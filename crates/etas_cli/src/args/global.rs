use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Args, Clone, Debug)]
pub struct GlobalOptions {
    #[arg(long, global = true, value_name = "PATH")]
    pub workspace: Option<PathBuf>,

    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true)]
    pub no_config: bool,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,

    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    #[arg(long, global = true)]
    pub quiet: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long, global = true, value_enum)]
    pub log_level: Option<LogLevel>,

    #[arg(long, global = true, value_enum, default_value_t = CacheMode::Auto)]
    pub cache: CacheMode,

    #[arg(long, global = true, value_name = "PATH")]
    pub cache_root: Option<PathBuf>,

    #[arg(long, global = true, value_name = "PATH")]
    pub profile_out: Option<PathBuf>,

    #[arg(long, global = true)]
    pub profile_tree: bool,

    #[arg(long, global = true, requires = "profile_tree")]
    pub profile_detail: bool,

    #[arg(long, global = true, requires = "profile_tree")]
    pub profile_pass_timing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Jsonl,
    Text,
    Mermaid,
    Dot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CacheMode {
    Auto,
    Off,
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
