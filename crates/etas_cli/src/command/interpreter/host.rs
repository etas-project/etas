use std::future::Future;

use etas_effects::{RequirementFact, TraceSpecClauseFact};
use etas_frontend::CheckedProject;
use etas_host::console::{ConsoleClient, ConsoleRequest, ConsoleResponse, LocalStdioClient};
use etas_host::{
    ApprovalDecision, ApprovalRequest, BrowserProtocolRequest, BrowserProtocolResponse, Budget,
    CommandClient, CommandRequest, CommandResponse, CostBudget, FilesystemClient,
    FilesystemRequest, FilesystemResponse, HostError, HostErrorCode, HostRequestId, HostValue,
    InMemoryMemoryClient, InMemorySessionClient, LocalCommandClient, LocalFilesystemClient,
    LocalStreamClient, LocalTcpClient, LocalTlsClient, MemoryClient, MemoryRequest, MemoryResponse,
    ModelClient, ModelRequest, ModelResponse, PolicyClient, PolicyDecision,
    PolicyEvaluationRequest, PolicyResponse, SecretOperation, SecretPayload, SecretRef,
    SecretRequest, SecretResponse, SecretValue, SessionClient, SessionRequest, SessionResponse,
    SqliteMemoryClient, SqliteSessionClient, StreamClient, StreamRequest, StreamResponse,
    TRACE_SPEC_RUNTIME_REF, TcpClient, TcpConnectRequest, TcpConnectResponse, TimeBudget,
    TlsClient, TlsConnectRequest, TlsConnectResponse, TokenBudget, ToolRequest, ToolResponse,
    TraceSpecRuntimeClient, UnsafeAllowAllLocalPolicyClient,
};
use etas_interpreter::{
    api::{
        EntryPoint, InterpreterCheckpoint, RunOptions, RunResult, entry_args_from_strings,
        resume_checkpoint_blocking, run_checked_blocking,
    },
    host::{HostFuture, HostServiceAvailability, HostServices},
};
use etas_utils::{ProfileHandle, ProfileSpanStatus};

use crate::error::CliError;

use super::{
    host_config::{
        ApprovalMode, CliHostConfig, CliModelAdapter, FilesystemMode, MemoryMode, PolicyMode,
        SecretMode, SessionMode,
    },
    host_tool::{CliToolRouter, unknown_tool_error},
};

#[derive(Clone)]
struct CliHost {
    availability: HostServiceAvailability,
    model: Option<CliModelAdapter>,
    tool: CliToolRouter,
    memory: Option<CliMemoryClient>,
    session: Option<CliSessionClient>,
    filesystem: Option<LocalFilesystemClient>,
    tcp: Option<LocalTcpClient>,
    stream: Option<LocalStreamClient>,
    tls: Option<LocalTlsClient>,
    secret: Option<EnvSecretClient>,
    command: Option<LocalCommandClient>,
    approval: ApprovalMode,
    policy: PolicyMode,
    policy_local: Option<etas_host::LocalStaticPolicyClient>,
    policy_http: Option<etas_host::HttpPolicyClient>,
    active_trace_specs: Vec<String>,
    trace_spec_facts: Vec<HostValue>,
    unavailable: UnavailableHostServices,
    profile: ProfileHandle,
}

#[derive(Clone, Copy, Default)]
struct UnavailableHostServices;

#[derive(Clone, Debug)]
enum CliMemoryClient {
    InMemory(InMemoryMemoryClient),
    Sqlite(SqliteMemoryClient),
}

#[derive(Clone, Debug)]
enum CliSessionClient {
    InMemory(InMemorySessionClient),
    Sqlite(SqliteSessionClient),
}

impl CliHost {
    fn dry_run(profile: ProfileHandle) -> Self {
        Self {
            availability: HostServiceAvailability::default(),
            model: None,
            tool: CliToolRouter::default(),
            memory: None,
            session: None,
            filesystem: None,
            tcp: None,
            stream: None,
            tls: None,
            secret: None,
            command: None,
            approval: ApprovalMode::Deny,
            policy: PolicyMode::DenyUnknown,
            policy_local: None,
            policy_http: None,
            active_trace_specs: Vec::new(),
            trace_spec_facts: Vec::new(),
            unavailable: UnavailableHostServices,
            profile,
        }
    }

    fn console(
        config: CliHostConfig,
        active_trace_specs: Vec<String>,
        trace_spec_facts: Vec<HostValue>,
        profile: ProfileHandle,
    ) -> Result<Self, CliError> {
        let tool =
            CliToolRouter::from_bindings(&config.tool_bindings, config.tool_private_resolution)?;
        let memory = memory_client(&config.memory_mode)?;
        let session = session_client(&config.session_mode)?;
        let byte_streams =
            (!config.program_network_allowlist.is_empty()).then(etas_host::ByteStreamStore::new);
        Ok(Self {
            availability: HostServiceAvailability {
                model: config.model.is_some(),
                console: true,
                tool: !tool.is_empty(),
                memory: config.memory_mode != MemoryMode::None,
                session: config.session_mode != SessionMode::None,
                filesystem: config.filesystem_mode != FilesystemMode::None,
                command: !config.command_allowed_programs.is_empty(),
                network: !config.program_network_allowlist.is_empty(),
                tcp: !config.program_network_allowlist.is_empty(),
                stream: !config.program_network_allowlist.is_empty(),
                tls: !config.program_network_allowlist.is_empty(),
                browser: false,
                secret_access: config.secret_mode != SecretMode::None,
                approval: config.approval_mode != ApprovalMode::Deny,
                policy: config.policy_mode != PolicyMode::DenyUnknown
                    || config.policy_local.is_some()
                    || config.policy_http.is_some()
                    || !trace_spec_facts.is_empty(),
                ..HostServiceAvailability::default()
            },
            model: config.model.map(|model| model.adapter),
            tool,
            memory,
            session,
            filesystem: (config.filesystem_mode != FilesystemMode::None)
                .then(LocalFilesystemClient::new),
            tcp: byte_streams.clone().map(LocalTcpClient::new),
            stream: byte_streams.clone().map(LocalStreamClient::new),
            tls: byte_streams.map(LocalTlsClient::new),
            secret: (config.secret_mode == SecretMode::Env).then_some(EnvSecretClient),
            command: (!config.command_allowed_programs.is_empty()).then(LocalCommandClient::new),
            approval: config.approval_mode,
            policy: config.policy_mode,
            policy_local: config.policy_local,
            policy_http: config.policy_http.map(|policy| policy.client),
            active_trace_specs,
            trace_spec_facts,
            unavailable: UnavailableHostServices,
            profile,
        })
    }

    fn profiled<'a, T, F>(
        &'a self,
        name: &'static str,
        future: F,
    ) -> HostFuture<'a, Result<T, HostError>>
    where
        T: Send + 'a,
        F: Future<Output = Result<T, HostError>> + Send + 'a,
    {
        let profile = self.profile.clone();
        Box::pin(async move {
            let mut span = profile.span(name, "host");
            let result = future.await;
            if result.is_ok() {
                span.finish(ProfileSpanStatus::Ok);
            } else {
                span.finish(ProfileSpanStatus::Error);
            }
            result
        })
    }
}

fn memory_client(mode: &MemoryMode) -> Result<Option<CliMemoryClient>, CliError> {
    match mode {
        MemoryMode::None => Ok(None),
        MemoryMode::Memory => Ok(Some(CliMemoryClient::InMemory(InMemoryMemoryClient::new()))),
        MemoryMode::Sqlite { path } => SqliteMemoryClient::open(path)
            .map(CliMemoryClient::Sqlite)
            .map(Some)
            .map_err(|error| {
                CliError::InvalidUsage(format!(
                    "failed to open SQLite memory backend `{}`: {}",
                    path.display(),
                    error.message
                ))
            }),
    }
}

fn session_client(mode: &SessionMode) -> Result<Option<CliSessionClient>, CliError> {
    match mode {
        SessionMode::None => Ok(None),
        SessionMode::Memory => Ok(Some(CliSessionClient::InMemory(
            InMemorySessionClient::new(),
        ))),
        SessionMode::Sqlite { path } => SqliteSessionClient::open(path)
            .map(CliSessionClient::Sqlite)
            .map(Some)
            .map_err(|error| {
                CliError::InvalidUsage(format!(
                    "failed to open SQLite session backend `{}`: {}",
                    path.display(),
                    error.message
                ))
            }),
    }
}

#[derive(Clone, Debug, Default)]
struct EnvSecretClient;

impl EnvSecretClient {
    async fn execute(&self, request: SecretRequest) -> Result<SecretResponse, HostError> {
        let result = match &request.operation {
            SecretOperation::Read { key } => match std::env::var(key) {
                Ok(_) => Ok(SecretPayload::Value(SecretValue::new(
                    SecretRef::new(key.clone()),
                    format!("{key}=<redacted>"),
                ))),
                Err(error) => Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "secret environment variable is not available",
                )
                .with_detail("key", key.clone())
                .with_detail("error", error.to_string())),
            },
            SecretOperation::HmacSha256 { key, body } => match std::env::var(key.id()) {
                Ok(secret) => Ok(SecretPayload::Bytes(
                    hmac_sha256(secret.as_bytes(), body).to_vec(),
                )),
                Err(error) => Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "secret environment variable is not available",
                )
                .with_detail("key", key.id().to_owned())
                .with_detail("error", error.to_string())),
            },
        };
        Ok(SecretResponse {
            id: request.id,
            result,
        })
    }
}

fn hmac_sha256(key: &[u8], body: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut key_block = [0u8; BLOCK];
    if key.len() > BLOCK {
        key_block[..32].copy_from_slice(&sha256(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut inner = [0x36u8; BLOCK];
    let mut outer = [0x5cu8; BLOCK];
    for index in 0..BLOCK {
        inner[index] ^= key_block[index];
        outer[index] ^= key_block[index];
    }
    let mut inner_data = Vec::with_capacity(BLOCK + body.len());
    inner_data.extend_from_slice(&inner);
    inner_data.extend_from_slice(body);
    let inner_hash = sha256(&inner_data);
    let mut outer_data = Vec::with_capacity(BLOCK + inner_hash.len());
    outer_data.extend_from_slice(&outer);
    outer_data.extend_from_slice(&inner_hash);
    sha256(&outer_data)
}

fn sha256(input: &[u8]) -> [u8; 32] {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut data = input.to_vec();
    let bit_len = (data.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = H0;
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, bytes) in chunk.chunks_exact(4).enumerate().take(16) {
            w[index] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (index, word) in h.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::hmac_sha256;

    #[test]
    fn hmac_sha256_matches_rfc4231_test_vector() {
        let digest = hmac_sha256(&[0x0b; 20], b"Hi There");
        assert_eq!(
            hex(&digest),
            "b0344c61d8db38535ca8afceaf0bf12b\
             881dc200c9833da726e9376c2e32cff7"
                .replace(' ', "")
        );
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl HostServices for CliHost {
    fn availability(&self) -> HostServiceAvailability {
        self.availability
    }

    fn model<'a>(
        &'a self,
        request: ModelRequest,
    ) -> HostFuture<'a, Result<ModelResponse, HostError>> {
        let future: HostFuture<'a, Result<ModelResponse, HostError>> = match &self.model {
            Some(CliModelAdapter::OpenAi(adapter)) => {
                Box::pin(async move { adapter.complete(request).await })
            }
            Some(CliModelAdapter::Anthropic(adapter)) => {
                Box::pin(async move { adapter.complete(request).await })
            }
            None => self.unavailable.model(request),
        };
        self.profiled("host.model", future)
    }

    fn tool<'a>(&'a self, request: ToolRequest) -> HostFuture<'a, Result<ToolResponse, HostError>> {
        let future = match self.tool.invoke(request.clone()) {
            Some(future) => future,
            None => Box::pin(async move { Err(unknown_tool_error(&request)) }),
        };
        self.profiled("host.tool", future)
    }

    fn memory<'a>(
        &'a self,
        request: MemoryRequest,
    ) -> HostFuture<'a, Result<MemoryResponse, HostError>> {
        let future: HostFuture<'a, Result<MemoryResponse, HostError>> = match &self.memory {
            Some(CliMemoryClient::InMemory(adapter)) => {
                Box::pin(async move { adapter.execute(request).await })
            }
            Some(CliMemoryClient::Sqlite(adapter)) => {
                Box::pin(async move { adapter.execute(request).await })
            }
            None => self.unavailable.memory(request),
        };
        self.profiled("host.memory", future)
    }

    fn session<'a>(
        &'a self,
        request: SessionRequest,
    ) -> HostFuture<'a, Result<SessionResponse, HostError>> {
        let future: HostFuture<'a, Result<SessionResponse, HostError>> = match &self.session {
            Some(CliSessionClient::InMemory(adapter)) => {
                Box::pin(async move { adapter.execute(request).await })
            }
            Some(CliSessionClient::Sqlite(adapter)) => {
                Box::pin(async move { adapter.execute(request).await })
            }
            None => self.unavailable.session(request),
        };
        self.profiled("host.session", future)
    }

    fn filesystem<'a>(
        &'a self,
        request: FilesystemRequest,
    ) -> HostFuture<'a, Result<FilesystemResponse, HostError>> {
        let future = match &self.filesystem {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.filesystem(request),
        };
        self.profiled("host.filesystem", future)
    }

    fn command<'a>(
        &'a self,
        request: CommandRequest,
    ) -> HostFuture<'a, Result<CommandResponse, HostError>> {
        let future = match &self.command {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.command(request),
        };
        self.profiled("host.command", future)
    }

    fn tcp<'a>(
        &'a self,
        request: TcpConnectRequest,
    ) -> HostFuture<'a, Result<TcpConnectResponse, HostError>> {
        let future = match &self.tcp {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.tcp(request),
        };
        self.profiled("host.tcp", future)
    }

    fn stream<'a>(
        &'a self,
        request: StreamRequest,
    ) -> HostFuture<'a, Result<StreamResponse, HostError>> {
        let future = match &self.stream {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.stream(request),
        };
        self.profiled("host.stream", future)
    }

    fn tls<'a>(
        &'a self,
        request: TlsConnectRequest,
    ) -> HostFuture<'a, Result<TlsConnectResponse, HostError>> {
        let future = match &self.tls {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.tls(request),
        };
        self.profiled("host.tls", future)
    }

    fn secret<'a>(
        &'a self,
        request: SecretRequest,
    ) -> HostFuture<'a, Result<SecretResponse, HostError>> {
        let future = match &self.secret {
            Some(adapter) => Box::pin(async move { adapter.execute(request).await }),
            None => self.unavailable.secret(request),
        };
        self.profiled("host.secret", future)
    }

    fn browser<'a>(
        &'a self,
        request: BrowserProtocolRequest,
    ) -> HostFuture<'a, Result<BrowserProtocolResponse, HostError>> {
        let future = self.unavailable.browser(request);
        self.profiled("host.browser", future)
    }

    fn approval<'a>(
        &'a self,
        request: ApprovalRequest,
    ) -> HostFuture<'a, Result<ApprovalDecision, HostError>> {
        let future = async move { approval_decision(self.approval, request) };
        self.profiled("host.approval", future)
    }

    fn policy<'a>(
        &'a self,
        request: PolicyEvaluationRequest,
    ) -> HostFuture<'a, Result<PolicyResponse, HostError>> {
        let mut request = request;
        if request.authority.policy.active_trace_specs.is_empty() {
            request.authority.policy.active_trace_specs = self.active_trace_specs.clone();
        }
        if request.authority.policy.trace_spec_facts.is_empty() {
            request.authority.policy.trace_spec_facts = self.trace_spec_facts.clone();
        }
        let future = async move {
            if let Some(adapter) = &self.policy_http {
                return adapter.evaluate(request).await;
            }
            if let Some(adapter) = &self.policy_local {
                return adapter.evaluate(request).await;
            }
            match self.policy {
                PolicyMode::DenyUnknown => {
                    let adapter = etas_host::DenyUnknownPolicyClient;
                    adapter.evaluate(request).await
                }
                PolicyMode::UnsafeTraceSpecRuntime => {
                    if is_trace_spec_runtime_ref(&request.policy_ref) {
                        let adapter = TraceSpecRuntimeClient;
                        adapter.evaluate(request).await
                    } else {
                        Err(HostError::new(
                            HostErrorCode::ProviderUnavailable,
                            "unsafe trace spec runtime only evaluates trace spec runtime references",
                        )
                        .with_detail("policy_ref", format!("{:?}", request.policy_ref)))
                    }
                }
                PolicyMode::UnsafeAllowLocalStatic => {
                    let adapter = UnsafeAllowAllLocalPolicyClient;
                    adapter.evaluate(request).await
                }
                PolicyMode::UnsafeRequireApprovalLocalStatic => {
                    let approval_id = HostRequestId(request.id.0.saturating_add(10_000_000));
                    Ok(PolicyResponse {
                        id: request.id,
                        decision: PolicyDecision::RequireApproval {
                            request: ApprovalRequest {
                                id: approval_id,
                                reason: format!(
                                    "CLI local policy requires approval for {} boundary",
                                    request.subject.kind
                                ),
                                requested_grants: Vec::new(),
                                trace: request.trace,
                            },
                        },
                    })
                }
                PolicyMode::LocalStatic => Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "local static policy mode requires configured local policy rules",
                )),
                PolicyMode::Http => Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "HTTP policy mode requires a configured HTTP policy client",
                )),
            }
        };
        self.profiled("host.policy", future)
    }

    fn console<'a>(
        &'a self,
        request: ConsoleRequest,
    ) -> HostFuture<'a, Result<ConsoleResponse, HostError>> {
        let future = async move {
            let adapter = LocalStdioClient::new();
            adapter.execute(request).await
        };
        self.profiled("host.console", future)
    }
}

fn is_trace_spec_runtime_ref(value: &HostValue) -> bool {
    matches!(value, HostValue::String(value) if value == TRACE_SPEC_RUNTIME_REF)
}

fn policy_label_terms(label: &str) -> Vec<String> {
    let trimmed = label.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed);
    inner
        .split(',')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn approval_decision(
    mode: ApprovalMode,
    request: ApprovalRequest,
) -> Result<ApprovalDecision, HostError> {
    match mode {
        ApprovalMode::Deny => Ok(ApprovalDecision::Denied {
            reason: "CLI approval mode denies host approval requests".to_owned(),
        }),
        ApprovalMode::Auto => Ok(ApprovalDecision::Approved {
            grant: etas_host::ApprovalGrant {
                id: request.id,
                grants: request.requested_grants,
                reason: "CLI approval mode auto-approved request".to_owned(),
            },
        }),
        ApprovalMode::Prompt => {
            if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                return Ok(ApprovalDecision::Denied {
                    reason: "CLI approval prompt requires an interactive TTY".to_owned(),
                });
            }
            eprintln!(
                "Etas approval requested: {}\nType `yes` to approve:",
                request.reason
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|error| {
                HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "failed to read approval input",
                )
                .with_detail("error", error.to_string())
            })?;
            if input.trim() == "yes" {
                Ok(ApprovalDecision::Approved {
                    grant: etas_host::ApprovalGrant {
                        id: request.id,
                        grants: request.requested_grants,
                        reason: "CLI approval prompt accepted request".to_owned(),
                    },
                })
            } else {
                Ok(ApprovalDecision::Denied {
                    reason: "CLI approval prompt denied request".to_owned(),
                })
            }
        }
    }
}

impl UnavailableHostServices {
    fn model(&self, request: ModelRequest) -> HostFuture<'_, Result<ModelResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "model host adapter is not configured",
            ))
        })
    }

    fn memory(&self, request: MemoryRequest) -> HostFuture<'_, Result<MemoryResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "memory host adapter is not configured",
            ))
        })
    }

    fn session(
        &self,
        request: SessionRequest,
    ) -> HostFuture<'_, Result<SessionResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "session host adapter is not configured",
            ))
        })
    }

    fn filesystem(
        &self,
        request: FilesystemRequest,
    ) -> HostFuture<'_, Result<FilesystemResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "filesystem host adapter is not configured",
            ))
        })
    }

    fn command(
        &self,
        request: CommandRequest,
    ) -> HostFuture<'_, Result<CommandResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "command host adapter is not configured",
            ))
        })
    }

    fn tcp(
        &self,
        request: TcpConnectRequest,
    ) -> HostFuture<'_, Result<TcpConnectResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "TCP host adapter is not configured",
            ))
        })
    }

    fn stream(&self, request: StreamRequest) -> HostFuture<'_, Result<StreamResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "stream host adapter is not configured",
            ))
        })
    }

    fn tls(
        &self,
        request: TlsConnectRequest,
    ) -> HostFuture<'_, Result<TlsConnectResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "TLS host adapter is not configured",
            ))
        })
    }

    fn secret(&self, request: SecretRequest) -> HostFuture<'_, Result<SecretResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "secret host adapter is not configured",
            ))
        })
    }

    fn browser(
        &self,
        request: BrowserProtocolRequest,
    ) -> HostFuture<'_, Result<BrowserProtocolResponse, HostError>> {
        Box::pin(async move {
            Err(unavailable(
                request.id,
                "browser protocol host adapter is not configured",
            ))
        })
    }
}

fn unavailable(id: etas_host::HostRequestId, message: &'static str) -> HostError {
    HostError::new(HostErrorCode::ProviderUnavailable, message)
        .with_detail("request_id", id.0.to_string())
}

pub(crate) fn run_checked(
    checked: &CheckedProject,
    dry_run: bool,
    allow_effects: bool,
    config: CliHostConfig,
    budget_overrides: RunBudgetOverrides,
    program_args: Vec<String>,
    profile: &ProfileHandle,
) -> Result<RunResult, CliError> {
    let effects_enabled = allow_effects || config.profile_name.is_some();
    let config = if dry_run || !effects_enabled {
        let mut hostless = CliHostConfig::none();
        hostless.execution_limits = config.execution_limits;
        hostless
    } else {
        config
    };
    let Some(item) = checked.entry else {
        return Err(CliError::InvalidUsage(
            "checked project does not contain a resolved interpreter entry".to_owned(),
        ));
    };
    let active_trace_specs = active_trace_specs_for_entry(checked, item);
    let trace_spec_facts = trace_spec_facts_for_entry(checked, item);
    if effects_enabled && !config.has_effect_adapters() {
        return Err(CliError::InvalidUsage(
            "`etas run` requires configured host adapters; use `--profile` or `--allow-effects` with legacy `ETAS_HOST_*` configuration"
                .to_owned(),
        ));
    }
    let host = if dry_run {
        CliHost::dry_run(profile.clone())
    } else {
        CliHost::console(
            config.clone(),
            active_trace_specs.clone(),
            trace_spec_facts.clone(),
            profile.clone(),
        )?
    };
    let entry = EntryPoint { item };
    let entry_args = entry_args_from_strings(checked, program_args);
    let mut options = config.run_options();
    options.profile = profile.clone();
    options.host_context.authority.policy.active_trace_specs = active_trace_specs;
    options.host_context.authority.policy.trace_spec_facts = trace_spec_facts;
    apply_budget_overrides(&mut options, budget_overrides);
    Ok(run_checked_blocking(
        checked, entry, entry_args, &host, options,
    ))
}

pub(crate) fn resume_checked(
    checked: &CheckedProject,
    checkpoint: &InterpreterCheckpoint,
    config: CliHostConfig,
) -> Result<RunResult, CliError> {
    let active_trace_specs = active_trace_specs_for_entry(checked, checkpoint.entry_item);
    let trace_spec_facts = trace_spec_facts_for_entry(checked, checkpoint.entry_item);
    let host = CliHost::console(
        config.clone(),
        active_trace_specs.clone(),
        trace_spec_facts.clone(),
        ProfileHandle::disabled(),
    )?;
    let mut options = config.run_options();
    options.host_context.authority.policy.active_trace_specs = active_trace_specs;
    options.host_context.authority.policy.trace_spec_facts = trace_spec_facts;
    Ok(resume_checkpoint_blocking(
        checked, checkpoint, &host, options,
    ))
}

fn active_trace_specs_for_entry(
    checked: &CheckedProject,
    item: etas_hir::HirItemId,
) -> Vec<String> {
    let mut policies = Vec::new();
    if let Some(requirements) = checked.effects.requirements.items.get(&item) {
        policies.extend(requirements.iter().filter_map(trace_spec_reference_name));
    }
    if let Some(trace_spec_facts) = checked.effects.trace_specs.items.get(&item) {
        policies.extend(
            trace_spec_facts
                .iter()
                .filter_map(trace_spec_clause_reference_name),
        );
    }
    if let Some(summary) = checked.effects.item_effects.get(&item) {
        policies.extend(
            summary
                .trace_spec_obligations
                .iter()
                .filter_map(trace_spec_reference_name),
        );
    }
    policies.sort();
    policies.dedup();
    policies
}

fn trace_spec_facts_for_entry(
    checked: &CheckedProject,
    item: etas_hir::HirItemId,
) -> Vec<HostValue> {
    let mut facts = Vec::new();
    if let Some(requirements) = checked.effects.requirements.items.get(&item) {
        facts.extend(
            requirements
                .iter()
                .filter_map(|requirement| match requirement {
                    RequirementFact::TraceSpec(policy) => Some(policy.clone()),
                    _ => None,
                }),
        );
    }
    if let Some(trace_spec_facts) = checked.effects.trace_specs.items.get(&item) {
        facts.extend(trace_spec_facts.iter().cloned());
    }
    if let Some(summary) = checked.effects.item_effects.get(&item) {
        facts.extend(
            summary
                .trace_spec_obligations
                .iter()
                .filter_map(|requirement| match requirement {
                    RequirementFact::TraceSpec(policy) => Some(policy.clone()),
                    _ => None,
                }),
        );
    }
    facts.sort();
    facts.dedup();
    facts
        .iter()
        .map(trace_spec_clause_fact_to_host_value)
        .collect::<Vec<_>>()
}

fn trace_spec_reference_name(fact: &RequirementFact) -> Option<String> {
    match fact {
        RequirementFact::TraceSpec(TraceSpecClauseFact::TraceSpecReference { name }) => {
            Some(name.clone())
        }
        _ => None,
    }
}

fn trace_spec_clause_reference_name(fact: &TraceSpecClauseFact) -> Option<String> {
    match fact {
        TraceSpecClauseFact::TraceSpecReference { name } => Some(name.clone()),
        _ => None,
    }
}

fn trace_spec_clause_fact_to_host_value(fact: &TraceSpecClauseFact) -> HostValue {
    let mut fields = vec![
        (
            "kind".to_owned(),
            HostValue::String(trace_spec_fact_kind(fact)),
        ),
        ("label".to_owned(), HostValue::String(fact.label())),
    ];
    match fact {
        TraceSpecClauseFact::PendingTypedTraceSpecModel => {}
        TraceSpecClauseFact::TraceSpecReference { name } => {
            fields.push(("name".to_owned(), HostValue::String(name.clone())));
        }
        TraceSpecClauseFact::Allow { label, .. } | TraceSpecClauseFact::Deny { label, .. } => {
            fields.push(("target_label".to_owned(), HostValue::String(label.clone())));
            fields.push((
                "target_patterns".to_owned(),
                policy_label_patterns_to_host_value(label),
            ));
        }
        TraceSpecClauseFact::RequireBefore {
            guard_label,
            target_label,
            ..
        } => {
            fields.push((
                "guard_label".to_owned(),
                HostValue::String(guard_label.clone()),
            ));
            fields.push((
                "guard_patterns".to_owned(),
                policy_label_patterns_to_host_value(guard_label),
            ));
            fields.push((
                "target_label".to_owned(),
                HostValue::String(target_label.clone()),
            ));
            fields.push((
                "target_patterns".to_owned(),
                policy_label_patterns_to_host_value(target_label),
            ));
        }
        TraceSpecClauseFact::RequireAfter {
            target_label,
            obligation_label,
            ..
        } => {
            fields.push((
                "target_label".to_owned(),
                HostValue::String(target_label.clone()),
            ));
            fields.push((
                "target_patterns".to_owned(),
                policy_label_patterns_to_host_value(target_label),
            ));
            fields.push((
                "obligation_label".to_owned(),
                HostValue::String(obligation_label.clone()),
            ));
            fields.push((
                "obligation_patterns".to_owned(),
                policy_label_patterns_to_host_value(obligation_label),
            ));
        }
        TraceSpecClauseFact::Limit { count } => {
            fields.push(("count".to_owned(), HostValue::UInt(*count as u128)));
        }
    }
    HostValue::Record(fields)
}

fn policy_label_patterns_to_host_value(label: &str) -> HostValue {
    HostValue::List(
        policy_label_terms(label)
            .into_iter()
            .filter_map(|term| {
                let (base, arg) = term
                    .split_once('[')
                    .map(|(base, rest)| {
                        (
                            base.trim().to_owned(),
                            rest.trim_end_matches(']').trim().to_owned(),
                        )
                    })
                    .unwrap_or((term.trim().to_owned(), String::new()));
                if base.is_empty() {
                    return None;
                }
                let mut fields = Vec::new();
                if let Some((effect, action)) = base.split_once('.') {
                    fields.push(("effect".to_owned(), HostValue::String(effect.to_owned())));
                    fields.push(("action".to_owned(), HostValue::String(action.to_owned())));
                } else {
                    fields.push(("effect".to_owned(), HostValue::String(base)));
                }
                if !arg.is_empty() {
                    fields.push((
                        "args".to_owned(),
                        HostValue::List(
                            arg.split(',')
                                .map(str::trim)
                                .filter(|arg| !arg.is_empty())
                                .map(|arg| HostValue::String(arg.to_owned()))
                                .collect(),
                        ),
                    ));
                }
                Some(HostValue::Record(fields))
            })
            .collect(),
    )
}

fn trace_spec_fact_kind(fact: &TraceSpecClauseFact) -> String {
    match fact {
        TraceSpecClauseFact::PendingTypedTraceSpecModel => "pending_typed_trace_spec_model",
        TraceSpecClauseFact::TraceSpecReference { .. } => "trace_spec_reference",
        TraceSpecClauseFact::Allow { .. } => "allow",
        TraceSpecClauseFact::Deny { .. } => "deny",
        TraceSpecClauseFact::RequireBefore { .. } => "require_before",
        TraceSpecClauseFact::RequireAfter { .. } => "require_after",
        TraceSpecClauseFact::Limit { .. } => "limit",
    }
    .to_owned()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunBudgetOverrides {
    pub tokens: Option<u64>,
    pub cost: Option<(u128, String)>,
    pub time: Option<u64>,
}

fn apply_budget_overrides(options: &mut RunOptions, overrides: RunBudgetOverrides) {
    if let Some(tokens) = overrides.tokens {
        options.host_context.budget.tokens = Some(TokenBudget { max_tokens: tokens });
        options.model_policy.options.max_output_tokens = Some(tokens);
        let budget = options
            .model_policy
            .budget
            .get_or_insert_with(Budget::default);
        budget.tokens = Some(TokenBudget { max_tokens: tokens });
    }
    if let Some((amount, currency)) = overrides.cost {
        let cost = CostBudget {
            max_micros: amount,
            currency,
        };
        options.host_context.budget.cost = Some(cost.clone());
        let budget = options
            .model_policy
            .budget
            .get_or_insert_with(Budget::default);
        budget.cost = Some(cost);
    }
    if let Some(max_millis) = overrides.time {
        let time = TimeBudget { max_millis };
        options.host_context.budget.time = Some(time);
        let budget = options
            .model_policy
            .budget
            .get_or_insert_with(Budget::default);
        budget.time = Some(time);
    }
}
