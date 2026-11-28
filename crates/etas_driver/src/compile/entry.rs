use super::{StdLinkage, compile_loaded_run, disk_cache_access};
use crate::{
    DriverError, DriverOptions, LoadProjectRequest, LoadedDriverProject, SourceDependencyMode,
    load_project,
    runtime_source::RuntimeSourceMaterializer,
    std_library::{install_receipt, provider::StdProvider},
};
use etas_frontend::{CheckedProgram, ExternalPackageId, RunEntryCache, SnapshotDetailLevel};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

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
    let loaded = load_project(options, request)?;
    let cache = entry_cache(options, &loaded);
    if let Some((cache, provider)) = &cache
        && let Some(entry) = cache.load(
            provider.registry.clone(),
            provider.interface.clone(),
            &options.profile,
        )
        && dependencies_match(&loaded, &entry.dependencies)
    {
        options.profile.counter("run.entry_cache_hit", 1);
        let sources = entry
            .program
            .sources
            .sources
            .iter()
            .filter_map(|s| Some((s.path.clone()?, s.clone())))
            .collect();
        return Ok(RunnableEntry {
            checked: Some(entry.program),
            diagnostics: entry.diagnostics,
            sources,
            definitions: None,
        });
    }
    options.profile.counter("run.entry_cache_hit", 0);
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
    if let Some((cache, _)) = cache
        && let Some(program) = &mut checked
    {
        cache.stage(program, &output.diagnostics, &options.profile);
    }
    Ok(RunnableEntry {
        checked,
        diagnostics: output.diagnostics,
        sources: result.loaded.loaded.sources,
        definitions,
    })
}

fn dependencies_match(
    loaded: &LoadedDriverProject,
    packages: &BTreeSet<ExternalPackageId>,
) -> bool {
    if packages.is_empty() {
        return true;
    }
    let (Some(root), Some(env)) = (&loaded.package_root, &loaded.package_environment) else {
        return false;
    };
    let mut materializer = RuntimeSourceMaterializer::new(root, env);
    packages.iter().all(|package| {
        materializer
            .locked_snapshot(*package, &|| Ok::<(), std::convert::Infallible>(()))
            .is_ok()
    })
}

fn entry_cache<'a>(
    options: &'a DriverOptions,
    loaded: &LoadedDriverProject,
) -> Option<(
    RunEntryCache,
    &'a crate::std_library::provider::ArtifactStdProvider,
)> {
    let StdProvider::Artifact(provider) = &options.std_provider else {
        return None;
    };
    provider.sdk.prepared_queries.as_ref()?;
    let access = disk_cache_access(options);
    let (directory, build) = install_receipt::run_cache_directory(access)?;
    // Resolver locations are part of the identity even when package metadata is identical.
    let lock = serde_json::to_vec(&loaded.package_environment).ok()?;
    let cache = RunEntryCache::new(
        directory,
        build,
        &provider.sdk.manifest.identity().ok()?,
        &provider.sdk.root.canonicalize().ok()?,
        &loaded.loaded.input,
        &lock,
        access,
    )?;
    Some((cache, provider))
}
