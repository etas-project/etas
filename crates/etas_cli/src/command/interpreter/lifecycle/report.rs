use crate::{
    args::global::{GlobalOptions, OutputFormat},
    error::CliError,
    exit::CliExit,
};
use etas_host::execution::{CancellationReason, ExternalOutcome, PendingWork};
use etas_interpreter::api::{RunOutcome, RunResult, codec::JsonDocument};

pub(crate) enum ControlledRun {
    Finished(Box<RunResult>),
    Forced(ForcedShutdown),
}

pub(crate) struct ForcedShutdown {
    pub(super) exit: CliExit,
    pub(super) trigger: &'static str,
    pub(super) pending: PendingWork,
    pub(super) events: Vec<JsonDocument>,
}

impl ForcedShutdown {
    pub(crate) fn json(&self) -> JsonDocument {
        let operations: Vec<_> = self.pending.operations().iter().map(|op| serde_json::json!({
            "id": op.id().as_u64(), "request_id": op.request().map(|id| id.0),
            "dispatched": op.dispatched(), "owner_lost": op.owner_lost(),
            "completed_units": op.completed_units(), "external_outcome": external_outcome(op.outcome()),
        })).collect();
        let mut report = JsonDocument::from(serde_json::json!({
            "schema": "etas.cli.interpreter-report.v2", "incomplete": true,
            "outcome": {"kind": "cancelled"}, "termination": null, "shutdown": "pending",
            "trigger": self.trigger, "exit_code": self.exit.code(), "events": null,
            "pending_scopes": self.pending.scopes().len(), "pending_operations": operations,
        }));
        report["events"] = serde_json::Value::Array(
            self.events
                .iter()
                .map(|event| {
                    let mut event = event.clone();
                    std::mem::take(&mut *event)
                })
                .collect(),
        );
        report
    }
    pub(crate) fn render(
        &self,
        global: &GlobalOptions,
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<CliExit, CliError> {
        let report = self.json();
        match global.format {
            OutputFormat::Json | OutputFormat::Jsonl => write_json_line(stdout, "", &report)?,
            _ => {
                writeln!(stderr,
                "execution stopped waiting ({}); termination unconfirmed: {} scopes, {} operations pending",
                self.trigger,
                self.pending.scopes().len(),
                self.pending.operations().len()
                ).map_err(|source| CliError::Io { path: "<shutdown-report>".into(), source })?;
                write_json_line(stderr, "", &report)?;
            }
        }
        Ok(self.exit)
    }
}

pub(crate) fn write_json_line(
    output: &mut dyn std::io::Write,
    prefix: &str,
    value: &serde_json::Value,
) -> Result<(), CliError> {
    use std::io::Write;
    let mut output = std::io::BufWriter::new(output);
    let write = (|| {
        output.write_all(prefix.as_bytes())?;
        etas_interpreter::api::codec::write_json(&mut output, value)?;
        output.write_all(b"\n")?;
        output.flush()
    })();
    write.map_err(|source| CliError::Io {
        path: "<output>".into(),
        source,
    })
}

fn external_outcome(value: Option<&ExternalOutcome>) -> serde_json::Value {
    match value {
        None => serde_json::json!({"kind": "pending"}),
        Some(ExternalOutcome::Unknown) => serde_json::json!({"kind": "unknown"}),
        Some(ExternalOutcome::StorageWrite(evidence)) => {
            serde_json::json!({"kind": "storage_write", "evidence": evidence})
        }
        Some(ExternalOutcome::NotDispatched) => serde_json::json!({"kind": "not_dispatched"}),
        Some(ExternalOutcome::Confirmed) => serde_json::json!({"kind": "confirmed"}),
        Some(ExternalOutcome::Partial { completed_units }) => {
            serde_json::json!({"kind": "partial", "completed_units": completed_units})
        }
        Some(ExternalOutcome::Failed(error)) => {
            serde_json::json!({"kind": "failed", "code": error.code.as_str()})
        }
    }
}

pub(crate) fn render_cancellation(
    global: &GlobalOptions,
    result: &RunResult,
    stderr: &mut dyn std::io::Write,
) -> Result<(), CliError> {
    if !matches!(global.format, OutputFormat::Human | OutputFormat::Text) {
        return Ok(());
    }
    let RunOutcome::Cancelled(cause) = &result.outcome else {
        return Ok(());
    };
    let uncertain = result
        .termination
        .operations()
        .iter()
        .filter(|op| op.outcome().is_some_and(ExternalOutcome::is_uncertain))
        .count();
    writeln!(stderr, "execution cancelled ({:?}); local work settled: true; uncertain/partial external operations: {uncertain}", cause.reason())
        .map_err(|source| CliError::Io { path: "<stderr>".into(), source })
}

pub(crate) fn result_exit(result: &RunResult) -> CliExit {
    match &result.outcome {
        RunOutcome::Cancelled(cause) => match cause.reason() {
            CancellationReason::Interrupt => CliExit::Interrupted,
            CancellationReason::Terminate => CliExit::Terminated,
            _ => CliExit::RuntimeFailure,
        },
        RunOutcome::Failed(_) => CliExit::RuntimeFailure,
        RunOutcome::Completed(_) => {
            if super::super::has_error(&result.diagnostics)
                || result
                    .termination
                    .operations()
                    .iter()
                    .any(|op| !op.cleanup_errors().is_empty())
            {
                CliExit::RuntimeFailure
            } else {
                CliExit::Success
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use etas_host::{HostError, HostErrorCode, TraceContext, TraceId, execution::ExecutionScope};
    use etas_interpreter::api::InterpValue;

    #[test]
    fn forced_report_keeps_deep_events_stack_safe_and_marks_them_incomplete() {
        let mut payload = serde_json::Value::Bool(true);
        for _ in 0..20_000 {
            payload = serde_json::Value::Array(vec![payload]);
        }
        let scope = ExecutionScope::new_owned();
        let report = ForcedShutdown {
            exit: CliExit::Interrupted,
            trigger: "second_signal",
            pending: scope.pending().unwrap(),
            events: vec![JsonDocument::from(payload)],
        };
        let document = report.json();
        assert_eq!(document["incomplete"], true);
        assert_eq!(document["shutdown"], "pending");
        assert_eq!(document["termination"], serde_json::Value::Null);
        let mut bytes = Vec::new();
        write_json_line(&mut bytes, "", &document).unwrap();
        assert!(bytes.ends_with(b"}\n"));
        assert_eq!(bytes.iter().filter(|byte| **byte == b'[').count(), 20_002);
        assert_eq!(document, report.json());
        scope.finish_body(false).unwrap();
    }

    fn completed_with_cleanup_error() -> RunResult {
        let scope = ExecutionScope::new_owned();
        let work = scope
            .register(None, None, TraceContext::root(TraceId(1)))
            .unwrap();
        work.begin_dispatch().unwrap();
        work.complete(
            ExternalOutcome::Confirmed,
            vec![HostError::new(
                HostErrorCode::ProviderUnavailable,
                "resource cleanup failed",
            )],
        )
        .unwrap();
        scope.finish_body(true).unwrap();
        RunResult {
            outcome: RunOutcome::Completed(InterpValue::Unit),
            termination: scope.termination().unwrap().unwrap(),
            diagnostics: vec![],
            events: vec![],
            checkpoints: vec![],
        }
    }

    #[test]
    fn completed_language_result_with_cleanup_error_is_runtime_failure() {
        let result = completed_with_cleanup_error();
        assert_eq!(result_exit(&result), CliExit::RuntimeFailure);
        assert!(matches!(result.outcome, RunOutcome::Completed(_)));
    }

    #[test]
    fn cleanup_failure_exit_code_is_observed_by_parent_process() {
        const CHILD: &str = "ETAS_TEST_CLEANUP_EXIT_CHILD";
        if std::env::var_os(CHILD).is_some() {
            std::process::exit(result_exit(&completed_with_cleanup_error()).code());
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "command::interpreter::lifecycle::report::tests::cleanup_failure_exit_code_is_observed_by_parent_process"])
            .env(CHILD, "1").output().unwrap();
        assert_eq!(output.status.code(), Some(3), "{output:?}");
    }
}
