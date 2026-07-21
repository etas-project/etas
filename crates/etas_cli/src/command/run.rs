use std::io::Write;

use etas_core::{Diagnostic, DiagnosticCode, EffectDiagnosticCode};
use etas_interpreter::api::{RunResult, codec, entry_requires_console};
use etas_utils::ProfileHandle;

use crate::{
    args::{global::GlobalOptions, run::RunArgs},
    error::CliError,
    exit::CliExit,
};

pub fn run(
    global: &GlobalOptions,
    args: RunArgs,
    profile: &ProfileHandle,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    validate_run_args(&args)?;
    let flow = args.flow.clone().unwrap_or_else(|| "main".to_owned());
    let host_config = super::interpreter::runtime_config_for_run(
        &args.input,
        Some(&flow),
        args.profile.as_deref(),
        args.runtime_config.as_deref(),
        &args.allow_net,
        args.max_call_depth,
    )?;
    if args.print_runtime_profile {
        writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&host_config.runtime_profile_json())
                .expect("runtime profile json serializes")
        )
        .map_err(|source| CliError::Io {
            path: std::path::PathBuf::from("<stdout>"),
            source,
        })?;
        return Ok(CliExit::Success);
    }
    let compiled = super::interpreter::compile_project(
        global,
        profile,
        args.input.clone(),
        args.all,
        Some(flow.clone()),
    )?;
    let has_compile_error = super::interpreter::has_error(&compiled.diagnostics);
    let visible_compile_diagnostics = visible_run_diagnostics(global, &compiled.diagnostics);
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
        render_compile_summary(global, stdout, &compiled)?;
        return Ok(CliExit::Diagnostics);
    }
    let Some(checked) = compiled.checked.as_ref() else {
        render_compile_summary(global, stdout, &compiled)?;
        return Ok(CliExit::Diagnostics);
    };

    let result = super::interpreter::run_checked(
        checked,
        args.dry_run,
        args.allow_effects,
        host_config.clone(),
        run_budget_overrides(&args)?,
        args.program_args.clone(),
        profile,
    )?;
    super::interpreter::render_diagnostics(global, stderr, &compiled.sources, &result.diagnostics)?;
    let source_paths = compiled
        .sources
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    if let Some(dir) = &args.checkpoint_dir {
        let runtime_profile = host_config.runtime_profile_json();
        super::interpreter::write_checkpoint_files(
            dir,
            &source_paths,
            &flow,
            &result,
            &runtime_profile,
        )?;
    }

    let report = codec::run_report_json("run", &source_paths, &flow, &result);
    if let Some(path) = &args.trace_out {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&report).expect("json serializes"),
        )
        .map_err(|source| CliError::Io {
            path: path.clone(),
            source,
        })?;
    }

    render_run_result(global, stdout, &compiled, &result, &report)?;

    if super::interpreter::has_error(&result.diagnostics) {
        Ok(CliExit::RuntimeFailure)
    } else {
        Ok(CliExit::Success)
    }
}

fn validate_run_args(args: &RunArgs) -> Result<(), CliError> {
    if args.dry_run && args.allow_effects {
        return Err(CliError::InvalidUsage(
            "`etas run --dry-run` cannot be combined with `--allow-effects`".to_owned(),
        ));
    }
    Ok(())
}

fn run_budget_overrides(
    args: &RunArgs,
) -> Result<super::interpreter::RunBudgetOverrides, CliError> {
    Ok(super::interpreter::RunBudgetOverrides {
        tokens: args.budget_tokens,
        cost: args
            .budget_cost
            .as_deref()
            .map(parse_cost_budget)
            .transpose()?,
        time: args
            .budget_time
            .as_deref()
            .map(parse_time_budget)
            .transpose()?,
    })
}

fn parse_time_budget(value: &str) -> Result<u64, CliError> {
    if let Some(milliseconds) = value.strip_suffix("ms") {
        return parse_u64(milliseconds, "--budget-time");
    }
    if let Some(seconds) = value.strip_suffix('s') {
        return parse_u64(seconds, "--budget-time").and_then(|seconds| {
            seconds.checked_mul(1000).ok_or_else(|| {
                CliError::InvalidUsage("`--budget-time` value is too large".to_owned())
            })
        });
    }
    parse_u64(value, "--budget-time")
}

fn parse_cost_budget(value: &str) -> Result<(u128, String), CliError> {
    let (currency, amount) = value.split_once(':').unwrap_or(("USD", value));
    if currency.is_empty() {
        return Err(CliError::InvalidUsage(
            "`--budget-cost` currency must not be empty".to_owned(),
        ));
    }
    let amount = amount.replace('_', "").parse::<u128>().map_err(|_| {
        CliError::InvalidUsage(
            "`--budget-cost` must be an integer micro-unit amount or CURRENCY:AMOUNT".to_owned(),
        )
    })?;
    Ok((amount, currency.to_owned()))
}

fn parse_u64(value: &str, flag: &str) -> Result<u64, CliError> {
    value
        .replace('_', "")
        .parse::<u64>()
        .map_err(|_| CliError::InvalidUsage(format!("`{flag}` must be a non-negative integer")))
}

fn visible_run_diagnostics(global: &GlobalOptions, diagnostics: &[Diagnostic]) -> Vec<Diagnostic> {
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

fn render_compile_summary(
    global: &GlobalOptions,
    stdout: &mut dyn Write,
    compiled: &super::interpreter::CompiledRunInput,
) -> Result<(), CliError> {
    match global.format {
        crate::args::global::OutputFormat::Json => {
            writeln!(
                stdout,
                "{}",
                serde_json::json!({
                    "command": "run",
                    "diagnostics": super::interpreter::diagnostics_json(&compiled.diagnostics, &compiled.sources),
                })
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            for diagnostic in &compiled.diagnostics {
                let Some(diagnostic_json) =
                    super::interpreter::diagnostic_json(diagnostic, &compiled.sources)
                else {
                    continue;
                };
                writeln!(
                    stdout,
                    "{}",
                    serde_json::json!({
                        "type": "diagnostic",
                        "diagnostic": diagnostic_json,
                    })
                )
                .map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
            }
        }
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {}
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas run`"
            )));
        }
    }
    Ok(())
}

fn render_run_result(
    global: &GlobalOptions,
    stdout: &mut dyn Write,
    compiled: &super::interpreter::CompiledRunInput,
    result: &RunResult,
    report: &serde_json::Value,
) -> Result<(), CliError> {
    match global.format {
        crate::args::global::OutputFormat::Json => {
            writeln!(stdout, "{report}").map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            for diagnostic in &result.diagnostics {
                let Some(diagnostic_json) =
                    super::interpreter::diagnostic_json(diagnostic, &compiled.sources)
                else {
                    continue;
                };
                writeln!(
                    stdout,
                    "{}",
                    serde_json::json!({
                        "type": "diagnostic",
                        "diagnostic": diagnostic_json,
                    })
                )
                .map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
            }
            writeln!(
                stdout,
                "{}",
                serde_json::json!({
                    "type": "run_result",
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
            if !global.quiet
                && !compiled
                    .checked
                    .as_ref()
                    .is_some_and(entry_requires_console)
            {
                let value = result
                    .value
                    .as_ref()
                    .map(codec::value_json)
                    .unwrap_or(serde_json::Value::Null);
                writeln!(stdout, "run value: {value}").map_err(|source| CliError::Io {
                    path: "<stdout>".into(),
                    source,
                })?;
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
                "`--format {other:?}` is not valid for `etas run`"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_budget_accepts_milliseconds_and_seconds() {
        assert_eq!(parse_time_budget("250").unwrap(), 250);
        assert_eq!(parse_time_budget("250ms").unwrap(), 250);
        assert_eq!(parse_time_budget("2s").unwrap(), 2000);
    }

    #[test]
    fn parse_cost_budget_defaults_to_usd_micro_units() {
        assert_eq!(
            parse_cost_budget("1_500").unwrap(),
            (1500, "USD".to_owned())
        );
        assert_eq!(
            parse_cost_budget("EUR:2500").unwrap(),
            (2500, "EUR".to_owned())
        );
    }
}
