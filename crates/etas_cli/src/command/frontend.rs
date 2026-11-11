use std::path::PathBuf;

use etas_driver::{
    CompileProjectRequest, DriverCacheMode, DriverError, DriverOptions, LoadProjectRequest,
    SourceDependencyMode,
};
use etas_frontend::{
    EntryPolicy, LoadedProjectInput, ProjectInput, ProjectOutput, SnapshotDetailLevel,
};
use etas_utils::ProfileHandle;

use crate::{
    args::global::{CacheMode, GlobalOptions},
    error::CliError,
};

pub fn check_project_once(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    input: ProjectInput,
) -> Result<ProjectOutput, CliError> {
    etas_driver::check_project_once(
        &driver_options(global, profile),
        CompileProjectRequest {
            input,
            snapshot_detail: SnapshotDetailLevel::None,
        },
    )
    .map(|response| response.response.output)
    .map_err(driver_error)
}

pub fn load_project(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    paths: Vec<PathBuf>,
    all: bool,
    entry_policy: EntryPolicy,
    flow: Option<String>,
    source_dependency_mode: SourceDependencyMode,
) -> Result<LoadedProjectInput, CliError> {
    load_project_with_optimization(
        global,
        profile,
        LoadProjectRequest {
            paths,
            all,
            entry_policy,
            flow,
            bin: None,
            source_dependency_mode,
        },
        None,
    )
}

pub fn load_project_with_optimization(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    request: LoadProjectRequest,
    optimization_level: Option<u8>,
) -> Result<LoadedProjectInput, CliError> {
    let options = driver_options_with_optimization(global, profile, optimization_level)?;
    etas_driver::load_project(&options, request)
        .map(|loaded| loaded.loaded)
        .map_err(driver_error)
}

pub(crate) fn driver_options(global: &GlobalOptions, profile: &ProfileHandle) -> DriverOptions {
    DriverOptions {
        workspace: global.workspace.clone(),
        cache_mode: match global.cache {
            CacheMode::Auto => DriverCacheMode::Auto,
            CacheMode::Off => DriverCacheMode::Off,
            CacheMode::ReadOnly => DriverCacheMode::ReadOnly,
            CacheMode::WriteOnly => DriverCacheMode::WriteOnly,
            CacheMode::ReadWrite => DriverCacheMode::ReadWrite,
        },
        cache_root: global.cache_root.clone(),
        profile: profile.clone(),
        #[cfg(feature = "component-runtime")]
        residual_target: Some(etas_interpreter::interpreter_residual_capabilities()),
        #[cfg(not(feature = "component-runtime"))]
        residual_target: None,
        optimization_level: None,
    }
}

pub(crate) fn driver_options_with_optimization(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    optimization_level: Option<u8>,
) -> Result<DriverOptions, CliError> {
    let options = driver_options(global, profile);
    optimization_level.map_or(Ok(options.clone()), |level| {
        options.with_optimization_level(level).map_err(driver_error)
    })
}

pub(crate) fn driver_error(error: DriverError) -> CliError {
    match error {
        DriverError::Io { path, source } => CliError::Io { path, source },
        DriverError::Package(error) => CliError::InvalidUsage(error.to_string()),
        DriverError::SourceSnapshot(error) => CliError::InvalidUsage(error.to_string()),
        DriverError::FrontendSession(message) => CliError::FrontendSession(message),
        DriverError::InvalidInput(message) => CliError::InvalidUsage(message),
    }
}
