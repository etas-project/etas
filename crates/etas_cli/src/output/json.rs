use std::{collections::HashMap, path::PathBuf};

use etas_core::{Diagnostic, SourceFile};
use serde_json::{Value, json};

use crate::output::diagnostic::{diagnostic_code, severity_text};

pub fn diagnostics_json(diagnostics: &[Diagnostic], source: &SourceFile) -> Value {
    Value::Array(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic_json(diagnostic, source))
            .collect(),
    )
}

pub fn diagnostic_json(diagnostic: &Diagnostic, source: &SourceFile) -> Value {
    let span = diagnostic.primary.span;
    let start = source.line_index.line_col(span.range.start);
    let end = source.line_index.line_col(span.range.end);
    json!({
        "code": diagnostic_code(&diagnostic.code),
        "phase": format!("{:?}", diagnostic.phase),
        "severity": severity_text(diagnostic.severity),
        "message": diagnostic.message,
        "primary": {
            "label": diagnostic.primary.label,
            "span": {
                "source": span.source.0,
                "start": span.range.start.0,
                "end": span.range.end.0,
                "startLine": start.line + 1,
                "startColumn": start.col + 1,
                "endLine": end.line + 1,
                "endColumn": end.col + 1,
            }
        },
        "labels": diagnostic.labels.iter().map(|label| {
            json!({
                "style": format!("{:?}", label.style),
                "message": label.message,
                "span": {
                    "source": label.span.source.0,
                    "start": label.span.range.start.0,
                    "end": label.span.range.end.0,
                }
            })
        }).collect::<Vec<_>>(),
        "notes": diagnostic.notes,
        "help": diagnostic.help,
        "suggestions": diagnostic.suggestions.iter().map(|suggestion| {
            json!({
                "title": suggestion.title,
                "applicability": format!("{:?}", suggestion.applicability),
                "edits": suggestion.edits.iter().map(|edit| {
                    json!({
                        "start": edit.range.start.0,
                        "end": edit.range.end.0,
                        "replacement": edit.replacement,
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>(),
    })
}

pub fn project_diagnostics_json(
    diagnostics: &[Diagnostic],
    sources: &[(PathBuf, SourceFile)],
) -> Value {
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
                    .map(|source| diagnostic_json(diagnostic, source))
            })
            .collect(),
    )
}
