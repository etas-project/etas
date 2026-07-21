#[cfg(feature = "cli-check")]
pub mod check;
#[cfg(any(
    feature = "cli-dump-ast",
    feature = "cli-dump-hir",
    feature = "cli-dump-air"
))]
pub mod dump;
pub mod global;
#[cfg(feature = "cli-graph")]
pub mod graph;
#[cfg(feature = "cli-lsp")]
pub mod lsp;
#[cfg(feature = "cli-pkg")]
pub mod pkg;
#[cfg(feature = "cli-repl")]
pub mod repl;
#[cfg(feature = "cli-run")]
pub mod run;
#[cfg(feature = "cli-watch")]
pub mod watch;

use clap::{Parser, Subcommand};

use self::global::GlobalOptions;

#[cfg(feature = "cli-check")]
use self::check::CheckArgs;
#[cfg(any(
    feature = "cli-dump-ast",
    feature = "cli-dump-hir",
    feature = "cli-dump-air"
))]
use self::dump::DumpArgs;
#[cfg(feature = "cli-graph")]
use self::graph::GraphArgs;
#[cfg(feature = "cli-lsp")]
use self::lsp::LspArgs;
#[cfg(feature = "cli-pkg")]
use self::pkg::PkgArgs;
#[cfg(feature = "cli-repl")]
use self::repl::ReplArgs;
#[cfg(feature = "cli-run")]
use self::run::RunArgs;
#[cfg(feature = "cli-watch")]
use self::watch::WatchArgs;

#[derive(Parser, Clone, Debug)]
#[command(name = "etas", version, about = "Etas language command line interface")]
pub struct CliArgs {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[cfg(feature = "cli-check")]
    Check(CheckArgs),
    #[cfg(any(
        feature = "cli-dump-ast",
        feature = "cli-dump-hir",
        feature = "cli-dump-air"
    ))]
    Dump(DumpArgs),
    #[cfg(feature = "cli-graph")]
    Graph(GraphArgs),
    #[cfg(feature = "cli-effects")]
    Effects(EffectsArgs),
    #[cfg(feature = "cli-policy")]
    Policy(PolicyArgs),
    #[cfg(feature = "cli-pkg")]
    Pkg(PkgArgs),
    #[cfg(feature = "cli-run")]
    Run(RunArgs),
    #[cfg(feature = "cli-replay")]
    Replay(ReplayArgs),
    #[cfg(feature = "cli-resume")]
    Resume(ResumeArgs),
    #[cfg(feature = "cli-watch")]
    Watch(WatchArgs),
    #[cfg(feature = "cli-repl")]
    Repl(ReplArgs),
    #[cfg(feature = "cli-lsp")]
    Lsp(LspArgs),
}

#[cfg(feature = "cli-effects")]
#[derive(clap::Args, Clone, Debug)]
pub struct EffectsArgs {
    #[arg(value_name = "FILE")]
    pub file: std::path::PathBuf,

    #[arg(long)]
    pub flow: Option<String>,
}

#[cfg(feature = "cli-policy")]
#[derive(clap::Args, Clone, Debug)]
pub struct PolicyArgs {
    #[arg(value_name = "FILE")]
    pub file: std::path::PathBuf,

    #[arg(long)]
    pub flow: Option<String>,
}

#[cfg(feature = "cli-replay")]
#[derive(clap::Args, Clone, Debug)]
pub struct ReplayArgs {
    #[arg(value_name = "TRACE")]
    pub trace: std::path::PathBuf,

    #[arg(long)]
    pub until: Option<String>,
}

#[cfg(feature = "cli-resume")]
#[derive(clap::Args, Clone, Debug)]
pub struct ResumeArgs {
    #[arg(value_name = "CHECKPOINT_ID")]
    pub checkpoint_id: String,

    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub runtime_config: Option<std::path::PathBuf>,

    #[arg(long, value_name = "PATH")]
    pub checkpoint_dir: Option<std::path::PathBuf>,

    #[arg(
        long,
        value_name = "DEPTH",
        help = "Maximum heap-backed interpreter call depth (1..=65536)"
    )]
    pub max_call_depth: Option<u32>,
}
