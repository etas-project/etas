use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args, Clone, Debug)]
pub struct DumpArgs {
    #[command(subcommand)]
    pub command: DumpCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum DumpCommand {
    #[cfg(feature = "cli-dump-ast")]
    Ast(DumpAstArgs),
    #[cfg(feature = "cli-dump-hir")]
    Hir(DumpHirArgs),
    #[cfg(feature = "cli-dump-air")]
    Air(DumpAirArgs),
}

#[cfg(feature = "cli-dump-ast")]
#[derive(Args, Clone, Debug)]
pub struct DumpAstArgs {
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    #[arg(long)]
    pub tokens: bool,

    #[arg(long)]
    pub spans: bool,

    #[arg(long)]
    pub diagnostics: bool,
}

#[cfg(feature = "cli-dump-hir")]
#[derive(Args, Clone, Debug)]
pub struct DumpHirArgs {
    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub symbols: bool,

    #[arg(long)]
    pub scopes: bool,

    #[arg(long = "source-map")]
    pub source_map: bool,

    #[arg(long)]
    pub spans: bool,

    #[arg(long)]
    pub diagnostics: bool,
}

#[cfg(feature = "cli-dump-air")]
#[derive(Args, Clone, Debug)]
pub struct DumpAirArgs {
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    #[arg(long)]
    pub flow: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}
