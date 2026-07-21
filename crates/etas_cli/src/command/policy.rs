use std::io::Write;

use crate::{
    args::{PolicyArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    _global: &GlobalOptions,
    _args: PolicyArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(super::unsupported(
        "policy",
        "policy explanation requires the future policy and driver reports",
    ))
}
