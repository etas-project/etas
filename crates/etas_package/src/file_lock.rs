use std::{
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

#[cfg(unix)]
use std::os::unix::{fs::MetadataExt, io::AsRawFd};

use crate::PackageError;

const LOCK_RETRY_COUNT: usize = 500;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(10);

pub(crate) struct PackageFileLock {
    path: PathBuf,
    #[cfg(unix)]
    _file: File,
    #[cfg(not(unix))]
    owner: String,
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
        acquire_platform(path)
    }
}

#[cfg(unix)]
fn acquire_platform(path: PathBuf) -> Result<PackageFileLock, PackageError> {
    for _ in 0..LOCK_RETRY_COUNT {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| PackageError::Io {
                path: path.clone(),
                source,
            })?;
        if !try_advisory_lock(&file).map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })? {
            thread::sleep(LOCK_RETRY_DELAY);
            continue;
        }

        let file_metadata = file.metadata().map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })?;
        let path_metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(PackageError::Io {
                    path: path.clone(),
                    source,
                });
            }
        };
        if file_metadata.dev() != path_metadata.dev() || file_metadata.ino() != path_metadata.ino()
        {
            continue;
        }

        write_lock_identity(&mut file, &new_owner_token()?, &path)?;
        return Ok(PackageFileLock { path, _file: file });
    }
    lock_timeout(path)
}

#[cfg(unix)]
impl Drop for PackageFileLock {
    fn drop(&mut self) {
        let Ok(locked) = self._file.metadata() else {
            return;
        };
        let Ok(current) = fs::metadata(&self.path) else {
            return;
        };
        if locked.dev() == current.dev() && locked.ino() == current.ino() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(unix)]
fn try_advisory_lock(file: &File) -> std::io::Result<bool> {
    // SAFETY: flock only reads the valid descriptor owned by `file`.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    let raw = error.raw_os_error();
    if raw == Some(libc::EWOULDBLOCK) || raw == Some(libc::EAGAIN) {
        Ok(false)
    } else {
        Err(error)
    }
}

#[cfg(not(unix))]
fn acquire_platform(path: PathBuf) -> Result<PackageFileLock, PackageError> {
    let owner = new_owner_token()?;
    for _ in 0..LOCK_RETRY_COUNT {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                write_lock_identity(&mut file, &owner, &path)?;
                return Ok(PackageFileLock { path, owner });
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(LOCK_RETRY_DELAY);
            }
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(PackageError::Io {
                    path: path.clone(),
                    source,
                });
            }
        }
    }
    lock_timeout(path)
}

fn write_lock_identity(file: &mut File, owner: &str, path: &Path) -> Result<(), PackageError> {
    file.set_len(0).map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.seek(SeekFrom::Start(0))
        .map_err(|source| PackageError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    writeln!(file, "owner={owner}").map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(file, "pid={}", std::process::id()).map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(
        file,
        "created_unix_millis={}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default()
    )
    .map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_data().map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn new_owner_token() -> Result<String, PackageError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| PackageError::Manifest {
        path: PathBuf::from("<package-lock>"),
        message: format!("failed to generate package lock owner token: {error}"),
    })?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn lock_timeout(path: PathBuf) -> Result<PackageFileLock, PackageError> {
    Err(PackageError::Manifest {
        path,
        message: "timed out waiting for package file lock".to_owned(),
    })
}

#[cfg(not(unix))]
impl Drop for PackageFileLock {
    fn drop(&mut self) {
        let Ok(contents) = fs::read_to_string(&self.path) else {
            return;
        };
        if contents
            .lines()
            .any(|line| line == format!("owner={}", self.owner))
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, time::Instant};

    use super::*;

    #[test]
    fn advisory_lock_file_can_be_reacquired_after_owner_release() {
        let path = test_lock_path("reuse");
        let first = PackageFileLock::acquire(&path).unwrap();
        assert!(path.exists());
        drop(first);
        assert!(!path.exists());

        let second = PackageFileLock::acquire(&path).unwrap();
        drop(second);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn waiting_acquirer_observes_release_without_lockfile_toctou() {
        let path = test_lock_path("waiter");
        let first = PackageFileLock::acquire(&path).unwrap();
        let waiter_path = path.clone();
        let (sender, receiver) = mpsc::channel();
        let waiter = thread::spawn(move || {
            let started = Instant::now();
            let lock = PackageFileLock::acquire(&waiter_path).unwrap();
            sender.send(started.elapsed()).unwrap();
            lock
        });

        thread::sleep(Duration::from_millis(30));
        assert!(receiver.try_recv().is_err());
        drop(first);

        let elapsed = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(elapsed >= Duration::from_millis(20));
        drop(waiter.join().unwrap());
        let _ = fs::remove_file(path);
    }

    #[cfg(unix)]
    #[test]
    fn old_owner_does_not_remove_a_replacement_lockfile() {
        let path = test_lock_path("replacement");
        let first = PackageFileLock::acquire(&path).unwrap();
        fs::remove_file(&path).unwrap();
        let second = PackageFileLock::acquire(&path).unwrap();

        drop(first);
        assert!(path.exists());
        drop(second);
        assert!(!path.exists());
    }

    fn test_lock_path(name: &str) -> PathBuf {
        let owner = new_owner_token().unwrap();
        std::env::temp_dir().join(format!("etas-package-lock-{name}-{owner}.lock"))
    }
}
