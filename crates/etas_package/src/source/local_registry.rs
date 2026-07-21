use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{PackageDiagnostic, PackageError, manifest, store, vendor};

use super::{ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest};

pub struct LocalRegistrySourceProvider;

impl SourceProvider for LocalRegistrySourceProvider {
    fn resolve(
        &self,
        request: &SourceResolveRequest<'_>,
    ) -> Result<Option<ResolvedSourcePackage>, Vec<PackageDiagnostic>> {
        let manifest::DependencySpec::Detailed(detail) = request.spec else {
            return Ok(None);
        };
        let Some(registry_name) = &detail.registry else {
            return Ok(None);
        };
        let Some(registry) = request.source_config.registries.get(registry_name) else {
            return Err(vec![PackageDiagnostic::new(
                "package.registry.unknown",
                format!(
                    "dependency `{}` uses registry `{registry_name}`, but package source config does not define it",
                    request.dependency_name
                ),
                Some(
                    request
                        .package_root
                        .join(".etas")
                        .join("source-config.json"),
                ),
            )]);
        };
        let registry_root = request.package_root.join(&registry.path);
        let registry_root = registry_root.canonicalize().map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.registry.unreadable",
                format!("registry `{registry_name}` path cannot be read: {source}"),
                Some(request.package_root.join(&registry.path)),
            )]
        })?;
        let package_name = request.spec.package_name(request.dependency_name);
        let index = read_local_registry_index(&registry_root, package_name).map_err(|error| {
            vec![PackageDiagnostic::new(
                "package.registry.index_unreadable",
                format!("registry index for package `{package_name}` could not be read: {error}"),
                Some(registry_index_path(&registry_root, package_name)),
            )]
        })?;
        let selected = select_registry_version(
            request.package_root,
            request.dependency_name,
            request.spec.version(),
            &registry_index_path(&registry_root, package_name),
            &index,
        )?;
        let package_root = selected.package_root(&registry_root, package_name);
        let package_root = package_root.canonicalize().map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.registry.payload_missing",
                format!(
                    "registry package `{package_name}` version `{}` payload cannot be read: {source}",
                    selected.version
                ),
                Some(selected.package_root(&registry_root, package_name)),
            )]
        })?;
        let artifact_path = vendor::package_metadata_artifact_path(&package_root);
        if !artifact_path.is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.registry.metadata_missing",
                format!(
                    "registry package `{package_name}` version `{}` is missing .etas/package.etasmeta",
                    selected.version
                ),
                Some(artifact_path),
            )]);
        }
        let package_index =
            vendor::read_package_metadata_artifact_required(&package_root).map_err(|error| {
            vec![PackageDiagnostic::new(
                "package.registry.metadata_unreadable",
                format!(
                    "registry package `{package_name}` version `{}` metadata artifact could not be read: {error}",
                    selected.version
                ),
                Some(vendor::package_metadata_artifact_path(&package_root)),
            )]
        })?;
        let payload_checksum =
            vendor::path_package_payload_checksum(&package_root).map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.registry.payload_unreadable",
                    format!(
                        "registry package `{package_name}` version `{}` payload could not be checksummed: {error}",
                        selected.version
                    ),
                    Some(package_root.clone()),
                )]
            })?;
        if !selected.checksum.starts_with("blake3:") {
            return Err(vec![PackageDiagnostic::new(
                "package.registry.invalid_checksum",
                format!(
                    "registry package `{package_name}` version `{}` checksum `{}` must be a blake3 payload digest",
                    selected.version, selected.checksum
                ),
                Some(registry_index_path(&registry_root, package_name)),
            )]);
        }
        if selected.checksum != payload_checksum {
            return Err(vec![PackageDiagnostic::new(
                "package.registry.checksum_mismatch",
                format!(
                    "registry package `{package_name}` version `{}` checksum `{payload_checksum}` does not match registry index checksum `{}`",
                    selected.version, selected.checksum
                ),
                Some(registry_index_path(&registry_root, package_name)),
            )]);
        }
        let store_path = store::store_package_payload(
            request.package_root,
            &package_root,
            &payload_checksum,
        )
        .map_err(|error| {
            vec![PackageDiagnostic::new(
                "package.registry.store_write_failed",
                format!(
                    "registry package `{package_name}` version `{}` could not be stored: {error}",
                    selected.version
                ),
                Some(request.package_root.join(".etas").join("store")),
            )]
        })?;
        Ok(Some(ResolvedSourcePackage {
            package_root,
            package_index,
            source: SourceCandidate::Registry {
                registry: file_url(&registry_root),
                checksum: selected.checksum.clone(),
                store: store_path,
            },
        }))
    }
}

#[derive(Debug, Deserialize)]
struct LocalRegistryIndex {
    #[serde(default)]
    versions: Vec<LocalRegistryVersion>,
}

#[derive(Debug, Deserialize)]
struct LocalRegistryVersion {
    version: String,
    #[serde(default)]
    path: Option<PathBuf>,
    checksum: String,
}

impl LocalRegistryVersion {
    fn package_root(&self, registry_root: &Path, package_name: &str) -> PathBuf {
        self.path.clone().map_or_else(
            || {
                registry_root
                    .join("packages")
                    .join(package_name)
                    .join(&self.version)
            },
            |path| {
                if path.is_absolute() {
                    path
                } else {
                    registry_root.join(path)
                }
            },
        )
    }
}

fn read_local_registry_index(
    registry_root: &Path,
    package_name: &str,
) -> Result<LocalRegistryIndex, PackageError> {
    let path = registry_index_path(registry_root, package_name);
    let text = std::fs::read_to_string(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| PackageError::Manifest {
        path,
        message: format!("local registry index is not valid JSON: {source}"),
    })
}

fn select_registry_version<'a>(
    package_root: &Path,
    dependency_name: &str,
    requirement: Option<&str>,
    index_path: &Path,
    index: &'a LocalRegistryIndex,
) -> Result<&'a LocalRegistryVersion, Vec<PackageDiagnostic>> {
    let requirement = requirement.unwrap_or("*");
    let requirement = semver::VersionReq::parse(requirement).map_err(|source| {
        vec![PackageDiagnostic::new(
            "package.manifest.invalid_version_requirement",
            format!(
                "dependency `{dependency_name}` has invalid version requirement `{requirement}`: {source}"
            ),
            Some(package_root.join("etas.toml")),
        )]
    })?;
    let mut matching = Vec::new();
    let mut diagnostics = Vec::new();
    for candidate in &index.versions {
        match semver::Version::parse(&candidate.version) {
            Ok(version) => {
                if requirement.matches(&version) {
                    matching.push((version, candidate));
                }
            }
            Err(source) => diagnostics.push(PackageDiagnostic::new(
                "package.registry.invalid_version",
                format!(
                    "registry version `{}` for dependency `{dependency_name}` is not SemVer: {source}",
                    candidate.version
                ),
                Some(index_path.to_path_buf()),
            )),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    matching
        .into_iter()
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, candidate)| candidate)
        .ok_or_else(|| {
            vec![PackageDiagnostic::new(
                "package.registry.version_unresolved",
                format!(
                    "no local registry version for dependency `{dependency_name}` satisfies `{requirement}`"
                ),
                Some(package_root.join("etas.toml")),
            )]
        })
}

fn registry_index_path(registry_root: &Path, package_name: &str) -> PathBuf {
    registry_root
        .join("index")
        .join(format!("{package_name}.json"))
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
