use std::path::PathBuf;

use etas_frontend::{CheckResponse, EntryPolicy, SnapshotDetailLevel, StandardLibraryMode};

use crate::{
    DriverError, DriverOptions, LoadProjectRequest, SourceDependencyMode, load_project,
    source::compile_loaded_project,
};

#[derive(Clone, Debug)]
pub struct StdLibraryCheckRequest {
    pub manifest_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct StdLibraryCheckOutput {
    pub package_root: PathBuf,
    pub manifest: etas_package::Manifest,
    pub definition_origin: etas_code::PackageKey,
    pub response: CheckResponse,
}

/// Load every source selected by the std manifest and run the normal frontend
/// pipeline. Diagnostics retain their source locations, including private bodies.
/// This is the source-bootstrap phase; bundle emission consumes its checked output.
pub fn check_standard_library_source(
    options: &DriverOptions,
    request: StdLibraryCheckRequest,
) -> Result<StdLibraryCheckOutput, DriverError> {
    let manifest_path = request
        .manifest_path
        .canonicalize()
        .map_err(|source| DriverError::Io {
            path: request.manifest_path,
            source,
        })?;
    if manifest_path.file_name().and_then(|name| name.to_str()) != Some("etas.toml") {
        return Err(DriverError::InvalidInput(format!(
            "std source build requires an etas.toml manifest, got `{}`",
            manifest_path.display()
        )));
    }
    let package_root = manifest_path
        .parent()
        .expect("manifest file has a parent")
        .to_owned();
    let manifest = etas_package::read_manifest(&package_root)?;
    if manifest.package.name != "std" {
        return Err(DriverError::InvalidInput(format!(
            "std source build requires package `std`, got `{}` in `{}`",
            manifest.package.name,
            manifest_path.display()
        )));
    }
    let mut loaded = load_project(
        options,
        LoadProjectRequest {
            paths: vec![package_root.clone()],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )?;
    let definition_origin = loaded
        .source_definition_provenance
        .local_package_key()
        .expect("loaded std package has a definition owner");
    loaded.loaded.input.options.standard_library_mode = StandardLibraryMode::SourceBootstrap;
    let compiled = compile_loaded_project(options, loaded.loaded, SnapshotDetailLevel::None)?;
    Ok(StdLibraryCheckOutput {
        package_root,
        manifest,
        definition_origin,
        response: compiled.response,
    })
}

#[cfg(test)]
mod tests;
