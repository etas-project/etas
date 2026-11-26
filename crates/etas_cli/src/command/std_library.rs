use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use etas_driver::std_library::{
    install_receipt::publish_install_receipt, sdk::install_standard_library,
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
            Some(StagedExecutable::prepare(&args.prefix)?),
        ),
    };
    let options = super::frontend::base_driver_options(global, profile);
    let output = if executable.is_some() {
        install_standard_library(&options, request, |program| {
            #[cfg(feature = "component-runtime")]
            {
                etas_interpreter::encode_installed_layouts(program)
                    .map_err(etas_driver::DriverError::InvalidInput)
            }
            #[cfg(not(feature = "component-runtime"))]
            {
                let _ = program;
                Ok(None)
            }
        })
    } else {
        build_standard_library(&options, request)
    }
    .map_err(super::frontend::driver_error)?;
    if let Some(executable) = executable {
        if output.manifest.effect_analysis_hash.is_some()
            && let Some(temporary) = &executable.temporary
        {
            let identity = output
                .manifest
                .identity()
                .map_err(super::frontend::driver_error)?;
            publish_install_receipt(temporary, &identity).map_err(super::frontend::driver_error)?;
        }
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
    temporary: Option<PathBuf>,
    destination: PathBuf,
}

impl StagedExecutable {
    fn prepare(prefix: &Path) -> Result<Self, CliError> {
        let bin = prefix.join("bin");
        fs::create_dir_all(&bin).map_err(|source| CliError::Io {
            path: bin.clone(),
            source,
        })?;
        let bin = bin
            .canonicalize()
            .map_err(|source| CliError::Io { path: bin, source })?;
        let destination = bin.join(format!("etas{}", std::env::consts::EXE_SUFFIX));
        #[cfg(not(target_os = "linux"))]
        let binary = std::env::current_exe().map_err(|source| CliError::Io {
            path: "<current executable>".into(),
            source,
        })?;
        #[cfg(not(target_os = "linux"))]
        if binary == destination {
            return Ok(Self {
                temporary: None,
                destination,
            });
        }
        // Cargo can replace the source executable while the SDK is compiling.
        #[cfg(target_os = "linux")]
        let binary = PathBuf::from("/proc/self/exe");
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
            temporary: Some(temporary.clone()),
            destination,
        };
        io::copy(&mut input, &mut output)
            .and_then(|_| set_installed_permissions(&input, &output))
            .map_err(|source| CliError::Io {
                path: temporary,
                source,
            })?;
        Ok(staged)
    }

    fn publish(&self) -> Result<(), CliError> {
        if let Some(temporary) = &self.temporary {
            fs::rename(temporary, &self.destination).map_err(|source| CliError::Io {
                path: self.destination.clone(),
                source,
            })?;
        }
        Ok(())
    }
}

fn set_installed_permissions(input: &fs::File, output: &fs::File) -> io::Result<()> {
    let permissions = input.metadata()?.permissions();
    #[cfg(target_os = "linux")]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        fs::Permissions::from_mode(permissions.mode() & !0o022)
    };
    output.set_permissions(permissions)
}

impl Drop for StagedExecutable {
    fn drop(&mut self) {
        if let Some(temporary) = &self.temporary {
            let _ = fs::remove_file(temporary);
        }
    }
}
