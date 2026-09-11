use crate::{error::CliError, exit::CliExit};
use etas_host::execution::CancellationReason;

#[derive(Clone, Copy, Debug)]
pub(super) enum Signal {
    Interrupt,
    Terminate,
}

pub(super) trait SignalSource {
    fn next(&mut self) -> impl std::future::Future<Output = Result<Signal, CliError>>;
}

impl Signal {
    pub(super) fn reason(self) -> CancellationReason {
        match self {
            Self::Interrupt => CancellationReason::Interrupt,
            Self::Terminate => CancellationReason::Terminate,
        }
    }
    pub(super) fn exit(self) -> CliExit {
        match self {
            Self::Interrupt => CliExit::Interrupted,
            Self::Terminate => CliExit::Terminated,
        }
    }
}

#[cfg(unix)]
pub(super) struct Signals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl Signals {
    pub(super) fn install() -> Result<Self, CliError> {
        use tokio::signal::unix::{SignalKind, signal};
        let error =
            |e| CliError::RuntimeState(format!("cannot install execution signal listener: {e}"));
        Ok(Self {
            interrupt: signal(SignalKind::interrupt()).map_err(error)?,
            terminate: signal(SignalKind::terminate()).map_err(error)?,
        })
    }
}

#[cfg(unix)]
impl SignalSource for Signals {
    async fn next(&mut self) -> Result<Signal, CliError> {
        let (event, signal) = tokio::select! {
            value = self.interrupt.recv() => (value, Signal::Interrupt),
            value = self.terminate.recv() => (value, Signal::Terminate),
        };
        event
            .ok_or_else(|| CliError::RuntimeState("execution signal channel closed".to_owned()))?;
        Ok(signal)
    }
}

#[cfg(not(unix))]
pub(super) struct Signals;
#[cfg(not(unix))]
impl Signals {
    pub(super) fn install() -> Result<Self, CliError> {
        Ok(Self)
    }
}
#[cfg(not(unix))]
impl SignalSource for Signals {
    async fn next(&mut self) -> Result<Signal, CliError> {
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| CliError::RuntimeState(e.to_string()))?;
        Ok(Signal::Interrupt)
    }
}
