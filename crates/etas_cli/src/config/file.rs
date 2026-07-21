use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct CliConfig {
    pub path: PathBuf,
    pub contents: String,
}
