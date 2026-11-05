use std::path::PathBuf;

use clap::Args;

use super::optimization::OptimizationArgs;

#[derive(Args, Clone, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    pub optimization: OptimizationArgs,

    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub phase1: bool,

    #[arg(long)]
    pub watch: bool,
}
