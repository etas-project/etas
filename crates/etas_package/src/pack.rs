use std::path::PathBuf;

use crate::{PackageError, source};

#[derive(Clone, Debug)]
pub struct PackPackageOptions {
    pub package_root: PathBuf,
    pub output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackedPackageResult {
    pub package_root: PathBuf,
    pub output: PathBuf,
    pub checksum: String,
}

pub fn pack_package(options: PackPackageOptions) -> Result<PackedPackageResult, PackageError> {
    let package_root = options
        .package_root
        .canonicalize()
        .map_err(|source| PackageError::Io {
            path: options.package_root.clone(),
            source,
        })?;
    let checksum = source::pack_etaspkg(&package_root, &options.output)?;
    Ok(PackedPackageResult {
        package_root,
        output: options.output,
        checksum,
    })
}
