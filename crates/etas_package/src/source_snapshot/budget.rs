use std::path::Path;

use super::{
    LockedPackageSourceSnapshot, SourceSnapshotError, SourceSnapshotLimits, SourceSnapshotResource,
    read_locked_package_sources,
};

/// Aggregate ownership budget for one compilation's locked input snapshots.
pub struct SourceSnapshotBudget {
    limits: SourceSnapshotLimits,
    entries: usize,
    bytes: usize,
}

impl Default for SourceSnapshotBudget {
    fn default() -> Self {
        Self::new(SourceSnapshotLimits::default())
    }
}

impl SourceSnapshotBudget {
    pub fn new(limits: SourceSnapshotLimits) -> Self {
        Self {
            limits,
            entries: 0,
            bytes: 0,
        }
    }

    pub fn read_locked<E>(
        &mut self,
        root: &Path,
        checksum: &str,
        mut checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<LockedPackageSourceSnapshot, SourceSnapshotError<E>> {
        checkpoint().map_err(SourceSnapshotError::Interrupted)?;
        let limits = SourceSnapshotLimits {
            max_entries: self.limits.max_entries.saturating_sub(self.entries),
            max_total_bytes: self.limits.max_total_bytes.saturating_sub(self.bytes),
            ..self.limits
        };
        if limits.max_entries == 0 {
            return Err(SourceSnapshotError::ResourceLimit {
                resource: SourceSnapshotResource::Entries,
                limit: self.limits.max_entries,
            });
        }
        if limits.max_total_bytes == 0 {
            return Err(SourceSnapshotError::ResourceLimit {
                resource: SourceSnapshotResource::TotalBytes,
                limit: self.limits.max_total_bytes,
            });
        }
        let snapshot = read_locked_package_sources(root, checksum, limits, checkpoint)?;
        self.entries += snapshot.entries;
        self.bytes += snapshot.payload_bytes;
        Ok(snapshot)
    }
}
