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
    optimization_level: u8,
    compiler_config: Option<etas_driver::compiler::NativeCompilerConfig>,
    defer_unused_std: bool,
) -> Result<CompiledRunInput, CliError> {
    let ordinary = defer_unused_std && compiler_config.is_none();
    let installed = ordinary
        && matches!(
            global.std_provider,
            crate::args::global::StdProviderChoice::Artifact
        );
    let options = if installed {
        super::super::frontend::base_driver_options(global, profile)
    } else {
        super::super::frontend::driver_options(global, profile)?
    }
    .with_optimization_level(optimization_level)
    .map_err(super::super::frontend::driver_error)?;
    let request = etas_driver::LoadProjectRequest {
        paths,
        all,
        entry_policy: EntryPolicy::Runnable,
        flow,
        bin: None,
        source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
    };
    if ordinary {
        let entry = if installed {
            etas_driver::load_installed_runnable_entry(options, request, global.sysroot.as_deref())
        } else {
            etas_driver::load_runnable_entry(&options, request)
        }
        .map_err(super::super::frontend::driver_error)?;
        return Ok(CompiledRunInput {
            sources: entry.sources,
            checked: entry.checked,
            diagnostics: entry.diagnostics,
            definitions: entry.definitions,
            compiler: None,
        });
    }
    let response = etas_driver::compile_runnable_project_with_loaded(
        &options,
        request,
        SnapshotDetailLevel::None,
    )
    .map_err(super::super::frontend::driver_error)?;
    let output = response.response.output.into_check_output();
    let definitions = output
        .checked
        .as_ref()
        .filter(|checked| compiler_config.is_some() || checked.requires_definition_catalog())
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
