use super::*;
use etas_interpreter::{
    Interpreter,
    api::code::PreparedCodeDefinitions,
    api::{EntryPoint, InterpValue, RunOptions},
};

pub(super) fn prepared(
    source: &str,
) -> (etas_frontend::CheckedProgram, Arc<NativeDefinitionStore>) {
    prepared_with_bodies(source, &["app.Worker"])
}

pub(super) fn prepared_with_bodies(
    source: &str,
    bodies: &[&str],
) -> (etas_frontend::CheckedProgram, Arc<NativeDefinitionStore>) {
    let mut input = ProjectInput::single_source(SourceInput::anonymous(source));
    input.options.reflection.retain_bodies = bodies.iter().map(|body| (*body).to_owned()).collect();
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let key = |name: &[u8]| {
        PackageKey::from_canonical_fingerprint(CodeFingerprint::of_bytes(
            "etas.test.source-code-service",
            name,
        ))
    };
    let origins = DefinitionOrigins {
        source_modules: DefinitionOrigins::declared_source_modules(&checked, key(b"app")).unwrap(),
        standard_package: key(b"std"),
        external_definitions: BTreeMap::new(),
    };
    let catalog =
        Arc::new(DefinitionCatalog::build(checked.clone(), &origins, Default::default()).unwrap());
    let definitions = Arc::new(NativeDefinitionStore::new(
        catalog,
        Arc::new(NativeCodeSyntaxOperations::default()),
        Default::default(),
    ));
    (checked, definitions)
}

#[test]
fn editing_agent_checks_cancellation_during_method_snapshot_rebuild() {
    struct CancelAt {
        calls: AtomicUsize,
        at: usize,
        kind: etas_utils::PipelineInterruptionKind,
    }
    impl etas_utils::PipelineExecutionControl for CancelAt {
        fn checkpoint(&self) -> Result<(), etas_utils::PipelineInterruption> {
            if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == self.at {
                return Err(etas_utils::PipelineInterruption {
                    kind: self.kind,
                    reason: "cancel during edit".into(),
                });
            }
            Ok(())
        }
    }
    let (checked, definitions) = prepared_with_bodies(
        r#"
module app;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow one() -> i32 ![] { return 1; }
    flow two() -> i32 ![] { return 2; }
}
flow main() -> unit { let definition = ^Worker; return; }
"#,
        &["app.Worker"],
    );
    let expression = checked
        .types
        .reflections
        .iter()
        .find_map(|(expr, fact)| {
            matches!(
                fact.target,
                etas_types::CheckedReflectionTarget::Agent { .. }
            )
            .then_some(*expr)
        })
        .unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let body = definitions
        .syntax_operations()
        .parse_body("return 2;")
        .unwrap();
    let retained = definitions.retained_bytes().unwrap();
    for kind in [
        etas_utils::PipelineInterruptionKind::Cancelled,
        etas_utils::PipelineInterruptionKind::ResourceLimit,
    ] {
        let control = CancelAt {
            calls: AtomicUsize::new(0),
            at: 3,
            kind,
        };
        let result = definitions.with_body_controlled(&original, body.clone(), &control);
        assert!(
            matches!(result, Err(etas_code::definition::DefinitionOperationError::Syntax(
            etas_code::syntax::SyntaxOperationError::Interrupted(interruption)
        )) if interruption.kind == kind)
        );
        assert_eq!(control.calls.load(Ordering::SeqCst), 3);
        assert_eq!(definitions.retained_bytes().unwrap(), retained);
    }
    assert!(definitions.with_body(&original, body).is_ok());
}

fn execute_source(body: &str, allowed: bool) -> etas_interpreter::api::RunResult {
    execute_source_with(body, allowed, |definitions| {
        NativeCompilerService::new(definitions, Default::default())
    })
}

fn execute_source_with<C: CompilerClient + Sync>(
    body: &str,
    allowed: bool,
    compiler: impl FnOnce(Arc<NativeDefinitionStore>) -> C,
) -> etas_interpreter::api::RunResult {
    let (checked, definitions) = prepared(&format!(
        r#"
module app;
import std.compiler.check;
import std.compiler.parse_body;
import std.compiler.CompileError;
import std.io.println;
agent Worker() -> i32 {{ return 1; }}
{body}
"#
    ));
    let host = CompilerHost::new(no_host::NoHost::default(), compiler(definitions.clone()));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    if allowed {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow("Code", "check"));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert_eq!(
        host.inner().calls(),
        0,
        "checking must not execute candidate model/tool/IO actions"
    );
    result
}

#[test]
fn source_code_check_uses_the_managed_compiler_and_returns_a_structured_accepted_report() {
    for expression in ["check(^Worker)", "perform Code.check(^Worker)"] {
        let result = execute_source(
            &format!(
                r#"
flow main() -> bool ![Error<CompileError>] {{
    let report = {expression};
    return report.status == "accepted" && match report.effects.escaping {{ Some(_) => true, None => false, }};
}}
"#
            ),
            true,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
        assert_eq!(
            result
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    etas_interpreter::api::WorkflowEvent::HostTrace(
                        etas_host::TraceEvent::HostRequestStarted {
                            kind: etas_host::HostRequestKind::Compiler,
                            ..
                        }
                    )
                ))
                .count(),
            1
        );
    }
}

#[test]
fn source_code_check_returns_invalid_source_as_a_rejected_report_not_a_service_error() {
    let result = execute_source(
        r#"
flow main() -> bool ![Error<CompileError>] {
    match parse_body("return missing_name;") {
        Ok(body) => {
            let report = check((^Worker).with_body(body));
            return report.status == "rejected" && report.diagnostics.len() > 0;
        },
        Err(_) => { return false; },
    }
}

"#,
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn source_compile_returns_an_agent_executed_inline_by_the_current_machine() {
    let (checked, definitions) = prepared(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] {
    match parse_body("return 2;") {
        Ok(body) => {
            let revised = compile((^Worker).with_body(body));
            return revised();
        },
        Err(_) => { return -1; },
    }
}
"#,
    );
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(no_host::NoHost::default(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    options
        .host_context
        .authority
        .grants
        .push(HostActionGrant::allow("Code", "compile"));
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(2)));
    assert_eq!(host.inner().calls(), 0);
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                etas_interpreter::api::WorkflowEvent::HostTrace(
                    etas_host::TraceEvent::HostRequestStarted {
                        kind: etas_host::HostRequestKind::Compiler,
                        ..
                    }
                )
            ))
            .count(),
        1
    );
}
#[test]
fn source_code_check_denial_is_not_captured_as_a_compile_error() {
    let result = execute_source(
        r#"
flow main() -> bool {
    match check(^Worker)? { Ok(_) => { return true; }, Err(_) => { return true; }, }
}
"#,
        false,
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains(HostErrorCode::AuthorityDenied.as_str())),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn source_code_check_keeps_expected_actual_constraint_and_analysis_path() {
    let result = execute_source(
        r#"
flow main() -> bool ![Error<CompileError>] {
    match parse_body("println(\"new action\")?; return 2;") {
        Ok(body) => {
            let report = check((^Worker).with_body(body));
            if report.status != "rejected" { return false; }
            for diagnostic in report.diagnostics limit Iterations(64) {
                if diagnostic.code == "code::AdditionalRequestedActions" {
                    match diagnostic.details {
                        Some(details) => {
                            match (details.expected, details.actual, details.constraint, diagnostic.location) {
                                (Some(expected), Some(actual), Some(constraint), Some(location)) => {
                                    return expected != actual && constraint.kind == "requested_actions"
                                        && details.analysis_path.len() > 0 && location.byte_end > location.byte_start;
                                },
                                _ => { return false; },
                            }
                        },
                        None => { return false; },
                    }
                }
            }
            return false;
        },
        Err(_) => { return false; },
    }
}
"#,
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

struct UnavailableCompiler;
impl CompilerClient for UnavailableCompiler {
    type ExecuteFuture<'a> = std::future::Ready<Result<CompilerResponse, HostError>>;
    fn execute<'a>(
        &'a self,
        request: CompilerRequest,
        _: &'a etas_host::execution::OperationContext,
    ) -> Self::ExecuteFuture<'a> {
        std::future::ready(Ok(CompilerResponse {
            id: request.id,
            result: Err(CompilerFailure::Service {
                diagnostics: vec![etas_code::diagnostic::DiagnosticRecord {
                    code: "code::ServiceUnavailable".into(),
                    severity: etas_code::diagnostic::DiagnosticSeverity::Error,
                    message: "compiler service unavailable".into(),
                    location: None,
                    details: None,
                }],
            }),
        }))
    }
}

#[test]
fn source_code_check_service_failure_is_captured_by_its_checked_error_contract() {
    for expression in ["check(^Worker)", "perform Code.check(^Worker)"] {
        let result = execute_source_with(
            &format!(
                r#"
flow main() -> bool {{
    match ({expression})? {{
        Ok(_) => {{ return false; }},
        Err(error) => {{ return error.kind == "service" && error.diagnostics.len() == 1; }},
    }}
}}
"#
            ),
            true,
            |_| UnavailableCompiler,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    }
}
