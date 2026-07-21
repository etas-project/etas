use std::{collections::HashMap, io::Write, path::PathBuf};

use etas_core::{Diagnostic, SourceFile};
use serde_json::Value;

use crate::{
    args::global::{GlobalOptions, OutputFormat},
    error::CliError,
    output::{diagnostic, json as json_output},
};

pub fn has_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error)
}

pub fn render_diagnostics(
    global: &GlobalOptions,
    stderr: &mut dyn Write,
    sources: &[(PathBuf, SourceFile)],
    diagnostics: &[Diagnostic],
) -> Result<(), CliError> {
    if global.quiet || diagnostics.is_empty() {
        return Ok(());
    }
    if matches!(global.format, OutputFormat::Human | OutputFormat::Text) {
        for (_, source) in sources {
            let source_diagnostics = diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.primary.span.source == source.id)
                .cloned()
                .collect::<Vec<_>>();
            if source_diagnostics.is_empty() {
                continue;
            }
            write!(
                stderr,
                "{}",
                diagnostic::render_human(&source_diagnostics, source)
            )
            .map_err(|source| CliError::Io {
                path: "<stderr>".into(),
                source,
            })?;
        }
    }
    Ok(())
}

pub fn diagnostics_json(diagnostics: &[Diagnostic], sources: &[(PathBuf, SourceFile)]) -> Value {
    let sources_by_id = sources
        .iter()
        .map(|(_, source)| (source.id, source))
        .collect::<HashMap<_, _>>();
    Value::Array(
        diagnostics
            .iter()
            .filter_map(|diagnostic| {
                sources_by_id
                    .get(&diagnostic.primary.span.source)
                    .map(|source| json_output::diagnostic_json(diagnostic, source))
            })
            .collect(),
    )
}

pub fn diagnostic_json(
    diagnostic: &Diagnostic,
    sources: &[(PathBuf, SourceFile)],
) -> Option<Value> {
    sources
        .iter()
        .find(|(_, source)| source.id == diagnostic.primary.span.source)
        .map(|(_, source)| json_output::diagnostic_json(diagnostic, source))
}
