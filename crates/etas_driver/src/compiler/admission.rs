use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use etas_host::{HostError, HostErrorCode};

pub(super) struct CompilerAdmission {
    maximum: NonZeroUsize,
    active: AtomicUsize,
}

impl CompilerAdmission {
    pub(super) fn new(maximum: NonZeroUsize) -> Self {
        Self {
            maximum,
            active: AtomicUsize::new(0),
        }
    }

    pub(super) fn try_acquire(self: &Arc<Self>) -> Result<CompilerPermit, HostError> {
        self.active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.maximum.get()).then(|| active + 1)
            })
            .map_err(|_| {
                HostError::new(
                    HostErrorCode::ProviderUnavailable,
                    "native compiler worker capacity unavailable",
                )
                .with_detail("resource", "active_compiler_workers")
                .with_detail("limit", self.maximum.to_string())
            })?;
        Ok(CompilerPermit(self.clone()))
    }
}

pub(super) struct CompilerPermit(Arc<CompilerAdmission>);

impl Drop for CompilerPermit {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_is_shared_and_released_only_by_the_owning_permit() {
        let admission = Arc::new(CompilerAdmission::new(NonZeroUsize::MIN));
        let clone = admission.clone();
        let permit = admission.try_acquire().unwrap();
        assert!(clone.try_acquire().is_err());
        drop(admission);
        assert!(clone.try_acquire().is_err());
        drop(permit);
        assert!(clone.try_acquire().is_ok());
    }
}
