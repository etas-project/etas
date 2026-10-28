use std::sync::Arc;

use etas_driver::compiler::NativeCompilerService;
use etas_interpreter::host::{CompilerHost, HostServices};

use crate::error::CliError;

use super::host::CliHost;

pub(super) fn compose_host(
    host: CliHost,
    compiler: Option<NativeCompilerService>,
    enabled: bool,
) -> Result<Box<dyn HostServices>, CliError> {
    if !enabled {
        return Ok(Box::new(host));
    }
    let compiler = compiler.ok_or_else(|| {
        CliError::RuntimeState(
            "native compiler requires definitions prepared from the current checked inputs".into(),
        )
    })?;
    let resolver = Arc::new(compiler.clone());
    Ok(Box::new(
        CompilerHost::new(host, compiler).with_candidate_resolver(resolver),
    ))
}
