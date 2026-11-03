use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use etas_code::{CodeFingerprint, PackageKey};
use etas_core::SourceId;
use etas_frontend::{ExternalPackageId, ProjectInput, SourceInput, SourceKind};
use etas_package::{
    PackageEnvironmentMetadata, PackageIdentity, ResolvedDependency, ResolvedDependencySource,
};

use super::invalid;
use crate::{DriverError, DriverProjectMode};

/// Captured by the filesystem owner before compilation. Later reflection and
/// candidate workers must not resolve mutable paths to recover package identity.
#[derive(Clone, Debug)]
pub(crate) struct SourceDefinitionProvenance {
    local_package: Option<PackageKey>,
    #[cfg(test)]
    single_file_root: Option<PathBuf>,
    single_files: HashMap<SourceId, (PathBuf, PackageKey)>,
    dependencies: HashMap<ExternalPackageId, DependencyOrigin>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyOrigin {
    package: Option<PackageKey>,
    import_root: String,
}

impl SourceDefinitionProvenance {
    #[cfg(test)]
    pub(crate) fn single_file_root(&self) -> Option<&Path> {
        self.single_file_root.as_deref()
    }
    pub(crate) fn local_package_key(&self) -> Option<PackageKey> {
        self.local_package
    }
    pub(crate) fn dependency_source_packages(
        &self,
    ) -> Vec<etas_frontend::ProjectDependencySourcePackage> {
        let mut packages: Vec<_> = self
            .dependencies
            .iter()
            .filter_map(|(package, origin)| {
                origin.package.map(|definition_package| {
                    etas_frontend::ProjectDependencySourcePackage {
                        package: *package,
                        import_root: origin.import_root.clone(),
                        definition_package,
                    }
                })
            })
            .collect();
        packages.sort_by_key(|entry| entry.package);
        packages
    }
    pub(crate) fn validate_external_definition_origins(
        &self,
        checked: &etas_frontend::CheckedProgramData,
    ) -> Result<(), DriverError> {
        for input in checked.external_definition_origins() {
            let origin = self
                .dependencies
                .get(&input.package)
                .ok_or_else(|| invalid("checked definition origin has no prepared dependency"))?;
            let expected = self.dependency_package_for(input.package, &origin.import_root)?;
            if input.origins.package != expected {
                return Err(invalid(
                    "package metadata definition origin disagrees with its prepared source origin",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn local_package_for(
        &self,
        source: &SourceInput,
    ) -> Result<PackageKey, DriverError> {
        if let Some(package) = self.local_package {
            return Ok(package);
        }
        let (path, package) = self
            .single_files
            .get(&source.id)
            .ok_or_else(|| invalid("source has no captured single-file package origin"))?;
        if source.path.as_ref() != Some(path) {
            return Err(invalid(
                "single-file source differs from its captured origin",
            ));
        }
        Ok(*package)
    }

    pub(crate) fn dependency_package_for(
        &self,
        package: ExternalPackageId,
        import_root: &str,
    ) -> Result<PackageKey, DriverError> {
        let origin = self
            .dependencies
            .get(&package)
            .ok_or_else(|| invalid("dependency source has no matching locked package origin"))?;
        if origin.import_root != import_root {
            return Err(invalid(
                "dependency source import binding disagrees with its package",
            ));
        }
        origin
            .package
            .ok_or_else(|| invalid("external source metadata has no canonical definition origin"))
    }
}

pub(crate) fn capture_source_definition_provenance(
    mode: DriverProjectMode,
    root: Option<&Path>,
    single_file_root: Option<&Path>,
    environment: Option<&PackageEnvironmentMetadata>,
    input: &ProjectInput,
) -> Result<SourceDefinitionProvenance, DriverError> {
    let local_package = match mode {
        DriverProjectMode::Package => {
            let root =
                root.ok_or_else(|| invalid("package definition has no prepared package root"))?;
            let identity = environment
                .and_then(|environment| environment.current_package.as_ref())
                .ok_or_else(|| invalid("package definition has no prepared package identity"))?;
            Some(local_package_key(root, identity)?)
        }
        DriverProjectMode::SourceProject => Some(local_source_key(
            "etas.code.source-project-origin.v1",
            &input.project_root,
        )?),
        DriverProjectMode::SingleFile => None,
    };
    let single_file_root = if mode == DriverProjectMode::SingleFile {
        Some(canonical_path(single_file_root.ok_or_else(|| {
            invalid("single-file entry origin is missing")
        })?)?)
    } else {
        None
    };
    let mut single_files = HashMap::new();
    if local_package.is_none() {
        let entry = single_file_root
            .as_ref()
            .ok_or_else(|| invalid("single-file entry origin is missing"))?;
        let package = local_source_key("etas.code.single-file-origin.v1", entry)?;
        let mut found_entry = false;
        for source in &input.sources {
            if !matches!(
                source.kind,
                SourceKind::SourceProjectFile | SourceKind::SingleFileInput
            ) {
                continue;
            }
            let path = source
                .path
                .as_ref()
                .ok_or_else(|| invalid("single-file definition has no canonical source origin"))?;
            found_entry |= path == entry;
            if single_files
                .insert(source.id, (path.clone(), package))
                .is_some()
            {
                return Err(invalid("loaded source identity is duplicated"));
            }
        }
        if !found_entry {
            return Err(invalid(
                "single-file entry does not belong to the loaded source graph",
            ));
        }
    }
    let mut dependencies = HashMap::new();
    if let (Some(root), Some(environment)) = (root, environment) {
        dependency_origins(root, &environment.dependencies, &mut dependencies)?;
    }
    Ok(SourceDefinitionProvenance {
        local_package,
        #[cfg(test)]
        single_file_root,
        single_files,
        dependencies,
    })
}

fn dependency_origins(
    root: &Path,
    dependencies: &[ResolvedDependency],
    output: &mut HashMap<ExternalPackageId, DependencyOrigin>,
) -> Result<(), DriverError> {
    let mut pending = dependencies
        .iter()
        .rev()
        .map(|dependency| (Some(root.to_owned()), dependency))
        .collect::<Vec<_>>();
    while let Some((owner, dependency)) = pending.pop() {
        let original_path = match &dependency.source {
            ResolvedDependencySource::Path { path, .. } if Path::new(path).is_absolute() => {
                Some(PathBuf::from(path))
            }
            ResolvedDependencySource::Path { path, .. } => {
                owner.as_ref().map(|owner| owner.join(path))
            }
            _ => None,
        };
        let package =
            match &dependency.source {
                ResolvedDependencySource::Builtin { .. } => continue,
                ResolvedDependencySource::Path { .. } => {
                    Some(local_package_key(root, &dependency.identity)?)
                }
                ResolvedDependencySource::Vendor {
                    definition_origin, ..
                } => {
                    let expected = local_package_key(root, &dependency.identity)?;
                    if definition_origin.is_some_and(|origin| origin != expected) {
                        return Err(invalid(
                            "vendored definition origin disagrees with package name and version",
                        ));
                    }
                    *definition_origin
                }
                ResolvedDependencySource::Registry {
                    registry, checksum, ..
                } => Some(locked_package_key(
                    root,
                    &dependency.identity,
                    &[registry, checksum],
                )?),
                ResolvedDependencySource::Git { url, rev, .. } => {
                    Some(locked_package_key(root, &dependency.identity, &[url, rev])?)
                }
                ResolvedDependencySource::GitHubClone { repo, rev, .. } => Some(
                    locked_package_key(root, &dependency.identity, &[repo, rev])?,
                ),
                ResolvedDependencySource::GitHubRelease {
                    repo,
                    release,
                    asset,
                    payload_checksum,
                    ..
                } => Some(locked_package_key(
                    root,
                    &dependency.identity,
                    &[repo, release, asset, payload_checksum],
                )?),
            };
        pending.extend(
            dependency
                .dependencies
                .iter()
                .rev()
                .map(|child| (original_path.clone(), child)),
        );
        let origin = DependencyOrigin {
            package,
            import_root: dependency.import_root.clone(),
        };
        match output.entry(dependency_id(dependency)) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(origin);
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                if entry.get() != &origin {
                    return Err(invalid("locked source package binding is ambiguous"));
                }
            }
        }
    }
    Ok(())
}

pub(super) fn dependency_id(dependency: &ResolvedDependency) -> ExternalPackageId {
    crate::package_env::stable_external_package_id(&crate::package_env::external_package_key(
        dependency,
    ))
}

fn local_package_key(root: &Path, identity: &PackageIdentity) -> Result<PackageKey, DriverError> {
    etas_package::local_definition_package_key(root, identity).map_err(DriverError::Package)
}

fn locked_package_key(
    root: &Path,
    identity: &PackageIdentity,
    origin: &[&str],
) -> Result<PackageKey, DriverError> {
    if [&identity.name, &identity.version, &identity.edition]
        .iter()
        .any(|part| part.is_empty() || part.contains('\0'))
        || origin
            .iter()
            .any(|part| part.is_empty() || part.contains('\0'))
    {
        return Err(invalid("locked source package identity is invalid"));
    }
    local_package_key(root, identity)
}

fn local_source_key(domain: &'static str, path: &Path) -> Result<PackageKey, DriverError> {
    let path = canonical_path(path)?;
    Ok(PackageKey::from_canonical_fingerprint(
        CodeFingerprint::of_bytes(domain, path.as_os_str().as_encoded_bytes()),
    ))
}

fn canonical_path(path: &Path) -> Result<PathBuf, DriverError> {
    path.canonicalize().map_err(|source| DriverError::Io {
        path: path.to_owned(),
        source,
    })
}
