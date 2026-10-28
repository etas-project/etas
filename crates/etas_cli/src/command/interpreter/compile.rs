use std::{path::PathBuf, sync::Arc};

use etas_core::{Diagnostic, SourceFile};
use etas_frontend::{CheckedProject, EntryPolicy, SnapshotDetailLevel};
use etas_utils::ProfileHandle;

use crate::{args::global::GlobalOptions, error::CliError};

pub struct CompiledRunInput {
    pub sources: Vec<(PathBuf, SourceFile)>,
    pub checked: Option<CheckedProject>,
    pub diagnostics: Vec<Diagnostic>,
    pub definitions: Option<Arc<etas_frontend::code::definition::NativeDefinitionStore>>,
    pub compiler: Option<etas_driver::compiler::NativeCompilerService>,
}

pub fn compile_project(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    paths: Vec<PathBuf>,
    all: bool,
    flow: Option<String>,
    compiler_config: Option<etas_driver::compiler::NativeCompilerConfig>,
) -> Result<CompiledRunInput, CliError> {
    let options = super::super::frontend::driver_options(global, profile);
    let response = etas_driver::compile_runnable_project_with_loaded(
        &options,
        etas_driver::LoadProjectRequest {
            paths,
            all,
            entry_policy: EntryPolicy::Runnable,
            flow,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        SnapshotDetailLevel::None,
    )
    .map_err(super::super::frontend::driver_error)?;
    let output = response.response.output.into_check_output();
    let definitions = output
        .checked
        .as_ref()
        .filter(|checked| compiler_config.is_some() || !checked.types.reflections.is_empty())
        .map(|checked| {
            etas_driver::compiler::prepare_native_definitions(
                &response.loaded,
                checked,
                Default::default(),
            )
        })
        .transpose()
        .map_err(super::super::frontend::driver_error)?;
    let compiler = match (compiler_config, definitions.as_ref()) {
        (Some(config), Some(definitions)) => Some(
            etas_driver::compiler::NativeCompilerService::new(definitions.clone(), config)
                .with_project_sources(&response.loaded)
                .map_err(super::super::frontend::driver_error)?,
        ),
        (Some(_), None) if output.checked.is_some() => {
            return Err(CliError::RuntimeState(
                "native compiler has no definitions for the checked project".into(),
            ));
        }
        _ => None,
    };
    Ok(CompiledRunInput {
        sources: response.loaded.loaded.sources,
        checked: output.checked,
        diagnostics: output.diagnostics,
        definitions,
        compiler,
    })
}
