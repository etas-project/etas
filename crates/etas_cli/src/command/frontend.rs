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
    etas_driver::load_project(
        &driver_options(global, profile),
        LoadProjectRequest {
            paths,
            all,
            entry_policy,
            flow,
            bin: None,
            source_dependency_mode,
        },
    )
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
    }
}

pub(crate) fn driver_error(error: DriverError) -> CliError {
    match error {
        DriverError::Io { path, source } => CliError::Io { path, source },
        DriverError::Package(error) => CliError::InvalidUsage(error.to_string()),
        DriverError::FrontendSession(message) => CliError::FrontendSession(message),
        DriverError::InvalidInput(message) => CliError::InvalidUsage(message),
    }
}
