use std::{future::Future, num::NonZeroUsize, pin::Pin, sync::Arc};

use etas_code::descriptor::DescriptorLimits;
use etas_frontend::code::definition::NativeDefinitionStore;
use etas_host::{
    HostError,
    compiler::{CompilerClient, CompilerRequest, CompilerResponse, limits::CompilerResourceLimits},
    execution::OperationContext,
};

use super::{admission::CompilerAdmission, publication::NativeCandidateStore};

#[derive(Clone, Copy, Debug)]
pub struct NativeCompilerConfig {
    pub descriptors: DescriptorLimits,
    pub max_retained_candidates: usize,
    pub max_retained_candidate_bytes: u64,
    pub resources: CompilerResourceLimits,
    pub max_active_workers: NonZeroUsize,
}

impl Default for NativeCompilerConfig {
    fn default() -> Self {
        Self {
            descriptors: Default::default(),
            max_retained_candidates: 64,
            max_retained_candidate_bytes: 256 * 1024 * 1024,
            resources: Default::default(),
            max_active_workers: NonZeroUsize::MIN,
        }
    }
}

#[derive(Clone)]
pub struct NativeCompilerService {
    pub(super) definitions: Arc<NativeDefinitionStore>,
    pub(super) candidates: Arc<NativeCandidateStore>,
    pub(super) config: NativeCompilerConfig,
    pub(super) admission: Arc<CompilerAdmission>,
    pub(super) package_sources: Option<Arc<super::sources::CompilerPackageSources>>,
}

impl NativeCompilerService {
    pub fn new(definitions: Arc<NativeDefinitionStore>, config: NativeCompilerConfig) -> Self {
        Self {
            definitions,
            candidates: Arc::new(NativeCandidateStore::default()),
            config,
            admission: Arc::new(CompilerAdmission::new(config.max_active_workers)),
            package_sources: None,
        }
    }

    pub fn with_project_sources(
        mut self,
        loaded: &crate::LoadedDriverProject,
    ) -> Result<Self, crate::DriverError> {
        let checked = self.definitions.catalog().checked_program();
        super::source_definition_origins(loaded, checked)?;
        loaded
            .source_definition_provenance
            .validate_external_definition_origins(checked)?;
        if loaded
            .loaded
            .input
            .environment
            .canonical_environment_fingerprint()
            != checked.project_environment_fingerprint
        {
            return Err(crate::DriverError::InvalidInput(
                "compiler source provider differs from the checked package environment".into(),
            ));
        }
        self.package_sources = match (&loaded.package_root, &loaded.package_environment) {
            (Some(root), Some(environment)) => {
                Some(Arc::new(super::sources::CompilerPackageSources {
                    root: root.clone(),
                    environment: environment.clone(),
                }))
            }
            (None, None) => None,
            _ => {
                return Err(crate::DriverError::InvalidInput(
                    "compiler package source provider is incomplete".into(),
                ));
            }
        };
        Ok(self)
    }
}

impl CompilerClient for NativeCompilerService {
    type ExecuteFuture<'a> =
        Pin<Box<dyn Future<Output = Result<CompilerResponse, HostError>> + Send + 'a>>;
    fn execute<'a>(
        &'a self,
        request: CompilerRequest,
        operation: &'a OperationContext,
    ) -> Self::ExecuteFuture<'a> {
        Box::pin(self.dispatch(request, operation))
    }
}
