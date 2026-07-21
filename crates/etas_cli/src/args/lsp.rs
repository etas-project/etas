use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct LspArgs {
    #[arg(long, default_value_t = true)]
    pub stdio: bool,
}
