use super::revision_checkpoint_values::save_captured;
use super::*;
use etas_interpreter::{
    Interpreter,
    api::{InterpValue, RunOptions, RunOutcome, code::PreparedCodeDefinitions, codec},
};

const PROGRAM: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    checkpoint("selected");
    let other = compile(^Worker);
    return selected() + other();
}
"#;

fn resume_source(
    artifact: serde_json::Value,
    allow: bool,
    cancel: bool,
    fuel: bool,
) -> etas_interpreter::api::RunResult {
    resume_source_for(PROGRAM, artifact, allow, cancel, fuel)
}

pub(super) fn resume_source_for(
    program: &str,
    artifact: serde_json::Value,
    allow: bool,
    cancel: bool,
    fuel: bool,
) -> etas_interpreter::api::RunResult {
    let (checked, definitions) = source::prepared(program);
    let bytes = codec::checkpoint_file_to_bytes(artifact, Default::default()).unwrap();
    let document = codec::checkpoint_file_from_bytes(&bytes, Default::default()).unwrap();
    let checkpoint = codec::SourceCheckpoint::from_document(document, &checked).unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(no_host::NoHost::with_checkpoint(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    if allow {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow("Code", "compile"));
    }
    if fuel {
        options.execution_limits.max_steps = std::num::NonZeroU64::new(1);
    }
    let invocation = Interpreter.create_source_resume(&checked, &checkpoint, &host, options);
    if cancel {
        invocation
            .control()
            .stop(CancellationReason::Requested)
            .unwrap();
    }
    let result = runtime().block_on(invocation.execute()).unwrap();
    assert_eq!(
        host.inner().calls(),
        0,
        "restoration must not replay the source checkpoint action"
    );
    result
}

fn compiler_ids(result: &etas_interpreter::api::RunResult) -> Vec<u32> {
    result
        .events
        .iter()
        .filter_map(|event| match event {
            etas_interpreter::api::WorkflowEvent::HostTrace(
                etas_host::TraceEvent::HostRequestStarted {
                    id,
                    kind: etas_host::HostRequestKind::Compiler,
                    ..
                },
            ) => Some(id.0),
            _ => None,
        })
        .collect()
}

#[test]
fn source_resume_recompiles_under_the_same_run_state_and_keeps_request_ids() {
    let (_, artifact) = save_captured(PROGRAM, &[None], 38);
    let next = artifact["checkpoint"]["trace"]["next_host_request"]
        .as_u64()
        .unwrap() as u32;
    let result = resume_source(artifact, true, false, false);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(38)));
    assert_eq!(compiler_ids(&result), vec![next, next + 1]);
}

#[test]
fn source_resume_requires_current_compile_authority() {
    let (_, artifact) = save_captured(PROGRAM, &[None], 38);
    let result = resume_source(artifact, false, false, false);
    assert!(result.value().is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("not authorized")),
        "{:?}",
        result.diagnostics
    );
    assert!(compiler_ids(&result).is_empty());
}

#[test]
fn source_resume_rejects_a_different_freshly_checked_fingerprint() {
    let (_, mut artifact) = save_captured(PROGRAM, &[None], 38);
    artifact["code_sources"][0]["snapshot_fingerprint"] =
        serde_json::to_value(CodeFingerprint::of_bytes("etas.test", b"tampered")).unwrap();
    let result = resume_source(artifact, true, false, false);
    assert!(result.value().is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("checked snapshot differs")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(compiler_ids(&result).len(), 1);
}

#[test]
fn source_resume_does_not_reset_the_saved_execution_fuel() {
    let (_, artifact) = save_captured(PROGRAM, &[None], 38);
    let result = resume_source(artifact, true, false, true);
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("maximum interpreter execution steps")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(compiler_ids(&result).len(), 1);
}

#[test]
fn source_resume_observes_cancellation_before_starting_compilation() {
    let (_, artifact) = save_captured(PROGRAM, &[None], 38);
    let result = resume_source(artifact, true, true, false);
    assert!(matches!(result.outcome, RunOutcome::Cancelled(_)));
    assert!(compiler_ids(&result).is_empty());
}

#[test]
fn source_resume_restores_an_active_native_program_without_replaying_its_checkpoint() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { checkpoint("inside candidate"); return 19; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected() + 1;
}
"#;
    let (_, artifact) = save_captured(program, &[None], 20);
    let result = resume_source_for(program, artifact, true, false, false);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
    assert_eq!(compiler_ids(&result).len(), 1);
}

#[test]
fn source_resume_restores_a_suspended_nominal_action_through_the_current_handler() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
agent Worker(value: UserId) -> UserId ![Query.choose] { return perform Query.choose(value); }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("restore");
    let result = handle { selected(id) } with {
        Query.choose(value) => { checkpoint("handler"); resume value; }
    };
    if result == id { return 20; }
    return 0;
}
"#;
    let (_, artifact) = save_captured(program, &[None], 20);
    let result = resume_source_for(program, artifact, true, false, false);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
    assert_eq!(compiler_ids(&result).len(), 1);
}
