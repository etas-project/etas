use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use super::{
    copy_directory, copy_file, package_index_path, package_metadata_artifact_path,
    path_package_payload_checksum,
};
use crate::{PackageError, ResolvedDependency, ResolvedDependencySource, manifest};

/// Copy only payloads named by the dependency graph, never unrelated vendor or
/// store contents. Each relative location is owned by its containing package.
pub(super) fn copy_materialized_payloads(
    source: &Path,
    destination: &Path,
    dependencies: &[ResolvedDependency],
) -> Result<(), PackageError> {
    let mut pending = dependencies
        .iter()
        .rev()
        .map(|dependency| (source.to_owned(), destination.to_owned(), dependency))
        .collect::<Vec<_>>();
    let mut copied = HashSet::new();
    while let Some((source_owner, destination_owner, dependency)) = pending.pop() {
        let Some((location, checksum)) = materialized_location(dependency, &source_owner)? else {
            continue;
        };
        let source_root = source_owner
            .join(location)
            .canonicalize()
            .map_err(|source| PackageError::Io {
                path: source_owner.join(location),
                source,
            })?;
        let destination_root = destination_owner.join(location);
        if destination_root.starts_with(&source_root) {
            return Err(invalid(
                &destination_root,
                "dependency payload copy would enter its own source",
            ));
        }
        if !copied.insert((source_root.clone(), destination_root.clone())) {
            continue;
        }
        copy_payload(&source_root, &destination_root, checksum)?;
        pending.extend(
            dependency
                .dependencies
                .iter()
                .rev()
                .map(|child| (source_root.clone(), destination_root.clone(), child)),
        );
    }
    Ok(())
}

pub(crate) fn validate_materialized_payloads(root: &Path) -> Result<(), PackageError> {
    let Some(index) = super::read_package_index(root)? else {
        return Ok(());
    };
    let mut pending = index
        .dependencies
        .iter()
        .rev()
        .map(|dependency| (root.to_owned(), dependency))
        .collect::<Vec<_>>();
    while let Some((owner, dependency)) = pending.pop() {
        let Some((location, checksum)) = materialized_location(dependency, &owner)? else {
            continue;
        };
        let root = owner.join(location);
        if path_package_payload_checksum(&root)? != checksum {
            return Err(invalid(
                &root,
                "stored dependency payload checksum does not match its graph",
            ));
        }
        pending.extend(
            dependency
                .dependencies
                .iter()
                .rev()
                .map(|child| (root.clone(), child)),
        );
    }
    Ok(())
}

fn materialized_location<'a>(
    dependency: &'a ResolvedDependency,
    owner: &Path,
) -> Result<Option<(&'a Path, &'a str)>, PackageError> {
    let (location, checksum) = match &dependency.source {
        ResolvedDependencySource::Vendor { path, checksum, .. } => (path, checksum),
        ResolvedDependencySource::Registry {
            store: Some(store),
            checksum,
            ..
        }
        | ResolvedDependencySource::Git {
            store: Some(store),
            checksum,
            ..
        }
        | ResolvedDependencySource::GitHubClone {
            store: Some(store),
            checksum,
            ..
        } => (store, checksum),
        ResolvedDependencySource::GitHubRelease {
            store: Some(store),
            payload_checksum,
            ..
        } => (store, payload_checksum),
        // No materialized payload was advertised. Metadata is not source availability.
        _ => return Ok(None),
    };
    let location = Path::new(location);
    if location.as_os_str().is_empty()
        || location
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(
            owner,
            "materialized dependency location is not a package-relative path",
        ));
    }
    Ok(Some((location, checksum)))
}

fn copy_payload(source: &Path, destination: &Path, expected: &str) -> Result<(), PackageError> {
    if !destination.exists() {
        let manifest = manifest::read_manifest(source)?;
        copy_file(&source.join("etas.toml"), &destination.join("etas.toml"))?;
        copy_directory(
            &source.join(&manifest.source.root),
            &destination.join(&manifest.source.root),
        )?;
        for (source, destination) in [
            (package_index_path(source), package_index_path(destination)),
            (
                package_metadata_artifact_path(source),
                package_metadata_artifact_path(destination),
            ),
        ] {
            if source.is_file() {
                copy_file(&source, &destination)?;
            }
        }
    }
    if path_package_payload_checksum(destination)? != expected {
        return Err(invalid(
            destination,
            "materialized dependency payload checksum does not match its graph",
        ));
    }
    Ok(())
}

fn invalid(path: &Path, message: &str) -> PackageError {
    PackageError::Manifest {
        path: PathBuf::from(path),
        message: message.into(),
    }
}
