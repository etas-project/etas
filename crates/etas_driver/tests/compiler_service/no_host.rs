use etas_host::{HostError, HostErrorCode, execution::OperationContext};
use etas_interpreter::host::{HostFuture, HostServiceAvailability, HostServices};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub(super) struct NoHost {
    calls: AtomicUsize,
    checkpoint: bool,
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
        console: etas_host::console::ConsoleRequest => etas_host::console::ConsoleResponse,
        approval: etas_host::ApprovalRequest => etas_host::ApprovalResponse,
        policy: etas_host::PolicyEvaluationRequest => etas_host::PolicyResponse,
    }
}
