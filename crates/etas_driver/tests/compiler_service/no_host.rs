use etas_host::{HostError, HostErrorCode, execution::OperationContext};
use etas_interpreter::host::{HostFuture, HostServiceAvailability, HostServices};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
pub(super) struct NoHost {
    calls: AtomicUsize,
    checkpoint: bool,
    console_enabled: bool,
    console_events: Mutex<Vec<etas_host::console::ConsoleOperation>>,
}
impl NoHost {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
    pub(super) fn with_checkpoint() -> Self {
        Self {
            checkpoint: true,
            ..Self::default()
        }
    }
    pub(super) fn with_console() -> Self {
        Self {
            console_enabled: true,
            ..Self::default()
        }
    }
    pub(super) fn console_events(&self) -> Vec<etas_host::console::ConsoleOperation> {
        self.console_events.lock().unwrap().clone()
    }
}

macro_rules! reject_requests {
    ($($method:ident: $request:path => $response:path),* $(,)?) => { $(
        fn $method<'a>(&'a self, _: OperationContext, _: $request) -> HostFuture<'a, Result<$response, HostError>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(HostError::new(HostErrorCode::ProviderUnavailable, "pure candidate unexpectedly requested a Host service")) })
        }
    )* };
}

impl HostServices for NoHost {
    fn availability(&self) -> HostServiceAvailability {
        HostServiceAvailability {
            checkpoint: self.checkpoint,
            console: self.console_enabled,
            ..Default::default()
        }
    }
    reject_requests! {
        compiler: etas_host::compiler::CompilerRequest => etas_host::compiler::CompilerResponse,
        model: etas_host::ModelRequest => etas_host::ModelResponse,
        tool: etas_host::ToolRequest => etas_host::ToolResponse,
        memory: etas_host::MemoryRequest => etas_host::MemoryResponse,
        memory_write: etas_host::memory::MemoryWriteRequest => etas_host::memory::MemoryWriteResponse,
        session: etas_host::SessionRequest => etas_host::SessionResponse,
        session_write: etas_host::session::SessionWriteRequest => etas_host::session::SessionWriteResponse,
        filesystem: etas_host::FilesystemRequest => etas_host::FilesystemResponse,
        command: etas_host::CommandRequest => etas_host::CommandResponse,
        tcp: etas_host::TcpConnectRequest => etas_host::TcpConnectResponse,
        stream: etas_host::StreamRequest => etas_host::StreamResponse,
        tls: etas_host::TlsConnectRequest => etas_host::TlsConnectResponse,
        secret: etas_host::SecretRequest => etas_host::SecretResponse,
        browser: etas_host::BrowserProtocolRequest => etas_host::BrowserProtocolResponse,
        approval: etas_host::ApprovalRequest => etas_host::ApprovalResponse,
        policy: etas_host::PolicyEvaluationRequest => etas_host::PolicyResponse,
    }

    fn console<'a>(
        &'a self,
        _: OperationContext,
        request: etas_host::console::ConsoleRequest,
    ) -> HostFuture<'a, Result<etas_host::console::ConsoleResponse, HostError>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            if !self.console_enabled {
                return Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "pure candidate unexpectedly requested a Host service",
                ));
            }
            match request.operation {
                operation @ (etas_host::console::ConsoleOperation::WriteStdout { .. }
                | etas_host::console::ConsoleOperation::WriteStderr { .. }) => {
                    self.console_events.lock().unwrap().push(operation);
                    Ok(etas_host::console::ConsoleResponse {
                        id: request.id,
                        result: etas_host::console::ConsoleResult::Written,
                    })
                }
                _ => Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "console input is not configured for this test",
                )),
            }
        })
    }
}
