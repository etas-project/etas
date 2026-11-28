use std::path::PathBuf;

use etas_core::SourceFile;
use etas_driver::{
    CompileProjectRequest, DriverCacheMode, DriverError, DriverOptions, LoadProjectRequest,
};
use etas_frontend::{ProjectOutput, SnapshotDetailLevel};
use etas_utils::ProfileHandle;

use crate::{
    args::global::{CacheMode, GlobalOptions},
    error::CliError,
};

pub fn check_paths(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    request: LoadProjectRequest,
    optimization_level: Option<u8>,
) -> Result<(Vec<(PathBuf, SourceFile)>, ProjectOutput), CliError> {
    let options = driver_options_with_optimization(global, profile, optimization_level)?;
    let loaded = etas_driver::load_project(&options, request)
        .map_err(driver_error)?
        .loaded;
    let output = etas_driver::check_project_interface_once(
        &options,
        CompileProjectRequest {
            input: loaded.input,
            snapshot_detail: SnapshotDetailLevel::None,
        },
    )
    .map_err(driver_error)?
    .response
    .output;
    Ok((loaded.sources, output))
}

pub(crate) fn driver_options(
    global: &GlobalOptions,
    profile: &ProfileHandle,
) -> Result<DriverOptions, CliError> {
    driver_options_for(global, profile, false)
}

pub(crate) fn driver_options_for(
    global: &GlobalOptions,
    profile: &ProfileHandle,
    run_entry: bool,
) -> Result<DriverOptions, CliError> {
    let mut options = base_driver_options(global, profile);
    options.std_provider = match global.std_provider {
        crate::args::global::StdProviderChoice::Legacy => {
            etas_driver::std_library::provider::StdProvider::Legacy
        }
        crate::args::global::StdProviderChoice::Artifact => {
            use etas_driver::std_library::provider::StdProvider;
            let open = if run_entry {
                StdProvider::artifact_for_run
            } else {
                StdProvider::artifact_profiled
            };
            open(global.sysroot.as_deref(), profile).map_err(driver_error)?
        }
    };
    Ok(options)
}

/// Source bootstrap builds its own std provider from the input package.
pub(crate) fn base_driver_options(
    global: &GlobalOptions,
    profile: &ProfileHandle,
) -> DriverOptions {
    DriverOptions {
        std_provider: etas_driver::std_library::provider::StdProvider::Legacy,
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
    let options = driver_options(global, profile)?;
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
