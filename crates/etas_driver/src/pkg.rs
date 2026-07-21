use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use etas_frontend::{
    EntryPolicy, PackageMetadataBinInput, PackageMetadataBuildInput,
    PackageMetadataToolBindingInput, SnapshotDetailLevel,
};

use crate::{
    DriverError, DriverOptions, DriverProjectMode, LoadProjectRequest, SourceDependencyMode,
    load_project, source::compile_loaded_project,
};

const PACKAGE_METADATA_COMPILER_VERSION: &str =
    concat!(env!("CARGO_PKG_VERSION"), "+frontend-package-metadata-v2");

#[derive(Clone, Debug)]
pub struct PackageLockRequest {
    pub package_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackageLockResponse {
    pub package_root: PathBuf,
    pub package_count: usize,
}

#[derive(Clone, Debug)]
pub struct PackagePrepareRequest {
    pub package_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackagePrepareResponse {
    pub package_root: PathBuf,
    pub package_graph_path: PathBuf,
    pub dependency_count: usize,
}

#[derive(Clone, Debug)]
pub struct PackageUpdateRequest {
    pub package_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackageUpdateResponse {
    pub package_root: PathBuf,
    pub package_graph_path: PathBuf,
    pub dependency_count: usize,
    pub locked_package_count: usize,
}

#[derive(Clone, Debug)]
pub struct PackageMetadataRequest {
    pub package_root: PathBuf,
    pub bin: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PackageMetadataResponse {
    pub package_root: PathBuf,
    pub artifact_path: PathBuf,
    pub artifact_hash: String,
}

#[derive(Clone, Debug)]
pub struct PackagePackRequest {
    pub package_root: PathBuf,
    pub output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackagePackResponse {
    pub package_root: PathBuf,
    pub output: PathBuf,
    pub checksum: String,
}

pub fn lock_package(
    options: &DriverOptions,
    request: PackageLockRequest,
) -> Result<PackageLockResponse, DriverError> {
    let span = options.profile.span("package.lock", "package");
    ensure_local_path_dependency_metadata(options, &request.package_root)?;
    let source_config = etas_package::PackageSourceConfig::read_project(&request.package_root)?;
    let materialize_span = options.profile.span("package.materialize", "package");
    let materialized =
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: request.package_root,
            source_config: Some(source_config),
        })?;
    materialize_span.finish_ok();
    let lock_span = options.profile.span("package.write_lockfile", "package");
    let locked = etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: materialized.package_root,
    })?;
    lock_span.finish_ok();
    options.profile.counter(
        "package.locked_packages",
        locked.lockfile.packages.len() as u64,
    );
    span.finish_ok();
    Ok(PackageLockResponse {
        package_root: locked.package_root,
        package_count: locked.lockfile.packages.len(),
    })
}

pub fn prepare_package(
    options: &DriverOptions,
    request: PackagePrepareRequest,
) -> Result<PackagePrepareResponse, DriverError> {
    let span = options.profile.span("package.prepare", "package");
    ensure_local_path_dependency_metadata(options, &request.package_root)?;
    let source_config = etas_package::PackageSourceConfig::read_project(&request.package_root)?;
    let materialize_span = options.profile.span("package.materialize", "package");
    let materialized =
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: request.package_root,
            source_config: Some(source_config),
        })?;
    materialize_span.finish_ok();
    options
        .profile
        .counter("package.dependencies", materialized.dependency_count as u64);
    span.finish_ok();
    Ok(PackagePrepareResponse {
        package_root: materialized.package_root,
        package_graph_path: materialized.package_graph_path,
        dependency_count: materialized.dependency_count,
    })
}

pub fn update_package(
    options: &DriverOptions,
    request: PackageUpdateRequest,
) -> Result<PackageUpdateResponse, DriverError> {
    let span = options.profile.span("package.update", "package");
    ensure_local_path_dependency_metadata(options, &request.package_root)?;
    let source_config = etas_package::PackageSourceConfig::read_project(&request.package_root)?;
    let materialize_span = options.profile.span("package.materialize", "package");
    let materialized =
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: request.package_root,
            source_config: Some(source_config),
        })?;
    materialize_span.finish_ok();
    let lock_span = options.profile.span("package.write_lockfile", "package");
    let locked = etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: materialized.package_root.clone(),
    })?;
    lock_span.finish_ok();
    options
        .profile
        .counter("package.dependencies", materialized.dependency_count as u64);
    options.profile.counter(
        "package.locked_packages",
        locked.lockfile.packages.len() as u64,
    );
    span.finish_ok();
    Ok(PackageUpdateResponse {
        package_root: materialized.package_root,
        package_graph_path: materialized.package_graph_path,
        dependency_count: materialized.dependency_count,
        locked_package_count: locked.lockfile.packages.len(),
    })
}

pub fn build_package_metadata(
    options: &DriverOptions,
    request: PackageMetadataRequest,
) -> Result<PackageMetadataResponse, DriverError> {
    let span = options.profile.span("package.metadata", "package");
    ensure_local_path_dependency_metadata(options, &request.package_root)?;
    let response = build_package_metadata_without_dependency_bootstrap(options, request)?;
    span.finish_ok();
    Ok(response)
}

fn build_package_metadata_without_dependency_bootstrap(
    options: &DriverOptions,
    request: PackageMetadataRequest,
) -> Result<PackageMetadataResponse, DriverError> {
    let span = options.profile.span(
        "package.metadata.build_without_dependency_bootstrap",
        "package",
    );
    let loaded = load_project(
        options,
        LoadProjectRequest {
            paths: vec![request.package_root],
            all: false,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: request.bin.clone(),
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )?;
    if loaded.mode != DriverProjectMode::Package {
        return Err(DriverError::InvalidInput(
            "`etas pkg metadata` requires a package root with etas.toml".to_owned(),
        ));
    }
    let package_root = loaded.package_root.clone().ok_or_else(|| {
        DriverError::InvalidInput("loaded package project did not report a package root".to_owned())
    })?;
    let compiled = compile_loaded_project(options, loaded.loaded, SnapshotDetailLevel::None)?;
    if compiled.response.output.checked.is_none() {
        return Err(DriverError::InvalidInput(format!(
            "package metadata artifact requires a checked project; frontend reported: {}",
            frontend_diagnostic_summary(&compiled.response.output.diagnostics)
        )));
    }
    let manifest = etas_package::read_manifest(&package_root)?;
    let prepared = etas_package::prepare_package(etas_package::PreparePackageOptions {
        package_root: package_root.clone(),
        selected_bin: request.bin,
    })?;
    let artifact_path = etas_package::package_metadata_artifact_path(&package_root);
    let artifact = etas_frontend::emit_package_metadata_artifact(
        &artifact_path,
        PackageMetadataBuildInput {
            package_id: manifest.package.name.clone(),
            package_version: manifest.package.version.clone(),
            package_edition: manifest.package.edition.clone(),
            compiler_version: PACKAGE_METADATA_COMPILER_VERSION.to_owned(),
            source_payload_hash: etas_package::package_source_payload_hash(&package_root)?,
            manifest_hash: etas_package::package_manifest_hash(&package_root)?,
            dependency_lock_hash: etas_package::package_dependency_lock_hash(&package_root)?,
            bins: manifest.bins.iter().map(bin_input).collect(),
            dependencies: prepared
                .environment
                .dependencies
                .iter()
                .map(etas_package::resolved_dependency_metadata)
                .collect::<Result<Vec<_>, _>>()?,
            tool_bindings: manifest
                .bindings
                .tools
                .iter()
                .map(|(tool, binding)| PackageMetadataToolBindingInput {
                    tool: tool.clone(),
                    kind: binding.kind.clone(),
                    provider: binding.provider_name().unwrap_or_default(),
                    effect_row: binding.effects.clone(),
                    action_row: binding.actions.clone(),
                })
                .collect(),
        },
        &compiled.response.output,
    )
    .map_err(|error| DriverError::InvalidInput(error.to_string()))?;
    etas_package::read_package_metadata_artifact(&package_root)?;
    span.finish_ok();
    Ok(PackageMetadataResponse {
        package_root,
        artifact_path,
        artifact_hash: artifact.artifact_hash,
    })
}

fn frontend_diagnostic_summary(diagnostics: &[etas_core::Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return "no diagnostics".to_owned();
    }
    diagnostics
        .iter()
        .take(5)
        .map(|diagnostic| format!("{:?}: {}", diagnostic.code, diagnostic.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn ensure_local_path_dependency_metadata(
    options: &DriverOptions,
    package_root: &Path,
) -> Result<(), DriverError> {
    let mut visiting = BTreeSet::new();
    ensure_local_path_dependency_metadata_inner(options, package_root, &mut visiting)
}

fn ensure_local_path_dependency_metadata_inner(
    options: &DriverOptions,
    package_root: &Path,
    visiting: &mut BTreeSet<PathBuf>,
) -> Result<(), DriverError> {
    let package_root = package_root
        .canonicalize()
        .map_err(|source| DriverError::Io {
            path: package_root.to_path_buf(),
            source,
        })?;
    if !visiting.insert(package_root.clone()) {
        return Ok(());
    }

    let manifest = etas_package::read_manifest(&package_root)?;
    for (dependency_name, spec) in &manifest.dependencies {
        let etas_package::manifest::DependencySpec::Detailed(detail) = spec else {
            continue;
        };
        let Some(path) = &detail.path else {
            continue;
        };
        let dependency_root = if path.is_absolute() {
            path.clone()
        } else {
            package_root.join(path)
        }
        .canonicalize()
        .map_err(|source| DriverError::Io {
            path: package_root.join(path),
            source,
        })?;

        ensure_local_path_dependency_metadata_inner(options, &dependency_root, visiting)
            .map_err(|error| dependency_metadata_error(dependency_name, &dependency_root, error))?;
        materialize_and_lock_package(&dependency_root)
            .map_err(|error| dependency_metadata_error(dependency_name, &dependency_root, error))?;
        if dependency_metadata_needs_rebuild(&dependency_root)? {
            build_package_metadata_without_dependency_bootstrap(
                options,
                PackageMetadataRequest {
                    package_root: dependency_root.clone(),
                    bin: None,
                },
            )
            .map_err(|error| dependency_metadata_error(dependency_name, &dependency_root, error))?;
        }
        verify_dependency_metadata_available(dependency_name, &dependency_root)
            .map_err(|error| dependency_metadata_error(dependency_name, &dependency_root, error))?;
    }
    Ok(())
}

fn dependency_metadata_error(
    dependency_name: &str,
    dependency_root: &Path,
    error: DriverError,
) -> DriverError {
    DriverError::InvalidInput(format!(
        "dependency `{dependency_name}` at `{}` could not produce usable package metadata: {error}",
        dependency_root.display()
    ))
}

fn materialize_and_lock_package(package_root: &Path) -> Result<(), DriverError> {
    let source_config = etas_package::PackageSourceConfig::read_project(package_root)?;
    let materialized =
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: package_root.to_path_buf(),
            source_config: Some(source_config),
        })?;
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: materialized.package_root,
    })?;
    Ok(())
}

fn dependency_metadata_needs_rebuild(package_root: &Path) -> Result<bool, DriverError> {
    match etas_package::read_package_metadata_artifact(package_root) {
        Ok(Some(_)) => match etas_package::read_package_metadata_artifact_info(package_root) {
            Ok(Some(info)) => Ok(info.header.compiler_version != PACKAGE_METADATA_COMPILER_VERSION),
            Ok(None) => Ok(true),
            Err(_) => Ok(true),
        },
        Ok(None) => Ok(true),
        Err(_) => Ok(true),
    }
}

fn verify_dependency_metadata_available(
    dependency_name: &str,
    package_root: &Path,
) -> Result<(), DriverError> {
    etas_package::read_package_metadata_artifact(package_root)?
        .ok_or_else(|| {
            DriverError::InvalidInput(format!(
                "path dependency `{dependency_name}` did not produce .etas/package.etasmeta at `{}`",
                package_root.display()
            ))
        })
        .map(|_| ())
}

pub fn pack_package(request: PackagePackRequest) -> Result<PackagePackResponse, DriverError> {
    let packed = etas_package::pack_package(etas_package::PackPackageOptions {
        package_root: request.package_root,
        output: request.output,
    })?;
    Ok(PackagePackResponse {
        package_root: packed.package_root,
        output: packed.output,
        checksum: packed.checksum,
    })
}

fn bin_input(bin: &etas_package::BinTarget) -> PackageMetadataBinInput {
    PackageMetadataBinInput {
        name: bin.name.clone(),
        module: bin.module.clone(),
        flow: bin.flow.clone(),
    }
}
