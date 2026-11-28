use std::path::Path;

use crate::DriverError;

static RUNNING_BUILD: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

/// Register the executable's compile-time stamp before loading an installed SDK.
/// Embedders without a build stamp retain the executable-digest receipt format.
pub fn register_toolchain_build(build: &'static str) {
    RUNNING_BUILD.get_or_init(|| build);
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{fs::File, io, os::unix::fs::MetadataExt, path::Path};

    use rustix::{fs::XattrFlags, io::Errno};

    use crate::DriverError;

    const EXECUTABLE: &str = "/proc/self/exe";
    const RECEIPT: &str = "user.etas-std-install";

    fn receipt(executable: &File, manifest_identity: &str) -> io::Result<String> {
        if let Some(build) = super::RUNNING_BUILD.get() {
            return Ok(format!(
                "etas-std-install-v2\n{build}\n{manifest_identity}\n"
            ));
        }
        let digest = blake3::Hasher::new().update_reader(executable)?.finalize();
        Ok(format!(
            "etas-std-install-v1\n{digest}\n{manifest_identity}\n"
        ))
    }

    fn read_receipt(manifest_identity: &str) -> Option<bool> {
        // The attribute belongs to the mapped toolchain binary. Neither a
        // selected sysroot nor a replacement directory can supply its value.
        let executable = File::open(EXECUTABLE).ok()?;
        if executable.metadata().ok()?.mode() & 0o022 != 0 {
            return None;
        }
        let mut data = [0_u8; 256];
        let size = rustix::fs::fgetxattr(&executable, RECEIPT, &mut data[..]).ok()?;
        let expected = receipt(&executable, manifest_identity).ok()?;
        Some(&data[..size] == expected.as_bytes())
    }

    pub(super) fn installed_artifact_is_trusted(manifest_identity: &str) -> bool {
        read_receipt(manifest_identity).unwrap_or(false)
    }

    pub(super) fn publish_install_receipt(
        executable: &Path,
        manifest_identity: &str,
    ) -> Result<(), DriverError> {
        let result = (|| -> io::Result<()> {
            let binary = File::open(executable)?;
            let recorded = receipt(&binary, manifest_identity)?;
            match rustix::fs::fsetxattr(&binary, RECEIPT, recorded.as_bytes(), XattrFlags::empty())
            {
                Ok(()) | Err(Errno::NOTSUP) => Ok(()),
                Err(error) => Err(error.into()),
            }
        })();
        result.map_err(|source| DriverError::Io {
            path: executable.to_owned(),
            source,
        })
    }
}

pub(crate) fn installed_artifact_is_trusted(_manifest_identity: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        linux::installed_artifact_is_trusted(_manifest_identity)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Bind an installed SDK to the staged toolchain executable that prepared it.
pub fn publish_install_receipt(
    _executable: &Path,
    _manifest_identity: &str,
) -> Result<(), DriverError> {
    #[cfg(target_os = "linux")]
    {
        linux::publish_install_receipt(_executable, _manifest_identity)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(())
    }
}

pub(crate) fn run_cache_directory(
    access: etas_frontend::DiskCacheAccess,
) -> Option<(std::path::PathBuf, &'static str)> {
    #[cfg(target_os = "linux")]
    {
        use std::{
            fs,
            os::unix::fs::{DirBuilderExt, MetadataExt},
            path::PathBuf,
        };
        let build = *RUNNING_BUILD.get()?;
        if access == etas_frontend::DiskCacheAccess::Disabled {
            return None;
        }
        let root = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache")))?;
        let directory = root.join("etas/checked-runs");
        if access.can_write() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(&directory)
                .ok()?;
        }
        // This directory contains locally produced executable plans, unlike the
        // workspace cache which can be supplied by a checked-out project.
        let metadata = fs::symlink_metadata(&directory).ok()?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
        {
            return None;
        }
        Some((directory, build))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = access;
        None
    }
}
