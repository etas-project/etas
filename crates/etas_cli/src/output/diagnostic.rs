use std::fmt::Write as _;

use etas_core::{Diagnostic, DiagnosticCode, LabelStyle, Severity, SourceFile};

pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

pub fn render_human(diagnostics: &[Diagnostic], source: &SourceFile) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        let primary_location = location(source, diagnostic.primary.span.range.start.to_usize());
        let severity = severity_text(diagnostic.severity);
        let code = diagnostic_code(&diagnostic.code);
        let _ = writeln!(output, "{severity}[{code}]: {}", diagnostic.message);
        let path = source
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<anonymous>".to_string());
        let _ = writeln!(
            output,
            " --> {path}:{}:{}",
            primary_location.line, primary_location.col
        );

        if let Some(line) = source_line(source, primary_location.zero_based_line) {
            let _ = writeln!(output, "  | {line}");
            let caret_col = primary_location.zero_based_col.min(line.len());
            let label = diagnostic.primary.label.as_deref().unwrap_or("");
            let _ = writeln!(output, "  | {}^ {label}", " ".repeat(caret_col));
        }

        for label in &diagnostic.labels {
            if label.style == LabelStyle::Secondary {
                let secondary = location(source, label.span.range.start.to_usize());
                let _ = writeln!(
                    output,
                    "  = note: {} at {}:{}",
                    label.message, secondary.line, secondary.col
                );
            }
        }

        for note in &diagnostic.notes {
            let _ = writeln!(output, "  = note: {note}");
        }

        if let Some(help) = &diagnostic.help {
            let _ = writeln!(output, "  = help: {help}");
        }
    }
    output
}

struct Location {
    line: u32,
    col: u32,
    zero_based_line: u32,
    zero_based_col: usize,
}

fn location(source: &SourceFile, offset: usize) -> Location {
    let line_col = source.line_index.line_col(etas_core::TextSize::new(offset));
    Location {
        line: line_col.line + 1,
        col: line_col.col + 1,
        zero_based_line: line_col.line,
        zero_based_col: line_col.col as usize,
    }
}

fn source_line(source: &SourceFile, line: u32) -> Option<String> {
    let range = source.line_index.line_range(line)?;
    let start = range.start.to_usize();
    let end = range.end.to_usize();
    Some(
        source.text()[start..end]
            .trim_end_matches(['\r', '\n'])
            .to_string(),
    )
}

pub fn severity_text(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

pub fn diagnostic_code(code: &DiagnosticCode) -> String {
    match code {
        DiagnosticCode::Syntax(code) => format!("syntax::{code:?}"),
        DiagnosticCode::Name(code) => format!("name::{code:?}"),
        DiagnosticCode::Type(code) => format!("type::{code:?}"),
        DiagnosticCode::Effect(code) => format!("effect::{code:?}"),
        DiagnosticCode::Analysis(code) => format!("analysis::{code:?}"),
    }
}
