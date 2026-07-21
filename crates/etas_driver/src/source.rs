use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
};

use etas_core::{SourceFile as CoreSourceFile, SourceId};
use etas_frontend::{
    EntryPolicy, LoadedProjectInput, ModulePath, ProjectEntry, ProjectSourceLoadOptions,
    ProjectSourceLoadScope, ProjectSourceLoader, SnapshotDetailLevel, SourceInput, SourceKind,
};
use etas_package::{PreparePackageOptions, prepare_package};

use crate::{
    DriverError, DriverOptions,
    package_env::{external_package_key, frontend_environment, stable_external_package_id},
    project_discovery::{
        absolute_input_path, choose_source_project_root, discover_package_root, resolve_workspace,
    },
    runtime_source::RuntimeSourcePlan,
};

#[derive(Clone, Debug)]
pub struct LoadProjectRequest {
    pub paths: Vec<PathBuf>,
    pub all: bool,
    pub entry_policy: EntryPolicy,
    pub flow: Option<String>,
    pub bin: Option<String>,
    pub source_dependency_mode: SourceDependencyMode,
}

#[derive(Clone, Debug, Default)]
pub enum SourceDependencyMode {
    #[default]
    MetadataOnly,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverProjectMode {
    Package,
    SourceProject,
    SingleFile,
}

#[derive(Clone, Debug)]
pub struct LoadedDriverProject {
    pub loaded: LoadedProjectInput,
    pub package_root: Option<PathBuf>,
    pub package_environment: Option<etas_package::PackageEnvironmentMetadata>,
    pub mode: DriverProjectMode,
}

pub fn load_project(
    options: &DriverOptions,
    request: LoadProjectRequest,
) -> Result<LoadedDriverProject, DriverError> {
    let _span = options.profile.span("driver.load_project", "driver");
    let local_project = resolve_project_inputs(options, &request)?;
    let import_search_roots =
        import_search_roots(&local_project.project_root, &local_project.roots);
    let source_kind = source_kind(&local_project.roots, request.all);
    let scope = project_source_load_scope(&request, &local_project.entry);
    let source_load_span = options.profile.span("driver.source_load", "driver");
    let mut loaded = ProjectSourceLoader::default()
        .load(ProjectSourceLoadOptions {
            project_root: local_project.project_root.clone(),
            roots: local_project.roots,
            entry: local_project.entry,
            scope,
            source_kind,
            import_search_roots,
            external_module_roots: standard_external_module_roots(),
        })
        .map_err(|source| DriverError::Io {
            path: local_project.project_root.clone(),
            source,
        })?;
    drop(source_load_span);
    options
        .profile
        .counter("frontend.loaded_source_files", loaded.sources.len() as u64);
    let environment_span = options
        .profile
        .span("driver.frontend_environment", "driver");
    loaded.input.environment = local_project.environment;
    drop(environment_span);
    loaded.input.options.entry_policy = request.entry_policy;
    append_source_dependency_sources(&mut loaded, &local_project.source_dependencies)?;
    options
        .profile
        .counter("frontend.total_source_files", loaded.sources.len() as u64);
    Ok(LoadedDriverProject {
        loaded,
        package_root: local_project.package_root,
        package_environment: local_project.package_environment,
        mode: local_project.mode,
    })
}

fn project_source_load_scope(
    request: &LoadProjectRequest,
    entry: &ProjectEntry,
) -> ProjectSourceLoadScope {
    if request.entry_policy == EntryPolicy::Runnable && !request.all && entry.module.is_some() {
        ProjectSourceLoadScope::ImportClosureFromEntry
    } else {
        ProjectSourceLoadScope::FullSourceTree
    }
}

pub fn compile_loaded_project(
    options: &DriverOptions,
    loaded: LoadedProjectInput,
    snapshot_detail: SnapshotDetailLevel,
) -> Result<crate::CompileProjectResponse, DriverError> {
    crate::check_project_once(
        options,
        crate::CompileProjectRequest {
            input: loaded.input,
            snapshot_detail,
        },
    )
}

#[derive(Clone, Debug)]
struct LocalSourceProject {
    project_root: PathBuf,
    package_root: Option<PathBuf>,
    roots: Vec<PathBuf>,
    entry: ProjectEntry,
    environment: etas_frontend::ProjectEnvironmentInput,
    package_environment: Option<etas_package::PackageEnvironmentMetadata>,
    mode: DriverProjectMode,
    source_dependencies: Vec<SourceDependencyOverlay>,
}

#[derive(Clone, Debug)]
struct SourceDependencyOverlay {
    package: etas_frontend::ExternalPackageId,
    import_root: String,
    package_root: PathBuf,
    source_root: PathBuf,
    source_files: Option<BTreeSet<PathBuf>>,
}

fn resolve_project_inputs(
    options: &DriverOptions,
    request: &LoadProjectRequest,
) -> Result<LocalSourceProject, DriverError> {
    let _span = options
        .profile
        .span("driver.resolve_project_inputs", "driver");
    let workspace_root = resolve_workspace(options)?;
    if request.paths.is_empty() {
        return resolve_directory_project(options, &workspace_root, request, true);
    }

    let absolute_paths = request
        .paths
        .iter()
        .cloned()
        .map(absolute_input_path)
        .collect::<Result<Vec<_>, _>>()?;

    if absolute_paths.len() == 1 && absolute_paths[0].is_dir() {
        return resolve_directory_project(options, &absolute_paths[0], request, false);
    }

    resolve_single_file_project(options, workspace_root, absolute_paths, request)
}

fn resolve_directory_project(
    options: &DriverOptions,
    root: &Path,
    request: &LoadProjectRequest,
    allow_ancestor_manifest: bool,
) -> Result<LocalSourceProject, DriverError> {
    if let Some(package_root) = directory_package_root(root, allow_ancestor_manifest) {
        let package_span = options.profile.span("package.prepare", "package");
        let prepared = prepare_package(PreparePackageOptions {
            package_root: package_root.clone(),
            selected_bin: request.bin.clone(),
        })?;
        drop(package_span);
        options.profile.counter(
            "package.dependencies",
            prepared.environment.dependencies.len() as u64,
        );
        let entry_span = options.profile.span("driver.package_entry", "driver");
        let entry = entry_from_package(&prepared, request.flow.clone(), request.entry_policy)?;
        drop(entry_span);
        let overlay_span = options
            .profile
            .span("driver.source_dependency_overlays", "driver");
        let source_dependencies = source_dependency_overlays_for_mode(
            &prepared.package_root,
            &prepared.environment.dependencies,
            &request.source_dependency_mode,
        )?;
        drop(overlay_span);
        let frontend_environment_span = options
            .profile
            .span("driver.frontend_environment_build", "driver");
        let frontend_environment = frontend_environment(prepared.environment.clone())?;
        drop(frontend_environment_span);
        let package_environment = Some(prepared.environment.clone());
        return Ok(LocalSourceProject {
            project_root: prepared.package_root.clone(),
            package_root: Some(prepared.package_root),
            roots: vec![prepared.source_root],
            entry,
            environment: frontend_environment,
            package_environment,
            mode: DriverProjectMode::Package,
            source_dependencies,
        });
    }

    let project_root = root.canonicalize().map_err(|source| DriverError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let source_root = choose_source_project_root(&project_root);
    Ok(LocalSourceProject {
        project_root,
        package_root: None,
        roots: vec![source_root],
        entry: fallback_entry(request.flow.clone()),
        environment: Default::default(),
        package_environment: None,
        mode: DriverProjectMode::SourceProject,
        source_dependencies: Vec::new(),
    })
}

fn directory_package_root(root: &Path, allow_ancestor_manifest: bool) -> Option<PathBuf> {
    if root.join("etas.toml").is_file() {
        return Some(root.to_path_buf());
    }
    if allow_ancestor_manifest {
        return discover_package_root(root);
    }
    None
}

fn resolve_single_file_project(
    options: &DriverOptions,
    workspace_root: PathBuf,
    paths: Vec<PathBuf>,
    request: &LoadProjectRequest,
) -> Result<LocalSourceProject, DriverError> {
    let package_root = paths.first().and_then(|path| discover_package_root(path));
    if let Some(package_root) = package_root {
        let package_span = options.profile.span("package.prepare", "package");
        let prepared = prepare_package(PreparePackageOptions {
            package_root: package_root.clone(),
            selected_bin: request.bin.clone(),
        })?;
        drop(package_span);
        options.profile.counter(
            "package.dependencies",
            prepared.environment.dependencies.len() as u64,
        );
        let source_dependencies = source_dependency_overlays_for_mode(
            &prepared.package_root,
            &prepared.environment.dependencies,
            &request.source_dependency_mode,
        )?;
        let package_environment = Some(prepared.environment.clone());
        return Ok(LocalSourceProject {
            project_root: prepared.package_root.clone(),
            package_root: Some(prepared.package_root.clone()),
            roots: paths,
            entry: entry_from_package(&prepared, request.flow.clone(), request.entry_policy)?,
            environment: frontend_environment(prepared.environment.clone())?,
            package_environment,
            mode: DriverProjectMode::Package,
            source_dependencies,
        });
    }

    let mode = if paths.len() == 1 && paths[0].is_file() {
        DriverProjectMode::SingleFile
    } else {
        DriverProjectMode::SourceProject
    };
    Ok(LocalSourceProject {
        project_root: workspace_root,
        package_root: None,
        roots: paths,
        entry: fallback_entry(request.flow.clone()),
        environment: Default::default(),
        package_environment: None,
        mode,
        source_dependencies: Vec::new(),
    })
}

fn source_dependency_overlays(
    package_root: &Path,
    dependencies: &[etas_package::ResolvedDependency],
) -> Result<Vec<SourceDependencyOverlay>, DriverError> {
    let mut overlays = Vec::new();
    collect_source_dependency_overlays(package_root, dependencies, &mut overlays)?;
    overlays.sort_by(|left, right| {
        left.import_root
            .cmp(&right.import_root)
            .then_with(|| left.package_root.cmp(&right.package_root))
    });
    overlays.dedup_by(|left, right| {
        left.import_root == right.import_root && left.package_root == right.package_root
    });
    Ok(overlays)
}

fn source_dependency_overlays_for_mode(
    package_root: &Path,
    dependencies: &[etas_package::ResolvedDependency],
    mode: &SourceDependencyMode,
) -> Result<Vec<SourceDependencyOverlay>, DriverError> {
    match mode {
        SourceDependencyMode::MetadataOnly => Ok(Vec::new()),
        SourceDependencyMode::All => source_dependency_overlays(package_root, dependencies),
    }
}

fn source_dependency_overlays_from_runtime_plan(
    plan: &RuntimeSourcePlan,
) -> Result<Vec<SourceDependencyOverlay>, DriverError> {
    let mut overlays = Vec::new();
    for dependency in plan.dependencies.values() {
        overlays.push(SourceDependencyOverlay {
            package: dependency.package,
            import_root: dependency.import_root.clone(),
            package_root: dependency.package_root.clone(),
            source_root: dependency.source_root.clone(),
            source_files: Some(dependency.source_files.clone()),
        });
    }
    Ok(overlays)
}

pub(crate) fn append_runtime_source_dependency_sources(
    loaded: &mut LoadedProjectInput,
    plan: &RuntimeSourcePlan,
) -> Result<Vec<SourceInput>, DriverError> {
    let previous_sources = loaded
        .input
        .sources
        .iter()
        .map(|source| source.id)
        .collect::<HashSet<_>>();
    let overlays = source_dependency_overlays_from_runtime_plan(plan)?;
    append_source_dependency_sources(loaded, &overlays)?;
    Ok(loaded
        .input
        .sources
        .iter()
        .filter(|source| !previous_sources.contains(&source.id))
        .cloned()
        .collect())
}

fn collect_source_dependency_overlays(
    base_package_root: &Path,
    dependencies: &[etas_package::ResolvedDependency],
    output: &mut Vec<SourceDependencyOverlay>,
) -> Result<(), DriverError> {
    for dependency in dependencies {
        if let Some(package_root) = source_dependency_package_root(base_package_root, dependency)? {
            let manifest = etas_package::read_manifest(&package_root)?;
            let source_root = package_root
                .join(&manifest.source.root)
                .canonicalize()
                .map_err(|source| DriverError::Io {
                    path: package_root.join(&manifest.source.root),
                    source,
                })?;
            output.push(SourceDependencyOverlay {
                package: stable_external_package_id(&external_package_key(dependency)),
                import_root: dependency.import_root.clone(),
                package_root: package_root.clone(),
                source_root,
                source_files: None,
            });
            collect_source_dependency_overlays(&package_root, &dependency.dependencies, output)?;
        }
    }
    Ok(())
}

fn source_dependency_package_root(
    base_package_root: &Path,
    dependency: &etas_package::ResolvedDependency,
) -> Result<Option<PathBuf>, DriverError> {
    match &dependency.source {
        etas_package::ResolvedDependencySource::Path { path, .. } => {
            Ok(Some(resolve_dependency_path(base_package_root, path)?))
        }
        etas_package::ResolvedDependencySource::Vendor { path, .. } => {
            Ok(Some(resolve_dependency_path(base_package_root, path)?))
        }
        _ => Ok(None),
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

fn append_source_dependency_sources(
    loaded: &mut LoadedProjectInput,
    dependencies: &[SourceDependencyOverlay],
) -> Result<(), DriverError> {
    if dependencies.is_empty() {
        return Ok(());
    }
    let Some(project_source_root) = loaded.input.source_root.clone() else {
        return Err(DriverError::InvalidInput(
            "source dependency execution requires a project source root".to_owned(),
        ));
    };
    let mut next_id = loaded
        .input
        .sources
        .iter()
        .map(|source| source.id.0)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let mut paths = loaded
        .input
        .sources
        .iter()
        .filter_map(|source| source.path.clone())
        .collect::<BTreeSet<_>>();

    for dependency in dependencies {
        let mut real_paths = match &dependency.source_files {
            Some(source_files) => source_files.iter().cloned().collect::<Vec<_>>(),
            None => {
                let mut real_paths = Vec::new();
                collect_source_files(&dependency.source_root, &mut real_paths)?;
                real_paths
            }
        };
        real_paths.sort();
        for real_path in real_paths {
            let relative = real_path
                .strip_prefix(&dependency.source_root)
                .map_err(|_| {
                    DriverError::InvalidInput(format!(
                        "dependency source file `{}` is outside source root `{}`",
                        real_path.display(),
                        dependency.source_root.display()
                    ))
                })?
                .to_path_buf();
            let virtual_path = project_source_root.join(relative);
            if !paths.insert(virtual_path.clone()) {
                return Err(DriverError::InvalidInput(format!(
                    "source dependency `{}` maps to duplicate project source path `{}`",
                    dependency.import_root,
                    virtual_path.display()
                )));
            }
            let text = std::fs::read_to_string(&real_path).map_err(|source| DriverError::Io {
                path: real_path.clone(),
                source,
            })?;
            let id = SourceId(next_id);
            next_id = next_id.saturating_add(1);
            let source = CoreSourceFile::new(id, Some(virtual_path.clone()), text.clone());
            loaded.sources.push((virtual_path.clone(), source));
            loaded.input.sources.push(SourceInput {
                id,
                path: Some(virtual_path),
                text,
                kind: SourceKind::DependencySourceOverlay {
                    package: dependency.package,
                    import_root: dependency.import_root.clone(),
                },
            });
        }
    }
    Ok(())
}

fn collect_source_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), DriverError> {
    for entry in std::fs::read_dir(root).map_err(|source| DriverError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| DriverError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, output)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("es") {
            output.push(path.canonicalize().map_err(|source| DriverError::Io {
                path: path.clone(),
                source,
            })?);
        }
    }
    Ok(())
}

fn entry_from_package(
    prepared: &etas_package::PreparedPackage,
    flow: Option<String>,
    entry_policy: EntryPolicy,
) -> Result<ProjectEntry, DriverError> {
    if let Some(bin) = &prepared.selected_bin {
        return Ok(ProjectEntry {
            module: Some(ModulePath {
                segments: bin.module.split('.').map(str::to_owned).collect(),
            }),
            flow: flow.unwrap_or_else(|| bin.flow.clone()),
        });
    }
    if let Some(flow) = flow {
        return Err(DriverError::InvalidInput(format!(
            "package `{}` has no selected [[bin]] module for --flow `{flow}`; select a bin or use source/single-file mode",
            prepared.manifest.package.name
        )));
    }
    if entry_policy == EntryPolicy::Runnable {
        return Err(DriverError::InvalidInput(format!(
            "package `{}` has no selected [[bin]] module; select a bin or use source/single-file mode",
            prepared.manifest.package.name
        )));
    }
    Ok(fallback_entry(None))
}

fn fallback_entry(flow: Option<String>) -> ProjectEntry {
    ProjectEntry {
        module: None,
        flow: flow.unwrap_or_else(|| "main".to_owned()),
    }
}

fn source_kind(paths: &[PathBuf], all: bool) -> Option<SourceKind> {
    if all || paths.is_empty() || paths.iter().any(|path| path.is_dir()) {
        Some(SourceKind::SourceProjectFile)
    } else {
        Some(SourceKind::SingleFileInput)
    }
}

fn import_search_roots(project_root: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = vec![project_root.to_path_buf()];
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for path in paths {
        let absolute = if path.is_absolute() {
            path.clone()
        } else {
            current_dir.join(path)
        };
        if absolute.is_file() {
            if let Some(root) = absolute.parent() {
                roots.push(root.to_path_buf());
            }
        } else if absolute.is_dir() {
            roots.push(absolute);
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn standard_external_module_roots() -> Vec<ModulePath> {
    vec![ModulePath {
        segments: vec!["std".to_owned()],
    }]
}
