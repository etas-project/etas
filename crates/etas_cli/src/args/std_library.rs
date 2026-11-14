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
    /// Install this executable and a matching std SDK into a toolchain prefix.
    Install(StdInstallArgs),
}

#[derive(clap::Args, Clone, Debug)]
pub struct StdBuildArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub out_dir: PathBuf,
}

#[derive(clap::Args, Clone, Debug)]
pub struct StdInstallArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub prefix: PathBuf,
}
