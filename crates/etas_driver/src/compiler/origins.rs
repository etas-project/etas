use std::collections::HashMap;

use etas_frontend::{CheckedProgramData, SourceKind, code::identity::SourceModuleOrigin};
use etas_hir::HirModuleId;

use crate::{DriverError, LoadedDriverProject};

mod provenance;
pub(crate) use provenance::{SourceDefinitionProvenance, capture_source_definition_provenance};

/// Bind native source modules to their prepared package origins. Portable
/// metadata definitions and the standard package have separate owning inputs.
pub fn source_definition_origins(
    loaded: &LoadedDriverProject,
    checked: &CheckedProgramData,
) -> Result<HashMap<HirModuleId, SourceModuleOrigin>, DriverError> {
    let provenance = &loaded.source_definition_provenance;
    let sources = loaded
        .loaded
        .input
        .sources
        .iter()
        .map(|source| (source.id, source))
        .collect::<HashMap<_, _>>();
    if sources.len() != loaded.loaded.input.sources.len() {
        return Err(invalid("loaded source identity is duplicated"));
    }
    let mut modules = HashMap::new();
    for module in &checked.hir.modules {
        let hir_module = checked
            .modules_arena
            .get(*module)
            .ok_or_else(|| invalid("checked source module is missing"))?;
        let index = checked
            .module_index
            .by_source
            .get(&hir_module.span.source)
            .and_then(|id| checked.module_index.modules.get(*id))
            .ok_or_else(|| invalid("checked source module has no project origin"))?;
        let mut owner = None;
        for part in &index.parts {
            let part = checked
                .module_index
                .parts
                .get(*part)
                .ok_or_else(|| invalid("checked module part is missing"))?;
            let input = sources.get(&part.source).ok_or_else(|| {
                invalid("checked source was not materialized by this driver input")
            })?;
            let archived = checked
                .definition_sources
                .source(part.source)
                .ok_or_else(|| invalid("checked source has no immutable definition archive"))?;
            if archived.kind != input.kind
                || archived.file.path != input.path
                || archived.file.text() != input.text
            {
                return Err(invalid(
                    "checked source differs from the prepared driver input",
                ));
            }
            let key = match &input.kind {
                SourceKind::SourceProjectFile | SourceKind::SingleFileInput => {
                    provenance.local_package_for(input)?
                }
                SourceKind::DependencySourceOverlay {
                    package,
                    import_root,
                } => provenance.dependency_package_for(*package, import_root)?,
                _ => {
                    return Err(invalid(
                        "virtual source requires an explicit definition origin",
                    ));
                }
            };
            if owner.is_some_and(|previous| previous != key) {
                return Err(invalid(
                    "source module parts belong to different package origins",
                ));
            }
            owner = Some(key);
        }
        let owner = owner.ok_or_else(|| invalid("checked source module has no source parts"))?;
        let origin = SourceModuleOrigin::declared(checked, *module, owner)
            .map_err(|error| invalid(&error.to_string()))?;
        modules.insert(*module, origin);
    }
    Ok(modules)
}

fn invalid(reason: &str) -> DriverError {
    DriverError::InvalidInput(format!("invalid source definition origin: {reason}"))
}

#[cfg(test)]
mod tests;
