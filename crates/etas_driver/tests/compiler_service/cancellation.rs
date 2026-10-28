use super::*;
use std::future::Future;

struct CancellingClock {
    base: Instant,
    checks: AtomicUsize,
    source: etas_host::execution::CancelSource,
}

impl MonotonicClock for CancellingClock {
    fn now(&self) -> Instant {
        if self.checks.fetch_add(1, Ordering::SeqCst) == 63 {
            self.source.stop(CancellationReason::Requested).unwrap();
        }
        self.base
    }
}

#[test]
fn cancelling_during_real_frontend_checkpoints_releases_the_worker_without_a_result() {
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(
        definitions,
        NativeCompilerConfig {
            max_retained_candidates: 1,
            ..Default::default()
        },
    );
    runtime().block_on(async {
        for operation in [CompilerOperation::Check, CompilerOperation::Compile] {
            let scope = ExecutionScope::new();
            let clock = Arc::new(CancellingClock {
                base: Instant::now(),
                checks: AtomicUsize::new(0),
                source: scope.cancel_source(),
            });
            let mut work = request(original.clone(), operation);
            work.budget = ExecutionBudget::start_with_clock(
                Budget {
                    time: Some(TimeBudget { max_millis: 60_000 }),
                    ..Default::default()
                },
                clock.clone(),
            );
            let parent = scope
                .register(None, Some(work.id), work.trace.clone())
                .unwrap();
            parent.begin_dispatch().unwrap();
            let error = service.execute(work, parent.context()).await.unwrap_err();
            assert_eq!(error.code, HostErrorCode::Cancelled);
            assert!(clock.checks.load(Ordering::SeqCst) >= 64);
            parent.complete(ExternalOutcome::Unknown, vec![]).unwrap();
            scope.finish_body(false).unwrap();
            let report = tokio::time::timeout(Duration::from_secs(5), scope.join())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(report.operations().len(), 2);
            assert!(report.operations()[1].dispatched());
            assert!(report.operations().iter().all(|operation| {
                operation.outcome().is_some()
                    && operation.cleanup_errors().is_empty()
                    && !operation.owner_lost()
            }));
        }
        assert!(
            execute(&service, request(original, CompilerOperation::Compile))
                .await
                .unwrap()
                .result
                .is_ok()
        );
    });
}

#[test]
fn dropping_the_receiver_does_not_abandon_registered_compiler_work() {
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(definitions, Default::default());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap();
    runtime.block_on(async {
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let busy = tokio::task::spawn_blocking(move || {
            started.send(()).unwrap();
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
        });
        ready.await.unwrap();
        let work = request(original.clone(), CompilerOperation::Compile);
        let scope = ExecutionScope::new();
        let parent = scope
            .register(None, Some(work.id), work.trace.clone())
            .unwrap();
        parent.begin_dispatch().unwrap();
        let context = parent.context().clone();
        {
            let job = service.execute(work, &context);
            tokio::pin!(job);
            std::future::poll_fn(|cx| {
                assert!(job.as_mut().poll(cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
        }
        assert_eq!(scope.pending().unwrap().operations().len(), 2);
        let error = execute(
            &service.clone(),
            request(original.clone(), CompilerOperation::Check),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, HostErrorCode::ProviderUnavailable);
        assert!(error.details.iter().any(|detail| {
            detail.key == "resource" && detail.value == "active_compiler_workers"
        }));
        scope
            .cancel_source()
            .stop(CancellationReason::Requested)
            .unwrap();
        parent.complete(ExternalOutcome::Unknown, vec![]).unwrap();
        scope.finish_body(false).unwrap();
        release.send(()).unwrap();
        busy.await.unwrap();
        let report = tokio::time::timeout(Duration::from_secs(5), scope.join())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(report.operations().len(), 2);
        assert_eq!(
            report.operations()[1].outcome(),
            Some(&ExternalOutcome::NotDispatched)
        );
        assert!(
            report
                .operations()
                .iter()
                .all(|operation| operation.cleanup_errors().is_empty())
        );
        assert!(
            execute(&service, request(original, CompilerOperation::Check))
                .await
                .unwrap()
                .result
                .is_ok()
        );
    });
}
