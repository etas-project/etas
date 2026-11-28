use super::{StdLinkage, compile_loaded_run, disk_cache_access};
use crate::{
    DriverError, DriverOptions, LoadProjectRequest, SourceDependencyMode,
    runtime_source::RuntimeSourceMaterializer,
    source::{DiscoveredProject, discover_project},
    std_library::{install_receipt, provider::StdProvider},
};
use etas_frontend::{CheckedProgram, ExternalPackageId, RunEntryCache, SnapshotDetailLevel};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct RunnableEntry {
    pub checked: Option<CheckedProgram>,
    pub sources: Vec<(PathBuf, etas_core::SourceFile)>,
    pub diagnostics: Vec<etas_core::Diagnostic>,
    pub definitions: Option<Arc<etas_frontend::code::definition::NativeDefinitionStore>>,
}

/// The ordinary interpreter entry point. Dynamic compilation and checkpoint
/// consumers request a full compilation with its native source environment.
pub fn load_runnable_entry(
    options: &DriverOptions,
    mut request: LoadProjectRequest,
) -> Result<RunnableEntry, DriverError> {
    request.source_dependency_mode = SourceDependencyMode::MetadataOnly;
    let project = discover_project(options, &request)?;
    let cache = match &options.std_provider {
        StdProvider::Artifact(provider) if provider.sdk.prepared_queries.is_some() => entry_cache(
            options,
            &project,
            &provider.sdk.root,
            &provider.sdk.manifest,
        ),
        _ => None,
    };
    if let Some(hit) = restore(options, &project, cache.as_ref()) {
        return Ok(hit);
    }
    compile_entry(options, project, cache)
}

/// Authenticate the SDK identity and check the source snapshot before opening
/// the installed query tables or building a frontend package environment.
pub fn load_installed_runnable_entry(
    mut options: DriverOptions,
    mut request: LoadProjectRequest,
    sysroot: Option<&Path>,
) -> Result<RunnableEntry, DriverError> {
    request.source_dependency_mode = SourceDependencyMode::MetadataOnly;
    let sysroot = StdProvider::sysroot(sysroot)?;
    let root = sysroot.join("std");
    let manifest = crate::std_library::sdk::compatible_manifest(&root)?;
    let project = discover_project(&options, &request)?;
    let cache = if manifest.prepared_queries_hash.is_some()
        && install_receipt::installed_artifact_is_trusted(&manifest.identity()?)
    {
        entry_cache(&options, &project, &root, &manifest)
    } else {
        None
    };
    if let Some(hit) = restore(&options, &project, cache.as_ref()) {
        return Ok(hit);
    }
    options.std_provider = StdProvider::artifact_for_run(Some(&sysroot), &options.profile)?;
    let cache = cache.filter(|_| matches!(&options.std_provider,
        StdProvider::Artifact(provider) if provider.sdk.manifest == manifest && provider.sdk.prepared_queries.is_some()));
    compile_entry(&options, project, cache)
}

fn restore(
    options: &DriverOptions,
    project: &DiscoveredProject,
    cache: Option<&RunEntryCache>,
) -> Option<RunnableEntry> {
    let entry = cache?.load(&options.profile)?;
    if !dependencies_match(project, &entry.dependencies) {
        return None;
    }
    options.profile.counter("run.entry_cache_hit", 1);
    let sources = entry
        .program
        .sources
        .sources
        .iter()
        .filter_map(|s| Some((s.path.clone()?, s.clone())))
        .collect();
    Some(RunnableEntry {
        checked: Some(entry.program),
        diagnostics: entry.diagnostics,
        sources,
        definitions: None,
    })
}

fn compile_entry(
    options: &DriverOptions,
    project: DiscoveredProject,
    cache: Option<RunEntryCache>,
) -> Result<RunnableEntry, DriverError> {
    options.profile.counter("run.entry_cache_hit", 0);
    let loaded = project.load(options)?;
    let result = compile_loaded_run(
        options,
        loaded,
        SnapshotDetailLevel::None,
        StdLinkage::Deferred,
    )?;
    let output = result.response.output;
    let mut checked = output.checked;
    let definitions = checked
        .as_ref()
        .filter(|p| p.requires_definition_catalog())
        .map(|p| crate::compiler::prepare_native_definitions(&result.loaded, p, Default::default()))
        .transpose()?;
    if let Some(cache) = cache
        && let StdProvider::Artifact(provider) = &options.std_provider
        && let Some(program) = &mut checked
    {
        cache.stage(
            program,
            &provider.interface,
            &output.diagnostics,
            &options.profile,
        );
    }
    Ok(RunnableEntry {
        checked,
        diagnostics: output.diagnostics,
        sources: result.loaded.loaded.sources,
        definitions,
    })
}

fn dependencies_match(project: &DiscoveredProject, packages: &BTreeSet<ExternalPackageId>) -> bool {
    if packages.is_empty() {
        return true;
    }
    let (Some(root), Some(env)) = (
        &project.package.package_root,
        &project.package.package_environment,
    ) else {
        return false;
    };
    let mut materializer = RuntimeSourceMaterializer::new(root, env);
    packages.iter().all(|package| {
        materializer
            .locked_snapshot(*package, &|| Ok::<(), std::convert::Infallible>(()))
            .is_ok()
    })
}

fn entry_cache(
    options: &DriverOptions,
    project: &DiscoveredProject,
    root: &Path,
    manifest: &crate::std_library::sdk::StdSdkManifest,
) -> Option<RunEntryCache> {
    let access = disk_cache_access(options);
    let (directory, build) = install_receipt::run_cache_directory(access)?;
    // Package environment conversion is deterministic for this toolchain. Its
    // locked input includes resolver locations, metadata and source digests.
    let inputs = serde_json::to_vec(&(
        &project.package.package_environment,
        project.package.mode as u8,
    ))
    .ok()?;
    RunEntryCache::new(
        directory,
        build,
        &manifest.identity().ok()?,
        &root.canonicalize().ok()?,
        &project.loaded.input,
        &inputs,
        access,
    )
}
