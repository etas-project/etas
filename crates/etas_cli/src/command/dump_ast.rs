use std::io::Write;

use etas_core::SourceId;
use etas_frontend::{DumpOptions, Frontend, dump_ast};
use serde_json::json;

use crate::{
    args::{dump::DumpAstArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
    output::{diagnostic, json as json_output, write_artifact},
    workspace,
};

pub fn run(
    global: &GlobalOptions,
    args: DumpAstArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let source = workspace::read_source_file(&args.file, SourceId(1))?;
    let frontend = Frontend;
    let parsed = frontend.parse(workspace::source_input(&source));
    let options = DumpOptions {
        include_spans: args.spans,
        include_tokens: args.tokens,
        include_diagnostics: args.diagnostics,
    };
    let artifact = dump_ast(&parsed, options);
    let has_errors = diagnostic::has_errors(&parsed.parsed.diagnostics);

    match global.format {
        crate::args::global::OutputFormat::Json => {
            let payload = json!({
                "kind": "ast",
                "path": args.file,
                "artifact": artifact,
                "diagnostics": json_output::diagnostics_json(&parsed.parsed.diagnostics, &source),
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
                json!({ "type": "artifact", "kind": "ast", "path": args.file, "artifact": artifact })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            if !global.quiet && !parsed.parsed.diagnostics.is_empty() {
                write!(
                    stderr,
                    "{}",
                    diagnostic::render_human(&parsed.parsed.diagnostics, &source)
                )
                .map_err(|source| CliError::Io {
                    path: "<stderr>".into(),
                    source,
                })?;
            }
            write_artifact(stdout, None, &artifact)?;
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas dump ast`"
            )));
        }
    }

    Ok(if has_errors {
        CliExit::Diagnostics
    } else {
        CliExit::Success
    })
}
