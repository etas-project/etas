use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use etas_core::{SourceFile, SourceId};
use etas_package::{ResolvedDependency, ResolvedDependencySource};
use etas_syntax::ast::{ImportTree, Item, Program};

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
}

pub fn materialize_runtime_source_plan(
    package_root: &Path,
    environment: &etas_package::PackageEnvironmentMetadata,
    requirements: &etas_frontend::RuntimeSourceRequirements,
) -> Result<RuntimeSourcePlan, DriverError> {
    let dependencies = flatten_dependencies(&environment.dependencies);
    let mut plans = BTreeMap::new();
    for requirement in requirements.dependencies.values() {
        let dependency = dependencies
            .iter()
            .copied()
            .find(|dependency| {
                stable_external_package_id(&external_package_key(dependency)) == requirement.package
            })
            .ok_or_else(|| {
                DriverError::InvalidInput(format!(
                    "runtime source requirement references unknown external package `{}`",
                    requirement.package.0
                ))
            })?;
        let dependency_package_root = dependency_package_root(package_root, dependency)?;
        let manifest = etas_package::read_manifest(&dependency_package_root)?;
        let source_root = dependency_package_root
            .join(&manifest.source.root)
            .canonicalize()
            .map_err(|source| DriverError::Io {
                path: dependency_package_root.join(&manifest.source.root),
                source,
            })?;
        let plan = plans
            .entry(dependency.import_root.clone())
            .or_insert_with(|| RuntimeSourceDependencyPlan {
                package: requirement.package,
                import_root: dependency.import_root.clone(),
                package_root: dependency_package_root.clone(),
                source_root: source_root.clone(),
                seed_modules: BTreeSet::new(),
                source_modules: BTreeSet::new(),
                source_files: BTreeSet::new(),
            });
        for module in &requirement.seed_modules {
            plan.seed_modules.insert(module.segments.clone());
            collect_reachable_dependency_sources(
                &plan.source_root,
                &dependency.import_root,
                module.segments.clone(),
                &mut plan.source_modules,
                &mut plan.source_files,
            )?;
        }
    }
    Ok(RuntimeSourcePlan {
        dependencies: plans,
    })
}

fn flatten_dependencies(dependencies: &[ResolvedDependency]) -> Vec<&ResolvedDependency> {
    fn walk<'a>(dependency: &'a ResolvedDependency, output: &mut Vec<&'a ResolvedDependency>) {
        output.push(dependency);
        for child in &dependency.dependencies {
            walk(child, output);
        }
    }
    let mut output = Vec::new();
    for dependency in dependencies {
        walk(dependency, &mut output);
    }
    output
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

fn collect_reachable_dependency_sources(
    source_root: &Path,
    import_root: &str,
    module: Vec<String>,
    source_modules: &mut BTreeSet<Vec<String>>,
    output: &mut BTreeSet<PathBuf>,
) -> Result<(), DriverError> {
    let mut pending = vec![module];
    let mut seen = BTreeSet::<Vec<String>>::new();
    while let Some(module) = pending.pop() {
        if !seen.insert(module.clone()) {
            continue;
        }
        let files = module_source_files(source_root, &module)?;
        source_modules.insert(module.clone());
        for file in files {
            if !output.insert(file.clone()) {
                continue;
            }
            let program = parse_dependency_source(&file)?;
            for imported in dependency_import_modules(import_root, &program, source_root)? {
                if !seen.contains(&imported) {
                    pending.push(imported);
                }
            }
        }
    }
    Ok(())
}

fn module_source_files(source_root: &Path, module: &[String]) -> Result<Vec<PathBuf>, DriverError> {
    let relative = module.iter().collect::<PathBuf>();
    let direct = source_root.join(&relative).with_extension("es");
    let mod_file = source_root.join(&relative).join("mod.es");
    let mut files = Vec::new();
    if direct.is_file() {
        files.push(direct.canonicalize().map_err(|source| DriverError::Io {
            path: direct,
            source,
        })?);
    }
    if mod_file.is_file() {
        files.push(mod_file.canonicalize().map_err(|source| DriverError::Io {
            path: mod_file,
            source,
        })?);
    }
    if files.is_empty() {
        return Err(DriverError::InvalidInput(format!(
            "runtime source module `{}` is missing under dependency source root `{}`",
            module.join("."),
            source_root.display()
        )));
    }
    collect_module_part_files(source_root, module, &mut files)?;
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_module_part_files(
    source_root: &Path,
    module: &[String],
    output: &mut Vec<PathBuf>,
) -> Result<(), DriverError> {
    let relative = module.iter().collect::<PathBuf>();
    let directory = source_root.join(relative);
    if !directory.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(&directory).map_err(|source| DriverError::Io {
        path: directory.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DriverError::Io {
            path: directory.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|extension| extension.to_str()) != Some("es")
        {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("mod.es") {
            continue;
        }
        if dependency_source_declares_module(&path, module)? {
            output.push(path.canonicalize().map_err(|source| DriverError::Io {
                path: path.clone(),
                source,
            })?);
        }
    }
    Ok(())
}

fn dependency_source_declares_module(path: &Path, module: &[String]) -> Result<bool, DriverError> {
    let program = parse_dependency_source(path)?;
    Ok(program
        .module
        .as_ref()
        .is_some_and(|decl| path_segments(&decl.path) == module))
}

fn parse_dependency_source(path: &Path) -> Result<Program, DriverError> {
    let text = std::fs::read_to_string(path).map_err(|source| DriverError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let source = SourceFile::new(SourceId(0), Some(path.to_path_buf()), text);
    let parsed = etas_syntax::parse_program(source);
    if let Some(diagnostic) = parsed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == etas_core::Severity::Error)
    {
        return Err(DriverError::InvalidInput(format!(
            "dependency source `{}` cannot be parsed for runtime source planning: {}",
            path.display(),
            diagnostic.message
        )));
    }
    Ok(parsed.value)
}

fn dependency_import_modules(
    import_root: &str,
    program: &Program,
    source_root: &Path,
) -> Result<Vec<Vec<String>>, DriverError> {
    let mut modules = BTreeSet::new();
    for import in &program.imports {
        collect_import_tree_modules(import_root, source_root, &import.tree, &mut modules)?;
    }
    Ok(modules.into_iter().collect())
}

fn collect_import_tree_modules(
    import_root: &str,
    source_root: &Path,
    tree: &ImportTree,
    output: &mut BTreeSet<Vec<String>>,
) -> Result<(), DriverError> {
    match tree {
        ImportTree::Single { path, .. } => {
            collect_candidate_import_path(import_root, source_root, &path_segments(path), output)?;
        }
        ImportTree::Group { prefix, items, .. } => {
            let prefix = path_segments(prefix);
            collect_candidate_import_path(import_root, source_root, &prefix, output)?;
            for item in items {
                let mut candidate = prefix.clone();
                candidate.push(item.name.text.clone());
                collect_candidate_import_path(import_root, source_root, &candidate, output)?;
            }
        }
        ImportTree::Wildcard { prefix, .. } => {
            collect_wildcard_import_path(import_root, source_root, &path_segments(prefix), output)?;
        }
        ImportTree::Error { .. } => {}
    }
    Ok(())
}

fn collect_wildcard_import_path(
    import_root: &str,
    source_root: &Path,
    path: &[String],
    output: &mut BTreeSet<Vec<String>>,
) -> Result<(), DriverError> {
    if !path.starts_with(&import_root_segments(import_root)) {
        return Ok(());
    }
    collect_candidate_import_path(import_root, source_root, path, output)?;
    let relative = path.iter().collect::<PathBuf>();
    let directory = source_root.join(relative);
    if directory.is_dir() {
        collect_module_files_under(source_root, &directory, output)?;
    }
    Ok(())
}

fn collect_module_files_under(
    source_root: &Path,
    directory: &Path,
    output: &mut BTreeSet<Vec<String>>,
) -> Result<(), DriverError> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_module_files_under(source_root, &path, output)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("es") {
            continue;
        }
        let Some(module) = module_path_for_source_file(source_root, &path) else {
            continue;
        };
        output.insert(module);
    }
    Ok(())
}

fn module_path_for_source_file(source_root: &Path, path: &Path) -> Option<Vec<String>> {
    let relative = path.strip_prefix(source_root).ok()?;
    if relative.file_name().and_then(|name| name.to_str()) == Some("mod.es") {
        let parent = relative.parent()?;
        return pathbuf_segments(parent);
    }
    let mut segments = pathbuf_segments(relative)?;
    let last = segments.last_mut()?;
    *last = last.strip_suffix(".es").unwrap_or(last).to_owned();
    Some(segments)
}

fn pathbuf_segments(path: &Path) -> Option<Vec<String>> {
    path.iter()
        .map(|segment| segment.to_str().map(str::to_owned))
        .collect()
}

fn collect_candidate_import_path(
    import_root: &str,
    source_root: &Path,
    path: &[String],
    output: &mut BTreeSet<Vec<String>>,
) -> Result<(), DriverError> {
    if !path.starts_with(&import_root_segments(import_root)) {
        return Ok(());
    }
    if module_file_exists(source_root, path) {
        output.insert(path.to_vec());
        return Ok(());
    }
    if path.len() > 1 {
        let parent = &path[..path.len() - 1];
        if module_file_exists(source_root, parent) {
            let item_name = path.last().expect("item path has last segment");
            if module_defines_item(source_root, parent, item_name)? {
                output.insert(parent.to_vec());
            }
        }
    }
    Ok(())
}

fn module_file_exists(source_root: &Path, module: &[String]) -> bool {
    let relative = module.iter().collect::<PathBuf>();
    source_root.join(&relative).with_extension("es").is_file()
        || source_root.join(&relative).join("mod.es").is_file()
}

fn module_defines_item(
    source_root: &Path,
    module: &[String],
    item_name: &str,
) -> Result<bool, DriverError> {
    for file in module_source_files(source_root, module)? {
        let program = parse_dependency_source(&file)?;
        if program
            .items
            .iter()
            .any(|item| item_name_in_source(&item.item).is_some_and(|name| name == item_name))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn item_name_in_source(item: &Item) -> Option<&str> {
    match item {
        Item::Alias(item) => Some(item.name.text.as_str()),
        Item::Type(item) => Some(item.name.text.as_str()),
        Item::Enum(item) => Some(item.name.text.as_str()),
        Item::Spec(item) => Some(item.name.text.as_str()),
        Item::Impl(_) => None,
        Item::Effect(item) => Some(item.name.text.as_str()),
        Item::Protocol(item) => Some(item.name.text.as_str()),
        Item::Flow(item) => Some(item.name.text.as_str()),
        Item::Agent(item) => Some(item.name.text.as_str()),
        Item::TopLevelLet(item) => Some(item.name.text.as_str()),
        Item::Tool(item) => item
            .path
            .segments
            .last()
            .map(|segment| segment.text.as_str()),
        Item::Error { .. } => None,
    }
}

fn path_segments(path: &etas_syntax::ast::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.text.clone())
        .collect()
}

fn import_root_segments(import_root: &str) -> Vec<String> {
    import_root
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
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
    fn runtime_source_plan_loads_referenced_module_import_closure_only() {
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
                checksum: "blake3:test".to_owned(),
            },
            dependencies: Vec::new(),
            public_metadata: PackagePublicMetadata {
                flows: vec![PackageFlowSignatureMetadata {
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

        let plan = materialize_runtime_source_plan(&root, &environment, &requirements)
            .expect("runtime source plan");
        let dependency_plan = plan.dependencies.get("dep").expect("dependency plan");
        assert_eq!(
            dependency_plan.source_modules,
            BTreeSet::from([
                vec!["dep".to_owned(), "api".to_owned()],
                vec![
                    "dep".to_owned(),
                    "internal".to_owned(),
                    "extra_helper".to_owned()
                ],
                vec!["dep".to_owned(), "internal".to_owned(), "helper".to_owned()],
                vec!["dep".to_owned(), "types".to_owned()],
            ])
        );
        assert!(
            dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/api.es").canonicalize().unwrap())
        );
        assert!(
            dependency_plan.source_files.contains(
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
            dependency_plan.source_files.contains(
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
            dependency_plan
                .source_files
                .contains(&dependency_src.join("dep/types.es").canonicalize().unwrap()),
            "source-compiled modules must include their source import closure"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_source_plan_follows_wildcard_imports_and_mod_files() {
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
        let requirements =
            runtime_requirements_for_modules([vec!["dep".to_owned(), "api".to_owned()]]);

        let plan = materialize_runtime_source_plan(&root, &environment, &requirements)
            .expect("runtime source plan");
        let dependency_plan = plan.dependencies.get("dep").expect("dependency plan");
        assert_eq!(
            dependency_plan.source_modules,
            BTreeSet::from([
                vec!["dep".to_owned(), "api".to_owned()],
                vec!["dep".to_owned(), "plugins".to_owned()],
                vec!["dep".to_owned(), "plugins".to_owned(), "json".to_owned()],
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
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_source_plan_fails_when_required_runtime_source_is_missing() {
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
        let environment = PackageEnvironmentMetadata {
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
        let _ = fs::remove_dir_all(root);
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
                checksum: "blake3:test".to_owned(),
            },
            dependencies: Vec::new(),
            public_metadata: PackagePublicMetadata {
                flows: vec![PackageFlowSignatureMetadata {
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
