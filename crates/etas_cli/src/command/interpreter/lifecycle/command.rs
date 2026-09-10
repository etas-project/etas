use super::{
    journal::EventJournal,
    report::ControlledRun,
    run::{StopState, drive},
    signals::{SignalSource, Signals},
};
use crate::{error::CliError, exit::CliExit};
use etas_interpreter::api::RunInvocation;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

const FINALIZATION_GRACE: Duration = Duration::from_millis(250);

/// Process-level owner, retained through diagnostics, artifacts and output flush.
#[derive(Clone)]
pub(crate) struct CommandLifecycle(Rc<RefCell<State>>);

struct State {
    runtime: Option<tokio::runtime::Runtime>,
    signals: Signals,
    stop: StopState,
    trace: Option<(PathBuf, serde_json::Value)>,
    profile: Option<(PathBuf, etas_utils::ProfileHandle)>,
    pending_output: Vec<String>,
    finalization_deadline: Option<tokio::time::Instant>,
    output_deadline: Option<tokio::time::Instant>,
}

impl Drop for State {
    fn drop(&mut self) {
        // Pending blocking output is reported, never implicitly joined forever.
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

impl CommandLifecycle {
    pub(crate) fn new() -> Result<Self, CliError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| CliError::RuntimeState(format!("cannot create command runtime: {e}")))?;
        let signals = {
            let _enter = runtime.enter();
            Signals::install()?
        };
        Ok(Self(Rc::new(RefCell::new(State {
            runtime: Some(runtime),
            signals,
            stop: StopState::default(),
            trace: None,
            profile: None,
            pending_output: vec![],
            finalization_deadline: None,
            output_deadline: None,
        }))))
    }

    pub(crate) fn execute(
        &self,
        invocation: RunInvocation<'_>,
        grace_ms: u64,
        journal: &EventJournal,
    ) -> Result<ControlledRun, CliError> {
        let mut state = self
            .0
            .try_borrow_mut()
            .map_err(|_| CliError::RuntimeState("command lifecycle re-entry".into()))?;
        let State {
            runtime,
            signals,
            stop,
            ..
        } = &mut *state;
        stop.grace = Duration::from_millis(grace_ms);
        let runtime = runtime
            .as_ref()
            .ok_or_else(|| CliError::RuntimeState("command runtime closed".into()))?;
        let control = invocation.control();
        let result = runtime.block_on(drive(control, invocation.execute(), signals, stop, journal));
        if matches!(&result, Ok(ControlledRun::Forced(_))) {
            state.finalization_deadline = Some(tokio::time::Instant::now() + FINALIZATION_GRACE);
            state.output_deadline = Some(tokio::time::Instant::now() + Duration::from_millis(25));
        }
        result
    }

    pub(crate) fn exit_override(&self) -> Option<CliExit> {
        self.0.borrow().stop.first.map(|signal| signal.exit())
    }
    pub(crate) fn status(&self, exit: Option<CliExit>) -> &'static str {
        let state = self.0.borrow();
        if state.stop.forced.is_some() {
            "forced"
        } else if state.stop.first.is_some()
            || matches!(exit, Some(CliExit::Interrupted | CliExit::Terminated))
        {
            "cancelled"
        } else if exit == Some(CliExit::Success) {
            "ok"
        } else {
            "error"
        }
    }

    pub(crate) fn trace(&self, path: &Path, report: serde_json::Value) {
        self.0.borrow_mut().trace = Some((path.to_owned(), report));
    }
    pub(crate) fn profile(&self, path: &Path, profile: etas_utils::ProfileHandle) {
        self.0.borrow_mut().profile = Some((path.to_owned(), profile));
    }
    pub(crate) fn forced(&self) -> bool {
        self.0.borrow().stop.forced.is_some()
    }

    /// Called after the last user-facing output, while signals are still serviced.
    pub(crate) fn finalize(&self, exit: CliExit) -> Result<(), CliError> {
        let trace_result = self.flush_trace();
        let profile = self.0.borrow_mut().profile.take();
        let profile_result = if let Some((path, profile)) = profile {
            if let Some(mut report) = profile.finish_report(self.status(Some(exit))) {
                for span in &mut report.spans {
                    if span.parent.is_none() && span.category == "cli" {
                        span.status = match report.status.as_str() {
                            "forced" => etas_utils::ProfileSpanStatus::Forced,
                            "cancelled" => etas_utils::ProfileSpanStatus::Cancelled,
                            "ok" => etas_utils::ProfileSpanStatus::Ok,
                            _ => etas_utils::ProfileSpanStatus::Error,
                        };
                    }
                }
                self.write_json(&path, report)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        trace_result.and(profile_result)
    }
    pub(crate) fn flush_trace(&self) -> Result<(), CliError> {
        let artifact = {
            let mut state = self.0.borrow_mut();
            let pending = state.pending_output.clone();
            let forced = state.stop.forced;
            state.trace.take().map(|(path, mut report)| {
                if let Some(trigger) = forced {
                    report["incomplete"] = true.into();
                    report["output_shutdown"] =
                        serde_json::json!({"trigger": trigger, "pending": pending});
                }
                (path, report)
            })
        };
        if let Some((path, report)) = artifact {
            self.write_json(&path, report)?;
        }
        Ok(())
    }

    pub(crate) fn write_artifact(&self, path: &Path, bytes: Vec<u8>) -> Result<(), CliError> {
        let owned = path.to_owned();
        self.io(path.display().to_string(), true, move || {
            std::fs::write(owned, bytes)
        })
        .map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        })
    }

    fn write_json(
        &self,
        path: &Path,
        value: impl serde::Serialize + Send + 'static,
    ) -> Result<(), CliError> {
        let owned = path.to_owned();
        self.io(path.display().to_string(), true, move || {
            let bytes = serde_json::to_vec_pretty(&value).map_err(std::io::Error::other)?;
            std::fs::write(owned, bytes)
        })
        .map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        })
    }

    pub(crate) fn io<T: Send + 'static>(
        &self,
        label: String,
        artifact: bool,
        write: impl FnOnce() -> std::io::Result<T> + Send + 'static,
    ) -> std::io::Result<T> {
        let mut state = self
            .0
            .try_borrow_mut()
            .map_err(|_| std::io::Error::other("command output re-entry"))?;
        if state.stop.forced.is_some() && state.finalization_deadline.is_none() {
            state.finalization_deadline = Some(tokio::time::Instant::now() + FINALIZATION_GRACE);
        }
        if state.stop.forced.is_some() && state.output_deadline.is_none() {
            state.output_deadline = Some(tokio::time::Instant::now() + Duration::from_millis(25));
        }
        let State {
            runtime,
            signals,
            stop,
            finalization_deadline,
            output_deadline,
            ..
        } = &mut *state;
        let runtime = runtime
            .as_ref()
            .ok_or_else(|| std::io::Error::other("command runtime closed"))?;
        let output_limit = if artifact {
            *finalization_deadline
        } else {
            *output_deadline
        };
        if stop.forced.is_some()
            && output_limit.is_some_and(|limit| limit <= tokio::time::Instant::now())
        {
            // The job has not started; do not record it as pending external work.
            return Err(interrupted());
        }
        let result = runtime.block_on(async {
            let mut task = tokio::task::spawn_blocking(write);
            loop {
                let deadline = if stop.forced.is_some() {
                    if artifact {
                        *finalization_deadline
                    } else {
                        *output_deadline
                    }
                } else {
                    stop.deadline
                };
                tokio::select! {
                    biased;
                    _ = wait_deadline(deadline) => {
                        stop.forced.get_or_insert("grace_expired");
                        return Err(interrupted());
                    }
                    signal = signals.next() => {
                        let signal = signal.map_err(std::io::Error::other)?;
                        stop.observe(signal);
                        if stop.forced.is_some() { return Err(interrupted()); }
                    }
                    result = &mut task => return result.map_err(std::io::Error::other)?,
                }
            }
        });
        if result
            .as_ref()
            .is_err_and(|error| error.kind() == std::io::ErrorKind::TimedOut)
        {
            state.pending_output.push(label);
            if state.finalization_deadline.is_none() {
                state.finalization_deadline =
                    Some(tokio::time::Instant::now() + FINALIZATION_GRACE);
            }
        }
        result
    }
}

async fn wait_deadline(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}
fn interrupted() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "command output did not settle before forced shutdown",
    )
}

pub(crate) fn write_artifact(
    lifecycle: Option<&CommandLifecycle>,
    path: &Path,
    bytes: Vec<u8>,
) -> Result<(), CliError> {
    match lifecycle {
        Some(lifecycle) => lifecycle.write_artifact(path, bytes),
        None => std::fs::write(path, bytes).map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

pub(crate) fn save_trace(
    lifecycle: Option<&CommandLifecycle>,
    path: &Path,
    report: serde_json::Value,
) -> Result<(), CliError> {
    if let Some(lifecycle) = lifecycle {
        lifecycle.trace(path, report);
        Ok(())
    } else {
        write_artifact(
            None,
            path,
            serde_json::to_vec_pretty(&report)
                .map_err(|e| CliError::RuntimeState(e.to_string()))?,
        )
    }
}
