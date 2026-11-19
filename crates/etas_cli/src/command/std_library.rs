use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use etas_driver::{StdLibraryBuildRequest, build_standard_library};
use etas_utils::ProfileHandle;

use crate::{
    args::{
        global::GlobalOptions,
        std_library::{StdArgs, StdCommand},
    },
    error::CliError,
    exit::CliExit,
};

pub fn run(
    global: &GlobalOptions,
    args: StdArgs,
    profile: &ProfileHandle,
    stdout: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let (request, executable) = match args.command {
        StdCommand::Build(args) => (
            StdLibraryBuildRequest {
                manifest_path: args.manifest_path,
                out_dir: args.out_dir,
            },
            None,
        ),
        StdCommand::Install(args) => (
            StdLibraryBuildRequest {
                manifest_path: args.manifest_path.unwrap_or_else(|| {
                    etas_driver::std_library::source_checkout_manifest().to_path_buf()
                }),
                out_dir: etas_driver::std_library::sdk::installed_sysroot(&args.prefix).join("std"),
            },
            StagedExecutable::prepare(&args.prefix)?,
        ),
    };
    let output = build_standard_library(
        &super::frontend::base_driver_options(global, profile),
        request,
    )
    .map_err(super::frontend::driver_error)?;
    if let Some(executable) = executable {
        executable.publish()?;
    }
    writeln!(
        stdout,
        "installed std {} at {} ({})",
        output.manifest.version,
        output.out_dir.display(),
        output.manifest.artifact_hash
    )
    .map_err(|source| CliError::Io {
        path: "<stdout>".into(),
        source,
    })?;
    Ok(CliExit::Success)
}

struct StagedExecutable {
    temporary: PathBuf,
    destination: PathBuf,
}

impl StagedExecutable {
    fn prepare(prefix: &Path) -> Result<Option<Self>, CliError> {
        let bin = prefix.join("bin");
        fs::create_dir_all(&bin).map_err(|source| CliError::Io {
            path: bin.clone(),
            source,
        })?;
        let bin = bin
            .canonicalize()
            .map_err(|source| CliError::Io { path: bin, source })?;
        let binary = std::env::current_exe().map_err(|source| CliError::Io {
            path: "<current executable>".into(),
            source,
        })?;
        let destination = bin.join(format!("etas{}", std::env::consts::EXE_SUFFIX));
        if binary == destination {
            return Ok(None);
        }
        // Cargo can replace the source executable while the SDK is compiling.
        let mut input = fs::File::open(&binary).map_err(|source| CliError::Io {
            path: binary,
            source,
        })?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = bin.join(format!(".etas-install-{}-{stamp}", std::process::id()));
        let mut output = fs::File::create_new(&temporary).map_err(|source| CliError::Io {
            path: temporary.clone(),
            source,
        })?;
        let staged = Self {
            temporary,
            destination,
        };
        io::copy(&mut input, &mut output)
            .and_then(|_| output.set_permissions(input.metadata()?.permissions()))
            .map_err(|source| CliError::Io {
                path: staged.temporary.clone(),
                source,
            })?;
        Ok(Some(staged))
    }

    fn publish(self) -> Result<(), CliError> {
        fs::rename(&self.temporary, &self.destination).map_err(|source| CliError::Io {
            path: self.destination.clone(),
            source,
        })
    }
}

impl Drop for StagedExecutable {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temporary);
    }
}
