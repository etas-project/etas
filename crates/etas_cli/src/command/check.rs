use std::{io::Write, path::PathBuf};

use etas_core::{Diagnostic, DiagnosticCode, EffectDiagnosticCode, SourceFile};
use etas_driver::SourceDependencyMode;
use etas_frontend::EntryPolicy;
use etas_utils::ProfileHandle;
use serde_json::json;

use crate::{
    args::{check::CheckArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
    output::{diagnostic, json as json_output, text},
};

pub fn run(
    global: &GlobalOptions,
    args: CheckArgs,
    profile: &ProfileHandle,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    if args.watch {
        return Err(CliError::Unsupported {
            command: "check --watch",
            reason: "watch mode depends on a stable compiler session model",
        });
    }

    let entry_policy = if args.phase1 {
        EntryPolicy::Runnable
    } else {
        EntryPolicy::CompileOnly
    };
    let loaded = super::frontend::load_project(
        global,
        profile,
        args.input,
        args.all,
        entry_policy,
        None,
        SourceDependencyMode::MetadataOnly,
    )?;
    let output = super::frontend::check_project_once(global, profile, loaded.input)?;
    let any_errors = has_blocking_check_errors(&output.diagnostics, args.phase1);

    let checked = loaded
        .sources
        .into_iter()
        .map(|(path, source)| {
            let diagnostics = output
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.primary.span.source == source.id)
                .cloned()
                .collect::<Vec<_>>();
            let visible_diagnostics = visible_check_diagnostics(global, args.phase1, &diagnostics);
            render_diagnostics(global, stderr, &source, &visible_diagnostics)?;
            Ok(CheckFile {
                path,
                source,
                diagnostics,
            })
        })
        .collect::<Result<Vec<_>, CliError>>()?;

    render_summary(global, stdout, &checked)?;

    if any_errors {
        Ok(CliExit::Diagnostics)
    } else {
        Ok(CliExit::Success)
    }
}

fn has_blocking_check_errors(diagnostics: &[Diagnostic], phase1: bool) -> bool {
    if phase1 {
        return diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == etas_core::Severity::Error
                || matches!(
                    diagnostic.code,
                    DiagnosticCode::Effect(EffectDiagnosticCode::RuntimeRequiredInPhase1)
                )
        });
    }
    diagnostics.iter().any(is_static_check_error)
}

fn is_static_check_error(diagnostic: &Diagnostic) -> bool {
    diagnostic::has_errors(std::slice::from_ref(diagnostic))
        && !matches!(
            diagnostic.code,
            DiagnosticCode::Effect(EffectDiagnosticCode::RuntimeRequiredInPhase1)
        )
}

fn visible_check_diagnostics(
    global: &GlobalOptions,
    phase1: bool,
    diagnostics: &[Diagnostic],
) -> Vec<Diagnostic> {
    if phase1 || global.verbose > 0 {
        return diagnostics.to_vec();
    }

    diagnostics
        .iter()
        .filter(|diagnostic| !is_phase1_runtime_readiness_warning(diagnostic))
        .cloned()
        .collect()
}

fn is_phase1_runtime_readiness_warning(diagnostic: &Diagnostic) -> bool {
    matches!(
        diagnostic.code,
        DiagnosticCode::Effect(EffectDiagnosticCode::RuntimeRequiredInPhase1)
    )
}

fn render_diagnostics(
    global: &GlobalOptions,
    stderr: &mut dyn Write,
    source: &SourceFile,
    diagnostics: &[Diagnostic],
) -> Result<(), CliError> {
    if global.quiet || diagnostics.is_empty() {
        return Ok(());
    }
    if matches!(
        global.format,
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text
    ) {
        write!(stderr, "{}", diagnostic::render_human(diagnostics, source)).map_err(|source| {
            CliError::Io {
                path: "<stderr>".into(),
                source,
            }
        })?;
    }
    Ok(())
}

fn render_summary(
    global: &GlobalOptions,
    stdout: &mut dyn Write,
    checked: &[CheckFile],
) -> Result<(), CliError> {
    match global.format {
        crate::args::global::OutputFormat::Json => {
            let files = checked
                .iter()
                .map(|file| {
                    json!({
                        "path": file.path,
                        "diagnostics": json_output::diagnostics_json(&file.diagnostics, &file.source),
                    })
                })
                .collect::<Vec<_>>();
            writeln!(stdout, "{}", json!({ "command": "check", "files": files })).map_err(
                |source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                },
            )?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            for file in checked {
                for diagnostic in &file.diagnostics {
                    writeln!(
                        stdout,
                        "{}",
                        json!({
                            "type": "diagnostic",
                            "path": file.path,
                            "diagnostic": json_output::diagnostic_json(diagnostic, &file.source),
                        })
                    )
                    .map_err(|source| CliError::Io {
                        path: "<stdout>".into(),
                        source,
                    })?;
                }
            }
            writeln!(
                stdout,
                "{}",
                json!({ "type": "summary", "checked": checked.len() })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            if !global.quiet {
                writeln!(
                    stdout,
                    "checked {}",
                    text::plural(checked.len(), "file", "files")
                )
                .map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
            }
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas check`"
            )));
        }
    }
    Ok(())
}

struct CheckFile {
    path: PathBuf,
    source: SourceFile,
    diagnostics: Vec<Diagnostic>,
}
