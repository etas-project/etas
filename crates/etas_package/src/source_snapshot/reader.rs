use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use super::{
    LockedPackageSourceSnapshot, SourceSnapshotError as Error, SourceSnapshotLimits,
    SourceSnapshotResource as Resource, hash_payload_file,
};
use crate::{Manifest, PackageError, vendor};

pub fn read_locked_package_sources<E>(
    package_root: &Path,
    expected_checksum: &str,
    limits: SourceSnapshotLimits,
    mut checkpoint: impl FnMut() -> Result<(), E>,
) -> Result<LockedPackageSourceSnapshot, Error<E>> {
    checkpoint().map_err(Error::Interrupted)?;
    let Some(digest) = expected_checksum.strip_prefix("blake3:") else {
        return Err(Error::InvalidChecksum);
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::InvalidChecksum);
    }
    let package_root = package_root
        .canonicalize()
        .map_err(|source| io(package_root, source))?;
    let mut reader = Reader {
        root: &package_root,
        limits,
        checkpoint: &mut checkpoint,
        entries: 0,
        total: 0,
        files: BTreeMap::new(),
        checksum_paths: Vec::new(),
    };
    let manifest_path = Path::new("etas.toml");
    reader.read_file(manifest_path)?;
    let text =
        std::str::from_utf8(&reader.files[manifest_path]).map_err(|_| Error::InvalidEntry {
            path: package_root.join(manifest_path),
            reason: "manifest is not UTF-8",
        })?;
    let manifest: Manifest = toml::from_str(text).map_err(|error| PackageError::Manifest {
        path: package_root.join(manifest_path),
        message: error.to_string(),
    })?;
    manifest.validate(&package_root.join(manifest_path))?;
    let mut source_root = PathBuf::new();
    for part in manifest.source.root.components() {
        match part {
            Component::Normal(part) => source_root.push(part),
            Component::CurDir => {}
            _ => {
                return Err(Error::InvalidEntry {
                    path: manifest.source.root.clone(),
                    reason: "source root must remain within its package",
                });
            }
        }
    }
    for path in [
        vendor::package_index_path(&package_root),
        vendor::package_metadata_artifact_path(&package_root),
    ] {
        let relative = path
            .strip_prefix(&package_root)
            .map_err(|_| Error::InvalidEntry {
                path: path.clone(),
                reason: "payload metadata is outside its package",
            })?;
        match fs::symlink_metadata(&path) {
            Ok(_) => reader.read_file(relative)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io(&path, error).into()),
        }
    }
    reader.read_sources(&source_root)?;
    reader.checksum_paths.sort();
    let mut hasher = blake3::Hasher::new();
    for path in &reader.checksum_paths {
        (reader.checkpoint)().map_err(Error::Interrupted)?;
        hash_payload_file(&mut hasher, path, &reader.files[path]);
    }
    let actual = format!("blake3:{}", hasher.finalize().to_hex());
    if actual != expected_checksum {
        return Err(Error::ChecksumMismatch {
            expected: expected_checksum.to_owned(),
            actual,
        });
    }
    (reader.checkpoint)().map_err(Error::Interrupted)?;
    Ok(LockedPackageSourceSnapshot {
        manifest,
        source_root,
        checksum: actual,
        entries: reader.entries,
        payload_bytes: reader.total,
        files: reader.files,
        package_root,
    })
}

struct Reader<'a, E, F: FnMut() -> Result<(), E>> {
    root: &'a Path,
    limits: SourceSnapshotLimits,
    checkpoint: &'a mut F,
    entries: usize,
    total: usize,
    files: BTreeMap<PathBuf, Arc<[u8]>>,
    checksum_paths: Vec<PathBuf>,
}

impl<E, F: FnMut() -> Result<(), E>> Reader<'_, E, F> {
    fn visit(&mut self, path: &Path) -> Result<(), Error<E>> {
        (self.checkpoint)().map_err(Error::Interrupted)?;
        self.entries = self.entries.checked_add(1).ok_or(Error::ResourceLimit {
            resource: Resource::Entries,
            limit: self.limits.max_entries,
        })?;
        if self.entries > self.limits.max_entries {
            return Err(Error::ResourceLimit {
                resource: Resource::Entries,
                limit: self.limits.max_entries,
            });
        }
        if path.components().count() > self.limits.max_depth {
            return Err(Error::ResourceLimit {
                resource: Resource::Depth,
                limit: self.limits.max_depth,
            });
        }
        Ok(())
    }

    fn checked_metadata(&self, relative: &Path) -> Result<fs::Metadata, Error<E>> {
        let mut path = self.root.to_owned();
        let mut metadata = fs::metadata(self.root).map_err(|source| io(self.root, source))?;
        for part in relative.components() {
            let Component::Normal(part) = part else {
                return Err(Error::InvalidEntry {
                    path: relative.to_owned(),
                    reason: "payload path is not package-relative",
                });
            };
            path.push(part);
            let current = fs::symlink_metadata(&path).map_err(|source| io(&path, source))?;
            if current.file_type().is_symlink() {
                return Err(Error::InvalidEntry {
                    path,
                    reason: "symlinks are not accepted in locked source payloads",
                });
            }
            metadata = current;
        }
        Ok(metadata)
    }

    fn read_sources(&mut self, source_root: &Path) -> Result<(), Error<E>> {
        let mut pending = vec![source_root.to_owned()];
        while let Some(relative) = pending.pop() {
            self.visit(&relative)?;
            let metadata = self.checked_metadata(&relative)?;
            if !metadata.is_dir() {
                return Err(Error::InvalidEntry {
                    path: self.root.join(&relative),
                    reason: "source root is not a directory",
                });
            }
            let path = self.root.join(&relative);
            for entry in fs::read_dir(&path).map_err(|source| io(&path, source))? {
                (self.checkpoint)().map_err(Error::Interrupted)?;
                let entry = entry.map_err(|source| io(&path, source))?;
                let child = relative.join(entry.file_name());
                let metadata = self.checked_metadata(&child)?;
                if metadata.is_dir() {
                    if self.entries.saturating_add(pending.len()).saturating_add(1)
                        > self.limits.max_entries
                    {
                        return Err(Error::ResourceLimit {
                            resource: Resource::Entries,
                            limit: self.limits.max_entries,
                        });
                    }
                    pending.push(child);
                } else if metadata.is_file() {
                    self.read_file(&child)?;
                } else {
                    return Err(Error::InvalidEntry {
                        path: self.root.join(child),
                        reason: "payload must contain only regular files and directories",
                    });
                }
            }
        }
        Ok(())
    }

    fn read_file(&mut self, relative: &Path) -> Result<(), Error<E>> {
        self.visit(relative)?;
        let metadata = self.checked_metadata(relative)?;
        if !metadata.is_file() {
            return Err(Error::InvalidEntry {
                path: self.root.join(relative),
                reason: "payload entry is not a regular file",
            });
        }
        if self.files.contains_key(relative) {
            // A package-root source tree also contains its manifest/metadata.
            // Preserve the existing payload hash multiplicity, sharing bytes.
            self.checksum_paths.push(relative.to_owned());
            return Ok(());
        }
        self.check_size(metadata.len())?;
        let path = self.root.join(relative);
        let mut file = open_regular(&path).map_err(|source| io(&path, source))?;
        if !file
            .metadata()
            .map_err(|source| io(&path, source))?
            .is_file()
        {
            return Err(Error::InvalidEntry {
                path,
                reason: "opened payload entry is not a regular file",
            });
        }
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 64 * 1024];
        loop {
            (self.checkpoint)().map_err(Error::Interrupted)?;
            let count = file.read(&mut chunk).map_err(|source| io(&path, source))?;
            if count == 0 {
                break;
            }
            let size = bytes.len().checked_add(count).ok_or(Error::ResourceLimit {
                resource: Resource::FileBytes,
                limit: self.limits.max_file_bytes,
            })?;
            self.check_size(size as u64)?;
            bytes.extend_from_slice(&chunk[..count]);
        }
        self.total += bytes.len();
        self.files.insert(relative.to_owned(), Arc::from(bytes));
        self.checksum_paths.push(relative.to_owned());
        Ok(())
    }

    fn check_size(&self, size: u64) -> Result<(), Error<E>> {
        if size > self.limits.max_file_bytes as u64 {
            return Err(Error::ResourceLimit {
                resource: Resource::FileBytes,
                limit: self.limits.max_file_bytes,
            });
        }
        if size > self.limits.max_total_bytes.saturating_sub(self.total) as u64 {
            return Err(Error::ResourceLimit {
                resource: Resource::TotalBytes,
                limit: self.limits.max_total_bytes,
            });
        }
        Ok(())
    }
}

fn io(path: &Path, source: std::io::Error) -> PackageError {
    PackageError::Io {
        path: path.to_owned(),
        source,
    }
}

fn open_regular(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    options.open(path)
}
