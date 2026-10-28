use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use etas_code::{
    CodeFingerprint, DefinitionKey, RevisionInfo,
    artifact::{CheckedCodeLease, CheckedCodeRef},
    definition::AgentDefinitionRef,
};
use etas_core::memory::MemoryFootprint;
use etas_frontend::code::service::{CandidatePublisher, CheckedCandidate};
use etas_host::{HostError, HostErrorCode};

use super::{NativeCompilerConfig, NativeCompilerService};

#[derive(etas_core::RetainedSize)]
struct NativeCandidate {
    candidate: CheckedCandidate,
    definition: AgentDefinitionRef,
}

impl std::fmt::Debug for NativeCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCheckedCode")
            .field("revision", self.candidate.revision())
            .finish()
    }
}

impl CheckedCodeLease for NativeCandidate {
    fn identity(&self) -> &DefinitionKey {
        self.candidate.identity()
    }
    fn revision(&self) -> &RevisionInfo {
        self.candidate.revision()
    }
    fn snapshot_fingerprint(&self) -> CodeFingerprint {
        self.candidate.snapshot_fingerprint()
    }
}

#[derive(Default)]
pub(super) struct NativeCandidateStore {
    objects: Mutex<HashMap<CodeFingerprint, Vec<Weak<NativeCandidate>>>>,
}

impl NativeCandidateStore {
    pub(super) fn retain(
        &self,
        candidate: CheckedCandidate,
        definition: AgentDefinitionRef,
        config: NativeCompilerConfig,
    ) -> Result<CheckedCodeRef, HostError> {
        let mut objects = self.objects.lock().map_err(lock_error)?;
        objects.retain(|_, bucket| {
            bucket.retain(|item| item.strong_count() != 0);
            !bucket.is_empty()
        });
        let fingerprint = candidate.snapshot_fingerprint();
        let mut live: Vec<_> = objects
            .values()
            .flatten()
            .filter_map(Weak::upgrade)
            .collect();
        // Do not merge leases across accepted bodies merely because a digest
        // matches. The actual native candidate remains the authority.
        if live.len() >= config.max_retained_candidates {
            return Err(limit_error("retained_candidates"));
        }
        let candidate = Arc::new(NativeCandidate {
            candidate,
            definition,
        });
        live.push(candidate.clone());
        if MemoryFootprint::of(&live).estimated_bytes() > config.max_retained_candidate_bytes {
            return Err(limit_error("retained_candidate_bytes"));
        }
        objects
            .entry(fingerprint)
            .or_default()
            .push(Arc::downgrade(&candidate));
        Ok(CheckedCodeRef::from_backend(candidate))
    }

    fn resolve(&self, reference: &CheckedCodeRef) -> Result<Arc<NativeCandidate>, HostError> {
        self.objects
            .lock()
            .map_err(lock_error)?
            .get(&reference.snapshot_fingerprint())
            .into_iter()
            .flatten()
            .filter_map(Weak::upgrade)
            .find(|candidate| reference.is_backed_by(candidate))
            .ok_or_else(|| {
                HostError::new(
                    HostErrorCode::InvalidRequest,
                    "checked code reference is not owned by this compiler service",
                )
            })
    }
}

#[derive(Debug)]
pub enum CandidatePublicationError<E> {
    Reference(HostError),
    Backend(E),
}

impl NativeCompilerService {
    pub fn reflect_compiled(
        &self,
        reference: &CheckedCodeRef,
    ) -> Result<
        AgentDefinitionRef,
        CandidatePublicationError<etas_frontend::code::definition::NativeDefinitionError>,
    > {
        let candidate = self
            .candidates
            .resolve(reference)
            .map_err(CandidatePublicationError::Reference)?;
        self.definitions
            .reflect_checked(
                &candidate.definition,
                &candidate.candidate,
                self.config.descriptors,
            )
            .map_err(CandidatePublicationError::Backend)
    }
    /// Called on the engine's owning executor, never inside the compiler worker.
    /// Preparing an implementation does not execute it or grant it authority.
    pub fn publish<P: CandidatePublisher>(
        &self,
        reference: &CheckedCodeRef,
        publisher: &mut P,
    ) -> Result<P::Artifact, CandidatePublicationError<P::Error>> {
        let candidate = self
            .candidates
            .resolve(reference)
            .map_err(CandidatePublicationError::Reference)?;
        publisher
            .publish(candidate.candidate.clone())
            .map_err(CandidatePublicationError::Backend)
    }
}

impl etas_frontend::code::service::CheckedCandidateResolver for NativeCompilerService {
    fn resolve_checked_candidate(
        &self,
        reference: &CheckedCodeRef,
    ) -> Result<CheckedCandidate, etas_frontend::code::service::CandidateResolutionError> {
        use etas_frontend::code::service::CandidateResolutionError;
        self.candidates
            .resolve(reference)
            .map(|candidate| candidate.candidate.clone())
            .map_err(|error| {
                if error.code == HostErrorCode::InvalidRequest {
                    CandidateResolutionError::UnownedReference
                } else {
                    CandidateResolutionError::Unavailable(error.message)
                }
            })
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> HostError {
    HostError::new(
        HostErrorCode::ProviderUnavailable,
        "native checked code store is poisoned",
    )
}
fn limit_error(resource: &'static str) -> HostError {
    HostError::new(
        HostErrorCode::BudgetExceeded,
        "compiler retained output limit exceeded",
    )
    .with_detail("resource", resource)
}
