pub mod diagnostic;
pub mod graph;
pub mod json;
pub mod text;

use std::{io::Write, path::Path};

use crate::error::CliError;

pub fn write_artifact(
    stdout: &mut dyn Write,
    path: Option<&Path>,
    contents: &str,
) -> Result<(), CliError> {
    match path {
        Some(path) => std::fs::write(path, contents).map_err(|source| CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
        None => {
            write!(stdout, "{contents}").map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            if !contents.ends_with('\n') {
                writeln!(stdout).map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
            }
            Ok(())
        }
    }
}
