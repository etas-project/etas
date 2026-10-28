use super::revision_checkpoint_values::{
    restore_captured, resume_captured, save_captured, save_checkpoint_at,
};
use etas_interpreter::api::InterpValue;

#[path = "revision_checkpoint_stack/root.rs"]
mod root;

const ACTIVE: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { checkpoint("inside candidate"); return 19; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected() + 1;
}
"#;

#[test]
fn an_active_revision_checkpoint_retains_the_run_root_and_program_boundary() {
    let (sources, artifact) = save_captured(ACTIVE, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(ACTIVE, &sources);
    assert_eq!(
        artifact["checkpoint"]["entry_item"],
        checked.entry.unwrap().0
    );
    assert!(artifact["checkpoint"]["code_revision"].is_null());
    assert!(
        artifact["checkpoint"]["machine"]["frames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|frame| frame["kind"] == "program_return")
    );
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}

const NESTED: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(depth: i32) -> i32 {
    if depth > 0 { return Worker(depth - 1); }
    checkpoint("inside frozen source");
    return 19;
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected(1) + 1;
}
"#;

#[test]
fn a_checkpoint_preserves_nested_candidate_and_frozen_source_program_frames() {
    let (sources, artifact) = save_captured(NESTED, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(NESTED, &sources);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}

#[test]
fn active_program_boundaries_reject_missing_revisions_changed_call_sites_and_scope_corruption() {
    use etas_interpreter::api::codec::checkpoint_from_json_with_revisions;
    let (sources, artifact) = save_captured(ACTIVE, &[None], 20);
    let (checked, _, revisions) = restore_captured(ACTIVE, &sources);
    let boundary = artifact["checkpoint"]["machine"]["frames"]
        .as_array()
        .unwrap()
        .iter()
        .position(|frame| frame["kind"] == "program_return")
        .unwrap();
    let decode = |artifact: &serde_json::Value, revisions| {
        checkpoint_from_json_with_revisions(&Default::default(), artifact, &checked, revisions)
    };
    assert!(
        decode(&artifact, None)
            .unwrap_err()
            .message()
            .contains("freshly prepared revisions")
    );
    let mut invalid_call_site =
        artifact["checkpoint"]["machine"]["frames"][boundary]["call_site"].clone();
    invalid_call_site["origin"]["expr"] = serde_json::json!(u32::MAX);
    for (field, value, expected) in [
        ("call_site", invalid_call_site, "checked call site"),
        ("handler_floor", serde_json::json!(1), "handler floor"),
        ("retry_depth", serde_json::json!(1), "retry depths"),
    ] {
        let mut tampered = artifact.clone();
        tampered["checkpoint"]["machine"]["frames"][boundary][field] = value;
        let error = decode(&tampered, Some(&revisions)).unwrap_err();
        assert!(
            error.message().contains(expected),
            "{field}: {}",
            error.message()
        );
    }
    let mut tampered = artifact.clone();
    tampered["checkpoint"]["machine"]["frames"]
        .as_array_mut()
        .unwrap()
        .remove(boundary);
    assert!(
        decode(&tampered, Some(&revisions)).is_err(),
        "child frames cannot be interpreted as root frames"
    );
}

const HANDLERS: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![Error<i32>] {
    return handle {
        checkpoint("candidate handler");
        return 19;
    } with { Error<i32>.raise(_) => { finish 7; } };
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return handle {
        return selected() + 1;
    } with { Error<i32>.raise(_) => { finish 8; } };
}
"#;

#[test]
fn checkpoint_validates_handler_arms_in_their_own_checked_program() {
    let (sources, artifact) = save_captured(HANDLERS, &[None], 20);
    assert_eq!(
        artifact["checkpoint"]["handlers"].as_array().unwrap().len(),
        2
    );
    let (checked, definitions, revisions) = restore_captured(HANDLERS, &sources);
    let mut wrong_floor = artifact.clone();
    let frames = wrong_floor["checkpoint"]["machine"]["frames"]
        .as_array_mut()
        .unwrap();
    let boundary = frames
        .iter_mut()
        .find(|frame| frame["kind"] == "program_return")
        .unwrap();
    boundary["handler_floor"] = serde_json::json!(0);
    assert!(
        etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &wrong_floor,
            &checked,
            Some(&revisions),
        )
        .unwrap_err()
        .message()
        .contains("handler floor")
    );
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

#[test]
fn an_active_named_agent_method_keeps_its_checked_callable_selection() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
impl agent Worker {
    flow extra() -> i32 { checkpoint("inside extra"); return 31; }
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected.extra() + 1;
}
"#;
    let (sources, artifact) = save_captured(program, &[None], 32);
    let (checked, definitions, revisions) = restore_captured(program, &sources);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(32)));
}

#[test]
fn active_revision_locals_and_results_use_the_recompiled_nominal_type_universe() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type Payload = { count: i32 };
agent Worker(value: Payload) -> Payload {
    let copied = Payload { count = value.count + 1 };
    checkpoint("nominal local");
    return copied;
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let result = selected(Payload { count = 18 });
    return result.count + 1;
}
"#;
    let (sources, artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &sources);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}

#[test]
fn a_resumed_candidate_can_checkpoint_again_without_changing_the_root_owner() {
    let program = ACTIVE.replace(
        "checkpoint(\"inside candidate\");",
        "checkpoint(\"first\"); checkpoint(\"second\");",
    );
    let (sources, first) = save_checkpoint_at(&program, &[None], 20, 2, 0);
    let (checked, definitions, revisions) = restore_captured(&program, &sources);
    let result = resume_captured(&checked, definitions, revisions, &first);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.checkpoints.len(), 1);
    let second =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    assert_eq!(
        second["checkpoint"]["entry_item"],
        first["checkpoint"]["entry_item"]
    );
    assert_eq!(
        second["checkpoint"]["compilation"],
        first["checkpoint"]["compilation"]
    );
    assert!(second["checkpoint"]["code_revision"].is_null());
    let (checked, definitions, revisions) = restore_captured(&program, &sources);
    let result = resume_captured(&checked, definitions, revisions, &second);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}
