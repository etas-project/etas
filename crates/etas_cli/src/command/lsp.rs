use std::io::Write;

use crate::{
    args::{global::GlobalOptions, lsp::LspArgs},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    _global: &GlobalOptions,
    _args: LspArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(super::unsupported(
        "lsp",
        "etas_lsp currently exposes only a placeholder binary, not a reusable server entry point",
    ))
}
