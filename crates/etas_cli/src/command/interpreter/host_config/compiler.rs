use std::{
    num::{NonZeroU64, NonZeroUsize},
    time::Duration,
};

use etas_driver::compiler::NativeCompilerConfig;
use etas_host::{HostActionGrant, compiler::limits::CompilerResourceLimits};
use etas_package::RuntimeCompilerProfile;

use crate::error::CliError;

#[derive(Clone, Copy)]
pub(crate) struct CliCompilerConfig {
    pub(crate) native: NativeCompilerConfig,
    allow_check: bool,
    allow_compile: bool,
}

impl CliCompilerConfig {
    pub(super) fn from_profile(
        previous: Option<Self>,
        profile: &RuntimeCompilerProfile,
    ) -> Result<Self, CliError> {
        match profile.backend.as_deref() {
            Some("native") => {}
            None if previous.is_some() => {}
            _ => {
                return Err(CliError::InvalidUsage(
                    "`runtime.compiler.backend` must explicitly select `native`".into(),
                ));
            }
        }
        let mut config = previous.unwrap_or(Self {
            native: NativeCompilerConfig::default(),
            allow_check: false,
            allow_compile: false,
        });
        if let Some(value) = profile.allow_check {
            config.allow_check = value;
        }
        if let Some(value) = profile.allow_compile {
            config.allow_compile = value;
        }
        if let Some(value) = profile.max_active_workers {
            config.native.max_active_workers =
                NonZeroUsize::new(value).ok_or_else(|| nonzero("max_active_workers"))?;
        }
        if let Some(value) = profile.max_retained_candidates {
            config.native.max_retained_candidates = NonZeroUsize::new(value)
                .ok_or_else(|| nonzero("max_retained_candidates"))?
                .get();
        }
        if let Some(value) = profile.max_retained_candidate_bytes {
            config.native.max_retained_candidate_bytes = NonZeroU64::new(value)
                .ok_or_else(|| nonzero("max_retained_candidate_bytes"))?
                .get();
        }
        let cpu = match profile.max_worker_cpu_ms {
            Some(value) => Duration::from_millis(
                NonZeroU64::new(value)
                    .ok_or_else(|| nonzero("max_worker_cpu_ms"))?
                    .get(),
            ),
            None => config.native.resources.max_worker_cpu_time(),
        };
        let memory = match profile.max_process_peak_resident_bytes {
            Some(value) => {
                NonZeroU64::new(value).ok_or_else(|| nonzero("max_process_peak_resident_bytes"))?
            }
            None => config.native.resources.max_process_peak_resident_bytes(),
        };
        config.native.resources = CompilerResourceLimits::new(cpu, memory)
            .map_err(|error| CliError::InvalidUsage(error.message))?;
        Ok(config)
    }

    pub(super) fn add_grants(self, grants: &mut Vec<HostActionGrant>) {
        if self.allow_check {
            grants.push(HostActionGrant::allow("Code", "check"));
        }
        if self.allow_compile {
            grants.push(HostActionGrant::allow("Code", "compile"));
        }
    }

    pub(super) fn json(self) -> serde_json::Value {
        serde_json::json!({
            "backend": "native",
            "allow_check": self.allow_check,
            "allow_compile": self.allow_compile,
            "max_active_workers": self.native.max_active_workers.get(),
            "max_retained_candidates": self.native.max_retained_candidates,
            "max_retained_candidate_bytes": self.native.max_retained_candidate_bytes,
            "max_worker_cpu_ms": self.native.resources.max_worker_cpu_time().as_millis(),
            "max_process_peak_resident_bytes": self.native.resources.max_process_peak_resident_bytes().get(),
        })
    }
}

fn nonzero(field: &str) -> CliError {
    CliError::InvalidUsage(format!("`runtime.compiler.{field}` must be nonzero"))
}
