use std::path::PathBuf;

use etas_core::{Diagnostic, SourceFile};
use etas_frontend::{CheckedProject, EntryPolicy, SnapshotDetailLevel};
use etas_utils::ProfileHandle;

use crate::{args::global::GlobalOptions, error::CliError};

pub struct CompiledRunInput {
    pub sources: Vec<(PathBuf, SourceFile)>,
    pub checked: Option<CheckedProject>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile_project(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    paths: Vec<PathBuf>,
    all: bool,
    flow: Option<String>,
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
    Ok(CompiledRunInput {
        sources: response.loaded.loaded.sources,
        checked: output.checked,
        diagnostics: output.diagnostics,
    })
}
