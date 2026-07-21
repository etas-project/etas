use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

use crate::PackageError;

const LOCK_RETRY_COUNT: usize = 500;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(10);
const STALE_LOCK_AFTER: Duration = Duration::from_secs(10 * 60);

pub(crate) struct PackageFileLock {
    path: PathBuf,
}

impl PackageFileLock {
    pub(crate) fn acquire(path: impl Into<PathBuf>) -> Result<Self, PackageError> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| PackageError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        for _ in 0..LOCK_RETRY_COUNT {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let _ = writeln!(file, "pid={}", std::process::id());
                    let _ = writeln!(
                        file,
                        "created_unix_millis={}",
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|duration| duration.as_millis())
                            .unwrap_or_default()
                    );
                    let _ = file.sync_all();
                    return Ok(Self { path });
                }
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                    if stale_lock(&path)? {
                        let _ = fs::remove_file(&path);
                    } else {
                        thread::sleep(LOCK_RETRY_DELAY);
                    }
                }
                Err(source) => {
                    return Err(PackageError::Io {
                        path: path.clone(),
                        source,
                    });
                }
            }
        }
        Err(PackageError::Manifest {
            path,
            message: "timed out waiting for package file lock".to_owned(),
        })
    }
}

impl Drop for PackageFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn stale_lock(path: &Path) -> Result<bool, PackageError> {
    let metadata = fs::metadata(path).map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let Ok(modified) = metadata.modified() else {
        return Ok(false);
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return Ok(false);
    };
    Ok(age > STALE_LOCK_AFTER)
}
