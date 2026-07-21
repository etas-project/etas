use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{PackageError, file_lock::PackageFileLock, manifest, metadata::PackageIndex};

use super::{
    checksum::{blake3_hash, file_checksum, optional_file_checksum, source_payload_checksum},
    convert::package_index_from_metadata,
    header::{
        ARTIFACT_SCHEMA_VERSION, CREATED_TARGET, MetadataArtifactHeader, MetadataArtifactInfo,
    },
};

pub fn package_metadata_artifact_path(package_root: &Path) -> PathBuf {
    etas_package_metadata::package_metadata_artifact_path(package_root)
}

pub fn package_source_payload_hash(package_root: &Path) -> Result<String, PackageError> {
    source_payload_checksum(package_root)
}

pub fn package_manifest_hash(package_root: &Path) -> Result<String, PackageError> {
    file_checksum(&package_root.join("etas.toml"))
}

pub fn package_dependency_lock_hash(package_root: &Path) -> Result<String, PackageError> {
    optional_file_checksum(&package_root.join("etas.lock"))
}

pub fn read_package_metadata_artifact(
    package_root: &Path,
) -> Result<Option<PackageIndex>, PackageError> {
    let path = package_metadata_artifact_path(package_root);
    if !path.exists() {
        return Ok(None);
    }
    let _lock = PackageFileLock::acquire(package_root.join(".etas").join("package.etasmeta.lock"))?;
    let bytes = fs::read(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    decode_artifact(package_root, &path, &bytes).map(Some)
}

pub fn read_package_metadata_artifact_info(
    package_root: &Path,
) -> Result<Option<MetadataArtifactInfo>, PackageError> {
    let path = package_metadata_artifact_path(package_root);
    if !path.exists() {
        return Ok(None);
    }
    let _lock = PackageFileLock::acquire(package_root.join(".etas").join("package.etasmeta.lock"))?;
    let bytes = fs::read(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    let decoded =
        etas_package_metadata::decode_metadata_artifact(&path, &bytes).map_err(package_error)?;
    verify_header(package_root, &path, &decoded.header)?;
    Ok(Some(MetadataArtifactInfo {
        header: decoded.header,
        artifact_hash: blake3_hash(&bytes),
    }))
}

pub fn metadata_artifact_hash(package_root: &Path) -> Result<Option<String>, PackageError> {
    let path = package_metadata_artifact_path(package_root);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|source| PackageError::Io { path, source })?;
    Ok(Some(blake3_hash(&bytes)))
}

fn decode_artifact(
    package_root: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<PackageIndex, PackageError> {
    let (header, metadata) = etas_package_metadata::package_metadata_from_artifact(path, bytes)
        .map_err(package_error)?;
    verify_header(package_root, path, &header)?;
    package_index_from_metadata(metadata, path)
}

fn verify_header(
    package_root: &Path,
    path: &Path,
    header: &MetadataArtifactHeader,
) -> Result<(), PackageError> {
    if header.artifact_schema_version != ARTIFACT_SCHEMA_VERSION {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact schema version `{}` is not supported; expected `{ARTIFACT_SCHEMA_VERSION}`",
                header.artifact_schema_version
            ),
        });
    }
    if header.created_target != CREATED_TARGET {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact was created by `{}`, expected `{CREATED_TARGET}`",
                header.created_target
            ),
        });
    }

    let manifest = manifest::read_manifest(package_root)?;
    if header.package_id != manifest.package.name {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact declares package `{}`, but manifest declares `{}`",
                header.package_id, manifest.package.name
            ),
        });
    }
    if header.package_version != manifest.package.version {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact declares version `{}`, but manifest declares `{}`",
                header.package_version, manifest.package.version
            ),
        });
    }

    let source_hash = source_payload_checksum(package_root)?;
    if header.source_payload_hash != source_hash {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact source hash `{}` is stale; current source hash is `{source_hash}`",
                header.source_payload_hash
            ),
        });
    }
    let manifest_hash = file_checksum(&package_root.join("etas.toml"))?;
    if header.manifest_hash != manifest_hash {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact manifest hash `{}` is stale; current manifest hash is `{manifest_hash}`",
                header.manifest_hash
            ),
        });
    }
    let lock_hash = optional_file_checksum(&package_root.join("etas.lock"))?;
    if header.dependency_lock_hash != lock_hash {
        return Err(PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "package metadata artifact dependency lock hash `{}` is stale; current lock hash is `{lock_hash}`",
                header.dependency_lock_hash
            ),
        });
    }
    Ok(())
}

fn package_error(error: etas_package_metadata::MetadataArtifactError) -> PackageError {
    PackageError::Manifest {
        path: PathBuf::from(".etas/package.etasmeta"),
        message: error.to_string(),
    }
}
