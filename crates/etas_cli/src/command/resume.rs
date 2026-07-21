use std::io::Write;

use etas_core::{Diagnostic, DiagnosticCode, EffectDiagnosticCode};
use etas_interpreter::api::codec;
use etas_utils::ProfileHandle;

use crate::{
    args::{ResumeArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    global: &GlobalOptions,
    args: ResumeArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let profile = ProfileHandle::disabled();
    let checkpoint_dir = args
        .checkpoint_dir
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let checkpoint_id = args
        .checkpoint_id
        .parse::<u32>()
        .map_err(|_| CliError::InvalidUsage("checkpoint id must be a numeric id".to_owned()))?;
    let checkpoint_path = super::interpreter::checkpoint_path(&checkpoint_dir, checkpoint_id);
    let bytes = std::fs::read(&checkpoint_path).map_err(|source| CliError::Io {
        path: checkpoint_path.clone(),
        source,
    })?;
    let checkpoint_json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| CliError::InvalidUsage("checkpoint artifact is not valid JSON".to_owned()))?;
    let (source_paths, flow) = codec::sources_and_flow_from_checkpoint_json(&checkpoint_json)
        .map_err(|error| CliError::InvalidUsage(error.to_string()))?;
    let checkpoint_profile = checkpoint_json.get("runtime_profile").ok_or_else(|| {
        CliError::InvalidUsage("checkpoint artifact is missing runtime profile metadata".to_owned())
    })?;
    let checkpoint_profile_name = checkpoint_profile
        .get("profile")
        .and_then(serde_json::Value::as_str);
    let checkpoint_max_call_depth = checkpoint_profile
        .get("execution")
        .and_then(|execution| execution.get("max_call_depth"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            CliError::InvalidUsage(
                "checkpoint runtime profile is missing a valid execution.max_call_depth".to_owned(),
            )
        })?;
    let checkpoint_max_call_depth = u32::try_from(checkpoint_max_call_depth).map_err(|_| {
        CliError::InvalidUsage(
            "checkpoint runtime profile execution.max_call_depth is too large".to_owned(),
        )
    })?;
    let requested_profile = args.profile.as_deref().or(checkpoint_profile_name);
    let host_config = super::interpreter::runtime_config_for_run(
        &source_paths,
        Some(&flow),
        requested_profile,
        args.runtime_config.as_deref(),
        &[],
        args.max_call_depth.or(Some(checkpoint_max_call_depth)),
    )?;
    let current_profile = host_config.runtime_profile_json();
    if checkpoint_profile != &current_profile {
        return Err(CliError::InvalidUsage(
            "current host runtime profile is not compatible with the checkpoint".to_owned(),
        ));
    }
    let compiled = super::interpreter::compile_project(
        global,
        &profile,
        source_paths.clone(),
        false,
        Some(flow.clone()),
    )?;
    let has_compile_error = super::interpreter::has_error(&compiled.diagnostics);
    let visible_compile_diagnostics = visible_resume_diagnostics(global, &compiled.diagnostics);
    super::interpreter::render_diagnostics(
        global,
        stderr,
        &compiled.sources,
        if has_compile_error {
            &compiled.diagnostics
        } else {
            &visible_compile_diagnostics
        },
    )?;
    if has_compile_error {
        return Ok(CliExit::Diagnostics);
    }
    let Some(checked) = compiled.checked.as_ref() else {
        return Ok(CliExit::Diagnostics);
    };
    let checkpoint = codec::checkpoint_from_json(&checkpoint_json, checked)
        .map_err(|error| CliError::InvalidUsage(error.to_string()))?;

    let result = super::interpreter::resume_checked(checked, &checkpoint, host_config)?;
    super::interpreter::render_diagnostics(global, stderr, &compiled.sources, &result.diagnostics)?;
    let report = codec::run_report_json("resume", &source_paths, &flow, &result);

    match global.format {
        crate::args::global::OutputFormat::Json => {
            writeln!(stdout, "{report}").map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            writeln!(
                stdout,
                "{}",
                serde_json::json!({
                    "type": "resume_result",
                    "checkpoint": checkpoint_id,
                    "value": result.value.as_ref().map(codec::value_json),
                    "events": result.events.len(),
                    "checkpoints": result.checkpoints.len(),
                })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            if !global.quiet {
                let value = result
                    .value
                    .as_ref()
                    .map(codec::value_json)
                    .unwrap_or(serde_json::Value::Null);
                writeln!(stdout, "resumed checkpoint {checkpoint_id}: {value}").map_err(
                    |source| CliError::Io {
                        path: "<stdout>".into(),
                        source,
                    },
                )?;
                writeln!(
                    stdout,
                    "events: {}, checkpoints: {}",
                    result.events.len(),
                    result.checkpoints.len()
                )
                .map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
            }
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas resume`"
            )));
        }
    }

    if super::interpreter::has_error(&result.diagnostics) {
        Ok(CliExit::RuntimeFailure)
    } else {
        Ok(CliExit::Success)
    }
}

fn visible_resume_diagnostics(
    global: &GlobalOptions,
    diagnostics: &[Diagnostic],
) -> Vec<Diagnostic> {
    if global.verbose > 0 {
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
