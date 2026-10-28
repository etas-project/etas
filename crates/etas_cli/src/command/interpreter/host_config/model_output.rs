use super::{
    CliError, CliModelAdapter, CliModelConfig, ModelProviderCapabilities,
    OpenAiTypedOutputEncoding, RuntimeEnvOverrides, RuntimeModelProfile,
};

pub(super) const ENV_KEY: &str = "ETAS_HOST_MODEL_OUTPUT_ENCODING";

pub(super) fn select(
    env: &RuntimeEnvOverrides,
    profile: Option<&RuntimeModelProfile>,
    current: Option<&CliModelConfig>,
) -> Result<Option<OpenAiTypedOutputEncoding>, CliError> {
    env.get(ENV_KEY)
        .or_else(|| profile.and_then(|profile| profile.typed_output_encoding.as_deref()))
        .map(parse)
        .transpose()
        .map(|encoding| encoding.or_else(|| current.and_then(|model| model.typed_output_encoding)))
}

pub(super) fn parse(value: &str) -> Result<OpenAiTypedOutputEncoding, CliError> {
    value
        .parse()
        .map_err(|error: etas_host::HostError| CliError::InvalidUsage(error.message))
}

pub(super) fn configure(
    adapter: CliModelAdapter,
    encoding: Option<OpenAiTypedOutputEncoding>,
) -> Result<
    (
        CliModelAdapter,
        ModelProviderCapabilities,
        Option<OpenAiTypedOutputEncoding>,
    ),
    CliError,
> {
    match adapter {
        CliModelAdapter::OpenAi(mut adapter) => {
            if let Some(encoding) = encoding {
                adapter = adapter
                    .with_typed_output_encoding(encoding)
                    .map_err(|error| CliError::InvalidUsage(error.message))?;
            }
            let effective = adapter
                .effective_typed_output_encoding()
                .map_err(|error| CliError::InvalidUsage(error.message))?;
            let capabilities = adapter
                .provider_capabilities()
                .map_err(|error| CliError::InvalidUsage(error.message))?;
            Ok((
                CliModelAdapter::OpenAi(adapter),
                capabilities,
                Some(effective),
            ))
        }
        CliModelAdapter::Anthropic(adapter) => {
            if encoding.is_some() {
                return Err(CliError::InvalidUsage(
                    "typed_output_encoding is only supported by OpenAI-compatible model adapters"
                        .into(),
                ));
            }
            Ok((
                CliModelAdapter::Anthropic(adapter),
                super::AnthropicProtocolAdapter::capabilities(),
                None,
            ))
        }
    }
}
