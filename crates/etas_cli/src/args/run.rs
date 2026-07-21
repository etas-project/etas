use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct RunArgs {
    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub flow: Option<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub allow_effects: bool,

    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub runtime_config: Option<PathBuf>,

    #[arg(long)]
    pub print_runtime_profile: bool,

    #[arg(long = "allow-net", value_name = "HOST:PORT")]
    pub allow_net: Vec<String>,

    #[arg(long = "args", value_name = "ARG", num_args = 0.., allow_hyphen_values = true)]
    pub program_args: Vec<String>,

    #[arg(long, value_name = "PATH")]
    pub trace_out: Option<PathBuf>,

    #[arg(long, value_name = "PATH")]
    pub checkpoint_dir: Option<PathBuf>,

    #[arg(
        long,
        value_name = "DEPTH",
        help = "Maximum heap-backed interpreter call depth (1..=65536)"
    )]
    pub max_call_depth: Option<u32>,

    #[arg(long)]
    pub budget_tokens: Option<u64>,

    #[arg(long)]
    pub budget_cost: Option<String>,

    #[arg(long)]
    pub budget_time: Option<String>,
}
