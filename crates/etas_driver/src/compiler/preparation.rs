use std::sync::Arc;

use etas_code::descriptor::DescriptorLimits;
use etas_frontend::{
    CheckedProgram,
    code::{
        definition::{DefinitionCatalog, NativeDefinitionStore},
        identity::DefinitionOrigins,
        service::NativeCodeSyntaxOperations,
    },
};

use crate::{DriverError, LoadedDriverProject};

/// Prepare inert native definitions from the exact checked input. This does
/// not configure a compiler Host service, grant Code authority, or run agents.
pub fn prepare_native_definitions(
    loaded: &LoadedDriverProject,
    checked: &CheckedProgram,
    limits: DescriptorLimits,
) -> Result<Arc<NativeDefinitionStore>, DriverError> {
    loaded
        .source_definition_provenance
        .validate_external_definition_origins(checked)?;
    let origins = DefinitionOrigins {
        source_modules: super::source_definition_origins(loaded, checked)?,
        standard_package: etas_frontend::code::identity::standard_package_key(
            &checked.std_registry,
        )
        .map_err(|error| DriverError::InvalidInput(error.to_string()))?,
        external_definitions: DefinitionOrigins::checked_external_definitions(checked)
            .map_err(|error| DriverError::InvalidInput(error.to_string()))?,
    };
    let catalog = DefinitionCatalog::build(checked.clone(), &origins, limits).map_err(|error| {
        DriverError::InvalidInput(format!("cannot prepare native code definitions: {error}"))
    })?;
    Ok(Arc::new(NativeDefinitionStore::new(
        Arc::new(catalog),
        Arc::new(NativeCodeSyntaxOperations::default()),
        Default::default(),
    )))
}
