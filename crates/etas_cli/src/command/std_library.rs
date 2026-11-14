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
    match args.command {
        StdCommand::Build(args) => {
            let output = build_standard_library(
                &super::frontend::driver_options(global, profile),
                StdLibraryBuildRequest {
                    manifest_path: args.manifest_path,
                    out_dir: args.out_dir,
                },
            )
            .map_err(super::frontend::driver_error)?;
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
    }
}
