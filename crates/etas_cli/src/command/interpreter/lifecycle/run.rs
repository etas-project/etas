use super::{ControlledRun, ForcedShutdown, signals::SignalSource};
use crate::error::CliError;
use etas_host::execution::StopWait;
use etas_interpreter::api::{RunControl, RunInfrastructureError, RunInvocation, RunResult};
use std::{future::Future, time::Duration};

pub(crate) fn run_controlled(
    grace_ms: u64,
    invocation: RunInvocation<'_>,
    lifecycle: Option<&super::CommandLifecycle>,
    journal: &super::EventJournal,
) -> Result<ControlledRun, CliError> {
    if let Some(lifecycle) = lifecycle {
        return lifecycle.execute(invocation, grace_ms, journal);
    }
    // Library CLI callers own their own interrupt policy; never install process handlers.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::RuntimeState(format!("cannot create execution runtime: {e}")))?;
    let result = runtime.block_on(invocation.execute());
    runtime.shutdown_background();
    finished(result)
}

pub(super) struct StopState {
    pub first: Option<super::signals::Signal>,
    pub deadline: Option<tokio::time::Instant>,
    pub grace: Duration,
    pub forced: Option<&'static str>,
}
impl Default for StopState {
    fn default() -> Self {
        Self {
            first: None,
            deadline: None,
            grace: Duration::from_millis(5000),
            forced: None,
        }
    }
}
impl StopState {
    pub(super) fn observe(&mut self, signal: super::signals::Signal) {
        if self.first.is_some() {
            self.forced = Some("second_signal");
        } else {
            self.first = Some(signal);
            self.deadline = Some(tokio::time::Instant::now() + self.grace);
        }
    }
}

pub(super) async fn drive(
    control: RunControl,
    run: impl Future<Output = Result<RunResult, RunInfrastructureError>>,
    signals: &mut impl SignalSource,
    stop: &mut StopState,
    journal: &super::EventJournal,
) -> Result<ControlledRun, CliError> {
    tokio::pin!(run);
    let signal = if let Some(signal) = stop.first {
        signal
    } else {
        let signal = tokio::select! {
            result = &mut run => return finished(result),
            signal = signals.next() => signal?,
        };
        stop.observe(signal);
        signal
    };
    control
        .stop(signal.reason())
        .map_err(|e| CliError::RuntimeState(e.to_string()))?;
    let trigger = tokio::select! {
        result = &mut run => return finished(result),
        next = signals.next() => { stop.observe(next?); "second_signal" },
        _ = tokio::time::sleep_until(stop.deadline.ok_or_else(|| CliError::RuntimeState("stop has no deadline".into()))?) => "grace_expired",
    };
    let pending = match control
        .wait_stopped(tokio::time::Instant::now())
        .await
        .map_err(|e| CliError::RuntimeState(e.to_string()))?
    {
        StopWait::Terminated(_) => return finished(run.await),
        StopWait::TimedOut(pending) => pending,
    };
    stop.forced = Some(trigger);
    Ok(ControlledRun::Forced(ForcedShutdown {
        exit: signal.exit(),
        trigger,
        pending,
        events: journal.snapshot()?,
    }))
}

fn finished(result: Result<RunResult, RunInfrastructureError>) -> Result<ControlledRun, CliError> {
    result
        .map(|result| ControlledRun::Finished(Box::new(result)))
        .map_err(|error| CliError::RuntimeState(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::signals::Signal;
    use super::*;
    use etas_frontend::{
        Frontend, ModulePath, ProjectCompileOptions, ProjectEntry, ProjectEnvironmentInput,
        ProjectInput, SourceInput, SourceKind,
    };
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, RunOptions},
    };

    struct TestSignals(Vec<Signal>);
    impl SignalSource for TestSignals {
        async fn next(&mut self) -> Result<Signal, CliError> {
            match self.0.pop() {
                Some(signal) => Ok(signal),
                None => std::future::pending().await,
            }
        }
    }

    async fn assert_forced(signals: Vec<Signal>, expected: &str) {
        let checked = Frontend
            .check_project(ProjectInput {
                project_root: "/workspace/test".into(),
                source_root: Some("/workspace/test/src".into()),
                options: ProjectCompileOptions::default(),
                environment: ProjectEnvironmentInput::default(),
                sources: vec![SourceInput {
                    id: etas_core::SourceId(0),
                    path: Some("/workspace/test/src/main.es".into()),
                    text: "module main; flow main() -> unit { return; }".into(),
                    kind: SourceKind::SourceProjectFile,
                }],
                entry: ProjectEntry {
                    module: Some(ModulePath {
                        segments: vec!["main".into()],
                    }),
                    flow: "main".into(),
                },
            })
            .checked
            .unwrap();
        let host = crate::command::interpreter::host::CliHost::dry_run(
            etas_utils::profile::ProfileHandle::disabled(),
        );
        let invocation = Interpreter.create_run(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            RunOptions::default(),
        );
        let control = invocation.control();
        // Isolate CLI deadline arbitration; an unpolled owner remains pending.
        let report = drive(
            control.clone(),
            std::future::pending(),
            &mut TestSignals(signals),
            &mut StopState {
                grace: Duration::from_millis(1),
                ..StopState::default()
            },
            &super::super::EventJournal::default(),
        )
        .await
        .unwrap();
        let ControlledRun::Forced(report) = report else {
            panic!("unfinished work must not become a completed run");
        };
        assert_eq!(report.trigger, expected);
        assert_eq!(report.exit, crate::exit::CliExit::Interrupted);
        assert_eq!(report.pending.scopes().len(), 1);
        assert_eq!(
            control.status().unwrap(),
            etas_host::execution::ScopeState::Stopping
        );
        drop(invocation);
        control.join().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn grace_expiry_preserves_pending_work_and_signal_status() {
        assert_forced(vec![Signal::Interrupt], "grace_expired").await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn second_signal_does_not_rewrite_initiating_signal_or_claim_termination() {
        assert_forced(vec![Signal::Terminate, Signal::Interrupt], "second_signal").await;
    }
}
