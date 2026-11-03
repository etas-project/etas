use std::path::PathBuf;

use etas_frontend::{
    FrontendSession,
    code::service::{CandidateBodyAnalysis, CandidateDependencySource, NativeCompilerError},
};
use etas_host::{
    HostError, HostErrorCode,
    compiler::{CompilerOperation, CompilerRequest},
};

use super::{NativeCompilerService, control::CompilerOperationControl};
use crate::{
    compile::RuntimeSourceMaterializedSet,
    runtime_source::{RuntimeSourceError, RuntimeSourceMaterializer, SourceInputError},
};

/// Prepared package locations and lock records, not a resolver attached to a
/// reflected definition. Source bytes are read only by an admitted Code job.
pub(super) struct CompilerPackageSources {
    pub root: PathBuf,
    pub environment: etas_package::PackageEnvironmentMetadata,
}

impl NativeCompilerService {
    pub(super) fn restore_external_source(
        &self,
        request: &CompilerRequest,
        session: &mut FrontendSession,
        control: std::sync::Arc<CompilerOperationControl>,
    ) -> Result<(Self, CompilerRequest), HostError> {
        let view = self
            .definitions
            .resolve(&request.definition)
            .map_err(super::error::definition_error)?;
        if view.environment().is_ok() {
            return Ok((self.clone(), request.clone()));
        }
        let Some(application) = view.external_application() else {
            // Metadata-only definitions without a checked application cannot be
            // restored here; candidate analysis reports their missing source environment.
            return Ok((self.clone(), request.clone()));
        };
        let package = application.package().ok_or_else(|| {
            HostError::new(
                HostErrorCode::InvalidRequest,
                "external definition has no resolved package identity",
            )
        })?;
        let sources = self.package_sources.as_ref().ok_or_else(|| {
            HostError::new(
                HostErrorCode::ProviderUnavailable,
                "locked package source provider is unavailable",
            )
        })?;
        let (owner, _) = crate::runtime_source::dependency_source_location(
            &sources.root,
            &sources.environment.dependencies,
            package,
        )
        .map_err(|error| HostError::new(HostErrorCode::InvalidRequest, error.to_string()))?;
        let expected = etas_code::PackageKey::from_name_and_version(
            &owner.identity.name,
            &owner.identity.version,
        )
        .map_err(|_| HostError::new(HostErrorCode::InvalidRequest, "invalid package identity"))?;
        if application
            .callable()
            .as_declaration()
            .is_none_or(|id| id.package() != expected)
        {
            return Err(HostError::new(
                HostErrorCode::InvalidRequest,
                "resolved package differs from checked definition identity",
            ));
        }
        let snapshot = RuntimeSourceMaterializer::new(&sources.root, &sources.environment)
            .locked_snapshot(package, &|| control.check())
            .map_err(source_error)?;
        let loaded =
            crate::source::load_locked_source_project(&snapshot, owner, &|| control.check())?;
        let project = session
            .try_open_project(loaded.loaded.input.clone())
            .map_err(|error| {
                HostError::new(
                    HostErrorCode::InvalidRequest,
                    format!("locked source project: {error:?}"),
                )
            })?;
        let response = session
            .check_controlled(
                project,
                etas_frontend::CheckRequest::one_shot(),
                control.clone(),
            )
            .map_err(|error| {
                control.check().err().unwrap_or_else(|| {
                    HostError::new(
                        HostErrorCode::InvalidRequest,
                        format!("locked source check: {error:?}"),
                    )
                })
            })?;
        control.check()?;
        if response
            .output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error)
        {
            return Err(HostError::new(
                HostErrorCode::InvalidRequest,
                "locked source project contains blocking diagnostics",
            ));
        }
        let checked = response.output.checked.as_ref().ok_or_else(|| {
            HostError::new(
                HostErrorCode::InvalidRequest,
                "locked source project did not produce checked facts",
            )
        })?;
        let producer = super::prepare_native_definitions(&loaded, checked, self.config.descriptors)
            .map_err(|error| HostError::new(HostErrorCode::InvalidRequest, error.to_string()))?;
        let definition = self
            .definitions
            .restore_external_application_controlled(
                &request.definition,
                &producer,
                self.config.descriptors,
                control.as_ref(),
            )
            .map_err(|error| {
                control.check().err().unwrap_or_else(|| {
                    HostError::new(HostErrorCode::InvalidRequest, error.to_string())
                })
            })?;
        control.check()?;
        let mut restored = request.clone();
        restored.definition = definition;
        let mut service = self.clone();
        service.package_sources = match (loaded.package_root, loaded.package_environment) {
            (Some(root), Some(environment)) => Some(std::sync::Arc::new(CompilerPackageSources {
                root,
                environment,
            })),
            _ => {
                return Err(HostError::new(
                    HostErrorCode::InvalidRequest,
                    "locked producer package environment is incomplete",
                ));
            }
        };
        Ok((service, restored))
    }

    pub(super) fn analyze_complete(
        &self,
        request: &CompilerRequest,
        session: &mut FrontendSession,
        control: std::sync::Arc<CompilerOperationControl>,
    ) -> Result<Result<CandidateBodyAnalysis, NativeCompilerError>, HostError> {
        let mut materializer = self
            .package_sources
            .as_ref()
            .map(|sources| RuntimeSourceMaterializer::new(&sources.root, &sources.environment));
        let mut materialized = RuntimeSourceMaterializedSet::default();
        let mut sources: Vec<CandidateDependencySource> = Vec::new();
        let first_id = self
            .definitions
            .catalog()
            .checked_program()
            .sources
            .sources
            .iter()
            .map(|source| source.id.0)
            .max()
            .and_then(|id| id.checked_add(1))
            .ok_or_else(|| {
                HostError::new(
                    HostErrorCode::InvalidRequest,
                    "compiler source identity is unavailable",
                )
            })?;
        loop {
            control.check()?;
            let analysis = match self.definitions.analyze_definition_with_dependencies(
                &request.definition,
                session,
                sources.clone(),
                self.config.descriptors,
                control.clone(),
            ) {
                Ok(analysis) => analysis,
                Err(error) => return Ok(Err(error)),
            };
            let source_rejected = analysis
                .output()
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error);
            let contract_rejected = matches!(analysis.contract_analysis(),
                Some(etas_frontend::code::contract::CandidateContractAnalysis::Analyzed { checks, .. })
                    if checks.iter().any(|check| matches!(check.relation, etas_frontend::code::contract::ContractRelation::Rejected(_))));
            if source_rejected || contract_rejected {
                return Ok(Ok(analysis));
            }
            let contract_proven = matches!(analysis.contract_analysis(),
                Some(etas_frontend::code::contract::CandidateContractAnalysis::Analyzed { checks, .. })
                    if !checks.is_empty() && checks.iter().all(|check| matches!(check.relation, etas_frontend::code::contract::ContractRelation::Proven)));
            if request.operation == CompilerOperation::Check
                && (contract_proven || materializer.is_none())
            {
                return Ok(Ok(analysis));
            }
            let Some(requirements) = &analysis.output().runtime_source_requirements else {
                return Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "candidate analysis did not produce runtime source requirements",
                ));
            };
            if requirements.is_empty() {
                return Ok(Ok(analysis));
            }
            let delta = materialized.delta(requirements);
            if delta.is_empty() {
                return Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "candidate runtime source completion made no progress",
                ));
            }
            let materializer = materializer.as_mut().ok_or_else(|| {
                HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "candidate requires a locked package source provider",
                )
            })?;
            let plan = materializer
                .plan_controlled(&delta, &|| control.check())
                .map_err(source_error)?;
            let count = u32::try_from(sources.len())
                .ok()
                .and_then(|count| first_id.checked_add(count))
                .ok_or_else(|| {
                    HostError::new(
                        HostErrorCode::BudgetExceeded,
                        "compiler source identity exhausted",
                    )
                })?;
            let added = plan
                .source_inputs_controlled(count, &|| control.check())
                .map_err(|error| match error {
                    SourceInputError::Driver(error) => {
                        HostError::new(HostErrorCode::InvalidRequest, error.to_string())
                    }
                    SourceInputError::Interrupted(error) => error,
                })?;
            if added.is_empty() {
                return Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "locked package source plan contains no required source files",
                ));
            }
            materialized.record_plan(&plan);
            for source in added {
                let etas_frontend::SourceKind::DependencySourceOverlay {
                    package,
                    import_root,
                } = &source.kind
                else {
                    return Err(HostError::new(
                        HostErrorCode::InvalidRequest,
                        "compiler source patch contains a non-dependency source",
                    ));
                };
                let path = source.path.as_ref().ok_or_else(|| {
                    HostError::new(HostErrorCode::InvalidRequest, "compiler source has no path")
                })?;
                let mut matching = plan.dependencies.values().filter(|dependency| {
                    dependency.package == *package
                        && dependency.import_root == *import_root
                        && dependency.source_files.contains(path)
                });
                let dependency = matching.next().ok_or_else(|| {
                    HostError::new(
                        HostErrorCode::InvalidRequest,
                        "compiler source has no matching locked dependency",
                    )
                })?;
                if matching.next().is_some() {
                    return Err(HostError::new(
                        HostErrorCode::InvalidRequest,
                        "compiler source has ambiguous locked dependencies",
                    ));
                }
                let identity =
                    etas_package::PackageIdentity::current(dependency.snapshot.manifest());
                let owner =
                    etas_code::PackageKey::from_name_and_version(&identity.name, &identity.version)
                        .map_err(|_| {
                            HostError::new(
                                HostErrorCode::InvalidRequest,
                                "invalid package identity",
                            )
                        })?;
                let key = crate::source::stable_package_source_key(
                    &source,
                    owner,
                    &dependency.package_root,
                )
                .map_err(|error| {
                    HostError::new(HostErrorCode::InvalidRequest, error.to_string())
                })?;
                sources.push(CandidateDependencySource {
                    source,
                    stable_key: key.key,
                });
            }
        }
    }
}

fn source_error(error: RuntimeSourceError<HostError>) -> HostError {
    use etas_package::SourceSnapshotError;
    match error {
        RuntimeSourceError::Interrupted(error) => error,
        RuntimeSourceError::ParseResource(error) => HostError::new(
            HostErrorCode::BudgetExceeded,
            format!(
                "runtime source parser {:?} limit {} exceeded",
                error.kind, error.limit
            ),
        ),
        RuntimeSourceError::Driver(error) => {
            HostError::new(HostErrorCode::InvalidRequest, error.to_string())
        }
        RuntimeSourceError::Snapshot(SourceSnapshotError::Interrupted(error)) => error,
        RuntimeSourceError::Snapshot(error @ SourceSnapshotError::ResourceLimit { .. }) => {
            HostError::new(HostErrorCode::BudgetExceeded, error.to_string())
        }
        RuntimeSourceError::Snapshot(error) => {
            HostError::new(HostErrorCode::InvalidRequest, error.to_string())
        }
    }
}
