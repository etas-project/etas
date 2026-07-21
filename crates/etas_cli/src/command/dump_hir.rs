use std::{io::Write, path::PathBuf};

use etas_core::{Diagnostic, DiagnosticPhase, Severity, SourceFile};
use etas_driver::SourceDependencyMode;
use etas_frontend::{EntryPolicy, HirDumpOptions, dump_hir};
use etas_utils::ProfileHandle;
use serde_json::json;

use crate::{
    args::{dump::DumpHirArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
    output::{diagnostic, json as json_output, write_artifact},
};

pub fn run(
    global: &GlobalOptions,
    args: DumpHirArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let profile = ProfileHandle::disabled();
    let loaded = super::frontend::load_project(
        global,
        &profile,
        args.input,
        args.all,
        EntryPolicy::CompileOnly,
        None,
        SourceDependencyMode::MetadataOnly,
    )?;
    let sources = loaded.sources;
    let output =
        super::frontend::check_project_once(global, &profile, loaded.input)?.into_check_output();
    let options = HirDumpOptions {
        include_spans: args.spans,
        include_diagnostics: args.diagnostics,
        include_symbols: args.symbols,
        include_scopes: args.scopes,
        include_source_map: args.source_map,
    };
    let artifact = dump_hir(&output, options);
    let has_errors = has_hir_dump_errors(&output.diagnostics);

    match global.format {
        crate::args::global::OutputFormat::Json => {
            let payload = json!({
                "kind": "hir",
                "paths": sources.iter().map(|(path, _)| path).collect::<Vec<_>>(),
                "artifact": artifact,
                "diagnostics": json_output::project_diagnostics_json(&output.diagnostics, &sources),
            });
            writeln!(stdout, "{payload}").map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            writeln!(
                stdout,
                "{}",
                json!({
                    "type": "artifact",
                    "kind": "hir",
                    "paths": sources.iter().map(|(path, _)| path).collect::<Vec<_>>(),
                    "artifact": artifact
                })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            if !global.quiet && !output.diagnostics.is_empty() {
                write_diagnostics(stderr, &output.diagnostics, &sources)?;
            }
            write_artifact(stdout, None, &artifact)?;
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas dump hir`"
            )));
        }
    }

    Ok(if has_errors {
        CliExit::Diagnostics
    } else {
        CliExit::Success
    })
}

fn write_diagnostics(
    stderr: &mut dyn Write,
    diagnostics: &[Diagnostic],
    sources: &[(PathBuf, SourceFile)],
) -> Result<(), CliError> {
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
    Ok(())
}

fn has_hir_dump_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Error
            && matches!(
                diagnostic.phase,
                DiagnosticPhase::Lex | DiagnosticPhase::Parse | DiagnosticPhase::Lower
            )
    })
}
