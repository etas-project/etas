use std::collections::BTreeMap;

use etas_host::{
    HostError, HostErrorCode, HttpToolProtocolAdapter, McpToolProtocolAdapter,
    PrivateResolutionPolicy, ProcessToolProtocolAdapter, ToolClient, ToolRequest, ToolResponse,
};
use etas_interpreter::host::HostFuture;
use etas_package::RuntimeToolsProfile;

use crate::error::CliError;

#[derive(Clone, Default)]
pub(super) struct CliToolRouter {
    tools: BTreeMap<String, CliToolAdapter>,
}

#[derive(Clone)]
enum CliToolAdapter {
    Http(HttpToolProtocolAdapter),
    Mcp(McpToolProtocolAdapter),
    Process(ProcessToolProtocolAdapter),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CliToolBinding {
    Http { name: String, base_url: String },
    Mcp { name: String, base_url: String },
    Process { name: String, program: String },
}

impl CliToolRouter {
    pub(super) fn from_bindings(
        bindings: &[CliToolBinding],
        private_resolution: PrivateResolutionPolicy,
    ) -> Result<Self, CliError> {
        let mut tools = BTreeMap::new();
        for binding in bindings {
            match binding {
                CliToolBinding::Http { name, base_url } => {
                    tools.insert(
                        name.clone(),
                        CliToolAdapter::Http(
                            HttpToolProtocolAdapter::try_new_with_policy(
                                base_url,
                                "/",
                                private_resolution,
                            )
                            .map_err(|error| {
                                CliError::InvalidUsage(format!(
                                    "invalid HTTP tool endpoint: {}",
                                    error.message
                                ))
                            })?,
                        ),
                    );
                }
                CliToolBinding::Mcp { name, base_url } => {
                    tools.insert(
                        name.clone(),
                        CliToolAdapter::Mcp(
                            McpToolProtocolAdapter::try_new_with_policy(
                                base_url,
                                private_resolution,
                            )
                            .map_err(|error| {
                                CliError::InvalidUsage(format!(
                                    "invalid MCP tool endpoint: {}",
                                    error.message
                                ))
                            })?,
                        ),
                    );
                }
                CliToolBinding::Process { name, program } => {
                    tools.insert(
                        name.clone(),
                        CliToolAdapter::Process(ProcessToolProtocolAdapter::new(
                            program,
                            Vec::new(),
                        )),
                    );
                }
            }
        }
        Ok(Self { tools })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub(super) fn invoke<'a>(
        &'a self,
        request: ToolRequest,
    ) -> Option<HostFuture<'a, Result<ToolResponse, HostError>>> {
        let tool_name = request
            .tool
            .qualified_name
            .as_ref()
            .unwrap_or(&request.tool.name);
        match self.tools.get(tool_name)? {
            CliToolAdapter::Http(adapter) => {
                Some(Box::pin(async move { adapter.invoke(request).await }))
            }
            CliToolAdapter::Mcp(adapter) => {
                Some(Box::pin(async move { adapter.invoke(request).await }))
            }
            CliToolAdapter::Process(adapter) => {
                Some(Box::pin(async move { adapter.invoke(request).await }))
            }
        }
    }
}

pub(super) fn parse_tool_bindings(
    http: Option<String>,
    mcp: Option<String>,
    process: Option<String>,
) -> Result<Vec<CliToolBinding>, CliError> {
    let mut bindings = Vec::new();
    if let Some(value) = http {
        bindings.extend(
            parse_named_values(&value, "ETAS_HOST_TOOL_HTTP")?
                .into_iter()
                .map(|(name, base_url)| CliToolBinding::Http { name, base_url }),
        );
    }
    if let Some(value) = mcp {
        bindings.extend(
            parse_named_values(&value, "ETAS_HOST_TOOL_MCP")?
                .into_iter()
                .map(|(name, base_url)| CliToolBinding::Mcp { name, base_url }),
        );
    }
    if let Some(value) = process {
        bindings.extend(
            parse_named_values(&value, "ETAS_HOST_TOOL_PROCESS")?
                .into_iter()
                .map(|(name, program)| CliToolBinding::Process { name, program }),
        );
    }
    validate_unique_tool_bindings(&bindings)?;
    Ok(bindings)
}

pub(super) fn tool_bindings_from_profile(
    profile: &RuntimeToolsProfile,
) -> Result<Vec<CliToolBinding>, CliError> {
    let mut bindings = Vec::new();
    bindings.extend(
        profile
            .http
            .iter()
            .map(|(name, base_url)| CliToolBinding::Http {
                name: name.clone(),
                base_url: base_url.clone(),
            }),
    );
    bindings.extend(
        profile
            .mcp
            .iter()
            .map(|(name, base_url)| CliToolBinding::Mcp {
                name: name.clone(),
                base_url: base_url.clone(),
            }),
    );
    bindings.extend(
        profile
            .process
            .iter()
            .map(|(name, program)| CliToolBinding::Process {
                name: name.clone(),
                program: program.clone(),
            }),
    );
    validate_unique_tool_bindings(&bindings)?;
    Ok(bindings)
}

fn validate_unique_tool_bindings(bindings: &[CliToolBinding]) -> Result<(), CliError> {
    let mut seen = BTreeMap::new();
    for binding in bindings {
        let name = match binding {
            CliToolBinding::Http { name, .. }
            | CliToolBinding::Mcp { name, .. }
            | CliToolBinding::Process { name, .. } => name,
        };
        if seen.insert(name.clone(), ()).is_some() {
            return Err(CliError::InvalidUsage(format!(
                "duplicate CLI tool binding `{name}`"
            )));
        }
    }
    Ok(())
}

pub(super) fn tool_binding_endpoint(binding: &CliToolBinding) -> Option<&str> {
    match binding {
        CliToolBinding::Http { base_url, .. } | CliToolBinding::Mcp { base_url, .. } => {
            Some(base_url)
        }
        CliToolBinding::Process { .. } => None,
    }
}

pub(super) fn tool_binding_name(binding: &CliToolBinding) -> &str {
    match binding {
        CliToolBinding::Http { name, .. }
        | CliToolBinding::Mcp { name, .. }
        | CliToolBinding::Process { name, .. } => name,
    }
}

pub(super) fn tool_binding_program(binding: &CliToolBinding) -> Option<&str> {
    match binding {
        CliToolBinding::Process { program, .. } => Some(program),
        CliToolBinding::Http { .. } | CliToolBinding::Mcp { .. } => None,
    }
}

fn parse_named_values(value: &str, env: &'static str) -> Result<Vec<(String, String)>, CliError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (name, value) = entry.split_once('=').ok_or_else(|| {
                CliError::InvalidUsage(format!("`{env}` entries must use `name=value` syntax"))
            })?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                return Err(CliError::InvalidUsage(format!(
                    "`{env}` entries must include non-empty name and value"
                )));
            }
            Ok((name.to_owned(), value.to_owned()))
        })
        .collect()
}

pub(super) fn unknown_tool_error(request: &ToolRequest) -> HostError {
    HostError::new(
        HostErrorCode::ToolUnavailable,
        "tool host adapter is not configured",
    )
    .with_detail("tool", &request.tool.name)
    .with_detail("request_id", request.id.0.to_string())
}
