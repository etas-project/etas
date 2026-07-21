use std::io::Write;

use etas_driver::{
    DriverCacheMode, DriverError, DriverOptions, PackageLockRequest, PackageMetadataRequest,
    PackagePackRequest, PackagePrepareRequest, PackageUpdateRequest, build_package_metadata,
    lock_package, pack_package, prepare_package, update_package,
};
use etas_utils::ProfileHandle;

use crate::{
    args::{
        global::{CacheMode, GlobalOptions},
        pkg::{PkgArgs, PkgCommand},
    },
    error::CliError,
    exit::CliExit,
};

pub fn run(
    global: &GlobalOptions,
    args: PkgArgs,
    profile: &ProfileHandle,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let options = driver_options(global, profile);
    match args.command {
        PkgCommand::Lock(args) => {
            let response = lock_package(
                &options,
                PackageLockRequest {
                    package_root: args.package_root,
                },
            )
            .map_err(driver_error)?;
            writeln!(
                stdout,
                "wrote {} with {} locked package{}",
                response.package_root.join("etas.lock").display(),
                response.package_count,
                if response.package_count == 1 { "" } else { "s" }
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            Ok(CliExit::Success)
        }
        PkgCommand::Update(args) => {
            let response = update_package(
                &options,
                PackageUpdateRequest {
                    package_root: args.package_root,
                },
            )
            .map_err(driver_error)?;
            writeln!(
                stdout,
                "updated {} and {} with {} dependenc{} and {} locked package{}",
                response.package_graph_path.display(),
                response.package_root.join("etas.lock").display(),
                response.dependency_count,
                if response.dependency_count == 1 {
                    "y"
                } else {
                    "ies"
                },
                response.locked_package_count,
                if response.locked_package_count == 1 {
                    ""
                } else {
                    "s"
                }
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            Ok(CliExit::Success)
        }
        PkgCommand::Prepare(args) => {
            let response = prepare_package(
                &options,
                PackagePrepareRequest {
                    package_root: args.package_root,
                },
            )
            .map_err(driver_error)?;
            writeln!(
                stdout,
                "materialized {} into {} with {} dependenc{}",
                response.package_root.display(),
                response.package_graph_path.display(),
                response.dependency_count,
                if response.dependency_count == 1 {
                    "y"
                } else {
                    "ies"
                }
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            Ok(CliExit::Success)
        }
        PkgCommand::Metadata(args) => {
            let response = build_package_metadata(
                &options,
                PackageMetadataRequest {
                    package_root: args.package_root,
                    bin: args.bin,
                },
            )
            .map_err(driver_error)?;
            writeln!(
                stdout,
                "wrote {} ({}) for {}",
                response.artifact_path.display(),
                response.artifact_hash,
                response.package_root.display()
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            Ok(CliExit::Success)
        }
        PkgCommand::Pack(args) => {
            let response = pack_package(PackagePackRequest {
                package_root: args.package_root,
                output: args.out,
            })
            .map_err(driver_error)?;
            writeln!(
                stdout,
                "packed {} into {} ({})",
                response.package_root.display(),
                response.output.display(),
                response.checksum
            )
            .map_err(|source| CliError::Io {
                path: "<stdout>".into(),
                source,
            })?;
            Ok(CliExit::Success)
        }
    }
}

fn driver_options(global: &GlobalOptions, profile: &ProfileHandle) -> DriverOptions {
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

fn driver_error(error: DriverError) -> CliError {
    match error {
        DriverError::Io { path, source } => CliError::Io { path, source },
        DriverError::Package(error) => CliError::InvalidUsage(error.to_string()),
        DriverError::FrontendSession(message) => CliError::FrontendSession(message),
        DriverError::InvalidInput(message) => CliError::InvalidUsage(message),
    }
}
