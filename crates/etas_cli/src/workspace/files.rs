use std::path::PathBuf;

use crate::{args::global::GlobalOptions, error::CliError};

pub fn resolve_workspace(global: &GlobalOptions) -> Result<PathBuf, CliError> {
    let path = match &global.workspace {
        Some(path) => path.clone(),
        None => std::env::current_dir().map_err(|source| CliError::Io {
            path: PathBuf::from("."),
            source,
        })?,
    };

    Ok(path)
}
