use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use etas_core::{SourceFile, SourceId};
use etas_package::{ResolvedDependency, ResolvedDependencySource};
use etas_syntax::ast::Program;

use crate::{
    DriverError,
    package_env::{external_package_key, stable_external_package_id},
};

#[derive(Clone, Debug, Default)]
pub struct RuntimeSourcePlan {
    pub dependencies: BTreeMap<String, RuntimeSourceDependencyPlan>,
}

impl RuntimeSourcePlan {
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }

    pub(crate) fn source_inputs(
        &self,
        mut next: u32,
    ) -> Result<Vec<etas_frontend::SourceInput>, DriverError> {
        let mut sources = Vec::new();
        let mut paths = BTreeSet::new();
        for dependency in self.dependencies.values() {
            for path in &dependency.source_files {
                if !path.starts_with(&dependency.source_root) || !paths.insert(path.clone()) {
                    return Err(DriverError::InvalidInput(
                        "runtime source path is duplicated or outside its source root".into(),
                    ));
                }
                let relative = path
                    .strip_prefix(dependency.snapshot.package_root())
                    .map_err(|_| {
                        DriverError::InvalidInput(
                            "runtime source is outside its locked snapshot".into(),
                        )
                    })?;
                let bytes = dependency.snapshot.file(relative).ok_or_else(|| {
                    DriverError::InvalidInput(
                        "runtime source is absent from its locked snapshot".into(),
                    )
                })?;
                let text = std::str::from_utf8(bytes)
                    .map_err(|_| {
                        DriverError::InvalidInput("locked runtime source is not UTF-8".into())
                    })?
                    .to_owned();
                sources.push(etas_frontend::SourceInput {
                    id: SourceId(next),
                    path: Some(path.clone()),
                    text,
                    kind: etas_frontend::SourceKind::DependencySourceOverlay {
                        package: dependency.package,
                        import_root: dependency.import_root.clone(),
                    },
                });
                next = next.checked_add(1).ok_or_else(|| {
                    DriverError::InvalidInput("runtime source identity exhausted".into())
                })?;
            }
        }
        Ok(sources)
    }
}

pub(crate) enum RuntimeSourceError<E> {
    Driver(DriverError),
    Snapshot(etas_package::SourceSnapshotError<E>),
    Interrupted(E),
    ParseResource(etas_syntax::BodyResourceError),
}

impl<E> From<DriverError> for RuntimeSourceError<E> {
    fn from(error: DriverError) -> Self {
        Self::Driver(error)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeSourceDependencyPlan {
    pub package: etas_frontend::ExternalPackageId,
    pub import_root: String,
    pub package_root: PathBuf,
    pub source_root: PathBuf,
    pub seed_modules: BTreeSet<Vec<String>>,
    pub source_modules: BTreeSet<Vec<String>>,
    pub source_files: BTreeSet<PathBuf>,
    pub(crate) snapshot: Arc<etas_package::LockedPackageSourceSnapshot>,
}

pub fn materialize_runtime_source_plan(
    package_root: &Path,
    environment: &etas_package::PackageEnvironmentMetadata,
    requirements: &etas_frontend::RuntimeSourceRequirements,
) -> Result<RuntimeSourcePlan, DriverError> {
    RuntimeSourceMaterializer::new(package_root, environment).plan(requirements)
}

pub(crate) struct RuntimeSourceMaterializer<'a> {
    package_root: &'a Path,
    environment: &'a etas_package::PackageEnvironmentMetadata,
    snapshots: BTreeMap<etas_frontend::ExternalPackageId, RuntimeSourcePackageSnapshot>,
    budget: etas_package::SourceSnapshotBudget,
}

struct RuntimeSourcePackageSnapshot {
    import_root: String,
    snapshot: Arc<etas_package::LockedPackageSourceSnapshot>,
}

impl<'a> RuntimeSourceMaterializer<'a> {
    pub(crate) fn new(
        package_root: &'a Path,
        environment: &'a etas_package::PackageEnvironmentMetadata,
    ) -> Self {
        Self {
            package_root,
            environment,
            snapshots: BTreeMap::new(),
            budget: Default::default(),
        }
    }

    fn snapshot<E>(
        &mut self,
        package: etas_frontend::ExternalPackageId,
        checkpoint: &impl Fn() -> Result<(), E>,
    ) -> Result<&RuntimeSourcePackageSnapshot, RuntimeSourceError<E>> {
        checkpoint().map_err(|error| {
            RuntimeSourceError::Snapshot(etas_package::SourceSnapshotError::Interrupted(error))
        })?;
        let slot = match self.snapshots.entry(package) {
            std::collections::btree_map::Entry::Vacant(slot) => slot,
            std::collections::btree_map::Entry::Occupied(slot) => return Ok(slot.into_mut()),
        };
        let (dependency, dependency_package_root) =
            dependency_source_location(self.package_root, &self.environment.dependencies, package)?;
        let checksum = dependency.source.payload_checksum().ok_or_else(|| {
            DriverError::InvalidInput(
                "runtime source dependency has no locked payload checksum".into(),
            )
        })?;
        let snapshot = Arc::new(
            self.budget
                .read_locked(&dependency_package_root, checksum, checkpoint)
                .map_err(RuntimeSourceError::Snapshot)?,
        );
        if etas_package::PackageIdentity::current(snapshot.manifest()) != dependency.identity {
            return Err(DriverError::InvalidInput(
                "locked runtime source package identity differs from its dependency".into(),
            )
            .into());
        }
        Ok(slot.insert(RuntimeSourcePackageSnapshot {
            import_root: dependency.import_root.clone(),
            snapshot,
        }))
    }

    pub(crate) fn plan(
        &mut self,
        requirements: &etas_frontend::RuntimeSourceRequirements,
    ) -> Result<RuntimeSourcePlan, DriverError> {
        self.plan_controlled(requirements, &|| Ok::<_, std::convert::Infallible>(()))
            .map_err(|error| match error {
                RuntimeSourceError::Driver(error) => error,
                RuntimeSourceError::Snapshot(error) => DriverError::SourceSnapshot(error),
                RuntimeSourceError::Interrupted(never) => match never {},
                RuntimeSourceError::ParseResource(error) => DriverError::InvalidInput(format!(
                    "runtime source parser {:?} limit {} exceeded",
                    error.kind, error.limit
                )),
            })
    }

    pub(crate) fn plan_controlled<E>(
        &mut self,
        requirements: &etas_frontend::RuntimeSourceRequirements,
        checkpoint: &impl Fn() -> Result<(), E>,
    ) -> Result<RuntimeSourcePlan, RuntimeSourceError<E>> {
        let mut plans = BTreeMap::new();
        for requirement in requirements.dependencies.values() {
            let package = self.snapshot(requirement.package, checkpoint)?;
            if requirement.import_root != package.import_root {
                return Err(DriverError::InvalidInput(
                    "runtime source requirement has a different locked import root".into(),
                )
                .into());
            }
            let snapshot = package.snapshot.clone();
            let source_root = snapshot.package_root().join(snapshot.source_root());
            let plan = plans.entry(package.import_root.clone()).or_insert_with(|| {
                RuntimeSourceDependencyPlan {
                    package: requirement.package,
                    import_root: package.import_root.clone(),
                    package_root: snapshot.package_root().to_owned(),
                    source_root: source_root.clone(),
                    seed_modules: BTreeSet::new(),
                    source_modules: BTreeSet::new(),
                    source_files: BTreeSet::new(),
                    snapshot,
                }
            });
            plan.seed_modules.extend(
                requirement
                    .seed_modules
                    .iter()
                    .map(|module| module.segments.clone()),
            );
            for module in &requirement.required_modules {
                checkpoint().map_err(RuntimeSourceError::Interrupted)?;
                let files = module_source_files(&plan.snapshot, &module.segments, checkpoint)?;
                plan.source_modules.insert(module.segments.clone());
                plan.source_files.extend(files);
            }
        }
        Ok(RuntimeSourcePlan {
            dependencies: plans,
        })
    }
}

fn dependency_source_location<'a>(
    root: &Path,
    dependencies: &'a [ResolvedDependency],
    target: etas_frontend::ExternalPackageId,
) -> Result<(&'a ResolvedDependency, PathBuf), DriverError> {
    let mut nodes: Vec<(&ResolvedDependency, Option<usize>)> = Vec::new();
    let mut pending = dependencies
        .iter()
        .rev()
        .map(|dependency| (dependency, None))
        .collect::<Vec<_>>();
    let mut selected: Option<usize> = None;
    while let Some((dependency, parent)) = pending.pop() {
        let index = nodes.len();
        nodes.push((dependency, parent));
        if stable_external_package_id(&external_package_key(dependency)) == target {
            if let Some(previous) = selected {
                if nodes[previous].0 != dependency {
                    return Err(DriverError::InvalidInput(
                        "runtime source package has conflicting locked dependency records".into(),
                    ));
                }
            } else {
                selected = Some(index);
            }
        }
        pending.extend(
            dependency
                .dependencies
                .iter()
                .rev()
                .map(|child| (child, Some(index))),
        );
    }
    let selected = selected.ok_or_else(|| {
        DriverError::InvalidInput(format!(
            "runtime source requirement references unknown external package `{}`",
            target.0
        ))
    })?;
    let mut chain = Vec::new();
    let mut next = Some(selected);
    while let Some(index) = next {
        chain.push(nodes[index].0);
        next = nodes[index].1;
    }
    let mut location = root.to_owned();
    for dependency in chain.into_iter().rev() {
        location = dependency_package_root(&location, dependency)?;
    }
    Ok((nodes[selected].0, location))
}

fn dependency_package_root(
    base_package_root: &Path,
    dependency: &ResolvedDependency,
) -> Result<PathBuf, DriverError> {
    let path = match &dependency.source {
        ResolvedDependencySource::Path { path, .. } => Some(path.as_str()),
        ResolvedDependencySource::Vendor { path, .. } => Some(path.as_str()),
        ResolvedDependencySource::Registry { store, .. }
        | ResolvedDependencySource::Git { store, .. }
        | ResolvedDependencySource::GitHubClone { store, .. }
        | ResolvedDependencySource::GitHubRelease { store, .. } => store.as_deref(),
        ResolvedDependencySource::Builtin { .. } => None,
    }
    .ok_or_else(|| {
        DriverError::InvalidInput(format!(
            "dependency `{}` cannot provide runtime source from source `{}`",
            dependency.identity.name,
            dependency_source_kind(&dependency.source)
        ))
    })?;
    resolve_dependency_path(base_package_root, path)
}

fn dependency_source_kind(source: &ResolvedDependencySource) -> &'static str {
    match source {
        ResolvedDependencySource::Builtin { .. } => "builtin",
        ResolvedDependencySource::Registry { .. } => "registry",
        ResolvedDependencySource::Git { .. } => "git",
        ResolvedDependencySource::GitHubClone { .. } => "github clone",
        ResolvedDependencySource::GitHubRelease { .. } => "github release",
        ResolvedDependencySource::Path { .. } => "path",
        ResolvedDependencySource::Vendor { .. } => "vendor",
    }
}

fn resolve_dependency_path(base_package_root: &Path, path: &str) -> Result<PathBuf, DriverError> {
    let path = PathBuf::from(path);
    let path = if path.is_absolute() {
        path
    } else {
        base_package_root.join(path)
    };
    path.canonicalize()
        .map_err(|source| DriverError::Io { path, source })
}

fn module_source_files<E>(
    snapshot: &etas_package::LockedPackageSourceSnapshot,
    module: &[String],
    checkpoint: &impl Fn() -> Result<(), E>,
) -> Result<Vec<PathBuf>, RuntimeSourceError<E>> {
    if module.is_empty()
        || module.iter().any(|segment| {
            segment.is_empty()
                || matches!(segment.as_str(), "." | "..")
                || segment.contains(['/', '\\', '\0'])
        })
    {
        return Err(DriverError::InvalidInput(
            "runtime source requirement contains an invalid module path".into(),
        )
        .into());
    }
    let source_root = snapshot.source_root();
    let relative = source_root.join(module.iter().collect::<PathBuf>());
    let direct = relative.with_extension("es");
    let mod_file = relative.join("mod.es");
    let mut files = Vec::new();
    for file in [&direct, &mod_file] {
        if let Some(bytes) = snapshot.file(file) {
            if !dependency_source_declares_module(file, bytes, module, checkpoint)? {
                return Err(DriverError::InvalidInput(format!(
                    "runtime source `{}` does not declare required module `{}`",
                    file.display(),
                    module.join("."),
                ))
                .into());
            }
            files.push(snapshot.package_root().join(file));
        }
    }
    if files.is_empty() {
        return Err(DriverError::InvalidInput(format!(
            "runtime source module `{}` is missing under dependency source root `{}`",
            module.join("."),
            snapshot.package_root().join(source_root).display()
        ))
        .into());
    }
    for (path, bytes) in snapshot.files() {
        checkpoint().map_err(RuntimeSourceError::Interrupted)?;
        if path.parent() != Some(relative.as_path())
            || path.extension().and_then(|extension| extension.to_str()) != Some("es")
        {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("mod.es") {
            continue;
        }
        if dependency_source_declares_module(path, bytes, module, checkpoint)? {
            files.push(snapshot.package_root().join(path));
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn dependency_source_declares_module<E>(
    path: &Path,
    bytes: &[u8],
    module: &[String],
    checkpoint: &impl Fn() -> Result<(), E>,
) -> Result<bool, RuntimeSourceError<E>> {
    let program = parse_dependency_source(path, bytes, checkpoint)?;
    Ok(program
        .module
        .as_ref()
        .is_some_and(|decl| path_segments(&decl.path) == module))
}

fn parse_dependency_source<E>(
    path: &Path,
    bytes: &[u8],
    checkpoint: &impl Fn() -> Result<(), E>,
) -> Result<Program, RuntimeSourceError<E>> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        DriverError::InvalidInput(format!(
            "locked runtime source `{}` is not UTF-8",
            path.display()
        ))
    })?;
    let source = SourceFile::new(SourceId(0), Some(path.to_path_buf()), text);
    let parsed = etas_syntax::parse_program_bounded(&source, Default::default(), checkpoint)
        .map_err(|error| match error {
            etas_syntax::BoundedBodyParseError::Interrupted(error) => {
                RuntimeSourceError::Interrupted(error)
            }
            etas_syntax::BoundedBodyParseError::Resource(error) => {
                RuntimeSourceError::ParseResource(error)
            }
            etas_syntax::BoundedBodyParseError::Syntax(error) => {
                RuntimeSourceError::Driver(DriverError::InvalidInput(format!(
                    "dependency source `{}` has invalid syntax: {error}",
                    path.display()
                )))
            }
        })?;
    if let Some(diagnostic) = parsed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == etas_core::Severity::Error)
    {
        return Err(DriverError::InvalidInput(format!(
            "dependency source `{}` cannot be parsed for runtime source planning: {}",
            path.display(),
            diagnostic.message
        ))
        .into());
    }
    Ok(parsed.value)
}

fn path_segments(path: &etas_syntax::ast::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.text.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use etas_frontend::{
        ExternalPackageId, ModulePath, RuntimeSourceRequirement, RuntimeSourceRequirements,
    };
    use etas_package::{
        PackageEnvironmentMetadata, PackageFlowSignatureMetadata, PackageIdentity,
        PackagePublicMetadata, PackageTypeMetadata, ResolvedDependency, ResolvedDependencySource,
    };

    use super::*;

    #[test]
    fn runtime_source_plan_materializes_only_explicitly_required_modules_and_parts() {
        let root = unique_temp_dir("runtime-source-plan");
        let dependency_root = root.join("dep");
        let dependency_src = dependency_root.join("src");
        fs::create_dir_all(dependency_src.join("dep/api")).expect("create api part dir");
        fs::create_dir_all(dependency_src.join("dep/internal")).expect("create dependency src");
        fs::write(
            dependency_root.join("etas.toml"),
            r#"[package]
name = "dep"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
        )
        .expect("write manifest");
        fs::write(
            dependency_src.join("dep/api.es"),
            r#"module dep.api;
import dep.internal.helper;
import dep.types.TypeOnly;
flow call() -> unit { return; }
"#,
        )
        .expect("write api module");
        fs::write(
            dependency_src.join("dep/api/extra.es"),
            r#"module dep.api;
import dep.internal.extra_helper;
flow extra() -> unit { return; }
"#,
        )
        .expect("write api extra module part");
        fs::write(
            dependency_src.join("dep/internal/helper.es"),
            "module dep.internal.helper;\nflow help() -> unit { return; }\n",
        )
        .expect("write helper module");
        fs::write(
            dependency_src.join("dep/internal/extra_helper.es"),
            "module dep.internal.extra_helper;\nflow help() -> unit { return; }\n",
        )
        .expect("write extra helper module");
        fs::write(
            dependency_src.join("dep/unused.es"),
            "module dep.unused;\nflow unused() -> unit { return; }\n",
        )
        .expect("write unused module");
        fs::write(
            dependency_src.join("dep/types.es"),
            "module dep.types;\npublic type TypeOnly = string;\n",
        )
        .expect("write type-only module");

        let dependency = ResolvedDependency {
            identity: PackageIdentity {
                name: "dep".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            import_root: "dep".to_owned(),
            source: ResolvedDependencySource::Path {
                path: dependency_root.to_string_lossy().into_owned(),
                checksum: etas_package::package_payload_checksum(&dependency_root).unwrap(),
            },
            dependencies: Vec::new(),
            public_metadata: PackagePublicMetadata {
                flows: vec![PackageFlowSignatureMetadata {
                    generic_params: Vec::new(),
                    path: vec!["dep".to_owned(), "api".to_owned(), "call".to_owned()],
                    param_names: Vec::new(),
                    params: Vec::new(),
                    output: PackageTypeMetadata::Primitive {
                        name: "unit".to_owned(),
                    },
                    effects: None,
                    visibility: "public".to_owned(),
                }],
                ..PackagePublicMetadata::default()
            },
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
        };
        let environment = PackageEnvironmentMetadata {
            dependencies: vec![dependency],
            ..PackageEnvironmentMetadata::default()
        };
        let requirements =
            runtime_requirements_for_modules([vec!["dep".to_owned(), "api".to_owned()]]);

        let mut materializer = RuntimeSourceMaterializer::new(&root, &environment);
        let plan = materializer
            .plan(&requirements)
            .expect("runtime source plan");
        let dependency_plan = plan.dependencies.get("dep").expect("dependency plan");
        assert_eq!(
            dependency_plan.source_modules,
            BTreeSet::from([vec!["dep".to_owned(), "api".to_owned()]])
        );
        assert!(
            dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/api.es").canonicalize().unwrap())
        );
        assert!(
            !dependency_plan.source_files.contains(
                &dependency_src
                    .join("dep/internal/helper.es")
                    .canonicalize()
                    .unwrap()
            )
        );
        assert!(
            dependency_plan.source_files.contains(
                &dependency_src
                    .join("dep/api/extra.es")
                    .canonicalize()
                    .unwrap()
            )
        );
        assert!(
            !dependency_plan.source_files.contains(
                &dependency_src
                    .join("dep/internal/extra_helper.es")
                    .canonicalize()
                    .unwrap()
            )
        );
        assert!(
            !dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/unused.es").canonicalize().unwrap())
        );
        assert!(
            !dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/types.es").canonicalize().unwrap()),
            "type-only imports stay in metadata until frontend explicitly requires their source"
        );
        fs::write(
            dependency_src.join("dep/api.es"),
            "module dep.api; flow call() -> unit { abort(\"changed\"); }",
        )
        .unwrap();
        fs::remove_file(dependency_src.join("dep/internal/helper.es")).unwrap();
        let later = runtime_requirements_for_modules([vec![
            "dep".into(),
            "internal".into(),
            "helper".into(),
        ]]);
        let later_plan = materializer.plan(&later).unwrap();
        let later_dependency = &later_plan.dependencies["dep"];
        assert!(Arc::ptr_eq(
            &dependency_plan.snapshot,
            &later_dependency.snapshot
        ));
        assert!(
            later_dependency
                .source_files
                .contains(&dependency_plan.source_root.join("dep/internal/helper.es"))
        );
        let mut loaded = etas_frontend::LoadedProjectInput {
            input: etas_frontend::ProjectInput::single_source(
                etas_frontend::SourceInput::anonymous(
                    "module app; flow main() -> unit { return; }",
                ),
            ),
            sources: Vec::new(),
        };
        let added =
            crate::source::append_runtime_source_dependency_sources(&mut loaded, &plan).unwrap();
        assert_eq!(added.len(), 2);
        assert!(
            added.iter().all(|source| !source.text.contains("changed")),
            "source materialization must use the verified snapshot, not reread mutable paths"
        );
        assert!(matches!(
            materialize_runtime_source_plan(&root, &environment, &requirements),
            Err(DriverError::SourceSnapshot(
                etas_package::SourceSnapshotError::ChecksumMismatch { .. }
            ))
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_source_plan_materializes_required_mod_files_without_expanding_wildcards() {
        let root = unique_temp_dir("runtime-source-plan-wildcard-mod");
        let dependency_root = root.join("dep");
        let dependency_src = dependency_root.join("src");
        fs::create_dir_all(dependency_src.join("dep/plugins")).expect("create dependency src");
        fs::write(
            dependency_root.join("etas.toml"),
            r#"[package]
name = "dep"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
        )
        .expect("write manifest");
        fs::write(
            dependency_src.join("dep/api.es"),
            r#"module dep.api;
import dep.plugins.*;
flow call() -> unit { return; }
"#,
        )
        .expect("write api module");
        fs::write(
            dependency_src.join("dep/plugins/mod.es"),
            "module dep.plugins;\nflow register() -> unit { return; }\n",
        )
        .expect("write plugins mod module");
        fs::write(
            dependency_src.join("dep/plugins/json.es"),
            "module dep.plugins.json;\nflow encode() -> unit { return; }\n",
        )
        .expect("write plugins json module");
        fs::write(
            dependency_src.join("dep/unused.es"),
            "module dep.unused;\nflow unused() -> unit { return; }\n",
        )
        .expect("write unused module");

        let dependency = test_dependency(
            &dependency_root,
            vec!["dep".to_owned(), "api".to_owned(), "call".to_owned()],
        );
        let environment = PackageEnvironmentMetadata {
            dependencies: vec![dependency],
            ..PackageEnvironmentMetadata::default()
        };
        let mut requirements = runtime_requirements_for_modules([
            vec!["dep".to_owned(), "api".to_owned()],
            vec!["dep".to_owned(), "plugins".to_owned()],
        ]);
        requirements
            .dependencies
            .get_mut(&test_external_package_id())
            .unwrap()
            .seed_modules = BTreeSet::from([ModulePath {
            segments: vec!["dep".into(), "api".into()],
        }]);

        let plan = materialize_runtime_source_plan(&root, &environment, &requirements)
            .expect("runtime source plan");
        let dependency_plan = plan.dependencies.get("dep").expect("dependency plan");
        assert_eq!(
            dependency_plan.source_modules,
            BTreeSet::from([
                vec!["dep".to_owned(), "api".to_owned()],
                vec!["dep".to_owned(), "plugins".to_owned()],
            ])
        );
        assert!(
            dependency_plan.source_files.contains(
                &dependency_src
                    .join("dep/plugins/mod.es")
                    .canonicalize()
                    .unwrap()
            )
        );
        assert!(
            !dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/unused.es").canonicalize().unwrap())
        );
        assert!(
            !dependency_plan.source_files.contains(
                &dependency_src
                    .join("dep/plugins/json.es")
                    .canonicalize()
                    .unwrap()
            )
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_source_plan_fails_when_required_runtime_source_is_missing_or_misidentified() {
        let root = unique_temp_dir("runtime-source-plan-missing");
        let dependency_root = root.join("dep");
        fs::create_dir_all(dependency_root.join("src/dep")).expect("create dependency src");
        fs::write(
            dependency_root.join("etas.toml"),
            r#"[package]
name = "dep"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
        )
        .expect("write manifest");
        let dependency = test_dependency(
            &dependency_root,
            vec!["dep".to_owned(), "missing".to_owned(), "call".to_owned()],
        );
        let mut environment = PackageEnvironmentMetadata {
            dependencies: vec![dependency],
            ..PackageEnvironmentMetadata::default()
        };
        let requirements =
            runtime_requirements_for_modules([vec!["dep".to_owned(), "missing".to_owned()]]);

        let error = materialize_runtime_source_plan(&root, &environment, &requirements)
            .expect_err("missing runtime source should fail closed");
        assert!(
            error
                .to_string()
                .contains("runtime source module `dep.missing` is missing"),
            "{error}"
        );
        fs::write(
            dependency_root.join("src/dep/missing.es"),
            "module dep.other; public flow call() -> unit { return; }",
        )
        .unwrap();
        if let ResolvedDependencySource::Path { checksum, .. } =
            &mut environment.dependencies[0].source
        {
            *checksum = etas_package::package_payload_checksum(&dependency_root).unwrap();
        }
        let error =
            materialize_runtime_source_plan(&root, &environment, &requirements).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("does not declare required module `dep.missing`"),
            "{error}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_source_plan_preserves_cancellation_during_cached_source_parsing() {
        let root = unique_temp_dir("runtime-source-cancel");
        let dependency = root.join("dep");
        fs::create_dir_all(dependency.join("src/dep")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
        )
        .unwrap();
        fs::write(
            dependency.join("src/dep/api.es"),
            "module dep.api; public flow run() -> unit { return; }",
        )
        .unwrap();
        let environment = PackageEnvironmentMetadata {
            dependencies: vec![test_dependency(
                &dependency,
                vec!["dep".into(), "api".into(), "run".into()],
            )],
            ..Default::default()
        };
        let requirements = runtime_requirements_for_modules([vec!["dep".into(), "api".into()]]);
        let mut materializer = RuntimeSourceMaterializer::new(&root, &environment);
        materializer.plan(&requirements).unwrap();
        let calls = std::cell::Cell::new(0);
        let result = materializer.plan_controlled(&requirements, &|| {
            let count = calls.get() + 1;
            calls.set(count);
            if count == 4 {
                Err("cancelled in cached parser")
            } else {
                Ok(())
            }
        });
        assert!(matches!(
            result,
            Err(RuntimeSourceError::Interrupted(
                "cancelled in cached parser"
            ))
        ));
        assert_eq!(calls.get(), 4);
        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("etas-{name}-{suffix}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn test_dependency(dependency_root: &Path, flow_path: Vec<String>) -> ResolvedDependency {
        ResolvedDependency {
            identity: PackageIdentity {
                name: "dep".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            import_root: "dep".to_owned(),
            source: ResolvedDependencySource::Path {
                path: dependency_root.to_string_lossy().into_owned(),
                checksum: etas_package::package_payload_checksum(dependency_root).unwrap(),
            },
            dependencies: Vec::new(),
            public_metadata: PackagePublicMetadata {
                flows: vec![PackageFlowSignatureMetadata {
                    generic_params: Vec::new(),
                    path: flow_path,
                    param_names: Vec::new(),
                    params: Vec::new(),
                    output: PackageTypeMetadata::Primitive {
                        name: "unit".to_owned(),
                    },
                    effects: None,
                    visibility: "public".to_owned(),
                }],
                ..PackagePublicMetadata::default()
            },
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
        }
    }

    fn test_external_package_id() -> ExternalPackageId {
        stable_external_package_id("dep|dep|0.1.0|2026")
    }

    fn runtime_requirements_for_modules(
        modules: impl IntoIterator<Item = Vec<String>>,
    ) -> RuntimeSourceRequirements {
        let package = test_external_package_id();
        let seed_modules = modules
            .into_iter()
            .map(|segments| ModulePath { segments })
            .collect::<BTreeSet<_>>();
        RuntimeSourceRequirements {
            dependencies: BTreeMap::from([(
                package,
                RuntimeSourceRequirement {
                    package,
                    import_root: "dep".to_owned(),
                    seed_modules: seed_modules.clone(),
                    required_modules: seed_modules,
                    reasons: Vec::new(),
                },
            )]),
        }
    }
}
