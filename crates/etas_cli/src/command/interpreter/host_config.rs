use etas_host::{
    AnthropicProtocolAdapter, AuthConfig, CommandPolicy, DestructiveOpPolicy, FilesystemPolicy,
    HttpPolicyClient, LocalPolicyDecision, LocalPolicyRule, LocalStaticPolicyClient, ModelName,
    ModelProviderCapabilities, ModelProviderId, NetworkEndpoint, NetworkPolicy,
    OpenAiProtocolAdapter, PrivateResolutionPolicy, RetryPolicy, SandboxPolicy, WorkspaceRoot,
};
use etas_interpreter::api::{
    ExecutionLimits, HostExecutionContext, ModelExecutionPolicy, RunOptions,
};
use etas_package::{
    RuntimeBackendProfile, RuntimeCommandProfile, RuntimeExecutionProfile, RuntimeModelProfile,
    RuntimeNetworkProfile, RuntimePolicyProfile, RuntimeProfile, RuntimeRetryProfile,
    RuntimeSection, RuntimeToolsProfile, discover_manifest, read_manifest,
};
use std::{
    collections::BTreeMap,
    num::{NonZeroU32, NonZeroU64},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::error::CliError;

use super::host_tool::{
    CliToolBinding, parse_tool_bindings, tool_binding_endpoint, tool_binding_name,
    tool_binding_program, tool_bindings_from_profile,
};

#[derive(Clone)]
pub(crate) struct CliHostConfig {
    pub(super) model: Option<CliModelConfig>,
    pub(super) command_allowed_programs: Vec<String>,
    pub(super) workspace_root: Option<WorkspaceRoot>,
    pub(super) filesystem_mode: FilesystemMode,
    pub(super) program_network_allowlist: Vec<NetworkEndpoint>,
    pub(super) adapter_transport_allowlist: Vec<NetworkEndpoint>,
    pub(super) memory_mode: MemoryMode,
    pub(super) approval_mode: ApprovalMode,
    pub(super) policy_mode: PolicyMode,
    pub(super) policy_local: Option<LocalStaticPolicyClient>,
    pub(super) policy_http: Option<CliPolicyHttpConfig>,
    pub(super) boundary_policy_ref: Option<String>,
    pub(super) session_id: Option<String>,
    pub(super) session_mode: SessionMode,
    pub(super) secret_mode: SecretMode,
    pub(super) tool_bindings: Vec<CliToolBinding>,
    pub(super) tool_private_resolution: PrivateResolutionPolicy,
    pub(super) profile_name: Option<String>,
    pub(super) execution_limits: ExecutionLimits,
}

#[derive(Clone)]
pub(super) struct CliModelConfig {
    pub(super) adapter: CliModelAdapter,
    pub(super) provider: ModelProviderId,
    pub(super) capabilities: ModelProviderCapabilities,
    pub(super) model: ModelName,
    base_url: String,
    endpoint: NetworkEndpoint,
    private_resolution: PrivateResolutionPolicy,
    retry: RetryPolicy,
}

#[derive(Clone)]
pub(super) struct CliPolicyHttpConfig {
    pub(super) client: HttpPolicyClient,
    base_url: String,
    endpoint: NetworkEndpoint,
    private_resolution: PrivateResolutionPolicy,
    retry: RetryPolicy,
}

#[derive(Clone)]
pub(super) enum CliModelAdapter {
    OpenAi(OpenAiProtocolAdapter),
    Anthropic(AnthropicProtocolAdapter),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum FilesystemMode {
    #[default]
    None,
    ReadOnly,
    ReadWrite,
    Destructive,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum MemoryMode {
    #[default]
    None,
    Memory,
    Sqlite {
        path: PathBuf,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum SessionMode {
    #[default]
    None,
    Memory,
    Sqlite {
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SecretMode {
    #[default]
    None,
    Env,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ApprovalMode {
    #[default]
    Deny,
    Auto,
    Prompt,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum PolicyMode {
    #[default]
    DenyUnknown,
    UnsafeTraceSpecRuntime,
    UnsafeAllowLocalStatic,
    UnsafeRequireApprovalLocalStatic,
    LocalStatic,
    Http,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RuntimeEnvOverrides {
    values: BTreeMap<String, String>,
}

impl RuntimeEnvOverrides {
    pub(crate) fn from_environment() -> Result<Self, CliError> {
        let mut values = BTreeMap::new();
        for (name, value) in std::env::vars_os() {
            let name = name.into_string().map_err(|_| {
                CliError::InvalidUsage(
                    "environment contains a non-unicode variable name".to_owned(),
                )
            })?;
            let value = value.into_string().map_err(|_| {
                CliError::InvalidUsage(format!(
                    "environment variable `{name}` is not valid unicode"
                ))
            })?;
            if !value.is_empty() {
                values.insert(name, value);
            }
        }
        Ok(Self { values })
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    fn optional(&self, name: &str) -> Option<String> {
        self.get(name).map(ToOwned::to_owned)
    }

    fn contains_any(&self, names: &[&str]) -> bool {
        names.iter().any(|name| self.values.contains_key(*name))
    }
}

impl CliHostConfig {
    pub(super) fn none() -> Self {
        Self {
            model: None,
            command_allowed_programs: Vec::new(),
            workspace_root: None,
            filesystem_mode: FilesystemMode::None,
            program_network_allowlist: Vec::new(),
            adapter_transport_allowlist: Vec::new(),
            memory_mode: MemoryMode::None,
            approval_mode: ApprovalMode::Deny,
            policy_mode: PolicyMode::DenyUnknown,
            policy_local: None,
            policy_http: None,
            boundary_policy_ref: None,
            session_id: None,
            session_mode: SessionMode::None,
            secret_mode: SecretMode::None,
            tool_bindings: Vec::new(),
            tool_private_resolution: PrivateResolutionPolicy::PublicOnly,
            profile_name: None,
            execution_limits: ExecutionLimits::default(),
        }
    }

    pub(super) fn from_legacy_environment() -> Result<Self, CliError> {
        let mut command_allowed_programs = optional_env("ETAS_HOST_COMMAND_ALLOWLIST")?
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let tool_bindings = parse_tool_bindings(
            optional_env("ETAS_HOST_TOOL_HTTP")?,
            optional_env("ETAS_HOST_TOOL_MCP")?,
            optional_env("ETAS_HOST_TOOL_PROCESS")?,
        )?;
        let tool_private_resolution = private_resolution_from_env_var(
            "ETAS_HOST_TOOL_ALLOW_PRIVATE",
            PrivateResolutionPolicy::PublicOnly,
        )?;
        command_allowed_programs.extend(
            tool_bindings
                .iter()
                .filter_map(tool_binding_program)
                .map(ToOwned::to_owned),
        );
        command_allowed_programs.sort();
        command_allowed_programs.dedup();

        let model = model_from_environment()?;
        let workspace_root = optional_env("ETAS_HOST_WORKSPACE_ROOT")?
            .map(|path| {
                WorkspaceRoot::new(&path).map_err(|error| {
                    CliError::InvalidUsage(format!(
                        "invalid ETAS_HOST_WORKSPACE_ROOT `{path}`: {}",
                        error.message
                    ))
                })
            })
            .transpose()?;
        let filesystem_mode = parse_filesystem_mode(optional_env("ETAS_HOST_FILESYSTEM")?)?;
        if filesystem_mode != FilesystemMode::None && workspace_root.is_none() {
            return Err(CliError::InvalidUsage(
                "`ETAS_HOST_FILESYSTEM` requires `ETAS_HOST_WORKSPACE_ROOT`".to_owned(),
            ));
        }
        let memory_mode = parse_memory_mode(
            optional_env("ETAS_HOST_MEMORY")?,
            optional_env("ETAS_HOST_MEMORY_PATH")?,
        )?;
        let approval_mode = parse_approval_mode(optional_env("ETAS_HOST_APPROVAL")?)?;
        let policy_mode = parse_policy_mode(optional_env("ETAS_HOST_POLICY")?)?;
        let policy_local = policy_local_from_environment(policy_mode)?;
        let policy_http = policy_http_from_environment(policy_mode)?;
        let boundary_policy_ref = optional_env("ETAS_HOST_BOUNDARY_POLICY")?;
        let session_id = optional_env("ETAS_HOST_SESSION_ID")?;
        let session_mode = parse_session_mode(
            optional_env("ETAS_HOST_SESSION")?,
            optional_env("ETAS_HOST_SESSION_PATH")?,
            session_id.as_ref(),
        )?;
        let secret_mode = parse_secret_mode(optional_env("ETAS_HOST_SECRET")?)?;
        let mut program_network_allowlist =
            parse_network_allowlist(optional_env("ETAS_HOST_NETWORK_ALLOWLIST")?)?;
        let mut adapter_transport_allowlist = Vec::new();
        if let Some(model) = &model {
            adapter_transport_allowlist.push(model.endpoint.clone());
        }
        for binding in &tool_bindings {
            if let Some(endpoint) = tool_binding_endpoint(binding) {
                adapter_transport_allowlist.push(network_endpoint_from_base_url(endpoint)?);
            }
        }
        if let Some(policy) = &policy_http {
            adapter_transport_allowlist.push(policy.endpoint.clone());
        }
        sort_dedup_network_allowlist(&mut program_network_allowlist);
        sort_dedup_network_allowlist(&mut adapter_transport_allowlist);

        Ok(Self {
            model,
            command_allowed_programs,
            workspace_root,
            filesystem_mode,
            program_network_allowlist,
            adapter_transport_allowlist,
            memory_mode,
            approval_mode,
            policy_mode,
            policy_local,
            policy_http,
            boundary_policy_ref,
            session_id,
            session_mode,
            secret_mode,
            tool_bindings,
            tool_private_resolution,
            profile_name: None,
            execution_limits: ExecutionLimits::default(),
        })
    }

    pub(super) fn from_environment_with_allow_net(allow_net: &[String]) -> Result<Self, CliError> {
        let mut config = Self::from_legacy_environment()?;
        let explicit_endpoints = parse_cli_network_allowlist(allow_net)?;
        config.program_network_allowlist.extend(explicit_endpoints);
        sort_dedup_network_allowlist(&mut config.program_network_allowlist);
        Ok(config)
    }

    pub(super) fn from_runtime_profile(
        profile_name: String,
        manifest_profile: Option<&RuntimeProfile>,
        local_profile: Option<&RuntimeProfile>,
        env: &RuntimeEnvOverrides,
        allow_net: &[String],
    ) -> Result<Self, CliError> {
        let mut config = Self::none();
        config.profile_name = Some(profile_name);
        if let Some(profile) = manifest_profile {
            config.apply_runtime_profile(profile, env)?;
        }
        if let Some(profile) = local_profile {
            config.apply_runtime_profile(profile, env)?;
        }
        config.apply_env_overrides(env)?;
        let explicit_endpoints = parse_cli_network_allowlist(allow_net)?;
        config.program_network_allowlist.extend(explicit_endpoints);
        config.materialize_adapter_transport_endpoints()?;
        config.command_allowed_programs.sort();
        config.command_allowed_programs.dedup();
        sort_dedup_network_allowlist(&mut config.program_network_allowlist);
        sort_dedup_network_allowlist(&mut config.adapter_transport_allowlist);
        Ok(config)
    }

    fn apply_runtime_profile(
        &mut self,
        profile: &RuntimeProfile,
        env: &RuntimeEnvOverrides,
    ) -> Result<(), CliError> {
        if let Some(model) = &profile.model {
            self.model = Some(model_from_runtime_profile(model, env, self.model.as_ref())?);
        }
        if let Some(memory) = &profile.memory {
            self.memory_mode = memory_mode_from_profile(memory)?;
        }
        if let Some(session) = &profile.session {
            self.session_id = session.id.clone().or(self.session_id.take());
            self.session_mode = session_mode_from_profile(session, self.session_id.as_ref())?;
        }
        if let Some(policy) = &profile.policy {
            self.apply_policy_profile(policy, env)?;
        }
        if let Some(approval) = &profile.approval {
            self.approval_mode = parse_approval_mode(approval.mode.clone())?;
        }
        if let Some(network) = &profile.network {
            let explicit_endpoints = network_allowlist_from_profile(network)?;
            self.program_network_allowlist.extend(explicit_endpoints);
        }
        if let Some(filesystem) = &profile.filesystem {
            self.filesystem_mode = parse_filesystem_mode(filesystem.mode.clone())?;
            if let Some(root) = &filesystem.workspace_root {
                self.workspace_root = Some(workspace_root_from_path(root)?);
            }
            if self.filesystem_mode != FilesystemMode::None && self.workspace_root.is_none() {
                return Err(CliError::InvalidUsage(
                    "`runtime.filesystem` requires `workspace_root` when filesystem access is enabled"
                        .to_owned(),
                ));
            }
        }
        if let Some(secret) = &profile.secret {
            self.secret_mode = parse_secret_mode(secret.mode.clone())?;
        }
        if let Some(command) = &profile.command {
            self.apply_command_profile(command);
        }
        if let Some(tools) = &profile.tools {
            self.apply_tools_profile(tools)?;
        }
        if let Some(boundary_policy) = &profile.boundary_policy {
            self.boundary_policy_ref = Some(boundary_policy.clone());
        }
        Ok(())
    }

    fn apply_execution_config(
        &mut self,
        manifest: &RuntimeExecutionProfile,
        local: &RuntimeExecutionProfile,
        cli_max_call_depth: Option<u32>,
    ) -> Result<(), CliError> {
        let default = ExecutionLimits::default();
        let max_call_depth = cli_max_call_depth
            .or(local.max_call_depth)
            .or(manifest.max_call_depth)
            .unwrap_or(default.max_call_depth.get());
        let max_call_depth = NonZeroU32::new(max_call_depth).ok_or_else(|| {
            CliError::InvalidUsage("`max_call_depth` must be greater than zero".to_owned())
        })?;
        let max_steps = local
            .max_steps
            .or(manifest.max_steps)
            .map(|value| {
                NonZeroU64::new(value).ok_or_else(|| {
                    CliError::InvalidUsage("`max_steps` must be greater than zero".to_owned())
                })
            })
            .transpose()?;
        self.execution_limits =
            ExecutionLimits::new(max_call_depth, max_steps).map_err(CliError::InvalidUsage)?;
        Ok(())
    }

    fn apply_policy_profile(
        &mut self,
        profile: &RuntimePolicyProfile,
        env: &RuntimeEnvOverrides,
    ) -> Result<(), CliError> {
        let mode = parse_policy_mode(profile.mode.clone())?;
        self.policy_mode = mode;
        self.policy_local = policy_local_from_rules(mode, &profile.rules)?;
        self.policy_http = policy_http_from_profile(mode, profile, env)?;
        Ok(())
    }

    fn apply_command_profile(&mut self, profile: &RuntimeCommandProfile) {
        self.command_allowed_programs
            .extend(profile.allow.iter().cloned());
    }

    fn apply_tools_profile(&mut self, profile: &RuntimeToolsProfile) -> Result<(), CliError> {
        if let Some(allow_private) = profile.allow_private {
            self.tool_private_resolution = private_resolution_from_flag(allow_private);
        }
        let bindings = tool_bindings_from_profile(profile)?;
        self.command_allowed_programs.extend(
            bindings
                .iter()
                .filter_map(tool_binding_program)
                .map(ToOwned::to_owned),
        );
        merge_tool_bindings(&mut self.tool_bindings, bindings);
        Ok(())
    }

    fn apply_env_overrides(&mut self, env: &RuntimeEnvOverrides) -> Result<(), CliError> {
        if env.contains_any(&[
            "ETAS_HOST_MODEL_ADAPTER",
            "ETAS_HOST_MODEL_NAME",
            "ETAS_HOST_MODEL_BASE_URL",
            "ETAS_HOST_MODEL_ALLOW_PRIVATE",
            "ETAS_HOST_MODEL_RETRY_ATTEMPTS",
            "ETAS_HOST_MODEL_RETRY_DELAY_MS",
        ]) {
            self.model = Some(model_from_env_overrides(env, self.model.as_ref())?);
        }
        if let Some(workspace_root) = env.optional("ETAS_HOST_WORKSPACE_ROOT") {
            self.workspace_root = Some(workspace_root_from_path(Path::new(&workspace_root))?);
        }
        if let Some(filesystem) = env.optional("ETAS_HOST_FILESYSTEM") {
            self.filesystem_mode = parse_filesystem_mode(Some(filesystem))?;
            if self.filesystem_mode != FilesystemMode::None && self.workspace_root.is_none() {
                return Err(CliError::InvalidUsage(
                    "`ETAS_HOST_FILESYSTEM` requires `ETAS_HOST_WORKSPACE_ROOT`".to_owned(),
                ));
            }
        }
        if env.contains_any(&["ETAS_HOST_MEMORY", "ETAS_HOST_MEMORY_PATH"]) {
            self.memory_mode = parse_memory_mode(
                env.optional("ETAS_HOST_MEMORY"),
                env.optional("ETAS_HOST_MEMORY_PATH"),
            )?;
        }
        if env.contains_any(&["ETAS_HOST_SESSION", "ETAS_HOST_SESSION_PATH"]) {
            self.session_mode = parse_session_mode(
                env.optional("ETAS_HOST_SESSION"),
                env.optional("ETAS_HOST_SESSION_PATH"),
                self.session_id.as_ref(),
            )?;
        }
        if let Some(session_id) = env.optional("ETAS_HOST_SESSION_ID") {
            self.session_id = Some(session_id);
            if self.session_mode == SessionMode::None {
                self.session_mode = SessionMode::Memory;
            }
        }
        if let Some(approval) = env.optional("ETAS_HOST_APPROVAL") {
            self.approval_mode = parse_approval_mode(Some(approval))?;
        }
        if env.contains_any(&[
            "ETAS_HOST_POLICY",
            "ETAS_HOST_POLICY_RULES",
            "ETAS_HOST_POLICY_URL",
            "ETAS_HOST_POLICY_TOKEN",
            "ETAS_HOST_POLICY_PATH",
            "ETAS_HOST_POLICY_ALLOW_PRIVATE",
            "ETAS_HOST_POLICY_RETRY_ATTEMPTS",
            "ETAS_HOST_POLICY_RETRY_DELAY_MS",
        ]) {
            let mode = env
                .optional("ETAS_HOST_POLICY")
                .map(|value| parse_policy_mode(Some(value)))
                .transpose()?
                .unwrap_or(self.policy_mode);
            self.policy_mode = mode;
            self.policy_local = policy_local_from_env_overrides(mode, env)?;
            self.policy_http =
                policy_http_from_env_overrides(mode, env, self.policy_http.as_ref())?;
        }
        if let Some(boundary_policy) = env.optional("ETAS_HOST_BOUNDARY_POLICY") {
            self.boundary_policy_ref = Some(boundary_policy);
        }
        if let Some(secret) = env.optional("ETAS_HOST_SECRET") {
            self.secret_mode = parse_secret_mode(Some(secret))?;
        }
        if let Some(command_allowlist) = env.optional("ETAS_HOST_COMMAND_ALLOWLIST") {
            self.command_allowed_programs.extend(
                command_allowlist
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned),
            );
        }
        if let Some(network_allowlist) = env.optional("ETAS_HOST_NETWORK_ALLOWLIST") {
            let explicit_endpoints = parse_network_allowlist(Some(network_allowlist))?;
            self.program_network_allowlist.extend(explicit_endpoints);
        }
        if env.contains_any(&[
            "ETAS_HOST_TOOL_HTTP",
            "ETAS_HOST_TOOL_MCP",
            "ETAS_HOST_TOOL_PROCESS",
            "ETAS_HOST_TOOL_ALLOW_PRIVATE",
        ]) {
            self.tool_private_resolution = private_resolution_from_env_override(
                env,
                "ETAS_HOST_TOOL_ALLOW_PRIVATE",
                self.tool_private_resolution,
            )?;
            let tool_bindings = parse_tool_bindings(
                env.optional("ETAS_HOST_TOOL_HTTP"),
                env.optional("ETAS_HOST_TOOL_MCP"),
                env.optional("ETAS_HOST_TOOL_PROCESS"),
            )?;
            self.command_allowed_programs.extend(
                tool_bindings
                    .iter()
                    .filter_map(tool_binding_program)
                    .map(ToOwned::to_owned),
            );
            merge_tool_bindings(&mut self.tool_bindings, tool_bindings);
        }
        Ok(())
    }

    fn materialize_adapter_transport_endpoints(&mut self) -> Result<(), CliError> {
        self.adapter_transport_allowlist.clear();
        if let Some(model) = &self.model {
            self.adapter_transport_allowlist
                .push(model.endpoint.clone());
        }
        for binding in &self.tool_bindings {
            if let Some(endpoint) = tool_binding_endpoint(binding) {
                self.adapter_transport_allowlist
                    .push(network_endpoint_from_base_url(endpoint)?);
            }
        }
        if let Some(policy) = &self.policy_http {
            self.adapter_transport_allowlist
                .push(policy.endpoint.clone());
        }
        Ok(())
    }

    pub(super) fn has_effect_adapters(&self) -> bool {
        self.model.is_some()
            || !self.command_allowed_programs.is_empty()
            || !self.tool_bindings.is_empty()
            || self.memory_mode != MemoryMode::None
            || self.filesystem_mode != FilesystemMode::None
            || !self.program_network_allowlist.is_empty()
            || self.approval_mode != ApprovalMode::Deny
            || self.policy_mode != PolicyMode::DenyUnknown
            || self.policy_local.is_some()
            || self.policy_http.is_some()
            || self.boundary_policy_ref.is_some()
            || self.session_mode != SessionMode::None
            || self.secret_mode != SecretMode::None
    }

    pub(crate) fn runtime_profile_json(&self) -> serde_json::Value {
        let model = self.model.as_ref().map(|model| {
            serde_json::json!({
                "provider": model.provider.0,
                "model": model.model.0,
                "private_resolution": private_resolution_name(model.private_resolution),
                "retry": retry_policy_json(model.retry),
                "capabilities": {
                    "forced_tool_output": model.capabilities.supports_forced_tool_output,
                    "json_schema_response_format": model.capabilities.supports_json_schema_response_format,
                    "plain_json_text_instruction": model.capabilities.supports_plain_json_text_instruction,
                    "tool_call_loop": model.capabilities.supports_tool_call_loop,
                    "required_tool_choice": model.capabilities.supports_required_tool_choice,
                }
            })
        });
        let tools = self
            .tool_bindings
            .iter()
            .map(|binding| match binding {
                CliToolBinding::Http { name, .. } => {
                    serde_json::json!({ "name": name, "kind": "http" })
                }
                CliToolBinding::Mcp { name, .. } => {
                    serde_json::json!({ "name": name, "kind": "mcp" })
                }
                CliToolBinding::Process { name, program } => {
                    serde_json::json!({ "name": name, "kind": "process", "program": program })
                }
            })
            .collect::<Vec<_>>();
        let mut profile = serde_json::json!({
            "schema": "etas.cli.runtime-profile.v1",
            "profile": self.profile_name.clone(),
            "model": model,
            "tools": tools,
            "tool_private_resolution": private_resolution_name(self.tool_private_resolution),
            "filesystem": format!("{:?}", self.filesystem_mode),
            "memory": memory_profile_json(&self.memory_mode),
            "approval": format!("{:?}", self.approval_mode),
            "policy": format!("{:?}", self.policy_mode),
            "policy_rules": self.policy_local.as_ref().map(|_| "local-static"),
            "policy_provider": self.policy_http.as_ref().map(|policy| {
                serde_json::json!({
                    "kind": "http",
                    "private_resolution": private_resolution_name(policy.private_resolution),
                    "retry": retry_policy_json(policy.retry),
                })
            }),
            "boundary_policy": self.boundary_policy_ref.clone(),
            "session": session_profile_json(&self.session_mode, self.session_id.as_deref()),
            "secret": format!("{:?}", self.secret_mode),
            "command_programs": self.command_allowed_programs,
            "program_network_endpoints": self.program_network_allowlist.iter().map(|endpoint| {
                serde_json::json!({
                    "scheme": endpoint.scheme,
                    "host": endpoint.host,
                    "port": endpoint.port,
                })
            }).collect::<Vec<_>>(),
            "adapter_transport_endpoints": self.adapter_transport_allowlist.iter().map(|endpoint| {
                serde_json::json!({
                    "scheme": endpoint.scheme,
                    "host": endpoint.host,
                    "port": endpoint.port,
                })
            }).collect::<Vec<_>>(),
            "workspace_root": self.workspace_root.as_ref().map(|root| root.canonical_root.display().to_string()),
            "execution": {
                "max_call_depth": self.execution_limits.max_call_depth.get(),
                "max_steps": self.execution_limits.max_steps.map(NonZeroU64::get),
            },
        });
        let fingerprint = runtime_profile_fingerprint(&profile);
        profile["profile_fingerprint"] = serde_json::Value::String(fingerprint);
        profile
    }

    pub(super) fn run_options(&self) -> RunOptions {
        let mut grants = vec![
            etas_host::HostActionGrant::allow("Console", "stdin_read_all"),
            etas_host::HostActionGrant::allow("Console", "stdin_read_line"),
            etas_host::HostActionGrant::allow("Console", "stdout_write"),
            etas_host::HostActionGrant::allow("Console", "stderr_write"),
        ];
        if self.model.is_some() {
            grants.push(etas_host::HostActionGrant::allow("Agentic", "infer"));
        }
        if !self.command_allowed_programs.is_empty() {
            grants.push(etas_host::HostActionGrant::allow("Command", "run"));
        }
        if self.filesystem_mode != FilesystemMode::None {
            grants.push(etas_host::HostActionGrant::allow("Fs", "read"));
            grants.push(etas_host::HostActionGrant::allow("Fs", "list"));
            grants.push(etas_host::HostActionGrant::allow("Fs", "stat"));
            if matches!(
                self.filesystem_mode,
                FilesystemMode::ReadWrite | FilesystemMode::Destructive
            ) {
                grants.push(etas_host::HostActionGrant::allow("Fs", "write"));
                grants.push(etas_host::HostActionGrant::allow("Fs", "atomic_replace"));
            }
        }
        if self.memory_mode != MemoryMode::None {
            grants.push(etas_host::HostActionGrant::allow("Memory", "read"));
            grants.push(etas_host::HostActionGrant::allow("Memory", "write"));
        }
        if self.session_mode != SessionMode::None {
            grants.push(etas_host::HostActionGrant::allow("Memory", "read"));
            grants.push(etas_host::HostActionGrant::allow("Memory", "write"));
        }
        if !self.program_network_allowlist.is_empty() {
            grants.push(etas_host::HostActionGrant::allow("Net", "tcp_connect"));
            grants.push(etas_host::HostActionGrant::allow("Tls", "handshake"));
            grants.push(etas_host::HostActionGrant::allow("Stream", "read"));
            grants.push(etas_host::HostActionGrant::allow("Stream", "write"));
            grants.push(etas_host::HostActionGrant::allow("Stream", "flush"));
            grants.push(etas_host::HostActionGrant::allow("Stream", "close"));
        }
        if !self.tool_bindings.is_empty() {
            grants.push(etas_host::HostActionGrant::allow("Tool", "call"));
        }
        if self.secret_mode != SecretMode::None {
            grants.push(etas_host::HostActionGrant::allow("Secret", "read"));
            grants.push(etas_host::HostActionGrant::allow("Secret", "use"));
        }
        let sandbox = if self.has_effect_adapters() {
            SandboxPolicy::allow_listed(
                filesystem_policy(self.workspace_root.clone(), self.filesystem_mode),
                NetworkPolicy::allow_endpoints(self.program_network_allowlist.clone()),
                CommandPolicy::allow_programs(self.command_allowed_programs.clone()),
                destructive_policy(self.filesystem_mode),
            )
        } else {
            SandboxPolicy::deny_all()
        };
        let model_policy = self
            .model
            .as_ref()
            .map(|model| ModelExecutionPolicy {
                provider: Some(model.provider.clone()),
                provider_capabilities: Some(model.capabilities),
                model: model.model.clone(),
                model_locked: true,
                ..ModelExecutionPolicy::phase1_default()
            })
            .unwrap_or_default();
        RunOptions {
            execution_limits: self.execution_limits,
            host_context: HostExecutionContext {
                authority: etas_host::AuthorityContext {
                    grants,
                    approvals: Vec::new(),
                    sandbox,
                    policy: etas_host::PolicyContext {
                        active_trace_specs: Vec::new(),
                        trace_spec_facts: Vec::new(),
                        labels: Vec::new(),
                        boundary_policy_ref: self
                            .boundary_policy_ref
                            .as_ref()
                            .map(|policy| etas_host::HostValue::String(policy.clone())),
                    },
                },
                trace: etas_host::TraceContext::root(etas_host::TraceId(1)),
                budget: etas_host::Budget::default(),
            },
            model_policy,
            current_session: self.session_id.clone(),
            ..RunOptions::default()
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeConfigFile {
    #[serde(default)]
    runtime: RuntimeSection,
}

pub(crate) fn runtime_config_for_run(
    inputs: &[PathBuf],
    flow: Option<&str>,
    requested_profile: Option<&str>,
    runtime_config: Option<&Path>,
    allow_net: &[String],
    max_call_depth: Option<u32>,
) -> Result<CliHostConfig, CliError> {
    let env = RuntimeEnvOverrides::from_environment()?;
    let Some(package_root) = package_root_for_inputs(inputs) else {
        if requested_profile.is_some() || runtime_config.is_some() {
            return Err(CliError::InvalidUsage(
                "`--profile` and `--runtime-config` require an `etas.toml` project".to_owned(),
            ));
        }
        let mut config = CliHostConfig::from_environment_with_allow_net(allow_net)?;
        config.apply_execution_config(
            &RuntimeExecutionProfile::default(),
            &RuntimeExecutionProfile::default(),
            max_call_depth,
        )?;
        return Ok(config);
    };
    let manifest = read_manifest(&package_root).map_err(|error| {
        CliError::InvalidUsage(format!(
            "failed to read runtime profile manifest `{}`: {error}",
            package_root.join("etas.toml").display()
        ))
    })?;
    let local_runtime = read_runtime_config_file(runtime_config, &package_root)?;
    let profile_name = requested_profile
        .map(ToOwned::to_owned)
        .or_else(|| bin_profile_for_flow(&manifest, flow))
        .or_else(|| local_runtime.default_profile.clone())
        .or_else(|| manifest.runtime.default_profile.clone());
    let mut config = if let Some(profile_name) = profile_name {
        let manifest_profile = manifest.runtime.profiles.get(&profile_name);
        let local_profile = local_runtime.profiles.get(&profile_name);
        if manifest_profile.is_none() && local_profile.is_none() {
            return Err(CliError::InvalidUsage(format!(
                "runtime profile `{profile_name}` is not defined in `etas.toml` or `etas.local.toml`"
            )));
        }
        CliHostConfig::from_runtime_profile(
            profile_name,
            manifest_profile,
            local_profile,
            &env,
            allow_net,
        )?
    } else {
        CliHostConfig::from_environment_with_allow_net(allow_net)?
    };
    config.apply_execution_config(
        &manifest.runtime.execution,
        &local_runtime.execution,
        max_call_depth,
    )?;
    Ok(config)
}

fn package_root_for_inputs(inputs: &[PathBuf]) -> Option<PathBuf> {
    let input = inputs
        .first()
        .map(PathBuf::as_path)
        .unwrap_or(Path::new("."));
    discover_manifest(input).and_then(|manifest| manifest.parent().map(Path::to_path_buf))
}

fn read_runtime_config_file(
    runtime_config: Option<&Path>,
    package_root: &Path,
) -> Result<RuntimeSection, CliError> {
    let path = runtime_config
        .map(Path::to_path_buf)
        .unwrap_or_else(|| package_root.join("etas.local.toml"));
    if !path.exists() {
        return Ok(RuntimeSection::default());
    }
    let text = std::fs::read_to_string(&path).map_err(|source| CliError::Config {
        path: path.clone(),
        source,
    })?;
    let config = toml::from_str::<RuntimeConfigFile>(&text).map_err(|source| {
        CliError::InvalidUsage(format!(
            "invalid runtime config `{}`: {source}",
            path.display()
        ))
    })?;
    Ok(config.runtime)
}

fn bin_profile_for_flow(manifest: &etas_package::Manifest, flow: Option<&str>) -> Option<String> {
    let flow = flow?;
    manifest
        .bins
        .iter()
        .find(|bin| bin.flow == flow)
        .and_then(|bin| bin.profile.clone())
}

fn merge_tool_bindings(existing: &mut Vec<CliToolBinding>, updates: Vec<CliToolBinding>) {
    for update in updates {
        let name = tool_binding_name(&update).to_owned();
        if let Some(slot) = existing
            .iter_mut()
            .find(|binding| tool_binding_name(binding) == name)
        {
            *slot = update;
        } else {
            existing.push(update);
        }
    }
}

fn model_from_runtime_profile(
    profile: &RuntimeModelProfile,
    env: &RuntimeEnvOverrides,
    current: Option<&CliModelConfig>,
) -> Result<CliModelConfig, CliError> {
    let adapter_name = env
        .optional("ETAS_HOST_MODEL_ADAPTER")
        .or_else(|| profile.adapter.clone())
        .or_else(|| current.map(|model| model.provider.0.clone()))
        .ok_or_else(|| {
            CliError::InvalidUsage("runtime model profile requires `adapter`".to_owned())
        })?;
    let model = env
        .optional("ETAS_HOST_MODEL_NAME")
        .or_else(|| profile.model.clone())
        .or_else(|| current.map(|model| model.model.0.clone()))
        .ok_or_else(|| {
            CliError::InvalidUsage("runtime model profile requires `model`".to_owned())
        })?;
    let base_url = env
        .optional("ETAS_HOST_MODEL_BASE_URL")
        .or_else(|| profile.base_url.clone())
        .or_else(|| current.map(|model| model.base_url.clone()));
    let api_key = env
        .optional("ETAS_HOST_MODEL_API_KEY")
        .or_else(|| env.optional("ETAS_HOST_OMLX_API_KEY"))
        .or_else(|| {
            profile
                .api_key_env
                .as_deref()
                .and_then(|name| env.optional(name))
        });
    let retry = retry_policy_from_env(
        env,
        "ETAS_HOST_MODEL_RETRY_ATTEMPTS",
        "ETAS_HOST_MODEL_RETRY_DELAY_MS",
        retry_policy_from_profile(
            profile.retry.as_ref(),
            current
                .map(|model| model.retry)
                .unwrap_or_else(default_transport_retry),
        )?,
    )?;
    let private_resolution = private_resolution_from_env_override(
        env,
        "ETAS_HOST_MODEL_ALLOW_PRIVATE",
        profile
            .allow_private
            .map(private_resolution_from_flag)
            .or_else(|| current.map(|model| model.private_resolution))
            .unwrap_or(PrivateResolutionPolicy::PublicOnly),
    )?;
    model_from_parts(
        adapter_name,
        model,
        base_url,
        api_key,
        retry,
        private_resolution,
    )
}

fn model_from_env_overrides(
    env: &RuntimeEnvOverrides,
    current: Option<&CliModelConfig>,
) -> Result<CliModelConfig, CliError> {
    let adapter_name = env
        .optional("ETAS_HOST_MODEL_ADAPTER")
        .or_else(|| current.map(|model| model.provider.0.clone()))
        .ok_or_else(|| {
            CliError::InvalidUsage(
                "`ETAS_HOST_MODEL_ADAPTER` is required for model env override".to_owned(),
            )
        })?;
    let model = env
        .optional("ETAS_HOST_MODEL_NAME")
        .or_else(|| current.map(|model| model.model.0.clone()))
        .ok_or_else(|| {
            CliError::InvalidUsage(
                "`ETAS_HOST_MODEL_NAME` is required for model env override".to_owned(),
            )
        })?;
    let base_url = env
        .optional("ETAS_HOST_MODEL_BASE_URL")
        .or_else(|| current.map(|model| model.base_url.clone()));
    let api_key = env
        .optional("ETAS_HOST_MODEL_API_KEY")
        .or_else(|| env.optional("ETAS_HOST_OMLX_API_KEY"));
    let retry = retry_policy_from_env(
        env,
        "ETAS_HOST_MODEL_RETRY_ATTEMPTS",
        "ETAS_HOST_MODEL_RETRY_DELAY_MS",
        current
            .map(|model| model.retry)
            .unwrap_or_else(default_transport_retry),
    )?;
    let private_resolution = private_resolution_from_env_override(
        env,
        "ETAS_HOST_MODEL_ALLOW_PRIVATE",
        current
            .map(|model| model.private_resolution)
            .unwrap_or(PrivateResolutionPolicy::PublicOnly),
    )?;
    model_from_parts(
        adapter_name,
        model,
        base_url,
        api_key,
        retry,
        private_resolution,
    )
}

fn model_from_parts(
    adapter_name: String,
    model: String,
    base_url_override: Option<String>,
    api_key: Option<String>,
    retry: RetryPolicy,
    private_resolution: PrivateResolutionPolicy,
) -> Result<CliModelConfig, CliError> {
    let auth = api_key.map_or(AuthConfig::None, AuthConfig::BearerToken);
    let (adapter, base_url, capabilities, private_resolution) = match adapter_name.as_str() {
        "openai" => {
            let base_url = base_url_override.ok_or_else(|| {
                CliError::InvalidUsage("`openai` model adapter requires `base_url`".to_owned())
            })?;
            (
                CliModelAdapter::OpenAi(
                    OpenAiProtocolAdapter::try_new_with_policy(
                        base_url.clone(),
                        private_resolution,
                    )
                    .map_err(host_transport_error)?,
                ),
                base_url,
                OpenAiProtocolAdapter::capabilities(),
                private_resolution,
            )
        }
        "anthropic" => {
            let base_url = base_url_override.ok_or_else(|| {
                CliError::InvalidUsage("`anthropic` model adapter requires `base_url`".to_owned())
            })?;
            (
                CliModelAdapter::Anthropic(
                    AnthropicProtocolAdapter::try_new_with_policy(
                        base_url.clone(),
                        private_resolution,
                    )
                    .map_err(host_transport_error)?,
                ),
                base_url,
                AnthropicProtocolAdapter::capabilities(),
                private_resolution,
            )
        }
        "omlx-openai" => {
            let base_url = base_url_override
                .unwrap_or_else(|| OpenAiProtocolAdapter::LOCAL_OMLX_BASE_URL.to_owned());
            (
                CliModelAdapter::OpenAi(
                    OpenAiProtocolAdapter::omlx_compatible(base_url.clone())
                        .map_err(host_transport_error)?,
                ),
                base_url,
                OpenAiProtocolAdapter::omlx_capabilities(),
                PrivateResolutionPolicy::AllowPrivate,
            )
        }
        "omlx-anthropic" => {
            let base_url = base_url_override
                .unwrap_or_else(|| AnthropicProtocolAdapter::LOCAL_OMLX_BASE_URL.to_owned());
            (
                CliModelAdapter::Anthropic(
                    AnthropicProtocolAdapter::omlx_compatible(base_url.clone())
                        .map_err(host_transport_error)?,
                ),
                base_url,
                AnthropicProtocolAdapter::capabilities(),
                PrivateResolutionPolicy::AllowPrivate,
            )
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "unsupported model adapter `{other}`"
            )));
        }
    };
    let adapter = match adapter {
        CliModelAdapter::OpenAi(adapter) => {
            CliModelAdapter::OpenAi(adapter.with_auth(auth).with_retry(retry))
        }
        CliModelAdapter::Anthropic(adapter) => {
            CliModelAdapter::Anthropic(adapter.with_auth(auth).with_retry(retry))
        }
    };
    Ok(CliModelConfig {
        adapter,
        provider: ModelProviderId(adapter_name),
        capabilities,
        model: ModelName(model),
        base_url: base_url.clone(),
        endpoint: network_endpoint_from_base_url(&base_url)?,
        private_resolution,
        retry,
    })
}

fn memory_mode_from_profile(profile: &RuntimeBackendProfile) -> Result<MemoryMode, CliError> {
    let backend = profile.backend.as_deref().unwrap_or("none");
    match backend {
        "none" => Ok(MemoryMode::None),
        "memory" => Ok(MemoryMode::Memory),
        "sqlite" => {
            let path = profile.path.clone().ok_or_else(|| {
                CliError::InvalidUsage("runtime memory `sqlite` backend requires `path`".to_owned())
            })?;
            Ok(MemoryMode::Sqlite { path })
        }
        other => Err(CliError::InvalidUsage(format!(
            "unsupported runtime memory backend `{other}`"
        ))),
    }
}

fn session_mode_from_profile(
    profile: &RuntimeBackendProfile,
    session_id: Option<&String>,
) -> Result<SessionMode, CliError> {
    let backend = profile.backend.as_deref().unwrap_or_else(|| {
        if session_id.is_some() {
            "memory"
        } else {
            "none"
        }
    });
    match backend {
        "none" => Ok(SessionMode::None),
        "memory" => Ok(SessionMode::Memory),
        "sqlite" => {
            let path = profile.path.clone().ok_or_else(|| {
                CliError::InvalidUsage(
                    "runtime session `sqlite` backend requires `path`".to_owned(),
                )
            })?;
            Ok(SessionMode::Sqlite { path })
        }
        other => Err(CliError::InvalidUsage(format!(
            "unsupported runtime session backend `{other}`"
        ))),
    }
}

fn policy_local_from_rules(
    mode: PolicyMode,
    rules: &[String],
) -> Result<Option<LocalStaticPolicyClient>, CliError> {
    if mode != PolicyMode::LocalStatic {
        return Ok(None);
    }
    let rules = rules
        .iter()
        .map(|rule| parse_local_policy_rule(rule))
        .collect::<Result<Vec<_>, _>>()?;
    if rules.is_empty() {
        return Err(CliError::InvalidUsage(
            "`runtime.policy.mode = \"local-static\"` requires at least one rule".to_owned(),
        ));
    }
    Ok(Some(LocalStaticPolicyClient::deny_by_default(rules)))
}

fn policy_local_from_env_overrides(
    mode: PolicyMode,
    env: &RuntimeEnvOverrides,
) -> Result<Option<LocalStaticPolicyClient>, CliError> {
    if mode != PolicyMode::LocalStatic {
        return Ok(None);
    }
    let rules = env.optional("ETAS_HOST_POLICY_RULES").ok_or_else(|| {
        CliError::InvalidUsage(
            "`ETAS_HOST_POLICY=local-static` requires `ETAS_HOST_POLICY_RULES`".to_owned(),
        )
    })?;
    let rules = rules
        .split(',')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .map(parse_local_policy_rule)
        .collect::<Result<Vec<_>, _>>()?;
    if rules.is_empty() {
        return Err(CliError::InvalidUsage(
            "`ETAS_HOST_POLICY_RULES` must include at least one rule".to_owned(),
        ));
    }
    Ok(Some(LocalStaticPolicyClient::deny_by_default(rules)))
}

fn policy_http_from_profile(
    mode: PolicyMode,
    profile: &RuntimePolicyProfile,
    env: &RuntimeEnvOverrides,
) -> Result<Option<CliPolicyHttpConfig>, CliError> {
    if mode != PolicyMode::Http {
        return Ok(None);
    }
    let base_url = profile.url.clone().ok_or_else(|| {
        CliError::InvalidUsage("`runtime.policy.mode = \"http\"` requires `url`".to_owned())
    })?;
    let endpoint = network_endpoint_from_base_url(&base_url)?;
    let auth = profile
        .token_env
        .as_deref()
        .and_then(|name| env.optional(name))
        .map_or(AuthConfig::None, AuthConfig::BearerToken);
    let retry = retry_policy_from_profile(profile.retry.as_ref(), default_transport_retry())?;
    let private_resolution = private_resolution_from_env_override(
        env,
        "ETAS_HOST_POLICY_ALLOW_PRIVATE",
        profile
            .allow_private
            .map(private_resolution_from_flag)
            .unwrap_or(PrivateResolutionPolicy::PublicOnly),
    )?;
    let mut client = HttpPolicyClient::try_new_with_policy(base_url.clone(), private_resolution)
        .map_err(host_transport_error)?
        .with_auth(auth)
        .with_retry(retry);
    if let Some(path) = &profile.path {
        client = client.with_path(path.clone());
    }
    Ok(Some(CliPolicyHttpConfig {
        client,
        base_url,
        endpoint,
        private_resolution,
        retry,
    }))
}

fn policy_http_from_env_overrides(
    mode: PolicyMode,
    env: &RuntimeEnvOverrides,
    current: Option<&CliPolicyHttpConfig>,
) -> Result<Option<CliPolicyHttpConfig>, CliError> {
    if mode != PolicyMode::Http {
        return Ok(None);
    }
    let base_url = env
        .optional("ETAS_HOST_POLICY_URL")
        .or_else(|| current.map(|policy| policy.base_url.clone()))
        .ok_or_else(|| {
            CliError::InvalidUsage(
                "`ETAS_HOST_POLICY=http` requires `ETAS_HOST_POLICY_URL`".to_owned(),
            )
        })?;
    let endpoint = network_endpoint_from_base_url(&base_url)?;
    let auth = env
        .optional("ETAS_HOST_POLICY_TOKEN")
        .map_or(AuthConfig::None, AuthConfig::BearerToken);
    let retry = retry_policy_from_env(
        env,
        "ETAS_HOST_POLICY_RETRY_ATTEMPTS",
        "ETAS_HOST_POLICY_RETRY_DELAY_MS",
        current
            .map(|policy| policy.retry)
            .unwrap_or_else(default_transport_retry),
    )?;
    let private_resolution = private_resolution_from_env_override(
        env,
        "ETAS_HOST_POLICY_ALLOW_PRIVATE",
        current
            .map(|policy| policy.private_resolution)
            .unwrap_or(PrivateResolutionPolicy::PublicOnly),
    )?;
    let mut client = HttpPolicyClient::try_new_with_policy(base_url.clone(), private_resolution)
        .map_err(host_transport_error)?
        .with_auth(auth)
        .with_retry(retry);
    if let Some(path) = env.optional("ETAS_HOST_POLICY_PATH") {
        client = client.with_path(path);
    }
    Ok(Some(CliPolicyHttpConfig {
        client,
        base_url,
        endpoint,
        private_resolution,
        retry,
    }))
}

fn host_transport_error(error: etas_host::HostError) -> CliError {
    CliError::InvalidUsage(format!(
        "invalid runtime transport endpoint: {}",
        error.message
    ))
}

fn private_resolution_from_flag(allow_private: bool) -> PrivateResolutionPolicy {
    if allow_private {
        PrivateResolutionPolicy::AllowPrivate
    } else {
        PrivateResolutionPolicy::PublicOnly
    }
}

fn private_resolution_name(policy: PrivateResolutionPolicy) -> &'static str {
    match policy {
        PrivateResolutionPolicy::PublicOnly => "public-only",
        PrivateResolutionPolicy::AllowPrivate => "allow-private",
    }
}

fn parse_private_resolution_flag(name: &str, value: &str) -> Result<bool, CliError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CliError::InvalidUsage(format!(
            "`{name}` must be `true` or `false`"
        ))),
    }
}

fn private_resolution_from_env_override(
    env: &RuntimeEnvOverrides,
    name: &'static str,
    base: PrivateResolutionPolicy,
) -> Result<PrivateResolutionPolicy, CliError> {
    env.optional(name)
        .map(|value| parse_private_resolution_flag(name, &value))
        .transpose()
        .map(|value| value.map(private_resolution_from_flag).unwrap_or(base))
}

fn private_resolution_from_env_var(
    name: &'static str,
    base: PrivateResolutionPolicy,
) -> Result<PrivateResolutionPolicy, CliError> {
    optional_env(name)?
        .map(|value| parse_private_resolution_flag(name, &value))
        .transpose()
        .map(|value| value.map(private_resolution_from_flag).unwrap_or(base))
}

fn network_allowlist_from_profile(
    profile: &RuntimeNetworkProfile,
) -> Result<Vec<NetworkEndpoint>, CliError> {
    parse_cli_network_allowlist(&profile.allow)
}

fn workspace_root_from_path(path: &Path) -> Result<WorkspaceRoot, CliError> {
    WorkspaceRoot::new(path).map_err(|error| {
        CliError::InvalidUsage(format!(
            "invalid runtime filesystem workspace root `{}`: {}",
            path.display(),
            error.message
        ))
    })
}

fn default_transport_retry() -> RetryPolicy {
    RetryPolicy {
        attempts: 3,
        delay: Duration::from_millis(25),
    }
}

fn retry_policy_from_profile(
    profile: Option<&RuntimeRetryProfile>,
    base: RetryPolicy,
) -> Result<RetryPolicy, CliError> {
    let attempts = profile
        .and_then(|profile| profile.attempts)
        .unwrap_or(base.attempts);
    let delay_ms = profile
        .and_then(|profile| profile.delay_ms)
        .unwrap_or(base.delay.as_millis() as u64);
    retry_policy_from_parts(attempts, delay_ms)
}

fn retry_policy_from_env(
    env: &RuntimeEnvOverrides,
    attempts_name: &'static str,
    delay_name: &'static str,
    base: RetryPolicy,
) -> Result<RetryPolicy, CliError> {
    let attempts = env
        .optional(attempts_name)
        .map(|value| parse_retry_attempts(attempts_name, &value))
        .transpose()?
        .unwrap_or(base.attempts);
    let delay_ms = env
        .optional(delay_name)
        .map(|value| parse_retry_delay_ms(delay_name, &value))
        .transpose()?
        .unwrap_or(base.delay.as_millis() as u64);
    retry_policy_from_parts(attempts, delay_ms)
}

fn retry_policy_from_env_vars(
    attempts_name: &'static str,
    delay_name: &'static str,
    base: RetryPolicy,
) -> Result<RetryPolicy, CliError> {
    let attempts = optional_env(attempts_name)?
        .map(|value| parse_retry_attempts(attempts_name, &value))
        .transpose()?
        .unwrap_or(base.attempts);
    let delay_ms = optional_env(delay_name)?
        .map(|value| parse_retry_delay_ms(delay_name, &value))
        .transpose()?
        .unwrap_or(base.delay.as_millis() as u64);
    retry_policy_from_parts(attempts, delay_ms)
}

fn retry_policy_from_parts(attempts: u8, delay_ms: u64) -> Result<RetryPolicy, CliError> {
    if attempts == 0 {
        return Err(CliError::InvalidUsage(
            "runtime retry attempts must be at least 1".to_owned(),
        ));
    }
    Ok(RetryPolicy {
        attempts,
        delay: Duration::from_millis(delay_ms),
    })
}

fn parse_retry_attempts(name: &'static str, value: &str) -> Result<u8, CliError> {
    value.parse::<u8>().map_err(|source| {
        CliError::InvalidUsage(format!(
            "`{name}` must be an integer between 1 and 255: {source}"
        ))
    })
}

fn parse_retry_delay_ms(name: &'static str, value: &str) -> Result<u64, CliError> {
    value.parse::<u64>().map_err(|source| {
        CliError::InvalidUsage(format!("`{name}` must be a non-negative integer: {source}"))
    })
}

fn runtime_profile_fingerprint(profile: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(profile).unwrap_or_default();
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn model_from_environment() -> Result<Option<CliModelConfig>, CliError> {
    let Some(adapter_name) = optional_env("ETAS_HOST_MODEL_ADAPTER")? else {
        return Ok(None);
    };
    let model = required_env("ETAS_HOST_MODEL_NAME")?;
    let base_url = optional_env("ETAS_HOST_MODEL_BASE_URL")?;
    let api_key =
        optional_env("ETAS_HOST_MODEL_API_KEY")?.or(optional_env("ETAS_HOST_OMLX_API_KEY")?);
    let retry = retry_policy_from_env_vars(
        "ETAS_HOST_MODEL_RETRY_ATTEMPTS",
        "ETAS_HOST_MODEL_RETRY_DELAY_MS",
        default_transport_retry(),
    )?;
    let private_resolution = private_resolution_from_env_var(
        "ETAS_HOST_MODEL_ALLOW_PRIVATE",
        PrivateResolutionPolicy::PublicOnly,
    )?;
    model_from_parts(
        adapter_name,
        model,
        base_url,
        api_key,
        retry,
        private_resolution,
    )
    .map(Some)
}

fn optional_env(name: &'static str) -> Result<Option<String>, CliError> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) | Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(CliError::InvalidUsage(format!(
            "environment variable `{name}` is not valid unicode"
        ))),
    }
}

fn required_env(name: &'static str) -> Result<String, CliError> {
    optional_env(name)?
        .ok_or_else(|| CliError::InvalidUsage(format!("environment variable `{name}` is required")))
}

fn parse_filesystem_mode(value: Option<String>) -> Result<FilesystemMode, CliError> {
    match value.as_deref().unwrap_or("none") {
        "none" => Ok(FilesystemMode::None),
        "readonly" => Ok(FilesystemMode::ReadOnly),
        "readwrite" => Ok(FilesystemMode::ReadWrite),
        "destructive" => Ok(FilesystemMode::Destructive),
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_FILESYSTEM mode `{other}`"
        ))),
    }
}

fn parse_memory_mode(
    value: Option<String>,
    path_override: Option<String>,
) -> Result<MemoryMode, CliError> {
    let value = value.unwrap_or_else(|| "none".to_owned());
    if let Some(path) = value.strip_prefix("sqlite:") {
        if path.is_empty() {
            return Err(CliError::InvalidUsage(
                "`ETAS_HOST_MEMORY=sqlite:<path>` requires a non-empty path".to_owned(),
            ));
        }
        return Ok(MemoryMode::Sqlite {
            path: PathBuf::from(path),
        });
    }
    match value.as_str() {
        "none" => Ok(MemoryMode::None),
        "memory" => Ok(MemoryMode::Memory),
        "sqlite" => {
            let path = path_override.ok_or_else(|| {
                CliError::InvalidUsage(
                    "`ETAS_HOST_MEMORY=sqlite` requires `ETAS_HOST_MEMORY_PATH`".to_owned(),
                )
            })?;
            Ok(MemoryMode::Sqlite {
                path: PathBuf::from(path),
            })
        }
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_MEMORY mode `{other}`"
        ))),
    }
}

fn memory_profile_json(mode: &MemoryMode) -> serde_json::Value {
    match mode {
        MemoryMode::None => serde_json::json!({ "kind": "none" }),
        MemoryMode::Memory => serde_json::json!({ "kind": "memory" }),
        MemoryMode::Sqlite { path } => serde_json::json!({
            "kind": "sqlite",
            "path": path.display().to_string(),
        }),
    }
}

fn parse_session_mode(
    value: Option<String>,
    path_override: Option<String>,
    session_id: Option<&String>,
) -> Result<SessionMode, CliError> {
    let Some(value) = value else {
        return if session_id.is_some() {
            Ok(SessionMode::Memory)
        } else {
            Ok(SessionMode::None)
        };
    };
    if let Some(path) = value.strip_prefix("sqlite:") {
        if path.is_empty() {
            return Err(CliError::InvalidUsage(
                "`ETAS_HOST_SESSION=sqlite:<path>` requires a non-empty path".to_owned(),
            ));
        }
        return Ok(SessionMode::Sqlite {
            path: PathBuf::from(path),
        });
    }
    match value.as_str() {
        "none" => Ok(SessionMode::None),
        "memory" => Ok(SessionMode::Memory),
        "sqlite" => {
            let path = path_override.ok_or_else(|| {
                CliError::InvalidUsage(
                    "`ETAS_HOST_SESSION=sqlite` requires `ETAS_HOST_SESSION_PATH`".to_owned(),
                )
            })?;
            Ok(SessionMode::Sqlite {
                path: PathBuf::from(path),
            })
        }
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_SESSION mode `{other}`"
        ))),
    }
}

fn session_profile_json(mode: &SessionMode, session_id: Option<&str>) -> serde_json::Value {
    match mode {
        SessionMode::None => serde_json::json!({
            "id": session_id,
            "kind": "none",
        }),
        SessionMode::Memory => serde_json::json!({
            "id": session_id,
            "kind": "memory",
        }),
        SessionMode::Sqlite { path } => serde_json::json!({
            "id": session_id,
            "kind": "sqlite",
            "path": path.display().to_string(),
        }),
    }
}

fn retry_policy_json(policy: RetryPolicy) -> serde_json::Value {
    serde_json::json!({
        "attempts": policy.attempts,
        "delay_ms": policy.delay.as_millis(),
    })
}

fn parse_secret_mode(value: Option<String>) -> Result<SecretMode, CliError> {
    match value.as_deref().unwrap_or("none") {
        "none" => Ok(SecretMode::None),
        "env" => Ok(SecretMode::Env),
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_SECRET mode `{other}`"
        ))),
    }
}

fn parse_approval_mode(value: Option<String>) -> Result<ApprovalMode, CliError> {
    match value.as_deref().unwrap_or("deny") {
        "deny" => Ok(ApprovalMode::Deny),
        "auto" => Ok(ApprovalMode::Auto),
        "prompt" => Ok(ApprovalMode::Prompt),
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_APPROVAL mode `{other}`"
        ))),
    }
}

fn parse_policy_mode(value: Option<String>) -> Result<PolicyMode, CliError> {
    match value.as_deref().unwrap_or("deny") {
        "deny" | "deny-unknown" => Ok(PolicyMode::DenyUnknown),
        "unsafe-trace-spec-runtime" => Ok(PolicyMode::UnsafeTraceSpecRuntime),
        "unsafe-allow-local-static" => Ok(PolicyMode::UnsafeAllowLocalStatic),
        "unsafe-require-approval-local-static" => Ok(PolicyMode::UnsafeRequireApprovalLocalStatic),
        "local-static" => Ok(PolicyMode::LocalStatic),
        "http" => Ok(PolicyMode::Http),
        other => Err(CliError::InvalidUsage(format!(
            "unsupported ETAS_HOST_POLICY mode `{other}`"
        ))),
    }
}

fn policy_local_from_environment(
    mode: PolicyMode,
) -> Result<Option<LocalStaticPolicyClient>, CliError> {
    if mode != PolicyMode::LocalStatic {
        return Ok(None);
    }
    let rules = required_env("ETAS_HOST_POLICY_RULES")?;
    let rules = rules
        .split(',')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .map(parse_local_policy_rule)
        .collect::<Result<Vec<_>, _>>()?;
    if rules.is_empty() {
        return Err(CliError::InvalidUsage(
            "`ETAS_HOST_POLICY=local-static` requires at least one rule in `ETAS_HOST_POLICY_RULES`"
                .to_owned(),
        ));
    }
    Ok(Some(LocalStaticPolicyClient::deny_by_default(rules)))
}

fn parse_local_policy_rule(rule: &str) -> Result<LocalPolicyRule, CliError> {
    let Some((selector, decision)) = rule.split_once('=') else {
        return Err(CliError::InvalidUsage(format!(
            "invalid local policy rule `{rule}`; expected `kind[:Qualified.Action<:resource_prefix>]=allow|approval|deny`"
        )));
    };
    let decision = parse_local_policy_decision(decision.trim())?;
    let mut parts = selector.split(':').map(str::trim);
    let Some(kind) = parts.next().filter(|kind| !kind.is_empty()) else {
        return Err(CliError::InvalidUsage(format!(
            "invalid local policy rule `{rule}`; subject kind is required"
        )));
    };
    let mut parsed = LocalPolicyRule::new(kind, decision);
    if let Some(action) = parts.next().filter(|action| !action.is_empty()) {
        parsed = parsed.qualified_action(action);
    }
    if let Some(resource) = parts.next().filter(|resource| !resource.is_empty()) {
        parsed = parsed.resource_prefix(resource);
    }
    if parts.next().is_some() {
        return Err(CliError::InvalidUsage(format!(
            "invalid local policy rule `{rule}`; too many selector components"
        )));
    }
    Ok(parsed)
}

fn parse_local_policy_decision(value: &str) -> Result<LocalPolicyDecision, CliError> {
    match value {
        "allow" => Ok(LocalPolicyDecision::Allow),
        "approval" | "require-approval" | "require_approval" => {
            Ok(LocalPolicyDecision::RequireApproval)
        }
        "deny" => Ok(LocalPolicyDecision::Deny),
        other => Err(CliError::InvalidUsage(format!(
            "unsupported local policy decision `{other}`"
        ))),
    }
}

fn policy_http_from_environment(mode: PolicyMode) -> Result<Option<CliPolicyHttpConfig>, CliError> {
    if mode != PolicyMode::Http {
        return Ok(None);
    }
    let base_url = required_env("ETAS_HOST_POLICY_URL")?;
    let endpoint = network_endpoint_from_base_url(&base_url)?;
    let auth =
        optional_env("ETAS_HOST_POLICY_TOKEN")?.map_or(AuthConfig::None, AuthConfig::BearerToken);
    let path = optional_env("ETAS_HOST_POLICY_PATH")?;
    let retry = retry_policy_from_env_vars(
        "ETAS_HOST_POLICY_RETRY_ATTEMPTS",
        "ETAS_HOST_POLICY_RETRY_DELAY_MS",
        default_transport_retry(),
    )?;
    let private_resolution = private_resolution_from_env_var(
        "ETAS_HOST_POLICY_ALLOW_PRIVATE",
        PrivateResolutionPolicy::PublicOnly,
    )?;
    let mut client = HttpPolicyClient::try_new_with_policy(base_url.clone(), private_resolution)
        .map_err(host_transport_error)?
        .with_auth(auth)
        .with_retry(retry);
    if let Some(path) = path {
        client = client.with_path(path);
    }
    Ok(Some(CliPolicyHttpConfig {
        client,
        base_url,
        endpoint,
        private_resolution,
        retry,
    }))
}

fn parse_network_allowlist(value: Option<String>) -> Result<Vec<NetworkEndpoint>, CliError> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(parse_network_allowlist_endpoint)
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn parse_cli_network_allowlist(values: &[String]) -> Result<Vec<NetworkEndpoint>, CliError> {
    let mut endpoints = Vec::new();
    for value in values {
        if value.contains("://") {
            endpoints.push(network_endpoint_from_base_url(value)?);
        } else {
            let tcp = network_endpoint_from_base_url(&format!("tcp://{value}"))?;
            let tls = NetworkEndpoint::new("tls", tcp.host.clone(), tcp.port);
            endpoints.push(tcp);
            endpoints.push(tls);
        }
    }
    Ok(endpoints)
}

fn parse_network_allowlist_endpoint(value: &str) -> Result<NetworkEndpoint, CliError> {
    if value.contains("://") {
        network_endpoint_from_base_url(value)
    } else {
        network_endpoint_from_base_url(&format!("tcp://{value}"))
    }
}

fn sort_dedup_network_allowlist(endpoints: &mut Vec<NetworkEndpoint>) {
    endpoints.sort_by(|left, right| {
        (&left.scheme, &left.host, left.port).cmp(&(&right.scheme, &right.host, right.port))
    });
    endpoints.dedup();
}

fn filesystem_policy(
    workspace_root: Option<WorkspaceRoot>,
    mode: FilesystemMode,
) -> FilesystemPolicy {
    let Some(root) = workspace_root else {
        return FilesystemPolicy::deny_all();
    };
    match mode {
        FilesystemMode::None => FilesystemPolicy::deny_all(),
        FilesystemMode::ReadOnly => FilesystemPolicy {
            read_roots: vec![root],
            write_roots: Vec::new(),
            delete_roots: Vec::new(),
        },
        FilesystemMode::ReadWrite => FilesystemPolicy::allow_workspace(root),
        FilesystemMode::Destructive => FilesystemPolicy::allow_destructive_workspace(root),
    }
}

fn destructive_policy(mode: FilesystemMode) -> DestructiveOpPolicy {
    match mode {
        FilesystemMode::Destructive => DestructiveOpPolicy { allow_delete: true },
        FilesystemMode::None | FilesystemMode::ReadOnly | FilesystemMode::ReadWrite => {
            DestructiveOpPolicy::deny_all()
        }
    }
}

fn network_endpoint_from_base_url(base_url: &str) -> Result<NetworkEndpoint, CliError> {
    let (scheme, rest) = base_url.split_once("://").ok_or_else(|| {
        CliError::InvalidUsage(format!("network endpoint `{base_url}` is missing scheme"))
    })?;
    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') || authority.starts_with('[') {
        return Err(CliError::InvalidUsage(format!(
            "network endpoint `{base_url}` has an unsupported authority"
        )));
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port.parse::<u16>().map_err(|_| {
                CliError::InvalidUsage(format!("network endpoint `{base_url}` has invalid port"))
            })?;
            (host, port)
        }
        None if scheme == "https" => (authority, 443),
        None if scheme == "http" => (authority, 80),
        None => {
            return Err(CliError::InvalidUsage(format!(
                "network endpoint `{base_url}` requires an explicit port for scheme `{scheme}`"
            )));
        }
    };
    if host.is_empty() {
        return Err(CliError::InvalidUsage(format!(
            "network endpoint `{base_url}` is missing host"
        )));
    }
    Ok(NetworkEndpoint::new(scheme, host, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_memory_mode_accepts_sqlite_path_inline() {
        assert_eq!(
            parse_memory_mode(Some("sqlite:/tmp/etas-memory.sqlite".to_owned()), None).unwrap(),
            MemoryMode::Sqlite {
                path: PathBuf::from("/tmp/etas-memory.sqlite"),
            }
        );
    }

    #[test]
    fn parse_memory_mode_accepts_sqlite_path_env() {
        assert_eq!(
            parse_memory_mode(
                Some("sqlite".to_owned()),
                Some("/tmp/etas-memory.sqlite".to_owned())
            )
            .unwrap(),
            MemoryMode::Sqlite {
                path: PathBuf::from("/tmp/etas-memory.sqlite"),
            }
        );
    }

    #[test]
    fn parse_memory_mode_rejects_sqlite_without_path() {
        let error = parse_memory_mode(Some("sqlite".to_owned()), None)
            .expect_err("sqlite memory mode must require a path");
        assert!(
            error.to_string().contains("ETAS_HOST_MEMORY_PATH"),
            "{error}"
        );
    }

    #[test]
    fn memory_profile_reports_sqlite_path() {
        assert_eq!(
            memory_profile_json(&MemoryMode::Sqlite {
                path: PathBuf::from("/tmp/etas-memory.sqlite"),
            }),
            serde_json::json!({
                "kind": "sqlite",
                "path": "/tmp/etas-memory.sqlite",
            })
        );
    }

    #[test]
    fn parse_session_mode_defaults_to_memory_when_session_id_is_present() {
        assert_eq!(
            parse_session_mode(None, None, Some(&"session-42".to_owned())).unwrap(),
            SessionMode::Memory
        );
    }

    #[test]
    fn parse_session_mode_accepts_sqlite_path_inline() {
        assert_eq!(
            parse_session_mode(
                Some("sqlite:/tmp/etas-session.sqlite".to_owned()),
                None,
                None
            )
            .unwrap(),
            SessionMode::Sqlite {
                path: PathBuf::from("/tmp/etas-session.sqlite"),
            }
        );
    }

    #[test]
    fn parse_session_mode_accepts_sqlite_path_env() {
        assert_eq!(
            parse_session_mode(
                Some("sqlite".to_owned()),
                Some("/tmp/etas-session.sqlite".to_owned()),
                None
            )
            .unwrap(),
            SessionMode::Sqlite {
                path: PathBuf::from("/tmp/etas-session.sqlite"),
            }
        );
    }

    #[test]
    fn parse_session_mode_rejects_sqlite_without_path() {
        let error = parse_session_mode(Some("sqlite".to_owned()), None, None)
            .expect_err("sqlite session mode must require a path");
        assert!(
            error.to_string().contains("ETAS_HOST_SESSION_PATH"),
            "{error}"
        );
    }

    #[test]
    fn session_profile_reports_sqlite_path() {
        assert_eq!(
            session_profile_json(
                &SessionMode::Sqlite {
                    path: PathBuf::from("/tmp/etas-session.sqlite"),
                },
                Some("session-42")
            ),
            serde_json::json!({
                "id": "session-42",
                "kind": "sqlite",
                "path": "/tmp/etas-session.sqlite",
            })
        );
    }

    #[test]
    fn parse_secret_mode_defaults_to_none_and_accepts_env() {
        assert_eq!(parse_secret_mode(None).unwrap(), SecretMode::None);
        assert_eq!(
            parse_secret_mode(Some("env".to_owned())).unwrap(),
            SecretMode::Env
        );
    }

    #[test]
    fn parse_secret_mode_rejects_unknown_provider() {
        let error = parse_secret_mode(Some("all-env".to_owned()))
            .expect_err("secret mode should be explicitly bounded to known providers");
        assert!(error.to_string().contains("ETAS_HOST_SECRET"), "{error}");
    }

    #[test]
    fn secret_grants_require_explicit_secret_provider() {
        let mut config = CliHostConfig::none();
        assert!(!has_secret_grant(&config.run_options()));

        config.secret_mode = SecretMode::Env;
        assert!(has_secret_grant(&config.run_options()));
    }

    #[test]
    fn run_options_include_configured_runtime_session() {
        let mut config = CliHostConfig::none();
        config.session_id = Some("session-42".to_owned());
        config.session_mode = SessionMode::Memory;
        let options = config.run_options();
        assert_eq!(options.current_session.as_deref(), Some("session-42"));
        assert_eq!(
            config.runtime_profile_json()["session"],
            serde_json::json!({
                "id": "session-42",
                "kind": "memory",
            })
        );
    }

    #[test]
    fn runtime_execution_config_merges_manifest_local_and_cli_precedence() {
        let manifest = RuntimeExecutionProfile {
            max_call_depth: Some(120),
            max_steps: Some(1_000_000),
        };
        let local = RuntimeExecutionProfile {
            max_call_depth: Some(96),
            max_steps: Some(500_000),
        };
        let mut config = CliHostConfig::none();
        config
            .apply_execution_config(&manifest, &local, Some(32))
            .expect("valid execution config should merge");

        assert_eq!(config.execution_limits.max_call_depth.get(), 32);
        assert_eq!(
            config.execution_limits.max_steps.map(NonZeroU64::get),
            Some(500_000)
        );
        assert_eq!(
            config.run_options().execution_limits,
            config.execution_limits
        );
        assert_eq!(
            config.runtime_profile_json()["execution"],
            serde_json::json!({
                "max_call_depth": 32,
                "max_steps": 500_000,
            })
        );
    }

    #[test]
    fn runtime_execution_config_rejects_zero_and_depth_above_hard_cap() {
        let mut config = CliHostConfig::none();
        let zero = RuntimeExecutionProfile {
            max_call_depth: Some(0),
            max_steps: None,
        };
        let error = config
            .apply_execution_config(&zero, &RuntimeExecutionProfile::default(), None)
            .expect_err("zero call depth must be rejected");
        assert!(error.to_string().contains("greater than zero"), "{error}");

        let error = config
            .apply_execution_config(
                &RuntimeExecutionProfile::default(),
                &RuntimeExecutionProfile::default(),
                Some(65_537),
            )
            .expect_err("call depth above the hard cap must be rejected");
        assert!(error.to_string().contains("hard cap 65536"), "{error}");
    }

    #[test]
    fn runtime_profile_merges_local_override_and_redacts_secret() {
        let manifest_profile = RuntimeProfile {
            model: Some(RuntimeModelProfile {
                adapter: Some("omlx-openai".to_owned()),
                model: Some("manifest-model".to_owned()),
                base_url: Some("http://127.0.0.1:8848/v1".to_owned()),
                api_key_env: Some("ETAS_TEST_OMLX_KEY".to_owned()),
                allow_private: Some(true),
                retry: Some(RuntimeRetryProfile {
                    attempts: Some(5),
                    delay_ms: Some(50),
                }),
            }),
            memory: Some(RuntimeBackendProfile {
                backend: Some("memory".to_owned()),
                path: None,
                id: None,
            }),
            ..RuntimeProfile::default()
        };
        let local_profile = RuntimeProfile {
            model: Some(RuntimeModelProfile {
                adapter: Some("omlx-openai".to_owned()),
                model: Some("local-model".to_owned()),
                base_url: Some("http://127.0.0.1:9999/v1".to_owned()),
                api_key_env: Some("ETAS_TEST_OMLX_KEY".to_owned()),
                allow_private: Some(true),
                retry: None,
            }),
            network: Some(RuntimeNetworkProfile {
                allow: vec!["127.0.0.1:9999".to_owned()],
            }),
            ..RuntimeProfile::default()
        };
        let mut env = RuntimeEnvOverrides::default();
        env.values
            .insert("ETAS_TEST_OMLX_KEY".to_owned(), "secret-value".to_owned());

        let config = CliHostConfig::from_runtime_profile(
            "local-omlx".to_owned(),
            Some(&manifest_profile),
            Some(&local_profile),
            &env,
            &[],
        )
        .expect("runtime profile should build");
        let json = config.runtime_profile_json();

        assert_eq!(json["profile"], "local-omlx");
        assert_eq!(json["model"]["model"], "local-model");
        assert_eq!(json["model"]["retry"]["attempts"], 5);
        assert_eq!(json["model"]["retry"]["delay_ms"], 50);
        assert!(
            !serde_json::to_string(&json)
                .expect("json serializes")
                .contains("secret-value")
        );
        assert!(
            config
                .program_network_allowlist
                .iter()
                .any(|endpoint| { endpoint.host == "127.0.0.1" && endpoint.port == 9999 }),
            "explicit profile network endpoint should be program-visible"
        );
        assert!(
            config
                .adapter_transport_allowlist
                .iter()
                .any(|endpoint| { endpoint.host == "127.0.0.1" && endpoint.port == 9999 }),
            "model endpoint should be materialized only as adapter transport authority"
        );
    }

    #[test]
    fn private_model_resolution_requires_explicit_adapter_authority() {
        let public_profile = RuntimeModelProfile {
            adapter: Some("openai".to_owned()),
            model: Some("mock-model".to_owned()),
            base_url: Some("http://127.0.0.1:8848/v1".to_owned()),
            ..RuntimeModelProfile::default()
        };
        let public =
            model_from_runtime_profile(&public_profile, &RuntimeEnvOverrides::default(), None)
                .expect("transport authority construction should not perform I/O");
        assert_eq!(
            public.private_resolution,
            PrivateResolutionPolicy::PublicOnly
        );

        let private_profile = RuntimeModelProfile {
            allow_private: Some(true),
            ..public_profile
        };
        let private =
            model_from_runtime_profile(&private_profile, &RuntimeEnvOverrides::default(), None)
                .expect("explicit private transport authority should be accepted");
        assert_eq!(
            private.private_resolution,
            PrivateResolutionPolicy::AllowPrivate
        );
    }

    #[test]
    fn private_resolution_environment_override_is_strict_boolean() {
        let mut env = RuntimeEnvOverrides::default();
        env.values
            .insert("ETAS_HOST_MODEL_ALLOW_PRIVATE".to_owned(), "yes".to_owned());
        let error = private_resolution_from_env_override(
            &env,
            "ETAS_HOST_MODEL_ALLOW_PRIVATE",
            PrivateResolutionPolicy::PublicOnly,
        )
        .expect_err("non-boolean private authority must be rejected");
        assert!(error.to_string().contains("true` or `false"), "{error}");
    }

    #[test]
    fn runtime_profile_merges_tool_bindings_by_name_and_env_overrides_retry() {
        let manifest_profile = RuntimeProfile {
            tools: Some(RuntimeToolsProfile {
                http: BTreeMap::from([
                    (
                        "app.tools.Search".to_owned(),
                        "http://127.0.0.1:7001".to_owned(),
                    ),
                    (
                        "app.tools.Score".to_owned(),
                        "http://127.0.0.1:7002".to_owned(),
                    ),
                ]),
                ..RuntimeToolsProfile::default()
            }),
            model: Some(RuntimeModelProfile {
                adapter: Some("omlx-openai".to_owned()),
                model: Some("manifest-model".to_owned()),
                base_url: Some("http://127.0.0.1:8848/v1".to_owned()),
                api_key_env: None,
                allow_private: Some(true),
                retry: Some(RuntimeRetryProfile {
                    attempts: Some(4),
                    delay_ms: Some(40),
                }),
            }),
            ..RuntimeProfile::default()
        };
        let local_profile = RuntimeProfile {
            tools: Some(RuntimeToolsProfile {
                http: BTreeMap::from([(
                    "app.tools.Score".to_owned(),
                    "http://127.0.0.1:7102".to_owned(),
                )]),
                ..RuntimeToolsProfile::default()
            }),
            model: Some(RuntimeModelProfile {
                adapter: None,
                model: Some("local-model".to_owned()),
                base_url: None,
                api_key_env: None,
                allow_private: None,
                retry: None,
            }),
            ..RuntimeProfile::default()
        };
        let mut env = RuntimeEnvOverrides::default();
        env.values.insert(
            "ETAS_HOST_TOOL_HTTP".to_owned(),
            "app.tools.Fetch=http://127.0.0.1:7201,app.tools.Search=http://127.0.0.1:7202"
                .to_owned(),
        );
        env.values
            .insert("ETAS_HOST_MODEL_RETRY_ATTEMPTS".to_owned(), "2".to_owned());

        let config = CliHostConfig::from_runtime_profile(
            "local-tools".to_owned(),
            Some(&manifest_profile),
            Some(&local_profile),
            &env,
            &[],
        )
        .expect("runtime profile should build");
        assert!(
            config.program_network_allowlist.is_empty(),
            "model and tool adapter endpoints must not become program network endpoints"
        );
        assert!(!config.adapter_transport_allowlist.is_empty());
        let options = config.run_options();
        assert!(!has_program_network_grant(&options));
        assert!(
            options
                .host_context
                .authority
                .sandbox
                .network
                .allowed_endpoints
                .is_empty(),
            "adapter transport endpoints must not enter the program sandbox policy"
        );
        let mut tools = config
            .tool_bindings
            .iter()
            .map(|binding| {
                (
                    tool_binding_name(binding).to_owned(),
                    tool_binding_endpoint(binding),
                )
            })
            .collect::<Vec<_>>();
        tools.sort_by(|left, right| left.0.cmp(&right.0));

        assert_eq!(
            tools,
            vec![
                ("app.tools.Fetch".to_owned(), Some("http://127.0.0.1:7201")),
                ("app.tools.Score".to_owned(), Some("http://127.0.0.1:7102")),
                ("app.tools.Search".to_owned(), Some("http://127.0.0.1:7202")),
            ]
        );
        let json = config.runtime_profile_json();
        assert_eq!(json["model"]["model"], "local-model");
        assert_eq!(json["model"]["retry"]["attempts"], 2);
        assert_eq!(json["model"]["retry"]["delay_ms"], 40);
    }

    fn has_secret_grant(options: &RunOptions) -> bool {
        options.host_context.authority.grants.iter().any(|grant| {
            matches!(
                grant,
                etas_host::HostActionGrant::Allow(etas_host::ActionPattern::Pattern {
                    effect,
                    action,
                    ..
                }) if effect == "Secret" && matches!(action.as_str(), "read" | "use")
            )
        })
    }

    fn has_program_network_grant(options: &RunOptions) -> bool {
        options.host_context.authority.grants.iter().any(|grant| {
            matches!(
                grant,
                etas_host::HostActionGrant::Allow(etas_host::ActionPattern::Pattern {
                    effect,
                    ..
                }) if matches!(effect.as_str(), "Net" | "Tls" | "Stream")
            )
        })
    }
}
