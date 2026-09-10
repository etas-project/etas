use crate::{
    args::global::{GlobalOptions, OutputFormat},
    error::CliError,
    exit::CliExit,
};
use etas_host::execution::{CancellationReason, ExternalOutcome, PendingWork};
use etas_interpreter::api::{RunOutcome, RunResult};

pub(crate) enum ControlledRun {
    Finished(Box<RunResult>),
    Forced(ForcedShutdown),
}

pub(crate) struct ForcedShutdown {
    pub(super) exit: CliExit,
    pub(super) trigger: &'static str,
    pub(super) pending: PendingWork,
    pub(super) events: Vec<serde_json::Value>,
}

impl ForcedShutdown {
    pub(crate) fn json(&self) -> serde_json::Value {
        let operations: Vec<_> = self.pending.operations().iter().map(|op| serde_json::json!({
            "id": op.id().as_u64(), "request_id": op.request().map(|id| id.0),
            "dispatched": op.dispatched(), "owner_lost": op.owner_lost(),
            "completed_units": op.completed_units(), "external_outcome": external_outcome(op.outcome()),
        })).collect();
        serde_json::json!({
            "schema": "etas.cli.interpreter-report.v1", "incomplete": true,
            "outcome": {"kind": "cancelled"}, "termination": null, "shutdown": "pending",
            "trigger": self.trigger, "exit_code": self.exit.code(), "events": self.events,
            "pending_scopes": self.pending.scopes().len(), "pending_operations": operations,
        })
    }
    pub(crate) fn render(
        &self,
        global: &GlobalOptions,
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
    ) -> Result<CliExit, CliError> {
        let report = self.json();
        let written = match global.format {
            OutputFormat::Json | OutputFormat::Jsonl => writeln!(stdout, "{report}"),
            _ => writeln!(
                stderr,
                "execution stopped waiting ({}); termination unconfirmed: {} scopes, {} operations pending\n{report}",
                self.trigger,
                self.pending.scopes().len(),
                self.pending.operations().len()
            ),
        };
        written.map_err(|source| CliError::Io {
            path: "<shutdown-report>".into(),
            source,
        })?;
        Ok(self.exit)
    }
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
