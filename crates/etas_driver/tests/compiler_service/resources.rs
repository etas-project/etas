use super::*;
use etas_host::compiler::limits::CompilerResourceLimits;
use std::num::NonZeroU64;

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn native_cpu_and_memory_exhaustion_never_publish_reports_or_candidates() {
    let (definitions, original) = fixture();
    runtime().block_on(async {
        for (resources, resource) in [
            (
                CompilerResourceLimits::new(Duration::from_nanos(1), NonZeroU64::MAX).unwrap(),
                "worker_cpu_time",
            ),
            (
                CompilerResourceLimits::new(Duration::from_secs(60), NonZeroU64::MIN).unwrap(),
                "process_peak_resident_memory",
            ),
        ] {
            let service = NativeCompilerService::new(
                definitions.clone(),
                NativeCompilerConfig {
                    resources,
                    ..Default::default()
                },
            );
            for operation in [CompilerOperation::Check, CompilerOperation::Compile] {
                let error = execute(&service, request(original.clone(), operation))
                    .await
                    .unwrap_err();
                assert_eq!(error.code, HostErrorCode::BudgetExceeded);
                assert!(
                    error
                        .details
                        .iter()
                        .any(|detail| { detail.key == "resource" && detail.value == resource }),
                    "{error:?}"
                );
            }
        }
        // Failed jobs release service admission and do not poison the definition store.
        let service = NativeCompilerService::new(definitions, Default::default());
        assert!(
            execute(&service, request(original, CompilerOperation::Compile))
                .await
                .unwrap()
                .result
                .is_ok()
        );
    });
}
