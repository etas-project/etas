use super::revision_checkpoint_values::{restore_captured, resume_captured, save_captured};
use etas_interpreter::api::InterpValue;

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
fn a_suspended_index_keeps_its_native_array_and_checked_type_context() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow pause() -> i32 { checkpoint("index"); return 0; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let values = [selected];
    let worker = values[pause()];
    return worker();
}
"#,
        19,
    );
}

#[test]
fn suspended_generic_index_and_map_access_keep_the_owner_type_bindings() {
    for (helpers, input) in [
        (
            "flow pause() -> i32 { checkpoint(\"generic index\"); return 0; } flow read<T>(values: Array<T>) -> T { return values[pause()]; }",
            "[selected]",
        ),
        (
            "flow pause() -> string { checkpoint(\"map lookup\"); return \"entry\"; } flow read<T>(values: Map<string, T>) -> T { return values[pause()]; }",
            "{ \"entry\" => selected }",
        ),
    ] {
        roundtrip(
            &format!(
                r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
{helpers}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let worker = read({input});
    return worker();
}}
"#
            ),
            19,
        );
    }
}

#[test]
fn suspended_slice_bounds_preserve_native_elements_and_generic_context() {
    for slice in ["values[start(), 1)", "values[0, end())"] {
        roundtrip(
            &format!(
                r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
flow start() -> i32 {{ checkpoint("slice start"); return 0; }}
flow end() -> i32 {{ checkpoint("slice end"); return 1; }}
flow read<T>(values: Array<T>) -> T {{
    let part = {slice};
    return part[0];
}}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let worker = read([selected]);
    return worker();
}}
"#
            ),
            19,
        );
    }
}

const COMPOSE: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(seed: i32) -> (i32 -> i32 ![]) { return (value: i32) => seed + value; }
flow identity(value: i32) -> i32 ![] { return value; }
flow choose() -> (i32 -> i32 ![]) { checkpoint("stage target"); return identity; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = selected(19);
    let composed = callback | choose();
    return composed(1);
}
"#;

#[test]
fn a_suspended_composition_retains_typed_native_stage_targets() {
    let mut artifact = roundtrip(COMPOSE, 20);
    let stage = find_kind(
        &mut artifact["checkpoint"]["machine"],
        "pipeline_stage_target",
    )
    .unwrap();
    assert!(stage.get("expr").is_some());
    assert!(stage.get("stages").is_none());
    assert!(stage.get("current_limits").is_none());
    assert_eq!(stage["targets"].as_array().unwrap().len(), 1);
}

#[test]
fn a_pipeline_can_suspend_while_selecting_a_stage_without_losing_its_agent_input() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow invoke(worker: Worker) -> i32 ![] { return worker(); }
flow choose() -> (Worker -> i32 ![]) { checkpoint("pipeline target"); return invoke; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected ~> choose();
}
"#,
        19,
    );
}

#[test]
fn a_pipeline_can_suspend_in_its_input_before_selecting_native_stages() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow invoke(worker: Worker) -> i32 ![] { return worker(); }
flow pause(worker: Worker) -> Worker { checkpoint("pipeline input"); return worker; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return pause(selected) ~> invoke;
}
"#,
        19,
    );
}

fn find_kind<'a>(
    value: &'a mut serde_json::Value,
    kind: &str,
) -> Option<&'a mut serde_json::Value> {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some(kind) {
        return Some(value);
    }
    match value {
        serde_json::Value::Object(fields) => {
            fields.values_mut().find_map(|value| find_kind(value, kind))
        }
        serde_json::Value::Array(values) => {
            values.iter_mut().find_map(|value| find_kind(value, kind))
        }
        _ => None,
    }
}

#[test]
fn pending_pipeline_cannot_replace_its_owner_or_resolved_prefix() {
    let (saved, artifact) = save_captured(COMPOSE, &[None], 20);
    let (checked, _, revisions) = restore_captured(COMPOSE, &saved);
    for (field, value, expected) in [
        ("expr", serde_json::json!(u32::MAX), "checked owner"),
        ("targets", serde_json::json!([]), "resolved target prefix"),
    ] {
        let mut damaged = artifact.clone();
        find_kind(
            &mut damaged["checkpoint"]["machine"],
            "pipeline_stage_target",
        )
        .unwrap()[field] = value;
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
    find_kind(
        &mut damaged["checkpoint"]["machine"],
        "pipeline_stage_target",
    )
    .unwrap()
    .as_object_mut()
    .unwrap()
    .remove("expr");
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &damaged,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.message().contains("expr"), "{error}");
}

#[test]
fn pending_index_and_slice_reject_erased_native_operands_and_mismatched_owners() {
    for (expression, kind, expected) in [
        ("values[pause()]", "index_value", "expr"),
        (
            "{ let part = values[pause(), 1); part[0] }",
            "slice_start",
            "eval",
        ),
        (
            "{ let part = values[0, end()); part[0] }",
            "slice_end",
            "eval",
        ),
    ] {
        let program = format!(
            r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
flow pause() -> i32 {{ checkpoint("index or slice"); return 0; }}
flow end() -> i32 {{ checkpoint("slice end"); return 1; }}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let values = [selected];
    let worker = {expression};
    return worker();
}}
"#
        );
        let (saved, artifact) = save_captured(&program, &[None], 19);
        let (checked, _, revisions) = restore_captured(&program, &saved);
        for field in ["base", "frame", expected] {
            let mut damaged = artifact.clone();
            let operand = find_kind(&mut damaged["checkpoint"]["machine"], kind).unwrap();
            match field {
                "base" => operand["base"] = serde_json::json!({"kind": "unit"}),
                "frame" => {
                    operand.as_object_mut().unwrap().remove("frame");
                }
                "expr" => operand["expr"] = serde_json::json!(u32::MAX),
                "eval" => operand["eval"]["expr"] = serde_json::json!(u32::MAX),
                _ => unreachable!(),
            }
            assert!(
                etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
                    &Default::default(),
                    &damaged,
                    &checked,
                    Some(&revisions)
                )
                .is_err(),
                "{kind} accepted invalid {field}"
            );
        }
    }
}

#[test]
fn a_pending_pipeline_rejects_erasing_its_checked_agent_input() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow invoke(worker: Worker) -> i32 ![] { return worker(); }
flow choose() -> (Worker -> i32 ![]) { checkpoint("select"); return invoke; }
flow main() -> i32 { let selected = compile(^Worker); return selected ~> choose(); }
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 19);
    let (checked, _, revisions) = restore_captured(program, &saved);
    find_kind(&mut artifact["checkpoint"]["machine"], "pipeline_target").unwrap()["input"] =
        serde_json::json!({"kind": "unit"});
    assert!(
        etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &artifact,
            &checked,
            Some(&revisions)
        )
        .is_err()
    );
}
