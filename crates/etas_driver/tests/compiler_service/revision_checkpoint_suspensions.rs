use super::revision_checkpoint_values::{restore_captured, resume_captured, save_captured};
use etas_interpreter::api::InterpValue;

const SUSPENDED: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
agent Worker(value: UserId) -> UserId ![Query.choose] {
    return perform Query.choose(value);
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("suspended");
    let result = handle { revised(id) } with {
        Query.choose(value) => { checkpoint("handler"); resume value; }
    };
    if result == id { return 20; }
    return 0;
}
"#;

#[test]
fn suspended_native_action_checkpoint_recompiles_and_resumes_the_exact_nominal_continuation() {
    let (saved, artifact) = save_captured(SUSPENDED, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(SUSPENDED, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(
        result.checkpoints.is_empty(),
        "resume repeated the handler checkpoint"
    );
}

#[test]
fn handler_finish_after_checkpoint_discards_the_suspended_native_suffix() {
    let source = SUSPENDED.replace("checkpoint(\"handler\"); resume value;", "checkpoint(\"handler\"); finish value;")
        .replace("return perform Query.choose(value);", "let selected = perform Query.choose(value); return abort(\"finished native suffix ran\");");
    let (saved, artifact) = save_captured(&source, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(&source, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

const CALLBACK: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
flow callback(value: UserId) -> UserId ![Query.choose] { return perform Query.choose(value); }
agent Worker(callback: (UserId) -> UserId ![Query.choose], value: UserId) -> UserId ![Query.choose] {
    return callback(value);
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("nested suspension");
    let result = handle { revised(callback, id) } with {
        Query.choose(value) => { checkpoint("nested"); resume value; }
    };
    if result == id { return 20; }
    return 0;
}
"#;

#[test]
fn source_callback_action_preserves_nested_single_shot_suspensions_across_checkpoint() {
    let (saved, artifact) = save_captured(CALLBACK, &[None], 20);
    assert_eq!(
        artifact["checkpoint"]["machine"]["suspended_programs"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let (checked, definitions, revisions) = restore_captured(CALLBACK, &saved);
    assert_eq!(
        artifact["checkpoint"]["machine"]["suspended_programs"][0]["action"]["kind"],
        "forwarded"
    );
    let mut wrong_action_owner = artifact.clone();
    wrong_action_owner["checkpoint"]["machine"]["suspended_programs"][0]["action"]["id"] =
        serde_json::json!(1);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &wrong_action_owner,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("not owned by its pending suspension"),
        "{error}"
    );
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}

#[test]
fn checkpointed_console_action_resumes_without_executing_a_default_host_boundary() {
    let source = r#"
module app;
import std.runtime.checkpoint;
import std.io.println;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { println("intercepted")?; return 20; }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Console.stdout_write(_) => { checkpoint("console"); resume; }
    };
}
"#;
    let (saved, artifact) = save_captured(source, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(source, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

#[test]
fn malformed_suspended_graphs_and_action_origins_are_rejected_before_restore() {
    let (saved, artifact) = save_captured(SUSPENDED, &[None], 20);
    let (checked, _, revisions) = restore_captured(SUSPENDED, &saved);
    let decode = |value: &serde_json::Value| {
        etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            value,
            &checked,
            Some(&revisions),
        )
    };
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"] = serde_json::json!([]);
    assert!(
        decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("missing suspended program")
    );
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"]
        .as_array_mut()
        .unwrap()
        .push(artifact["checkpoint"]["machine"]["suspended_programs"][0].clone());
    assert!(
        decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("unreachable suspended program")
    );
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"][0]["continuation"] =
        serde_json::json!({"kind":"program_resume", "id":1});
    assert!(
        decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("cyclic suspended program")
    );
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"][0]["action"]["origin"]["expr"] =
        serde_json::json!(u32::MAX);
    assert!(
        decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("not a checked perform")
    );
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"][0]["boundary"]["call_site"]["origin"]["expr"] =
        serde_json::json!(u32::MAX);
    assert!(decode(&bad).unwrap_err().to_string().contains("call site"));
    let mut bad = artifact;
    bad["checkpoint"]["machine"]["suspended_programs"][0]["action"]["origin"]["bindings"] =
        serde_json::json!([["T", 0], ["T", 0]]);
    assert!(
        decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("duplicate type bindings")
    );
}

#[test]
fn detached_native_handler_scopes_are_restored_and_rebased_before_resumption() {
    let source = SUSPENDED
        .replace("effect Query {", "effect Other { action inspect() -> unit; } effect Query {")
        .replace("return perform Query.choose(value);", "return handle { perform Query.choose(value) } with { Other.inspect() => { resume; } };");
    let (saved, artifact) = save_captured(&source, &[None], 20);
    assert_eq!(
        artifact["checkpoint"]["machine"]["suspended_programs"][0]["handlers"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let (checked, definitions, revisions) = restore_captured(&source, &saved);
    let result = resume_captured(&checked, definitions.clone(), revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    let mut bad = artifact.clone();
    bad["checkpoint"]["machine"]["suspended_programs"][0]["handlers"][0]["id"] =
        artifact["checkpoint"]["handlers"][0]["id"].clone();
    assert!(
        etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &bad,
            &checked,
            Some(&revisions)
        )
        .is_err()
    );
}

#[test]
fn inference_action_checkpoint_preserves_the_checked_output_schema_without_a_model_host() {
    let source = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> UserId { return perform infer<UserId>(Prompt.new()); }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("inference suspension");
    let result = handle { revised() } with {
        Agentic.infer<_, UserId>(_, _) => { checkpoint("inference"); resume id; }
    };
    if result == id { return 20; }
    return 0;
}

"#;
    let (saved, artifact) = save_captured(source, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(source, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

#[test]
fn resumed_suspension_can_checkpoint_again_with_the_original_root_and_without_consumed_nodes() {
    let source = SUSPENDED.replace(
        "return perform Query.choose(value);",
        "let selected = perform Query.choose(value); checkpoint(\"native\"); return selected;",
    );
    let (saved, artifact) =
        super::revision_checkpoint_values::save_checkpoint_at(&source, &[None], 20, 2, 0);
    let (checked, definitions, revisions) = restore_captured(&source, &saved);
    let result = resume_captured(&checked, definitions.clone(), revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert_eq!(result.checkpoints.len(), 1);
    let next =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    assert!(
        next["checkpoint"]["machine"]["suspended_programs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let resumed = resume_captured(&checked, definitions, revisions, &next);
    assert!(resumed.diagnostics.is_empty(), "{:?}", resumed.diagnostics);
    assert_eq!(resumed.value(), Some(&InterpValue::i32(20)));
    assert!(resumed.checkpoints.is_empty());
}

#[test]
fn suspended_generic_action_restores_its_checked_selector_and_payload_substitution() {
    let source = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose<T>(value: T) -> T; }
flow relay<T>(value: T) -> T ![Query.choose<T>] { return perform Query.choose<T>(value); }
agent Worker(value: UserId) -> UserId ![Query.choose<UserId>] { return relay<UserId>(value); }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("generic suspension");
    let result = handle { revised(id) } with {
        Query.choose<UserId>(value) => { checkpoint("generic"); resume value; }
    };
    if result == id { return 20; }
    return 0;
}
"#;
    let (saved, artifact) = save_captured(source, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(source, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

#[test]
fn direct_resume_counts_detached_calls_against_saved_and_current_limits() {
    use etas_driver::compiler::NativeCompilerService;
    use etas_interpreter::{
        Interpreter,
        api::{ExecutionLimits, RunOptions, code::PreparedCodeDefinitions},
        host::CompilerHost,
    };
    use std::{num::NonZeroU32, sync::Arc};
    let (saved, artifact) = save_captured(SUSPENDED, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(SUSPENDED, &saved);
    let checkpoint = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(super::no_host::NoHost::with_checkpoint(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let options = |limits| RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions.clone()))),
        checkpoint_revisions: Some(revisions.clone()),
        execution_limits: limits,
        ..Default::default()
    };
    let one = ExecutionLimits::new(NonZeroU32::new(1).unwrap(), None).unwrap();
    let result = super::runtime()
        .block_on(Interpreter.resume_checkpoint(&checked, &checkpoint, &host, options(one)))
        .unwrap();
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("active and suspended calls exceed current execution limits")),
        "{:?}",
        result.diagnostics
    );
    let mut bad = checkpoint;
    bad.execution_progress.original_limits = one;
    let result = super::runtime()
        .block_on(Interpreter.resume_checkpoint(&checked, &bad, &host, options(Default::default())))
        .unwrap();
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("active and suspended calls exceed its original call-depth limit")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn detached_retry_attempts_survive_checkpoint_and_reject_inconsistent_ordinals() {
    let source = SUSPENDED
        .replace(
            "import std.runtime.checkpoint;",
            "import std.runtime.checkpoint; import std.runtime.limits.Attempts;",
        )
        .replace(
            "return perform Query.choose(value);",
            "retry limit Attempts(2) { return perform Query.choose(value); } return value;",
        );
    let (saved, artifact) = save_captured(&source, &[None], 20);
    assert_eq!(
        artifact["checkpoint"]["machine"]["suspended_programs"][0]["retries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let (checked, definitions, revisions) = restore_captured(&source, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    let mut bad = artifact;
    bad["checkpoint"]["machine"]["suspended_programs"][0]["retries"][0]["ordinal"] =
        serde_json::json!(1);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &bad,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("retry scope topology mismatch"),
        "{error}"
    );
}

#[test]
fn recursive_native_agent_calls_restore_a_multi_segment_suspension_graph() {
    let source = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
effect Query { action next(value: i32) -> i32; }
agent Worker(next: Worker, depth: i32) -> i32 ![Query.next] {
    if depth == 0 { return perform Query.next(20); }
    return next(next, depth - 1);
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised(revised, 32) } with {
        Query.next(value) => { checkpoint("recursive"); resume value; }
    };
}
"#;
    let (saved, artifact) = save_captured(source, &[None], 20);
    assert_eq!(
        artifact["checkpoint"]["machine"]["suspended_programs"]
            .as_array()
            .unwrap()
            .len(),
        33
    );
    let nodes = artifact["checkpoint"]["machine"]["suspended_programs"]
        .as_array()
        .unwrap();
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node["action"]["kind"] == "native")
            .count(),
        1
    );
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node["action"]["kind"] == "forwarded")
            .count(),
        32
    );
    let (checked, definitions, revisions) = restore_captured(source, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}
