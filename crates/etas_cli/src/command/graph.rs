use std::io::Write;

use crate::{
    args::{global::GlobalOptions, graph::GraphArgs},
    error::CliError,
    exit::CliExit,
    output,
};

pub fn run(
    _global: &GlobalOptions,
    _args: GraphArgs,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    Err(super::unsupported(
        "graph",
        output::graph::GRAPH_DRIVER_REASON,
    ))
}
