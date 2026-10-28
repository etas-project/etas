use std::sync::Arc;

use etas_code::{
    definition::DefinitionOperationError,
    descriptor::DescriptorError,
    diagnostic::{DiagnosticRecord, DiagnosticSeverity},
    syntax::SyntaxOperationError,
};
use etas_frontend::code::{
    definition::NativeDefinitionError,
    descriptor::ProjectionError,
    draft::OverlayError,
    service::{CandidateAnalysisError, CheckReportError, NativeCompilerError},
};
use etas_host::{HostError, HostErrorCode, compiler::CompilerFailure};

pub(super) fn definition_error(error: DefinitionOperationError) -> HostError {
    match error {
        DefinitionOperationError::ForeignDefinition
        | DefinitionOperationError::IncompatibleDefinitions => {
            HostError::new(HostErrorCode::InvalidRequest, error.to_string())
        }
        DefinitionOperationError::ResourceLimit { resource, limit } => {
            resource_error(resource, limit)
        }
        DefinitionOperationError::Syntax(error) => syntax_error(error),
        DefinitionOperationError::BackendUnavailable { reason } => {
            HostError::new(HostErrorCode::ProviderUnavailable, reason)
        }
    }
}

pub(super) fn compiler_failure(error: NativeCompilerError) -> Result<CompilerFailure, HostError> {
    match error {
        NativeCompilerError::Rejected(report) => Ok(CompilerFailure::Rejected(Arc::from(report))),
        NativeCompilerError::Analysis(CandidateAnalysisError::Interrupted(error)) => {
            Err(super::control::interrupted(error))
        }
        NativeCompilerError::Definition(NativeDefinitionError::Operation(error)) => {
            Err(definition_error(error))
        }
        NativeCompilerError::Definition(NativeDefinitionError::MissingSourceEnvironment) => {
            Ok(CompilerFailure::Service {
                diagnostics: vec![DiagnosticRecord {
                    code: "code::MissingSourceEnvironment".into(),
                    severity: DiagnosticSeverity::Error,
                    message: "compilation requires restoring the target's source and locked lexical environment; package descriptors are not executable evidence".into(),
                    location: None,
                    details: None,
                }],
            })
        }
        NativeCompilerError::Definition(NativeDefinitionError::Overlay(OverlayError::Syntax(
            error,
        ))) => Err(syntax_error(error)),
        NativeCompilerError::Definition(NativeDefinitionError::Projection(error))
        | NativeCompilerError::Report(CheckReportError::Projection(error)) => {
            projection_failure(error)
        }
        error => Ok(service_error(error.to_string())),
    }
}

fn projection_failure(error: ProjectionError) -> Result<CompilerFailure, HostError> {
    match error {
        ProjectionError::Limit { resource, limit } => Err(resource_error(resource, limit as u64)),
        ProjectionError::Descriptor(DescriptorError::Limit { resource, limit }) => {
            Err(resource_error(resource, limit as u64))
        }
        ProjectionError::Syntax(error) => Err(syntax_error(error)),
        error => Ok(service_error(error.to_string())),
    }
}

fn syntax_error(error: SyntaxOperationError) -> HostError {
    match error {
        SyntaxOperationError::Interrupted(error) => super::control::interrupted(error),
        SyntaxOperationError::ResourceLimit { resource, limit } => resource_error(resource, limit),
        SyntaxOperationError::ForeignSyntaxObject => {
            HostError::new(HostErrorCode::InvalidRequest, error.to_string())
        }
        SyntaxOperationError::BackendUnavailable { reason } => {
            HostError::new(HostErrorCode::ProviderUnavailable, reason)
        }
    }
}

fn resource_error(resource: impl std::fmt::Debug, limit: u64) -> HostError {
    HostError::new(
        HostErrorCode::BudgetExceeded,
        "compiler resource limit exceeded",
    )
    .with_detail("resource", format!("{resource:?}"))
    .with_detail("limit", limit.to_string())
}

fn service_error(message: String) -> CompilerFailure {
    CompilerFailure::Service {
        diagnostics: vec![DiagnosticRecord {
            code: "code::CompilerServiceFailure".into(),
            severity: DiagnosticSeverity::Error,
            message,
            location: None,
            details: None,
        }],
    }
}
