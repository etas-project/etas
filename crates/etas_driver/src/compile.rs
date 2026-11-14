use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use etas_core::Severity;
use etas_frontend::{
    CheckRequest, CheckResponse, CheckScope, DependencyOverlayChange, DiskCacheAccess,
    EnvironmentChange, ExternalPackageId, FrontendSession, FrontendSessionOptions,
    FrontendSessionStore, ModulePath, OptimizationLevel, ProjectChangeSet, ProjectInput,
    ProjectRevision, RuntimeSourceRequirement, RuntimeSourceRequirements, SnapshotDetailLevel,
    SourceInput, SourceKind,
};
use etas_utils::ProfileHandle;

use crate::{
    DriverError, LoadProjectRequest, LoadedDriverProject, SourceDependencyMode, load_project,
    runtime_source::{RuntimeSourceMaterializer, RuntimeSourcePlan},
    source::append_runtime_source_dependency_sources,
};

#[derive(Clone, Debug)]
pub struct DriverOptions {
    pub std_provider: crate::std_library::provider::StdProvider,
    pub workspace: Option<PathBuf>,
    pub cache_mode: DriverCacheMode,
    pub cache_root: Option<PathBuf>,
    pub profile: ProfileHandle,
    pub residual_target: Option<etas_code::residual::target::ResidualTargetCapabilities>,
    pub optimization_level: Option<OptimizationLevel>,
}

impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            std_provider: Default::default(),
            workspace: None,
            cache_mode: DriverCacheMode::Auto,
            cache_root: None,
            profile: ProfileHandle::disabled(),
            residual_target: None,
            optimization_level: None,
        }
    }
}

impl DriverOptions {
    pub fn with_optimization_level(mut self, level: u8) -> Result<Self, DriverError> {
        self.optimization_level = Some(
            OptimizationLevel::try_from(level)
                .map_err(|error| DriverError::InvalidInput(error.to_string()))?,
        );
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverCacheMode {
    Auto,
    Off,
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Clone, Debug)]
pub struct CompileProjectRequest {
    pub input: ProjectInput,
    pub snapshot_detail: SnapshotDetailLevel,
}

#[derive(Clone, Debug)]
pub struct CompileProjectResponse {
    pub response: CheckResponse,
}

#[derive(Clone, Debug)]
pub struct CompileRunnableProjectResponse {
    pub loaded: LoadedDriverProject,
    pub response: CheckResponse,
}

pub fn check_project_once(
    options: &DriverOptions,
    mut request: CompileProjectRequest,
) -> Result<CompileProjectResponse, DriverError> {
    options.std_provider.configure(&mut request.input);
    if let Some(level) = options.optimization_level {
        request.input.options.optimization_level = level;
    }
    if let Some(target) = &options.residual_target {
        request.input.options.residual_target = target.clone();
    }
    let _span = options.profile.span("frontend.check", "frontend");
    let mut session = if request.input.options.standard_library_mode
        == etas_frontend::StandardLibraryMode::SourceBootstrap
        && let crate::std_library::provider::StdProvider::Artifact(provider) = &options.std_provider
    {
        // Source builds consume the verified primitive ABI, independently of
        // which public exports the previous SDK has already migrated.
        FrontendSession::with_options_and_std_registry(
            session_options(options)?,
            provider.library.program.std_registry.clone(),
        )
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?
    } else {
        frontend_session(options)?
    };
    let project = session
        .try_open_project(request.input)
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
    let mut check = check_request(options).with_snapshot_detail(request.snapshot_detail);
    if !check.disk_artifact_access.can_write() {
        check.retention = etas_frontend::CheckRetention::OneShot;
    }
    let response = session
        .check(project, check)
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
    record_pipeline_profile(options, &response);
    Ok(CompileProjectResponse { response })
}

pub fn compile_runnable_project(
    options: &DriverOptions,
    request: LoadProjectRequest,
    snapshot_detail: SnapshotDetailLevel,
) -> Result<CompileProjectResponse, DriverError> {
    Ok(CompileProjectResponse {
        response: compile_runnable_project_with_loaded(options, request, snapshot_detail)?.response,
    })
}

pub fn compile_runnable_project_with_loaded(
    options: &DriverOptions,
    mut request: LoadProjectRequest,
    snapshot_detail: SnapshotDetailLevel,
) -> Result<CompileRunnableProjectResponse, DriverError> {
    request.source_dependency_mode = SourceDependencyMode::MetadataOnly;
    let metadata_loaded = load_project(options, request)?;
    let mut session = frontend_session(options)?;
    let project = session
        .try_open_project(metadata_loaded.loaded.input.clone())
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
    let mut response = {
        let mut attrs = BTreeMap::new();
        attrs.insert("phase".to_owned(), "metadata".to_owned());
        let _span = options
            .profile
            .span_with_attrs("frontend.check", "frontend", attrs);
        let response = session
            .check(
                project,
                check_request(options)
                    .with_snapshot_detail(snapshot_detail)
                    .with_scope(CheckScope::EntryReachable),
            )
            .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
        record_pipeline_profile(options, &response);
        response
    };
    if has_blocking_diagnostics(&response.output.diagnostics) {
        return Ok(CompileRunnableProjectResponse {
            loaded: metadata_loaded,
            response,
        });
    }
    let Some(package_root) = metadata_loaded.package_root.as_ref() else {
        return Ok(CompileRunnableProjectResponse {
            loaded: metadata_loaded,
            response,
        });
    };
    let Some(package_environment) = metadata_loaded.package_environment.as_ref() else {
        return Ok(CompileRunnableProjectResponse {
            loaded: metadata_loaded,
            response,
        });
    };
    let mut runtime_loaded = metadata_loaded.clone();
    let mut materialized = RuntimeSourceMaterializedSet::default();
    let mut source_materializer = RuntimeSourceMaterializer::new(package_root, package_environment);
    let mut revision = 1;
    let mut iterations = 0_u64;
    loop {
        let requirements = response
            .output
            .runtime_source_requirements
            .as_ref()
            .ok_or_else(|| {
                DriverError::FrontendSession(
                    "runnable compilation has no checked runtime source requirements".into(),
                )
            })?;
        let delta = materialized.delta(requirements);
        if delta.is_empty() {
            options
                .profile
                .counter("runtime_overlay.iterations", iterations);
            return Ok(CompileRunnableProjectResponse {
                loaded: runtime_loaded,
                response,
            });
        }

        let runtime_plan = source_materializer.plan(&delta)?;
        if runtime_plan.is_empty() {
            return Err(DriverError::InvalidInput(
                "runtime source requirements could not be materialized".to_owned(),
            ));
        }
        let added_sources =
            append_runtime_source_dependency_sources(&mut runtime_loaded.loaded, &runtime_plan)?;
        if added_sources.is_empty() {
            return Err(DriverError::InvalidInput(
                "runtime source overlay made no progress".to_owned(),
            ));
        }
        let mut added_keys = Vec::with_capacity(added_sources.len());
        for source in &added_sources {
            let SourceKind::DependencySourceOverlay {
                package,
                import_root,
            } = &source.kind
            else {
                return Err(DriverError::InvalidInput(
                    "runtime source patch contains a non-dependency source".into(),
                ));
            };
            let path = source.path.as_ref().ok_or_else(|| {
                DriverError::InvalidInput("runtime source has no locked path".into())
            })?;
            let mut dependencies = runtime_plan.dependencies.values().filter(|dependency| {
                dependency.package == *package
                    && dependency.import_root == *import_root
                    && dependency.source_files.contains(path)
            });
            let dependency = dependencies.next().ok_or_else(|| {
                DriverError::InvalidInput("runtime source has no locked dependency plan".into())
            })?;
            if dependencies.next().is_some() {
                return Err(DriverError::InvalidInput(
                    "runtime source has ambiguous locked dependency plans".into(),
                ));
            }
            let owner = runtime_loaded
                .source_definition_provenance
                .dependency_package_for(*package, import_root)?;
            added_keys.push(crate::source::stable_package_source_key(
                source,
                owner,
                &dependency.package_root,
            )?);
        }
        options
            .profile
            .counter("runtime_overlay.added_sources", added_sources.len() as u64);
        options.profile.counter(
            "frontend.total_source_files",
            runtime_loaded.loaded.sources.len() as u64,
        );
        materialized.record_plan(&runtime_plan);
        session
            .apply_changes(
                project,
                ProjectChangeSet {
                    revision: ProjectRevision(revision),
                    source_changes: Vec::new(),
                    dependency_overlay_changes: dependency_overlay_changes_from_sources(
                        added_sources,
                    )?,
                    environment_change: Some(EnvironmentChange::AddOverlaySourceKeys(
                        added_keys.clone(),
                    )),
                    option_changes: Vec::new(),
                },
            )
            .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
        runtime_loaded
            .loaded
            .input
            .environment
            .stable_source_keys
            .extend(added_keys);
        iterations += 1;
        revision += 1;
        response = {
            let mut attrs = BTreeMap::new();
            attrs.insert("phase".to_owned(), "runtime-overlay".to_owned());
            attrs.insert("iteration".to_owned(), iterations.to_string());
            let _span = options
                .profile
                .span_with_attrs("frontend.check", "frontend", attrs);
            let response = session
                .check(
                    project,
                    CheckRequest::incremental()
                        .with_scope(CheckScope::EntryReachable)
                        .with_disk_artifact_access(disk_cache_access(options))
                        .with_snapshot_detail(snapshot_detail)
                        .with_pipeline_timing(options.profile.is_enabled()),
                )
                .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
            record_pipeline_profile(options, &response);
            response
        };
        if has_blocking_diagnostics(&response.output.diagnostics) {
            return Ok(CompileRunnableProjectResponse {
                loaded: runtime_loaded,
                response,
            });
        }
    }
}

#[derive(Default)]
pub(crate) struct RuntimeSourceMaterializedSet {
    modules: BTreeMap<ExternalPackageId, BTreeSet<ModulePath>>,
}

impl RuntimeSourceMaterializedSet {
    pub(crate) fn delta(
        &self,
        requirements: &RuntimeSourceRequirements,
    ) -> RuntimeSourceRequirements {
        let mut delta = RuntimeSourceRequirements::default();
        for (package, requirement) in &requirements.dependencies {
            let materialized = self.modules.get(package);
            let required_modules = requirement
                .required_modules
                .iter()
                .filter(|module| !materialized.is_some_and(|modules| modules.contains(*module)))
                .cloned()
                .collect::<BTreeSet<_>>();
            if required_modules.is_empty() {
                continue;
            }
            let seed_modules = requirement
                .seed_modules
                .iter()
                .filter(|module| required_modules.contains(*module))
                .cloned()
                .collect::<BTreeSet<_>>();
            delta.dependencies.insert(
                *package,
                RuntimeSourceRequirement {
                    package: requirement.package,
                    import_root: requirement.import_root.clone(),
                    seed_modules,
                    required_modules,
                    reasons: requirement.reasons.clone(),
                },
            );
        }
        delta
    }

    pub(crate) fn record_plan(&mut self, plan: &RuntimeSourcePlan) {
        for dependency in plan.dependencies.values() {
            self.modules.entry(dependency.package).or_default().extend(
                dependency
                    .source_modules
                    .iter()
                    .cloned()
                    .map(|segments| ModulePath { segments }),
            );
        }
    }
}

fn dependency_overlay_changes_from_sources(
    sources: Vec<SourceInput>,
) -> Result<Vec<DependencyOverlayChange>, DriverError> {
    let mut grouped =
        BTreeMap::<(etas_frontend::ExternalPackageId, String), Vec<SourceInput>>::new();
    for source in sources {
        let SourceKind::DependencySourceOverlay {
            package,
            import_root,
        } = &source.kind
        else {
            return Err(DriverError::InvalidInput(format!(
                "runtime source materialization produced non-overlay source `{}`",
                source.id.0
            )));
        };
        grouped
            .entry((*package, import_root.clone()))
            .or_default()
            .push(source);
    }
    Ok(grouped
        .into_iter()
        .map(
            |((package, import_root), added_sources)| DependencyOverlayChange {
                package,
                import_root,
                added_sources,
            },
        )
        .collect())
}

pub fn frontend_session(
    options: &DriverOptions,
) -> Result<FrontendSession<FrontendSessionStore>, DriverError> {
    let session = match &options.std_provider {
        crate::std_library::provider::StdProvider::Legacy => {
            FrontendSession::with_options(session_options(options)?)
        }
        crate::std_library::provider::StdProvider::Artifact(provider) => {
            FrontendSession::with_options_and_std_registry(
                session_options(options)?,
                provider.registry.clone(),
            )
            .map(|session| session.with_standard_library(provider.library.clone()))
        }
    };
    session.map_err(|error| DriverError::FrontendSession(format!("{error:?}")))
}

pub fn session_options(options: &DriverOptions) -> Result<FrontendSessionOptions, DriverError> {
    if options.cache_mode == DriverCacheMode::Off
        || (options.cache_mode == DriverCacheMode::Auto && options.cache_root.is_none())
    {
        return Ok(FrontendSessionOptions::memory_only());
    }
    let root = match options.cache_root.clone() {
        Some(root) => root,
        None => crate::project_discovery::resolve_workspace(options)?
            .join(".etas")
            .join("cache"),
    };
    Ok(FrontendSessionOptions::disk_cache(root).with_disk_cache_access(disk_cache_access(options)))
}

pub fn check_request(options: &DriverOptions) -> CheckRequest {
    CheckRequest::full_project()
        .with_disk_artifact_access(disk_cache_access(options))
        .with_snapshot_detail(SnapshotDetailLevel::None)
        .with_pipeline_timing(options.profile.is_enabled())
}

fn disk_cache_access(options: &DriverOptions) -> DiskCacheAccess {
    match options.cache_mode {
        DriverCacheMode::Auto | DriverCacheMode::ReadWrite => DiskCacheAccess::read_write(),
        DriverCacheMode::Off => DiskCacheAccess::disabled(),
        DriverCacheMode::ReadOnly => DiskCacheAccess::read_only(),
        DriverCacheMode::WriteOnly => DiskCacheAccess::write_only(),
    }
}

fn record_pipeline_profile(options: &DriverOptions, response: &CheckResponse) {
    if !options.profile.is_enabled() {
        return;
    }
    options.profile.counter(
        "frontend.pipeline.records",
        response.pipeline_records.len() as u64,
    );
    options.profile.counter(
        "frontend.hir_index_builds",
        response.memory.hir_index_builds,
    );
    options.profile.counter(
        "frontend.retained_revisions",
        u64::from(response.memory.retained_revision.is_some()),
    );
    options.profile.counter(
        "frontend.retained_revision.estimated_bytes",
        response.memory.retained_revision_estimated_bytes,
    );
    if let Some(cache) = &response.memory.cache_after {
        options
            .profile
            .counter("frontend.cache.budget_bytes", cache.budget_bytes);
        options.profile.counter(
            "frontend.cache.estimated_retained_bytes",
            cache.estimated_retained_bytes,
        );
        options.profile.counter(
            "frontend.cache.payload_estimated_bytes",
            cache.payload_estimated_bytes,
        );
        options.profile.counter(
            "frontend.cache.bookkeeping_estimated_bytes",
            cache.bookkeeping_estimated_bytes,
        );
        options
            .profile
            .counter("frontend.cache.evicted_artifacts", cache.evicted_artifacts);
        options.profile.counter(
            "frontend.cache.rejected_insertions",
            cache.rejected_insertions,
        );
        for kind in &cache.by_kind {
            options.profile.counter(
                format!(
                    "frontend.cache.{}.{}.referenced_payload_estimated_bytes",
                    kind.namespace, kind.kind
                ),
                kind.referenced_payload_estimated_bytes,
            );
            options.profile.counter(
                format!(
                    "frontend.cache.{}.{}.entry_bookkeeping_estimated_bytes",
                    kind.namespace, kind.kind
                ),
                kind.entry_bookkeeping_estimated_bytes,
            );
        }
    }
    options.profile.counter(
        "frontend.parsed_source_files",
        response
            .output
            .parsed_sources
            .len()
            .saturating_sub(response.output.reused_parsed_sources) as u64,
    );
    options.profile.counter(
        "frontend.reused_parsed_sources",
        response.output.reused_parsed_sources as u64,
    );
    if let Some(reachability) = response.output.reachability.as_ref() {
        options.profile.counter(
            "frontend.reachable_items",
            reachability.reachable_items.len() as u64,
        );
        options.profile.counter(
            "frontend.reachable_bodies",
            reachability.reachable_bodies.len() as u64,
        );
    }
    options.profile.counter(
        "frontend.type_checked_bodies",
        response.output.type_body_outputs.len() as u64,
    );
    options.profile.counter(
        "frontend.effect_checked_bodies",
        response.output.effect_body_outputs.len() as u64,
    );
    for record in &response.pipeline_records {
        let Some(timing) = &record.timing else {
            continue;
        };
        let duration_ns = timing.duration.as_nanos().min(u128::from(u64::MAX)) as u64;
        options.profile.counter(
            format!("frontend.pass.{}.duration_ns", record.pass),
            duration_ns,
        );
    }
    if let Some(artifacts) = response.output.effect_pipeline_artifacts.as_ref() {
        let total_effect_pass_ns = artifacts.pass_timings.iter().fold(0_u64, |total, timing| {
            total.saturating_add(timing.duration_ns)
        });
        let mut next_effect_pass_start = options
            .profile
            .elapsed_ns()
            .unwrap_or(0)
            .saturating_sub(total_effect_pass_ns);
        for timing in &artifacts.pass_timings {
            options.profile.counter(
                format!("effects.pass.{}.duration_ns", timing.pass),
                timing.duration_ns,
            );
            options.profile.completed_span(
                timing.pass.clone(),
                "effects",
                next_effect_pass_start,
                timing.duration_ns,
            );
            next_effect_pass_start = next_effect_pass_start.saturating_add(timing.duration_ns);
        }
    }
}

fn has_blocking_diagnostics(diagnostics: &[etas_core::Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimization_level_override_reaches_checked_output_and_schedule() {
        const OPTIMIZATION_PASSES: [&str; 12] = [
            "AgentEffectLegalityPass",
            "LowerOwnedSnapshotCowPass",
            "FuseSerializationBoundaryPass",
            "SuffixDemandHistoryPass",
            "PruneUnusedQualityContextPass",
            "PrepareVersionedRequestPass",
            "VersionedPreparationCsePass",
            "FuseProjectionScansPass",
            "BatchStorageSemijoinPass",
            "SynthesizeScopedIndexPass",
            "ScheduleBoundedPipelinePass",
            "LowerResourceRegionPass",
        ];
        let expected_counts = [0, 5, 10, 12];

        for level in 0..=3 {
            let response = check_project_once(
                &DriverOptions::default()
                    .with_optimization_level(level)
                    .unwrap(),
                CompileProjectRequest {
                    input: ProjectInput::single_source(SourceInput::anonymous(
                        "flow main() -> i32 { return 42; }",
                    )),
                    snapshot_detail: SnapshotDetailLevel::None,
                },
            )
            .unwrap()
            .response;
            let checked = response.output.checked.as_ref().unwrap();
            assert_eq!(checked.compile_options.optimization_level.number(), level);
            let scheduled = response
                .pipeline_records
                .iter()
                .filter(|record| OPTIMIZATION_PASSES.contains(&record.pass))
                .map(|record| record.pass)
                .collect::<Vec<_>>();
            assert_eq!(
                scheduled,
                OPTIMIZATION_PASSES[..expected_counts[usize::from(level)]],
                "O{level}"
            );
        }
    }

    #[test]
    fn cache_off_one_shot_check_does_not_retain_an_incremental_revision() {
        for snapshot_detail in [SnapshotDetailLevel::None, SnapshotDetailLevel::Full] {
            let output = check_project_once(
                &DriverOptions {
                    cache_mode: DriverCacheMode::Off,
                    ..Default::default()
                },
                CompileProjectRequest {
                    input: ProjectInput::single_source(SourceInput::anonymous(
                        "flow main() -> i32 { return 42; }",
                    )),
                    snapshot_detail,
                },
            )
            .unwrap()
            .response;
            assert!(output.output.checked.is_some());
            assert!(output.memory.retained_revision.is_none());
            assert_eq!(output.memory.retained_revision_estimated_bytes, 0);
            assert_eq!(
                output
                    .memory
                    .cache_after
                    .as_ref()
                    .unwrap()
                    .estimated_retained_bytes,
                0
            );
            assert!(output.cache.stored_artifacts.is_empty());
            assert_eq!(
                output.snapshot.is_some(),
                snapshot_detail == SnapshotDetailLevel::Full
            );
            assert_eq!(output.memory.hir_index_builds, 2);
        }
    }

    #[test]
    fn memory_profile_counters_come_from_the_checked_cache_graph() {
        let profile = ProfileHandle::enabled("memory-profile");
        let options = DriverOptions {
            profile: profile.clone(),
            ..Default::default()
        };
        let mut session = FrontendSession::new();
        let project = session.open_project(ProjectInput::single_source(SourceInput::anonymous(
            "flow main() -> i32 { return 42; }",
        )));
        let response = session.check(project, CheckRequest::default()).unwrap();
        record_pipeline_profile(&options, &response);
        let report = profile.finish_report("ok").unwrap();
        let counter = |name: &str| {
            report
                .counters
                .iter()
                .find(|counter| counter.name == name)
                .map(|counter| counter.value)
                .unwrap()
        };
        let usage = response.memory.cache_after.as_ref().unwrap();
        assert_eq!(
            counter("frontend.cache.estimated_retained_bytes"),
            usage.estimated_retained_bytes
        );
        assert_eq!(
            counter("frontend.cache.payload_estimated_bytes"),
            usage.payload_estimated_bytes
        );
        assert_eq!(
            counter("frontend.retained_revision.estimated_bytes"),
            response.memory.retained_revision_estimated_bytes
        );
        let checked = usage
            .by_kind
            .iter()
            .find(|kind| kind.kind == "checked_project")
            .unwrap();
        assert_eq!(
            counter("frontend.cache.frontend.checked_project.referenced_payload_estimated_bytes"),
            checked.referenced_payload_estimated_bytes
        );
        assert!(checked.referenced_payload_estimated_bytes > 0);
    }
}
