use etas_host::compiler::limits::{CompilerResourceLimits, CompilerResourceMeter};
use etas_host::{ExecutionBudget, HostError, HostErrorCode, execution::OperationContext};
use etas_utils::{PipelineExecutionControl, PipelineInterruption, PipelineInterruptionKind};

pub(super) struct CompilerOperationControl {
    operation: OperationContext,
    budget: ExecutionBudget,
    resources: CompilerResourceMeter,
}

impl CompilerOperationControl {
    pub(super) fn new(
        operation: OperationContext,
        budget: ExecutionBudget,
        limits: CompilerResourceLimits,
    ) -> Result<Self, HostError> {
        operation.signal().check()?;
        budget.check_time()?;
        Ok(Self {
            operation,
            budget,
            resources: CompilerResourceMeter::start(limits)?,
        })
    }

    pub(super) fn check(&self) -> Result<(), HostError> {
        self.operation.signal().check()?;
        self.budget.check_time()?;
        self.resources.check()?;
        Ok(())
    }
}

impl PipelineExecutionControl for CompilerOperationControl {
    fn checkpoint(&self) -> Result<(), PipelineInterruption> {
        self.check().map_err(|error| PipelineInterruption {
            kind: match error.code {
                HostErrorCode::Cancelled => PipelineInterruptionKind::Cancelled,
                HostErrorCode::BudgetExceeded | HostErrorCode::TimedOut => {
                    PipelineInterruptionKind::ResourceLimit
                }
                _ => PipelineInterruptionKind::Unavailable,
            },
            reason: error.to_string(),
        })
    }
}

pub(super) fn interrupted(error: PipelineInterruption) -> HostError {
    HostError::new(
        match error.kind {
            PipelineInterruptionKind::Cancelled => HostErrorCode::Cancelled,
            PipelineInterruptionKind::ResourceLimit => HostErrorCode::BudgetExceeded,
            PipelineInterruptionKind::Unavailable => HostErrorCode::ProviderUnavailable,
        },
        error.reason,
    )
}
