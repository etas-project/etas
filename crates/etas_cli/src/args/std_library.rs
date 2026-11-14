use std::path::PathBuf;

#[derive(clap::Args, Clone, Debug)]
pub struct StdArgs {
    #[command(subcommand)]
    pub command: StdCommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum StdCommand {
    /// Build and install checked std sources and their matching artifact.
    Build(StdBuildArgs),
}

#[derive(clap::Args, Clone, Debug)]
pub struct StdBuildArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub out_dir: PathBuf,
}
