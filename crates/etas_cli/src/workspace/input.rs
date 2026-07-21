use std::path::Path;

use etas_core::{SourceFile, SourceId};
#[cfg(feature = "component-frontend")]
use etas_frontend::{SourceInput, SourceKind};

use crate::error::CliError;

pub fn read_source_file(path: &Path, id: SourceId) -> Result<SourceFile, CliError> {
    let text = std::fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(SourceFile::new(id, Some(path.to_path_buf()), text))
}

#[cfg(feature = "component-frontend")]
pub fn source_input(source: &SourceFile) -> SourceInput {
    SourceInput {
        id: source.id,
        path: source.path.clone(),
        text: source.text().to_owned(),
        kind: SourceKind::SingleFileInput,
    }
}
