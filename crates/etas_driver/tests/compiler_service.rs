use etas_code::{
    CodeFingerprint, DefinitionKey, PackageKey, RevisionInfo,
    artifact::{CheckedCodeLease, CheckedCodeRef},
    definition::{CodeDefinitionOperations, DefinitionRef},
    report::CheckStatus,
    syntax::CodeSyntaxOperations,
};
use etas_driver::compiler::{
    CandidatePublicationError, NativeCompilerConfig, NativeCompilerService,
};
use etas_frontend::{
    EntryPolicy, Frontend, ProjectInput, SourceInput,
    code::{
        definition::{DefinitionCatalog, NativeDefinitionStore, ReflectionOptions},
        identity::DefinitionOrigins,
        service::NativeCodeSyntaxOperations,
    },
};
use etas_host::{
    AuthorityContext, Budget, ExecutionBudget, HostActionGrant, HostError, HostErrorCode,
    HostRequestId, HostTraceRequest, MonotonicClock, TimeBudget, TokenBudget, TraceContext,
    TraceId,
    compiler::{
        CompilerClient, CompilerFailure, CompilerOperation, CompilerRequest, CompilerResponse,
        CompilerResult, validation::validate_compiler_response,
    },
    execution::{CancellationReason, ExecutionScope, ExternalOutcome},
};
use etas_interpreter::api::code::InterpreterCandidatePublisher;
use etas_interpreter::host::{CompilerHost, HostServices};
#[path = "compiler_service/cancellation.rs"]
mod cancellation;
#[path = "compiler_service/dependency_namespaces.rs"]
mod dependency_namespaces;
#[path = "compiler_service/external_agents.rs"]
mod external_agents;
#[path = "compiler_service/external_callable_runtime.rs"]
mod external_callable_runtime;
#[path = "compiler_service/external_methods.rs"]
mod external_methods;
#[path = "compiler_service/external_reflection.rs"]
mod external_reflection;
#[path = "compiler_service/external_returned_callables.rs"]
mod external_returned_callables;
#[path = "compiler_service/external_spec_methods.rs"]
mod external_spec_methods;
#[path = "compiler_service/flow_revisions.rs"]
mod flow_revisions;
#[path = "compiler_service/loaded_origins.rs"]
mod loaded_origins;
#[path = "compiler_service/no_host.rs"]
mod no_host;
#[path = "compiler_service/origin_metadata.rs"]
mod origin_metadata;
#[path = "compiler_service/resources.rs"]
mod resources;
#[path = "compiler_service/retained_metadata.rs"]
mod retained_metadata;
#[path = "compiler_service/revision_actions.rs"]
mod revision_actions;
#[path = "compiler_service/revision_callables.rs"]
mod revision_callables;
#[path = "compiler_service/revision_checkpoint.rs"]
mod revision_checkpoint;
#[path = "compiler_service/revision_checkpoint_argument_slots.rs"]
mod revision_checkpoint_argument_slots;
#[path = "compiler_service/revision_checkpoint_callable_types.rs"]
mod revision_checkpoint_callable_types;
#[path = "compiler_service/revision_checkpoint_callbacks.rs"]
mod revision_checkpoint_callbacks;
#[path = "compiler_service/revision_checkpoint_composition.rs"]
mod revision_checkpoint_composition;
#[path = "compiler_service/revision_checkpoint_literal_slots.rs"]
mod revision_checkpoint_literal_slots;
#[path = "compiler_service/revision_checkpoint_model_policy.rs"]
mod revision_checkpoint_model_policy;
#[path = "compiler_service/revision_checkpoint_operand_slots.rs"]
mod revision_checkpoint_operand_slots;
#[path = "compiler_service/revision_checkpoint_returned_bindings.rs"]
mod revision_checkpoint_returned_bindings;
#[path = "compiler_service/revision_checkpoint_source_resume.rs"]
mod revision_checkpoint_source_resume;
#[path = "compiler_service/revision_checkpoint_stack.rs"]
mod revision_checkpoint_stack;
#[path = "compiler_service/revision_checkpoint_suspensions.rs"]
mod revision_checkpoint_suspensions;
#[path = "compiler_service/revision_checkpoint_values.rs"]
mod revision_checkpoint_values;
#[path = "compiler_service/revision_frozen_behavior.rs"]
mod revision_frozen_behavior;
#[path = "compiler_service/revision_inference.rs"]
mod revision_inference;
#[path = "compiler_service/revision_native_actions.rs"]
mod revision_native_actions;
#[path = "compiler_service/revision_returned_anonymous.rs"]
mod revision_returned_anonymous;
#[path = "compiler_service/revision_stream_values.rs"]
mod revision_stream_values;
#[path = "compiler_service/revisions.rs"]
mod revisions;
#[path = "compiler_service/source.rs"]
mod source;
#[path = "compiler_service/source_restoration.rs"]
mod source_restoration;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

fn fixture() -> (Arc<NativeDefinitionStore>, DefinitionRef) {
    fixture_with_retention(true)
}

fn fixture_with_retention(retain: bool) -> (Arc<NativeDefinitionStore>, DefinitionRef) {
    fixture_from_source(
        "module app; agent Worker() -> i32 { return 1; } flow inspect() -> unit { let d = ^Worker; return; }",
        retain,
    )
}

fn fixture_from_source(source: &str, retain: bool) -> (Arc<NativeDefinitionStore>, DefinitionRef) {
    let mut input = ProjectInput::single_source(SourceInput::anonymous(source));
    input.options.entry_policy = EntryPolicy::Optional;
    input.options.reflection = ReflectionOptions {
        retain_bodies: if retain {
            vec!["app.Worker".into()]
        } else {
            vec![]
        },
        export_bodies: vec![],
    };
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let key = |bytes| {
        PackageKey::from_canonical_fingerprint(CodeFingerprint::of_bytes(
            "etas.test.compiler-service",
            bytes,
        ))
    };
    let origins = DefinitionOrigins {
        source_modules: DefinitionOrigins::declared_source_modules(&checked, key(b"owner"))
            .unwrap(),
        standard_package: key(b"std"),
        external_definitions: BTreeMap::new(),
    };
    let catalog =
        Arc::new(DefinitionCatalog::build(checked, &origins, Default::default()).unwrap());
    let syntax = Arc::new(NativeCodeSyntaxOperations::default());
    let definitions = Arc::new(NativeDefinitionStore::new(
        catalog,
        syntax,
        Default::default(),
    ));
    let definition = definitions.reflect_agent(expression).unwrap();
    (definitions, definition)
}

fn request(definition: DefinitionRef, operation: CompilerOperation) -> CompilerRequest {
    let mut authority = AuthorityContext::deny_all();
    authority
        .grants
        .push(HostActionGrant::allow("Code", operation.action()));
    CompilerRequest {
        id: HostRequestId(17),
        definition,
        operation,
        authority,
        trace: TraceContext::root(TraceId(11)),
        budget: Default::default(),
    }
}

async fn execute(
    service: &NativeCompilerService,
    request: CompilerRequest,
) -> Result<CompilerResponse, HostError> {
    let scope = ExecutionScope::new();
    let parent = scope
        .register(None, Some(request.id), request.trace.clone())
        .unwrap();
    parent.begin_dispatch().unwrap();
    let result = service.execute(request, parent.context()).await;
    parent
        .complete(
            if result.is_ok() {
                ExternalOutcome::Confirmed
            } else {
                ExternalOutcome::Unknown
            },
            vec![],
        )
        .unwrap();
    scope.finish_body(result.is_ok()).unwrap();
    scope.join().await.unwrap();
    result
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn native_compiler_is_composed_through_the_interpreter_host_port_without_authority_inheritance() {
    let (definitions, original) = fixture();
    let body = definitions
        .syntax_operations()
        .parse_body("return unresolved_name();")
        .unwrap();
    let definition = definitions.with_body(&original, body).unwrap();
    let host = CompilerHost::new(
        no_host::NoHost::default(),
        NativeCompilerService::new(definitions, Default::default()),
    );
    let available = host.availability();
    assert!(available.compiler);
    assert!(
        !available.model
            && !available.network
            && !available.filesystem
            && !available.host_authority
    );
    runtime().block_on(async {
        let scope = ExecutionScope::new();
        let request = request(definition.clone(), CompilerOperation::Check);
        let operation = scope
            .register(None, Some(request.id), request.trace.clone())
            .unwrap();
        operation.begin_dispatch().unwrap();
        let response = host
            .compiler(operation.context().clone(), request.clone())
            .await
            .unwrap();
        validate_compiler_response(&request, &response, Default::default()).unwrap();
        let Ok(CompilerResult::Checked(report)) = response.result else {
            panic!("checked report")
        };
        assert_eq!(report.status, CheckStatus::Rejected);
        assert!(!report.diagnostics.is_empty());
        operation
            .complete(ExternalOutcome::Confirmed, vec![])
            .unwrap();

        let mut denied = request.clone();
        denied.authority = AuthorityContext::deny_all();
        let operation = scope
            .register(None, Some(denied.id), denied.trace.clone())
            .unwrap();
        operation.begin_dispatch().unwrap();
        let error = host
            .compiler(operation.context().clone(), denied)
            .await
            .unwrap_err();
        assert_eq!(error.code, HostErrorCode::AuthorityDenied);
        operation
            .complete(ExternalOutcome::Confirmed, vec![])
            .unwrap();
        scope.finish_body(true).unwrap();
        scope.join().await.unwrap();
    });
    assert_eq!(
        host.inner().calls(),
        0,
        "the compiler must not execute model/tool/candidate services"
    );
}

#[test]
fn compiler_response_validation_checks_report_roots_not_only_the_descriptor_arena() {
    use etas_code::{
        descriptor::{EffectRowInfoRef, TraceInfoRef, TypeInfoRef},
        diagnostic::{DescriptorDetail, DiagnosticDetails, DiagnosticRecord, DiagnosticSeverity},
    };
    let (definitions, definition) = fixture();
    let service = NativeCompilerService::new(definitions, Default::default());
    runtime().block_on(async {
        let request = request(definition, CompilerOperation::Check);
        let response = execute(&service, request.clone()).await.unwrap();
        validate_compiler_response(&request, &response, Default::default()).unwrap();
        let Ok(CompilerResult::Checked(report)) = response.result else {
            panic!("report")
        };
        for mutation in 0..4 {
            let mut invalid = (*report).clone();
            match mutation {
                0 => invalid.effects.requested = Some(EffectRowInfoRef(u32::MAX)),
                1 => invalid.trace = Some(TraceInfoRef(u32::MAX)),
                2 => invalid.effects.escaping = None,
                3 => invalid.diagnostics.push(DiagnosticRecord {
                    code: "test::InvalidDetail".into(),
                    severity: DiagnosticSeverity::Warning,
                    message: "invalid type root".into(),
                    location: None,
                    details: Some(Arc::new(DiagnosticDetails {
                        expected: Some(DescriptorDetail::Type {
                            graph: invalid.descriptors.clone(),
                            ty: TypeInfoRef(u32::MAX),
                        }),
                        ..Default::default()
                    })),
                }),
                _ => unreachable!(),
            }
            let response = CompilerResponse {
                id: request.id,
                result: Ok(CompilerResult::Checked(Arc::new(invalid))),
            };
            let error =
                validate_compiler_response(&request, &response, Default::default()).unwrap_err();
            assert_eq!(
                error.code,
                HostErrorCode::InvalidResponse,
                "mutation {mutation}"
            );
        }
        let diagnostic = DiagnosticRecord {
            code: "test::InvalidServiceDetail".into(),
            severity: DiagnosticSeverity::Error,
            message: "invalid trace root".into(),
            location: None,
            details: Some(Arc::new(DiagnosticDetails {
                actual: Some(DescriptorDetail::Trace {
                    graph: report.descriptors.clone(),
                    trace: TraceInfoRef(u32::MAX),
                }),
                ..Default::default()
            })),
        };
        let response = CompilerResponse {
            id: request.id,
            result: Err(CompilerFailure::Service {
                diagnostics: vec![diagnostic],
            }),
        };
        assert_eq!(
            validate_compiler_response(&request, &response, Default::default())
                .unwrap_err()
                .code,
            HostErrorCode::InvalidResponse
        );
    });
}

#[test]
fn native_host_check_compile_and_engine_publication_use_the_same_frozen_revision() {
    let (definitions, original) = fixture();
    let body = definitions
        .syntax_operations()
        .parse_body("return 2;")
        .unwrap();
    let edited = definitions.with_body(&original, body).unwrap();
    let standard = definitions
        .resolve(&original)
        .unwrap()
        .environment()
        .unwrap()
        .checked()
        .std_registry
        .clone();
    let service = NativeCompilerService::new(definitions, Default::default());
    runtime().block_on(async {
        let check = request(edited.clone(), CompilerOperation::Check);
        let before = check.budget.snapshot().unwrap();
        let report = execute(&service, check.clone()).await.unwrap();
        validate_compiler_response(&check, &report, Default::default()).unwrap();
        let Ok(CompilerResult::Checked(report)) = report.result else {
            panic!("report")
        };
        assert_eq!(report.status, CheckStatus::Accepted);
        assert_eq!(&report.revision, edited.revision());
        assert_eq!(
            before,
            check.budget.snapshot().unwrap(),
            "compilation must not fabricate model token usage"
        );
        let compile = request(edited.clone(), CompilerOperation::Compile);
        let Ok(CompilerResult::Compiled(artifact)) =
            execute(&service, compile.clone()).await.unwrap().result
        else {
            panic!("artifact")
        };
        let prepared = service
            .publish(&artifact, &mut InterpreterCandidatePublisher)
            .unwrap();
        assert_eq!(prepared.revision(), edited.revision());
        assert_eq!(prepared.identity(), original.identity());
        assert_eq!(
            prepared.snapshot_fingerprint(),
            artifact.snapshot_fingerprint()
        );
        assert!(Arc::ptr_eq(&prepared.program().std_registry, &standard));
        assert_eq!(prepared.plan().entry.item, prepared.entry().item);
        let host = no_host::NoHost::default();
        let result = prepared
            .create_run(vec![], &host, Default::default())
            .execute()
            .await
            .unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(host.calls(), 0);
        assert_eq!(
            result.value(),
            Some(&etas_interpreter::api::InterpValue::i32(2))
        );
    });
}

#[test]
fn invalid_source_is_a_check_report_and_a_compile_rejection_not_a_service_failure() {
    let (definitions, original) = fixture();
    let edited = definitions
        .with_body(
            &original,
            definitions
                .syntax_operations()
                .parse_body("return \"wrong\";")
                .unwrap(),
        )
        .unwrap();
    let service = NativeCompilerService::new(definitions, Default::default());
    runtime().block_on(async {
        let Ok(CompilerResult::Checked(report)) =
            execute(&service, request(edited.clone(), CompilerOperation::Check))
                .await
                .unwrap()
                .result
        else {
            panic!("normal report")
        };
        assert_eq!(report.status, CheckStatus::Rejected);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "type::TypeMismatch")
        );
        let Err(CompilerFailure::Rejected(report)) =
            execute(&service, request(edited, CompilerOperation::Compile))
                .await
                .unwrap()
                .result
        else {
            panic!("compile rejection")
        };
        assert_eq!(report.status, CheckStatus::Rejected);
    });
}

#[test]
fn compiled_leases_pin_definition_provenance_and_preserve_original_body_visibility() {
    runtime().block_on(async {
        for retain in [true, false] {
            let (definitions, original) = fixture_with_retention(retain);
            let frozen = definitions
                .resolve(&original)
                .unwrap()
                .environment()
                .unwrap()
                .clone();
            let edited = definitions
                .with_body(
                    &original,
                    definitions
                        .syntax_operations()
                        .parse_body("return 2;")
                        .unwrap(),
                )
                .unwrap();
            let service = NativeCompilerService::new(definitions.clone(), Default::default());
            let expected = edited.revision().clone();
            let Ok(CompilerResult::Compiled(artifact)) =
                execute(&service, request(edited, CompilerOperation::Compile))
                    .await
                    .unwrap()
                    .result
            else {
                panic!("artifact")
            };
            drop(original);
            let current = service.reflect_compiled(&artifact).unwrap();
            let view = definitions.resolve(&current).unwrap();
            assert_eq!(view.revision(), &expected);
            assert_eq!(view.body().is_some(), retain);
            assert!(view.current_analysis().is_ok());
            assert!(Arc::ptr_eq(view.environment().unwrap(), &frozen));
        }
    });
}

#[test]
fn denied_foreign_and_wrong_operation_context_requests_fail_closed() {
    let (definitions, original) = fixture();
    let (foreign, other) = fixture();
    let service = NativeCompilerService::new(definitions, Default::default());
    runtime().block_on(async {
        let mut denied = request(original.clone(), CompilerOperation::Check);
        denied.authority = AuthorityContext::deny_all();
        assert_eq!(
            execute(&service, denied).await.unwrap_err().code,
            HostErrorCode::AuthorityDenied
        );
        assert_eq!(
            execute(&service, request(other, CompilerOperation::Check))
                .await
                .unwrap_err()
                .code,
            HostErrorCode::InvalidRequest
        );
        let scope = ExecutionScope::new();
        let parent = scope
            .register(
                None,
                Some(HostRequestId(99)),
                TraceContext::root(TraceId(11)),
            )
            .unwrap();
        parent.begin_dispatch().unwrap();
        assert_eq!(
            service
                .execute(
                    request(original, CompilerOperation::Check),
                    parent.context()
                )
                .await
                .unwrap_err()
                .code,
            HostErrorCode::InvalidRequest
        );
        parent.complete(ExternalOutcome::Confirmed, vec![]).unwrap();
        scope.finish_body(false).unwrap();
        scope.join().await.unwrap();
    });
    drop(foreign);
}

#[test]
fn checked_reference_cannot_be_forged_or_published_by_another_service() {
    #[derive(Debug, etas_core::RetainedSize)]
    struct Forged {
        identity: DefinitionKey,
        revision: RevisionInfo,
        snapshot: CodeFingerprint,
    }
    impl CheckedCodeLease for Forged {
        fn identity(&self) -> &DefinitionKey {
            &self.identity
        }
        fn revision(&self) -> &RevisionInfo {
            &self.revision
        }
        fn snapshot_fingerprint(&self) -> CodeFingerprint {
            self.snapshot
        }
    }
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let other = NativeCompilerService::new(definitions, Default::default());
    let artifact = runtime().block_on(async {
        let Ok(CompilerResult::Compiled(artifact)) =
            execute(&service, request(original, CompilerOperation::Compile))
                .await
                .unwrap()
                .result
        else {
            panic!("artifact")
        };
        artifact
    });
    let forged = CheckedCodeRef::from_backend(Arc::new(Forged {
        identity: artifact.identity().clone(),
        revision: artifact.revision().clone(),
        snapshot: artifact.snapshot_fingerprint(),
    }));
    for (service, reference) in [(&service, &forged), (&other, &artifact)] {
        assert!(matches!(
            etas_frontend::code::service::CheckedCandidateResolver::resolve_checked_candidate(
                service, reference
            ),
            Err(etas_frontend::code::service::CandidateResolutionError::UnownedReference)
        ));
        assert!(matches!(
            service.publish(reference, &mut InterpreterCandidatePublisher),
            Err(CandidatePublicationError::Reference(HostError {
                code: HostErrorCode::InvalidRequest,
                ..
            }))
        ));
    }
    assert!(
        service
            .publish(&artifact, &mut InterpreterCandidatePublisher)
            .is_ok()
    );
}

#[test]
fn output_retention_is_bounded_and_dead_leases_release_admission() {
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(
        definitions,
        NativeCompilerConfig {
            max_retained_candidates: 1,
            ..Default::default()
        },
    );
    runtime().block_on(async {
        let first = execute(
            &service,
            request(original.clone(), CompilerOperation::Compile),
        )
        .await
        .unwrap();
        assert_eq!(
            execute(
                &service,
                request(original.clone(), CompilerOperation::Compile)
            )
            .await
            .unwrap_err()
            .code,
            HostErrorCode::BudgetExceeded
        );
        drop(first);
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
fn descriptor_and_retained_byte_exhaustion_are_host_limits_not_source_rejections() {
    let (definitions, original) = fixture();
    let no_bytes = NativeCompilerService::new(
        definitions.clone(),
        NativeCompilerConfig {
            max_retained_candidate_bytes: 0,
            ..Default::default()
        },
    );
    let no_descriptors = NativeCompilerService::new(
        definitions,
        NativeCompilerConfig {
            descriptors: etas_code::descriptor::DescriptorLimits {
                max_nodes: 0,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    runtime().block_on(async {
        for (service, operation) in [
            (&no_bytes, CompilerOperation::Compile),
            (&no_descriptors, CompilerOperation::Check),
            (&no_descriptors, CompilerOperation::Compile),
        ] {
            assert_eq!(
                execute(service, request(original.clone(), operation))
                    .await
                    .unwrap_err()
                    .code,
                HostErrorCode::BudgetExceeded
            );
        }
    });
}

#[test]
fn every_descriptor_graph_budget_is_enforced_by_both_check_and_compile() {
    use etas_code::descriptor::DescriptorLimits;
    let (definitions, original) = fixture();
    runtime().block_on(async {
        for descriptors in [
            DescriptorLimits {
                max_nodes: 0,
                ..Default::default()
            },
            DescriptorLimits {
                max_edges: 0,
                ..Default::default()
            },
            DescriptorLimits {
                max_expansion_depth: 0,
                ..Default::default()
            },
            DescriptorLimits {
                max_text_bytes: 0,
                ..Default::default()
            },
        ] {
            let service = NativeCompilerService::new(
                definitions.clone(),
                NativeCompilerConfig {
                    descriptors,
                    ..Default::default()
                },
            );
            for operation in [CompilerOperation::Check, CompilerOperation::Compile] {
                let error = execute(&service, request(original.clone(), operation))
                    .await
                    .unwrap_err();
                assert_eq!(
                    error.code,
                    HostErrorCode::BudgetExceeded,
                    "{descriptors:?}: {error:?}"
                );
            }
        }
    });
}

#[test]
fn annotation_projection_exhaustion_remains_a_resource_error_for_check_and_compile() {
    let source = format!(
        "module app; @custom(\"{}\") agent Worker() -> i32 {{ return 1; }} flow inspect() -> unit {{ let d = ^Worker; return; }}",
        "x".repeat(16_384)
    );
    let (definitions, original) = fixture_from_source(&source, true);
    let service = NativeCompilerService::new(
        definitions,
        NativeCompilerConfig {
            descriptors: etas_code::descriptor::DescriptorLimits {
                max_text_bytes: 1024,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    runtime().block_on(async {
        for operation in [CompilerOperation::Check, CompilerOperation::Compile] {
            let error = execute(&service, request(original.clone(), operation))
                .await
                .unwrap_err();
            assert_eq!(error.code, HostErrorCode::BudgetExceeded, "{error:?}");
        }
    });
}

#[test]
fn response_identity_and_result_kind_are_bound_to_the_request() {
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    runtime().block_on(async {
        let check = request(original.clone(), CompilerOperation::Check);
        let response = execute(&service, check.clone()).await.unwrap();
        let mut wrong = response.clone();
        wrong.id = HostRequestId(99);
        assert_eq!(
            validate_compiler_response(&check, &wrong, Default::default())
                .unwrap_err()
                .code,
            HostErrorCode::InvalidResponse
        );
        let mut compile = check.clone();
        compile.operation = CompilerOperation::Compile;
        assert!(validate_compiler_response(&compile, &response, Default::default()).is_err());
        let edited = definitions
            .with_body(
                &original,
                definitions
                    .syntax_operations()
                    .parse_body("return 3;")
                    .unwrap(),
            )
            .unwrap();
        let edited_request = request(edited, CompilerOperation::Check);
        assert!(
            validate_compiler_response(&edited_request, &response, Default::default()).is_err()
        );
    });
}

#[test]
fn compiler_trace_contains_snapshot_identity_and_never_generated_source() {
    let (definitions, original) = fixture();
    let edited = definitions
        .with_body(
            &original,
            definitions
                .syntax_operations()
                .parse_body("let private_marker = \"secret source\"; return 2;")
                .unwrap(),
        )
        .unwrap();
    let payload = request(edited, CompilerOperation::Compile).trace_payload();
    assert_eq!(payload.qualified_action, "Code.compile");
    assert_eq!(payload.fields.len(), 4);
    assert!(!format!("{payload:?}").contains("secret source"));
}

struct AdvancingClock {
    base: Instant,
    calls: AtomicUsize,
}
impl MonotonicClock for AdvancingClock {
    fn now(&self) -> Instant {
        self.base
            + if self.calls.fetch_add(1, Ordering::SeqCst) >= 8 {
                Duration::from_secs(10)
            } else {
                Duration::ZERO
            }
    }
}

#[test]
fn parent_wall_time_budget_reaches_the_actual_frontend_pass_boundaries() {
    let (definitions, original) = fixture();
    let service = NativeCompilerService::new(definitions, Default::default());
    let clock = Arc::new(AdvancingClock {
        base: Instant::now(),
        calls: AtomicUsize::new(0),
    });
    let mut request = request(original, CompilerOperation::Compile);
    request.budget = ExecutionBudget::start_with_clock(
        Budget {
            time: Some(TimeBudget { max_millis: 1000 }),
            tokens: Some(TokenBudget { max_tokens: 0 }),
            cost: None,
        },
        clock.clone(),
    );
    let error = runtime().block_on(execute(&service, request)).unwrap_err();
    assert_eq!(error.code, HostErrorCode::BudgetExceeded);
    assert!(clock.calls.load(Ordering::SeqCst) >= 9);
}

#[test]
fn cancelled_queued_compilation_remains_owned_until_the_worker_proves_non_dispatch() {
    use std::future::Future;
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
        let request = request(original, CompilerOperation::Compile);
        let scope = ExecutionScope::new();
        let parent = scope
            .register(None, Some(request.id), request.trace.clone())
            .unwrap();
        parent.begin_dispatch().unwrap();
        let context = parent.context().clone();
        let job = service.execute(request, &context);
        tokio::pin!(job);
        std::future::poll_fn(|cx| {
            assert!(job.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        scope
            .cancel_source()
            .stop(CancellationReason::Requested)
            .unwrap();
        release.send(()).unwrap();
        busy.await.unwrap();
        assert_eq!(job.await.unwrap_err().code, HostErrorCode::Cancelled);
        parent.complete(ExternalOutcome::Unknown, vec![]).unwrap();
        scope.finish_body(false).unwrap();
        let report = scope.join().await.unwrap();
        assert_eq!(report.operations().len(), 2);
        assert_eq!(
            report.operations()[1].outcome(),
            Some(&ExternalOutcome::NotDispatched)
        );
    });
}
