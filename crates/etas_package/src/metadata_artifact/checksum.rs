use std::path::Path;

use crate::{PackageError, manifest};

pub(super) fn blake3_hash(bytes: &[u8]) -> String {
    etas_package_metadata::blake3_hash(bytes)
}

pub(super) fn source_payload_checksum(package_root: &Path) -> Result<String, PackageError> {
    let manifest = manifest::read_manifest(package_root)?;
    etas_package_metadata::source_payload_checksum(
        package_root,
        &package_root.join(&manifest.source.root),
    )
    .map_err(package_error)
}

pub(super) fn file_checksum(path: &Path) -> Result<String, PackageError> {
    etas_package_metadata::file_checksum(path).map_err(package_error)
}

pub(super) fn optional_file_checksum(path: &Path) -> Result<String, PackageError> {
    etas_package_metadata::optional_file_checksum(path).map_err(package_error)
}

fn package_error(error: etas_package_metadata::MetadataArtifactError) -> PackageError {
    PackageError::Manifest {
        path: std::path::PathBuf::from(".etas/package.etasmeta"),
        message: error.to_string(),
    }
}
