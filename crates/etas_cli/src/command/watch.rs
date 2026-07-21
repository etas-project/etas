use std::io::Write;

use crate::{
    args::{global::GlobalOptions, watch::WatchArgs},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    _global: &GlobalOptions,
    _args: WatchArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(CliError::Unsupported {
        command: "watch",
        reason: "watch mode is deferred until it owns one live FrontendSession and drives apply_changes plus incremental check from filesystem events",
    })
}
