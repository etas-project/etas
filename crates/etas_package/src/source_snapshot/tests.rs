use std::{
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::*;

struct Package(PathBuf);
impl Package {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-locked-sources-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            "[package]\nname = \"library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
        )
        .unwrap();
        fs::write(
            root.join("src/main.es"),
            "module library; public flow answer() -> i32 { return 7; }",
        )
        .unwrap();
        Self(root)
    }
    fn checksum(&self) -> String {
        crate::package_payload_checksum(&self.0).unwrap()
    }
    fn read(
        &self,
        checksum: &str,
        limits: SourceSnapshotLimits,
    ) -> Result<LockedPackageSourceSnapshot, SourceSnapshotError<Infallible>> {
        read_locked_package_sources(&self.0, checksum, limits, || Ok(()))
    }
}
impl Drop for Package {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn locked_source_snapshot_uses_the_existing_payload_checksum_and_immutable_bytes() {
    let package = Package::new();
    fs::create_dir(package.0.join(".etas")).unwrap();
    fs::write(package.0.join(".etas/package-index.json"), "metadata").unwrap();
    fs::write(package.0.join(".etas/package.etasmeta"), "binary artifact").unwrap();
    fs::write(package.0.join("src/notes.txt"), "non-source payload bytes").unwrap();
    let checksum = package.checksum();
    let snapshot = package.read(&checksum, Default::default()).unwrap();
    assert_eq!(snapshot.checksum(), checksum);
    assert_eq!(snapshot.files().count(), 5);
    assert_eq!(snapshot.manifest().package.name, "library");
    assert_eq!(
        snapshot.file(Path::new("src/notes.txt")),
        Some(b"non-source payload bytes".as_slice())
    );
    let old = snapshot.file(Path::new("src/main.es")).unwrap().to_owned();
    fs::write(package.0.join("src/main.es"), "changed after snapshot").unwrap();
    assert_eq!(
        snapshot.file(Path::new("src/main.es")),
        Some(old.as_slice())
    );
    assert!(matches!(
        package.read(&checksum, Default::default()),
        Err(SourceSnapshotError::ChecksumMismatch { .. })
    ));
}

#[test]
fn locked_source_snapshot_checks_the_read_bytes_not_an_earlier_filesystem_hash() {
    let package = Package::new();
    let checksum = package.checksum();
    let mut changed = false;
    let result = read_locked_package_sources(&package.0, &checksum, Default::default(), || {
        if !changed {
            fs::write(package.0.join("src/main.es"), "changed during request").unwrap();
            changed = true;
        }
        Ok::<_, Infallible>(())
    });
    assert!(matches!(
        result,
        Err(SourceSnapshotError::ChecksumMismatch { .. })
    ));
}

#[test]
fn locked_source_snapshot_rejects_noncanonical_or_forged_checksums() {
    let package = Package::new();
    for checksum in ["", "blake3:test", "blake3:ABCDEF", "sha256:123"] {
        assert!(matches!(
            package.read(checksum, Default::default()),
            Err(SourceSnapshotError::InvalidChecksum)
        ));
    }
    assert!(matches!(
        package.read(&format!("blake3:{}", "0".repeat(64)), Default::default()),
        Err(SourceSnapshotError::ChecksumMismatch { .. })
    ));
}

#[test]
fn locked_source_snapshot_preserves_relative_and_package_root_source_layouts() {
    for source_root in ["./src", "."] {
        let package = Package::new();
        let manifest = package.0.join("etas.toml");
        let source = fs::read_to_string(&manifest).unwrap();
        fs::write(
            &manifest,
            format!("{source}\n[source]\nroot = {source_root:?}\n"),
        )
        .unwrap();
        let checksum = package.checksum();
        let snapshot = package.read(&checksum, Default::default()).unwrap();
        assert!(snapshot.file(Path::new("src/main.es")).is_some());
        assert_eq!(snapshot.checksum(), checksum);
        assert_eq!(snapshot.files().count(), 2);
    }
}

#[test]
fn locked_source_snapshot_preserves_the_interrupt_identity() {
    let package = Package::new();
    #[derive(Debug, PartialEq, Eq)]
    struct Cancelled(u64);
    let mut calls = 0;
    let result =
        read_locked_package_sources(&package.0, &package.checksum(), Default::default(), || {
            calls += 1;
            if calls == 5 {
                Err(Cancelled(73))
            } else {
                Ok(())
            }
        });
    assert!(matches!(
        result,
        Err(SourceSnapshotError::Interrupted(Cancelled(73)))
    ));
}

#[test]
fn locked_source_snapshot_enforces_per_file_and_aggregate_byte_limits() {
    let package = Package::new();
    fs::write(package.0.join("src/large.es"), vec![b'x'; 16 * 1024]).unwrap();
    let checksum = package.checksum();
    let limits = SourceSnapshotLimits {
        max_file_bytes: 1024,
        ..Default::default()
    };
    assert!(matches!(
        package.read(&checksum, limits),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::FileBytes,
            limit: 1024
        })
    ));
    let limits = SourceSnapshotLimits {
        max_total_bytes: 1024,
        ..Default::default()
    };
    assert!(matches!(
        package.read(&checksum, limits),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::TotalBytes,
            limit: 1024
        })
    ));
}

#[test]
fn locked_source_snapshot_bounds_directory_width_and_depth_without_recursion() {
    let package = Package::new();
    fs::create_dir_all(package.0.join("src/a/b/c")).unwrap();
    fs::write(package.0.join("src/a/b/c/main.es"), "deep").unwrap();
    let checksum = package.checksum();
    let limits = SourceSnapshotLimits {
        max_depth: 3,
        ..Default::default()
    };
    assert!(matches!(
        package.read(&checksum, limits),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::Depth,
            limit: 3
        })
    ));
    let limits = SourceSnapshotLimits {
        max_entries: 3,
        ..Default::default()
    };
    assert!(matches!(
        package.read(&checksum, limits),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::Entries,
            limit: 3
        })
    ));
}

#[test]
fn locked_source_snapshot_accepts_exact_entry_and_byte_budgets() {
    let package = Package::new();
    let total = fs::read(package.0.join("etas.toml")).unwrap().len()
        + fs::read(package.0.join("src/main.es")).unwrap().len();
    let limits = SourceSnapshotLimits {
        max_entries: 3,
        max_depth: 2,
        max_total_bytes: total,
        ..Default::default()
    };
    assert_eq!(
        package
            .read(&package.checksum(), limits)
            .unwrap()
            .files()
            .count(),
        2
    );
}

#[test]
fn locked_source_snapshot_budget_is_shared_across_packages() {
    let first = Package::new();
    let second = Package::new();
    let first_snapshot = first.read(&first.checksum(), Default::default()).unwrap();
    let bytes = first_snapshot
        .files()
        .map(|(_, bytes)| bytes.len())
        .sum::<usize>();
    let mut budget = SourceSnapshotBudget::new(SourceSnapshotLimits {
        max_total_bytes: bytes * 2 - 1,
        ..Default::default()
    });
    budget
        .read_locked(&first.0, &first.checksum(), || Ok::<_, Infallible>(()))
        .unwrap();
    assert!(matches!(
        budget.read_locked(&second.0, &second.checksum(), || Ok::<_, Infallible>(())),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::TotalBytes,
            ..
        })
    ));

    let mut budget = SourceSnapshotBudget::new(SourceSnapshotLimits {
        max_entries: 6,
        ..Default::default()
    });
    budget
        .read_locked(&first.0, &first.checksum(), || Ok::<_, Infallible>(()))
        .unwrap();
    budget
        .read_locked(&second.0, &second.checksum(), || Ok::<_, Infallible>(()))
        .unwrap();
    assert!(matches!(
        budget.read_locked(&first.0, &first.checksum(), || Ok::<_, Infallible>(())),
        Err(SourceSnapshotError::ResourceLimit {
            resource: SourceSnapshotResource::Entries,
            limit: 6
        })
    ));
    assert!(matches!(
        budget.read_locked(&first.0, &first.checksum(), || Err::<(), _>(73)),
        Err(SourceSnapshotError::Interrupted(73))
    ));
}

#[test]
fn failed_locked_source_reads_do_not_retain_snapshot_quota() {
    let package = Package::new();
    let mut budget = SourceSnapshotBudget::new(SourceSnapshotLimits {
        max_entries: 3,
        ..Default::default()
    });
    assert!(matches!(
        budget.read_locked(&package.0, &format!("blake3:{}", "0".repeat(64)), || Ok::<
            _,
            Infallible,
        >(
            ()
        )),
        Err(SourceSnapshotError::ChecksumMismatch { .. })
    ));
    assert!(
        budget
            .read_locked(&package.0, &package.checksum(), || Ok::<_, Infallible>(()))
            .is_ok()
    );
}

#[test]
fn locked_source_snapshot_rejects_package_escaping_source_roots() {
    let package = Package::new();
    let manifest = package.0.join("etas.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        format!("{text}\n[source]\nroot = \"../outside\"\n"),
    )
    .unwrap();
    assert!(matches!(
        package.read(&format!("blake3:{}", "0".repeat(64)), Default::default()),
        Err(SourceSnapshotError::InvalidEntry { .. })
    ));
}

#[cfg(unix)]
#[test]
fn locked_source_snapshot_rejects_source_and_metadata_symlinks() {
    use std::os::unix::fs::symlink;
    let package = Package::new();
    let checksum = package.checksum();
    symlink(package.0.join("etas.toml"), package.0.join("src/alias.es")).unwrap();
    assert!(matches!(
        package.read(&checksum, Default::default()),
        Err(SourceSnapshotError::InvalidEntry { .. })
    ));
    fs::remove_file(package.0.join("src/alias.es")).unwrap();
    symlink(package.0.join("src"), package.0.join(".etas")).unwrap();
    fs::write(package.0.join("src/package-index.json"), "outside metadata").unwrap();
    assert!(matches!(
        package.read(&checksum, Default::default()),
        Err(SourceSnapshotError::InvalidEntry { .. })
    ));
}
