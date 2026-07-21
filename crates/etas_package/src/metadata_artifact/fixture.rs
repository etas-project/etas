#![cfg(any(test, feature = "test-support"))]

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use etas_package_metadata::EncodedMetadataSection;

use crate::{PackageError, file_lock::PackageFileLock, metadata::PackageIndex};

use super::{
    checksum::{blake3_hash, file_checksum, optional_file_checksum, source_payload_checksum},
    container::package_metadata_artifact_path,
    encode::package_index_to_metadata,
    header::{
        ARTIFACT_SCHEMA_VERSION, COMPILER_VERSION, CREATED_TARGET, MetadataArtifactHeader,
        MetadataArtifactInfo,
    },
};

pub fn write_fixture_package_metadata_artifact(
    package_root: &Path,
    index: &PackageIndex,
) -> Result<MetadataArtifactInfo, PackageError> {
    let path = package_metadata_artifact_path(package_root);
    let lock_path = package_root.join(".etas").join("package.etasmeta.lock");
    let _lock = PackageFileLock::acquire(lock_path)?;
    let parent = path.parent().ok_or_else(|| PackageError::Manifest {
        path: path.clone(),
        message: "package metadata artifact path has no parent".to_owned(),
    })?;

    let header = MetadataArtifactHeader {
        artifact_schema_version: ARTIFACT_SCHEMA_VERSION,
        compiler_version: COMPILER_VERSION.to_owned(),
        package_id: index.package.name.clone(),
        package_version: index.package.version.clone(),
        source_payload_hash: source_payload_checksum(package_root)?,
        manifest_hash: file_checksum(&package_root.join("etas.toml"))?,
        dependency_lock_hash: optional_file_checksum(&package_root.join("etas.lock"))?,
        created_target: CREATED_TARGET.to_owned(),
    };
    let bytes = encode_artifact(&header, index)?;
    etas_package_metadata::write_metadata_artifact_file(&path, &bytes).map_err(package_error)?;
    sync_directory(parent)?;
    Ok(MetadataArtifactInfo {
        header,
        artifact_hash: blake3_hash(&bytes),
    })
}

fn encode_artifact(
    header: &MetadataArtifactHeader,
    index: &PackageIndex,
) -> Result<Vec<u8>, PackageError> {
    let sections = build_sections(index)?;
    etas_package_metadata::encode_metadata_artifact(header, sections).map_err(package_error)
}

fn build_sections(index: &PackageIndex) -> Result<Vec<EncodedMetadataSection>, PackageError> {
    let metadata = package_index_to_metadata(index)?;
    Ok(etas_package_metadata::package_metadata_to_sections(
        &metadata,
    ))
}

#[cfg(any(test, feature = "test-support"))]
fn sync_directory(path: &Path) -> Result<(), PackageError> {
    let directory = File::open(path).map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    directory.sync_all().map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn package_error(error: etas_package_metadata::MetadataArtifactError) -> PackageError {
    PackageError::Manifest {
        path: PathBuf::from(".etas/package.etasmeta"),
        message: error.to_string(),
    }
}
