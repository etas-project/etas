use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{PackageError, file_lock::PackageFileLock, vendor};

pub fn package_store_root(package_root: &Path) -> PathBuf {
    package_root.join(".etas").join("store").join("packages")
}

pub fn package_store_relative_path(checksum: &str) -> Result<String, PackageError> {
    let Some(digest) = checksum.strip_prefix("blake3:") else {
        return Err(PackageError::Manifest {
            path: PathBuf::from(".etas/store/packages"),
            message: format!("package store only accepts blake3 checksums, got `{checksum}`"),
        });
    };
    if digest.is_empty()
        || !digest
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(PackageError::Manifest {
            path: PathBuf::from(".etas/store/packages"),
            message: format!("package store checksum digest is invalid: `{checksum}`"),
        });
    }
    Ok(format!(".etas/store/packages/blake3/{digest}"))
}

pub fn store_package_payload(
    package_root: &Path,
    payload_root: &Path,
    checksum: &str,
) -> Result<String, PackageError> {
    let relative = package_store_relative_path(checksum)?;
    let destination = package_root.join(&relative);
    let store_root = package_store_root(package_root);
    fs::create_dir_all(&store_root).map_err(|source| PackageError::Io {
        path: store_root.clone(),
        source,
    })?;
    let _lock = PackageFileLock::acquire(store_root.join(".lock"))?;
    if destination.exists() {
        let actual = vendor::path_package_payload_checksum(&destination)?;
        if actual == checksum {
            return Ok(relative);
        }
        return Err(PackageError::Manifest {
            path: destination,
            message: format!(
                "package store entry for `{checksum}` is corrupted: existing payload checksum is `{actual}`"
            ),
        });
    }
    let Some(parent) = destination.parent() else {
        return Err(PackageError::Manifest {
            path: destination,
            message: "package store destination has no parent".to_owned(),
        });
    };
    fs::create_dir_all(parent).map_err(|source| PackageError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temp = unique_temp_path(parent, "store-write");
    if temp.exists() {
        fs::remove_dir_all(&temp).map_err(|source| PackageError::Io {
            path: temp.clone(),
            source,
        })?;
    }
    copy_directory(payload_root, &temp)?;
    let actual = vendor::path_package_payload_checksum(&temp)?;
    if actual != checksum {
        let _ = fs::remove_dir_all(&temp);
        return Err(PackageError::Manifest {
            path: temp,
            message: format!(
                "copied package store payload checksum `{actual}` does not match expected `{checksum}`"
            ),
        });
    }
    fs::rename(&temp, &destination).map_err(|source| PackageError::Io {
        path: destination.clone(),
        source,
    })?;
    Ok(relative)
}

fn unique_temp_path(parent: &Path, prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    parent.join(format!(".{prefix}-{}-{stamp}", std::process::id()))
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), PackageError> {
    fs::create_dir_all(destination).map_err(|source_error| PackageError::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    let entries = fs::read_dir(source).map_err(|source_error| PackageError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source_error| PackageError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let path = entry.path();
        let target = destination.join(entry.file_name());
        let metadata = entry.metadata().map_err(|source_error| PackageError::Io {
            path: path.clone(),
            source: source_error,
        })?;
        if metadata.is_dir() {
            copy_directory(&path, &target)?;
        } else if metadata.is_file() {
            copy_file(&path, &target)?;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), PackageError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source| PackageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(source, destination).map_err(|source_error| PackageError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn store_package_payload_reuses_existing_digest_under_concurrent_writes() {
        let root = temp_dir("store-concurrent-root");
        let payload = temp_dir("store-concurrent-payload");
        write_payload_package(&payload, "payload");
        let checksum = vendor::path_package_payload_checksum(&payload).unwrap();

        let mut handles = Vec::new();
        for _ in 0..8 {
            let root = root.clone();
            let payload = payload.clone();
            let checksum = checksum.clone();
            handles.push(thread::spawn(move || {
                store_package_payload(&root, &payload, &checksum)
            }));
        }

        let mut paths = Vec::new();
        for handle in handles {
            paths.push(handle.join().unwrap().unwrap());
        }
        assert!(paths.iter().all(|path| path == &paths[0]));
        let stored = root.join(&paths[0]);
        assert_eq!(
            vendor::path_package_payload_checksum(&stored).unwrap(),
            checksum
        );
    }

    fn write_payload_package(root: &Path, module: &str) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "{module}"
version = "0.1.0"
edition = "2026"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("src").join("lib.es"),
            format!("module {module};\n"),
        )
        .unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("etas-package-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
