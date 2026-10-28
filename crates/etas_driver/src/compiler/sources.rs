use std::path::PathBuf;

use etas_frontend::{
    FrontendSession, SourceInput,
    code::service::{CandidateBodyAnalysis, NativeCompilerError},
};
use etas_host::{HostError, HostErrorCode, compiler::CompilerRequest};

use super::{NativeCompilerService, control::CompilerOperationControl};
use crate::{
    compile::RuntimeSourceMaterializedSet,
    runtime_source::{RuntimeSourceError, RuntimeSourceMaterializer},
};

/// Prepared package locations and lock records, not a resolver attached to a
/// reflected definition. Source bytes are read only by an admitted Code job.
pub(super) struct CompilerPackageSources {
    pub root: PathBuf,
    pub environment: etas_package::PackageEnvironmentMetadata,
}

impl NativeCompilerService {
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
        let mut sources: Vec<SourceInput> = Vec::new();
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
            let added = plan.source_inputs(count).map_err(|error| {
                HostError::new(HostErrorCode::InvalidRequest, error.to_string())
            })?;
            if added.is_empty() {
                return Err(HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "locked package source plan contains no required source files",
                ));
            }
            materialized.record_plan(&plan);
            sources.extend(added);
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
