use super::revision_checkpoint_values::{restore_captured, resume_captured, save_captured};
use etas_interpreter::api::InterpValue;

const TARGET: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(value: i32) -> i32 { return value + 19; }
flow pause() -> i32 { checkpoint("argument"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected(pause());
}
"#;

#[test]
fn a_pending_compiled_agent_call_keeps_its_checked_callee_identity() {
    let (saved, artifact) = save_captured(TARGET, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(TARGET, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}

fn roundtrip(program: &str, expected: i32) -> serde_json::Value {
    let (saved, artifact) = save_captured(program, &[None], expected);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(expected)));
    assert!(result.checkpoints.is_empty());
    artifact
}

#[test]
fn an_evaluated_nominal_argument_keeps_its_nested_compiled_agent_type() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
type Holder = { worker: Worker };
flow combine(holder: Holder, value: i32) -> i32 {
    let worker = holder.worker;
    return worker() + value;
}
flow pause() -> i32 { checkpoint("argument"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return combine(Holder { worker = selected }, pause());
}
"#,
        20,
    );
}

#[test]
fn a_pending_scoped_closure_call_keeps_its_checked_function_type() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(seed: i32) -> ((i32) -> i32 ![]) { return (value: i32) => seed + value; }
flow pause() -> i32 { checkpoint("argument"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = selected(19);
    return callback(pause());
}
"#,
        20,
    );
}

const METHOD: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(value: i32) -> i32 { return value + 19; }
impl agent Worker { flow other(value: i32) -> i32 { return value + 29; } }
flow pause() -> i32 { checkpoint("argument"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected.other(pause());
}
"#;

#[test]
fn a_pending_auxiliary_method_keeps_the_checked_method_selection() {
    roundtrip(METHOD, 30);
}

#[test]
fn a_suspended_generic_callee_expression_retains_the_outer_call_site() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(value: i32) -> i32 { return value + 19; }
flow choose<T>(value: T) -> T { checkpoint("callee"); return value; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return choose(selected)(1);
}
"#,
        20,
    );
}

fn pending_call(value: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some("call_args") {
        return Some(value);
    }
    match value {
        serde_json::Value::Object(fields) => fields.values_mut().find_map(pending_call),
        serde_json::Value::Array(values) => values.iter_mut().find_map(pending_call),
        _ => None,
    }
}

#[test]
fn pending_call_metadata_cannot_drop_or_change_its_checked_site_or_argument_prefix() {
    let (saved, artifact) = save_captured(TARGET, &[None], 20);
    let (checked, _, revisions) = restore_captured(TARGET, &saved);
    for (field, value, expected) in [
        ("call", serde_json::json!(u32::MAX), "checked call site"),
        (
            "evaluated_args",
            serde_json::json!([{"kind":"unit"}]),
            "evaluated call argument prefix",
        ),
    ] {
        let mut damaged = artifact.clone();
        pending_call(&mut damaged["checkpoint"]["machine"]).unwrap()[field] = value;
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &damaged,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        assert!(error.message().contains(expected), "{field}: {error}");
    }
    let mut damaged = artifact;
    pending_call(&mut damaged["checkpoint"]["machine"])
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("call");
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &damaged,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.message().contains("call"), "{error}");
}

#[test]
fn pending_method_metadata_cannot_select_another_callable_with_the_same_abi() {
    let (saved, mut artifact) = save_captured(METHOD, &[None], 30);
    let (checked, _, revisions) = restore_captured(METHOD, &saved);
    let default_method = artifact["code_sources"][0]["source"]["revision"]["definition"].clone();
    let call = pending_call(&mut artifact["checkpoint"]["machine"]).unwrap();
    assert_eq!(call["target"]["kind"], "agent_revision");
    call["target"]["reference"]["callable"] = default_method;
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.message().contains("checked call selection"),
        "{error}"
    );
}

#[test]
fn source_only_resume_recompiles_the_replacement_before_restoring_pending_arguments() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, with_body, parse_body, CompileError};
agent Worker(value: i32) -> i32 { return 19; }
flow pause() -> i32 { checkpoint("argument"); return 1; }
flow build() -> Worker ![Error<CompileError>] {
    let draft = match parse_body("return 29;") {
        Ok(body) => with_body(^Worker, body),
        Err(_) => abort("invalid body"),
    };
    return compile(draft);
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = build();
    return selected(pause());
}
"#;
    let (_, artifact) = save_captured(program, &[Some("return 29;")], 29);
    let result = super::revision_checkpoint_source_resume::resume_source_for(
        program, artifact, true, false, false,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(29)));
    assert!(result.checkpoints.is_empty());
}
