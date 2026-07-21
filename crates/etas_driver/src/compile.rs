use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use etas_core::Severity;
use etas_frontend::{
    CheckRequest, CheckResponse, CheckScope, DependencyOverlayChange, DiskCacheAccess,
    ExternalPackageId, FrontendSession, FrontendSessionOptions, FrontendSessionStore, ModulePath,
    ProjectChangeSet, ProjectInput, ProjectRevision, RuntimeSourceRequirement,
    RuntimeSourceRequirements, SnapshotDetailLevel, SourceInput, SourceKind,
};
use etas_utils::ProfileHandle;

use crate::{
    DriverError, LoadProjectRequest, LoadedDriverProject, SourceDependencyMode, load_project,
    runtime_source::{RuntimeSourcePlan, materialize_runtime_source_plan},
    source::append_runtime_source_dependency_sources,
};

#[derive(Clone, Debug)]
pub struct DriverOptions {
    pub workspace: Option<PathBuf>,
    pub cache_mode: DriverCacheMode,
    pub cache_root: Option<PathBuf>,
    pub profile: ProfileHandle,
}

impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            workspace: None,
            cache_mode: DriverCacheMode::Auto,
            cache_root: None,
            profile: ProfileHandle::disabled(),
        }
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
    request: CompileProjectRequest,
) -> Result<CompileProjectResponse, DriverError> {
    let _span = options.profile.span("frontend.check", "frontend");
    let mut session = frontend_session(options)?;
    let project = session
        .try_open_project(request.input)
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
    let response = session
        .check(
            project,
            check_request(options).with_snapshot_detail(request.snapshot_detail),
        )
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
    let mut revision = 1;
    let mut iterations = 0_u64;
    loop {
        let requirements = response
            .output
            .runtime_source_requirements
            .clone()
            .unwrap_or_default();
        let delta = materialized.delta(&requirements);
        if delta.is_empty() {
            options
                .profile
                .counter("runtime_overlay.iterations", iterations);
            return Ok(CompileRunnableProjectResponse {
                loaded: runtime_loaded,
                response,
            });
        }

        let runtime_plan =
            materialize_runtime_source_plan(package_root, package_environment, &delta)?;
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
                    environment_change: None,
                    option_changes: Vec::new(),
                },
            )
            .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))?;
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
struct RuntimeSourceMaterializedSet {
    modules: BTreeMap<ExternalPackageId, BTreeSet<ModulePath>>,
}

impl RuntimeSourceMaterializedSet {
    fn delta(&self, requirements: &RuntimeSourceRequirements) -> RuntimeSourceRequirements {
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

    fn record_plan(&mut self, plan: &RuntimeSourcePlan) {
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
    FrontendSession::with_options(session_options(options)?)
        .map_err(|error| DriverError::FrontendSession(format!("{error:?}")))
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
