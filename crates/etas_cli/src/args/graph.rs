use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Args, Clone, Debug)]
pub struct GraphArgs {
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    #[arg(long)]
    pub flow: Option<String>,

    #[arg(long, value_enum, default_value_t = GraphKind::Source)]
    pub kind: GraphKind,

    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum GraphKind {
    Source,
    Air,
    Effects,
}
