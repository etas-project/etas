use std::path::{Path, PathBuf};

use crate::{
    BinTarget, PackageDiagnostic, PackageError, PackageIdentity, ResolvedDependency,
    ResolvedDependencySource, manifest, metadata, source, vendor,
};

use super::version::version_satisfies;

#[derive(Clone, Debug)]
pub struct MaterializePackageOptions {
    pub package_root: PathBuf,
    pub source_config: Option<source::PackageSourceConfig>,
}

#[derive(Clone, Debug)]
pub struct MaterializedPackageResult {
    pub package_root: PathBuf,
    pub package_graph_path: PathBuf,
    pub dependency_count: usize,
}

pub fn materialize_package(
    options: MaterializePackageOptions,
) -> Result<MaterializedPackageResult, PackageError> {
    let MaterializePackageOptions {
        package_root,
        source_config,
    } = options;
    let package_root = canonical_package_root(&package_root)?;
    let manifest = manifest::read_manifest(&package_root)?;
    let source_config = match source_config {
        Some(source_config) => source_config,
        None => source::PackageSourceConfig::read_project(&package_root)?,
    };
    let existing_index = vendor::read_package_index(&package_root)?;
    let mut diagnostics = Vec::new();
    let mut dependencies = Vec::new();

    for (dependency_name, spec) in &manifest.dependencies {
        if metadata::is_builtin_package(dependency_name) {
            continue;
        }

        let request = source::SourceResolveRequest {
            package_root: &package_root,
            source_config: &source_config,
            dependency_name,
            spec,
        };
        let source_package = match source::resolve_dependency_source(&request) {
            Ok(Some(source_package)) => source_package,
            Ok(None) => {
                diagnostics.push(PackageDiagnostic::new(
                    "package.materialize.unsupported_source",
                    format!(
                        "dependency `{dependency_name}` cannot be materialized; supported sources are `path`, configured local registries, `git`, GitHub clone, and GitHub release `.etaspkg` dependencies"
                    ),
                    Some(package_root.join("etas.toml")),
                ));
                continue;
            }
            Err(mut source_diagnostics) => {
                diagnostics.append(&mut source_diagnostics);
                continue;
            }
        };

        let source =
            match materialized_source(&package_root, dependency_name, spec, &source_package) {
                Ok(source) => source,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };

        match dependency_from_source_index(
            dependency_name,
            spec,
            &source_package,
            &package_root,
            source,
        ) {
            Ok(dependency) => dependencies.push(dependency),
            Err(mut dependency_diagnostics) => diagnostics.append(&mut dependency_diagnostics),
        }
    }

    if !diagnostics.is_empty() {
        return Err(PackageError::Diagnostics(diagnostics));
    }

    let mut index = existing_index.unwrap_or_else(|| empty_package_index(&manifest));
    index.version = 1;
    index.package = PackageIdentity::current(&manifest);
    index.dependencies = dependencies;
    if index.bins.is_empty() {
        index.bins = manifest.bins.clone();
    }
    vendor::write_package_index(&package_root, &index)?;

    Ok(MaterializedPackageResult {
        package_graph_path: vendor::package_index_path(&package_root),
        dependency_count: index.dependencies.len(),
        package_root,
    })
}

fn materialized_source(
    package_root: &Path,
    dependency_name: &str,
    spec: &manifest::DependencySpec,
    source_package: &source::ResolvedSourcePackage,
) -> Result<ResolvedDependencySource, PackageDiagnostic> {
    match &source_package.source {
        source::SourceCandidate::Path => {
            let vendored_payload = vendor::write_vendored_path_dependency(
                package_root,
                &spec.import_root(dependency_name),
                &source_package.package_root,
            )
            .map_err(|error| {
                PackageDiagnostic::new(
                    "package.materialize.vendor_write_failed",
                    format!("path dependency `{dependency_name}` could not be vendored: {error}"),
                    Some(package_root.join(".etas").join("vendor")),
                )
            })?;
            Ok(ResolvedDependencySource::Vendor {
                path: vendored_payload.relative_path,
                checksum: vendored_payload.checksum,
                store: Some(vendored_payload.store_path),
            })
        }
        source::SourceCandidate::Registry {
            registry,
            checksum,
            store,
        } => Ok(ResolvedDependencySource::Registry {
            registry: registry.clone(),
            checksum: checksum.clone(),
            store: Some(store.clone()),
        }),
        source::SourceCandidate::Git {
            url,
            rev,
            checksum,
            store,
        } => Ok(ResolvedDependencySource::Git {
            url: url.clone(),
            rev: rev.clone(),
            checksum: checksum.clone(),
            store: Some(store.clone()),
        }),
        source::SourceCandidate::GitHubClone {
            repo,
            rev,
            checksum,
            store,
        } => Ok(ResolvedDependencySource::GitHubClone {
            repo: repo.clone(),
            rev: rev.clone(),
            checksum: checksum.clone(),
            store: Some(store.clone()),
        }),
        source::SourceCandidate::GitHubRelease {
            repo,
            release,
            asset,
            asset_checksum,
            payload_checksum,
            store,
        } => Ok(ResolvedDependencySource::GitHubRelease {
            repo: repo.clone(),
            release: release.clone(),
            asset: asset.clone(),
            asset_checksum: asset_checksum.clone(),
            payload_checksum: payload_checksum.clone(),
            store: Some(store.clone()),
        }),
    }
}

fn canonical_package_root(package_root: &Path) -> Result<PathBuf, PackageError> {
    package_root
        .canonicalize()
        .map_err(|source| PackageError::Io {
            path: package_root.to_path_buf(),
            source,
        })
}

fn dependency_from_source_index(
    dependency_name: &str,
    spec: &manifest::DependencySpec,
    source_package: &source::ResolvedSourcePackage,
    package_root: &Path,
    source: ResolvedDependencySource,
) -> Result<ResolvedDependency, Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    let dependency_index = source_package.package_index.clone();
    if let Some(package_name) = spec.package_name_constraint(dependency_name)
        && dependency_index.package.name != package_name
    {
        diagnostics.push(PackageDiagnostic::new(
            "package.materialize.package_name_mismatch",
            format!(
                "dependency `{dependency_name}` expected package `{package_name}` but metadata declares `{}`",
                dependency_index.package.name
            ),
            Some(vendor::package_metadata_artifact_path(&source_package.package_root)),
        ));
    }
    if let Some(requirement) = spec.version() {
        match version_satisfies(&dependency_index.package.version, requirement) {
            Ok(true) => {}
            Ok(false) => diagnostics.push(PackageDiagnostic::new(
                "package.materialize.version_mismatch",
                format!(
                    "path dependency `{dependency_name}` version `{}` does not satisfy manifest requirement `{requirement}`",
                    dependency_index.package.version
                ),
                Some(package_root.join("etas.toml")),
            )),
            Err(message) => diagnostics.push(PackageDiagnostic::new(
                "package.manifest.invalid_version_requirement",
                format!(
                    "dependency `{dependency_name}` has invalid version requirement `{requirement}`: {message}"
                ),
                Some(package_root.join("etas.toml")),
            )),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let dependency = ResolvedDependency {
        identity: dependency_index.package,
        import_root: spec.import_root(dependency_name),
        source,
        dependencies: dependency_index.dependencies,
        public_metadata: dependency_index.public_metadata,
        effect_metadata: dependency_index.effect_metadata,
        tool_bindings: dependency_index.tool_bindings,
    };
    Ok(dependency)
}

fn empty_package_index(manifest: &manifest::Manifest) -> metadata::PackageIndex {
    metadata::PackageIndex {
        version: 1,
        package: PackageIdentity::current(manifest),
        dependencies: Vec::new(),
        external_modules: Vec::new(),
        public_metadata: Default::default(),
        effect_metadata: Default::default(),
        tool_bindings: Vec::new(),
        bins: Vec::<BinTarget>::new(),
    }
}
