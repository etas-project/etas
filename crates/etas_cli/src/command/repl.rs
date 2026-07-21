use std::io::Write;

use crate::{
    args::{global::GlobalOptions, repl::ReplArgs},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    _global: &GlobalOptions,
    _args: ReplArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(super::unsupported(
        "repl",
        "the REPL must be a client of stable driver APIs",
    ))
}
