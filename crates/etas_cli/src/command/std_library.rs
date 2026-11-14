use std::io::Write;

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
    let (request, prefix) = match args.command {
        StdCommand::Build(args) => (
            StdLibraryBuildRequest {
                manifest_path: args.manifest_path,
                out_dir: args.out_dir,
            },
            None,
        ),
        StdCommand::Install(args) => (
            StdLibraryBuildRequest {
                manifest_path: args.manifest_path,
                out_dir: args
                    .prefix
                    .join("lib/etas")
                    .join(etas_driver::std_library::sdk::STD_TOOLCHAIN_ID)
                    .join("std"),
            },
            Some(args.prefix),
        ),
    };
    let output =
        build_standard_library(&super::frontend::driver_options(global, profile)?, request)
            .map_err(super::frontend::driver_error)?;
    if let Some(prefix) = prefix {
        let bin = prefix.join("bin");
        std::fs::create_dir_all(&bin).map_err(|source| CliError::Io {
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
        if binary != destination {
            std::fs::copy(&binary, &destination).map_err(|source| CliError::Io {
                path: destination,
                source,
            })?;
        }
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
