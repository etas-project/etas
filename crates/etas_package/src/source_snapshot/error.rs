use std::{fmt, path::PathBuf};

use crate::PackageError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceSnapshotResource {
    Entries,
    Depth,
    FileBytes,
    TotalBytes,
}

#[derive(Debug)]
pub enum SourceSnapshotError<E> {
    Package(PackageError),
    Interrupted(E),
    InvalidChecksum,
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    ResourceLimit {
        resource: SourceSnapshotResource,
        limit: usize,
    },
    InvalidEntry {
        path: PathBuf,
        reason: &'static str,
    },
}

impl<E> From<PackageError> for SourceSnapshotError<E> {
    fn from(error: PackageError) -> Self {
        Self::Package(error)
    }
}

impl<E: fmt::Display> fmt::Display for SourceSnapshotError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Package(error) => error.fmt(f),
            Self::Interrupted(error) => write!(f, "package source snapshot interrupted: {error}"),
            Self::InvalidChecksum => {
                f.write_str("locked package source checksum must be a canonical BLAKE3 digest")
            }
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "locked package source checksum mismatch: expected {expected}, read {actual}"
            ),
            Self::ResourceLimit { resource, limit } => write!(
                f,
                "package source snapshot exceeds {resource:?} limit {limit}"
            ),
            Self::InvalidEntry { path, reason } => write!(
                f,
                "invalid package source entry `{}`: {reason}",
                path.display()
            ),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SourceSnapshotError<E> {}
