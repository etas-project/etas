use std::sync::Arc;

use etas_frontend::FrontendSession;
use etas_host::{
    HostError,
    compiler::{
        CompilerOperation, CompilerRequest, CompilerResponse, CompilerResult,
        validation::{validate_compiler_request, validate_compiler_response},
    },
    execution::OperationContext,
};

use super::{NativeCompilerService, control::CompilerOperationControl};

enum NativeWorkResult {
    Checked(Arc<etas_code::report::CheckReport>),
    Compiled(Box<etas_frontend::code::service::CheckedCandidate>),
}

impl NativeCompilerService {
    pub(super) async fn dispatch(
        &self,
        request: CompilerRequest,
        operation: &OperationContext,
    ) -> Result<CompilerResponse, HostError> {
        validate_compiler_request(&request, operation)?;
        request.budget.check_time()?;
        let permit = self.admission.try_acquire()?;
        self.definitions
            .resolve(&request.definition)
            .map_err(super::error::definition_error)?;
        let std_registry = self
            .definitions
            .catalog()
            .checked_program()
            .std_registry
            .clone();
        let service = self.clone();
        let work = request.clone();
        let response = operation
            .run_blocking(request.budget.clone(), move |context| {
                let _permit = permit;
                let control = Arc::new(CompilerOperationControl::new(
                    context,
                    work.budget.clone(),
                    service.config.resources,
                )?);
                let mut session = FrontendSession::with_std_registry(std_registry);
                let analysis = service.analyze_complete(&work, &mut session, control.clone())?;
                let result = match (work.operation, analysis) {
                    (CompilerOperation::Check, Ok(analysis)) => analysis
                        .check_report(service.config.descriptors)
                        .map(|report| NativeWorkResult::Checked(Arc::new(report)))
                        .map_err(etas_frontend::code::service::NativeCompilerError::Report),
                    (
                        CompilerOperation::Check,
                        Err(etas_frontend::code::service::NativeCompilerError::Rejected(report)),
                    ) => Ok(NativeWorkResult::Checked(Arc::from(report))),
                    (CompilerOperation::Compile, Ok(analysis)) => analysis
                        .into_checked_candidate(service.config.descriptors)
                        .map(|candidate| NativeWorkResult::Compiled(Box::new(candidate)))
                        .map_err(etas_frontend::code::service::NativeCompilerError::from),
                    (_, Err(error)) => Err(error),
                };
                control.check()?;
                let result = match result {
                    Ok(NativeWorkResult::Checked(report)) => Ok(CompilerResult::Checked(report)),
                    Ok(NativeWorkResult::Compiled(candidate)) => {
                        Ok(CompilerResult::Compiled(service.candidates.retain(
                            *candidate,
                            work.definition.clone(),
                            service.config,
                        )?))
                    }
                    Err(error) => Err(super::error::compiler_failure(error)?),
                };
                let response = CompilerResponse {
                    id: work.id,
                    result,
                };
                validate_compiler_response(&work, &response, service.config.descriptors)?;
                control.check()?;
                Ok(response)
            })
            .await?;
        operation.signal().check()?;
        request.budget.check_time()?;
        Ok(response)
    }
}
