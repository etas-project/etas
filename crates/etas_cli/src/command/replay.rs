use std::io::Write;

use crate::{
    args::{ReplayArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    global: &GlobalOptions,
    args: ReplayArgs,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    if args.until.is_some() {
        return Err(CliError::InvalidUsage(
            "`etas replay --until` requires cursor-addressable trace events".to_owned(),
        ));
    }
    let bytes = std::fs::read(&args.trace).map_err(|source| CliError::Io {
        path: args.trace.clone(),
        source,
    })?;
    let report: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| CliError::InvalidUsage("replay trace is not valid JSON".to_owned()))?;
    if report.get("schema").and_then(serde_json::Value::as_str)
        != Some("etas.cli.interpreter-report.v1")
    {
        return Err(CliError::InvalidUsage(
            "replay currently expects an `etas run --trace-out` interpreter report".to_owned(),
        ));
    }

    match global.format {
        crate::args::global::OutputFormat::Json => {
            writeln!(
                stdout,
                "{}",
                serde_json::json!({
                    "command": "replay",
                    "source_trace": args.trace,
                    "until": args.until,
                    "report": report,
                })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            if let Some(events) = report.get("events").and_then(serde_json::Value::as_array) {
                for event in events {
                    writeln!(
                        stdout,
                        "{}",
                        serde_json::json!({ "type": "event", "event": event })
                    )
                    .map_err(|source| CliError::Io {
                        path: "<stdout>".into(),
                        source,
                    })?;
                }
            }
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            if !global.quiet {
                let events = report
                    .get("events")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len);
                let checkpoints = report
                    .get("checkpoints")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len);
                writeln!(
                    stdout,
                    "replayed interpreter report: {events} events, {checkpoints} checkpoints"
                )
                .map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
                if let Some(value) = report.get("value") {
                    writeln!(stdout, "value: {value}").map_err(|source| CliError::Io {
                        path: "<stdout>".into(),
                        source,
                    })?;
                }
            }
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas replay`"
            )));
        }
    }
    Ok(CliExit::Success)
}
