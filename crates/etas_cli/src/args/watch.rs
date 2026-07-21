use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct WatchArgs {
    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(long)]
    pub all: bool,
}
