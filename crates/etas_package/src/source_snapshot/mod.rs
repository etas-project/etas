use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Manifest;

mod budget;
mod error;
mod reader;
#[cfg(test)]
mod tests;

pub use budget::SourceSnapshotBudget;
pub use error::{SourceSnapshotError, SourceSnapshotResource};
pub use reader::read_locked_package_sources;

#[derive(Clone, Copy, Debug)]
pub struct SourceSnapshotLimits {
    pub max_entries: usize,
    pub max_depth: usize,
    pub max_file_bytes: usize,
    pub max_total_bytes: usize,
}

impl Default for SourceSnapshotLimits {
    fn default() -> Self {
        Self {
            max_entries: 16_384,
            max_depth: 64,
            max_file_bytes: 16 * 1024 * 1024,
            max_total_bytes: 128 * 1024 * 1024,
        }
    }
}

/// Bytes verified against the locked payload, not a lease on mutable paths.
pub struct LockedPackageSourceSnapshot {
    package_root: PathBuf,
    manifest: Manifest,
    source_root: PathBuf,
    checksum: String,
    files: BTreeMap<PathBuf, Arc<[u8]>>,
    entries: usize,
    payload_bytes: usize,
}

impl std::fmt::Debug for LockedPackageSourceSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockedPackageSourceSnapshot")
            .field("package_root", &self.package_root)
            .field("checksum", &self.checksum)
            .field("files", &self.files.len())
            .finish()
    }
}

impl LockedPackageSourceSnapshot {
    pub fn package_root(&self) -> &Path {
        &self.package_root
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }
    pub fn checksum(&self) -> &str {
        &self.checksum
    }
    pub fn file(&self, relative: &Path) -> Option<&[u8]> {
        self.files.get(relative).map(AsRef::as_ref)
    }
    pub fn files(&self) -> impl Iterator<Item = (&Path, &[u8])> {
        self.files
            .iter()
            .map(|(path, bytes)| (path.as_path(), bytes.as_ref()))
    }
}

pub(crate) fn hash_payload_file(hasher: &mut blake3::Hasher, relative: &Path, bytes: &[u8]) {
    hasher.update(relative.to_string_lossy().as_bytes());
    hasher.update(&[0]);
    hasher.update(bytes);
    hasher.update(&[0]);
}
