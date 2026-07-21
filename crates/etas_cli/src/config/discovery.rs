use std::path::{Path, PathBuf};

use crate::{args::global::GlobalOptions, error::CliError};

use super::CliConfig;

pub fn load_config(
    global: &GlobalOptions,
    workspace: &Path,
) -> Result<Option<CliConfig>, CliError> {
    if global.no_config {
        return Ok(None);
    }

    if let Some(path) = &global.config {
        return read_config(path.clone()).map(Some);
    }

    let default = workspace.join("etas.toml");
    if default.is_file() {
        return read_config(default).map(Some);
    }

    Ok(None)
}

fn read_config(path: PathBuf) -> Result<CliConfig, CliError> {
    let contents = std::fs::read_to_string(&path).map_err(|source| CliError::Config {
        path: path.clone(),
        source,
    })?;
    Ok(CliConfig { path, contents })
}
