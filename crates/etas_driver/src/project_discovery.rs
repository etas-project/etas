use std::path::{Path, PathBuf};

use etas_package::discover_manifest;

use crate::{DriverError, DriverOptions};

pub fn resolve_workspace(options: &DriverOptions) -> Result<PathBuf, DriverError> {
    match &options.workspace {
        Some(path) => Ok(path.clone()),
        None => std::env::current_dir().map_err(|source| DriverError::Io {
            path: PathBuf::from("."),
            source,
        }),
    }
}

pub fn discover_package_root(start: &Path) -> Option<PathBuf> {
    discover_manifest(start).and_then(|manifest| manifest.parent().map(Path::to_path_buf))
}

pub fn absolute_input_path(path: PathBuf) -> Result<PathBuf, DriverError> {
    if path.is_absolute() {
        return Ok(path);
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|source| DriverError::Io {
            path: PathBuf::from("."),
            source,
        })
}

pub fn choose_source_project_root(root: &Path) -> PathBuf {
    let src = root.join("src");
    if src.is_dir() {
        src
    } else {
        root.to_path_buf()
    }
}
