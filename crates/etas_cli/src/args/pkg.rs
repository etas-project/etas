use std::path::PathBuf;

#[derive(clap::Args, Clone, Debug)]
pub struct PkgArgs {
    #[command(subcommand)]
    pub command: PkgCommand,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum PkgCommand {
    Lock(PkgTargetArgs),
    Update(PkgTargetArgs),
    Prepare(PkgPrepareArgs),
    Metadata(PkgMetadataArgs),
    Pack(PkgPackArgs),
}

#[derive(clap::Args, Clone, Debug)]
pub struct PkgTargetArgs {
    #[arg(value_name = "PACKAGE_ROOT", default_value = ".")]
    pub package_root: PathBuf,
}

#[derive(clap::Args, Clone, Debug)]
pub struct PkgPrepareArgs {
    #[arg(value_name = "PACKAGE_ROOT", default_value = ".")]
    pub package_root: PathBuf,
}

#[derive(clap::Args, Clone, Debug)]
pub struct PkgMetadataArgs {
    #[arg(value_name = "PACKAGE_ROOT", default_value = ".")]
    pub package_root: PathBuf,

    #[arg(long, value_name = "BIN")]
    pub bin: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct PkgPackArgs {
    #[arg(value_name = "PACKAGE_ROOT", default_value = ".")]
    pub package_root: PathBuf,

    #[arg(long, value_name = "PATH")]
    pub out: PathBuf,
}
