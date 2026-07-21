use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct CheckArgs {
    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub phase1: bool,

    #[arg(long)]
    pub watch: bool,
}
