use std::io::Write;

use crate::{
    args::{dump::DumpAirArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    _global: &GlobalOptions,
    _args: DumpAirArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(super::unsupported(
        "dump air",
        "AIR lowering and artifact production belong in the future etas_driver boundary",
    ))
}
