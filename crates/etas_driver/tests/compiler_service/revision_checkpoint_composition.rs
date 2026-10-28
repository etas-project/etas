use super::revision_checkpoint_values::{restore_captured, resume_captured, save_checkpoint_at};
use etas_interpreter::api::InterpValue;
use serde_json::{Value, json};

const FIRST_CLASS: &str = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
agent Worker(value: i32) -> i32 { checkpoint("native"); return value; }
flow identity(value: i32) -> i32 ![] { return value; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let composed = selected | identity;
    return composed(19);
}
"#;

fn find_kind<'a>(value: &'a mut Value, kind: &str) -> Option<&'a mut Value> {
    if value.get("kind").and_then(Value::as_str) == Some(kind) {
        return Some(value);
    }
    match value {
        Value::Object(fields) => fields.values_mut().find_map(|value| find_kind(value, kind)),
        Value::Array(values) => values.iter_mut().find_map(|value| find_kind(value, kind)),
        _ => None,
    }
}

#[test]
fn a_first_class_composition_preserves_the_native_stage_invocation_across_checkpoint() {
    let (saved, artifact) = save_checkpoint_at(FIRST_CLASS, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(FIRST_CLASS, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn an_immediate_composition_does_not_require_a_caller_local_holding_the_target() {
    let program = FIRST_CLASS.replace(
        "let composed = selected | identity;\n    return composed(19);",
        "return (selected | identity)(19);",
    );
    assert!(!program.contains("let composed"));
    let (saved, artifact) = save_checkpoint_at(&program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(&program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn composed_invocation_rejects_missing_forged_or_inconsistent_stage_evidence() {
    let (saved, artifact) = save_checkpoint_at(FIRST_CLASS, &[None], 19, 1, 0);
    let (checked, _, revisions) = restore_captured(FIRST_CLASS, &saved);
    for damage in [
        "missing", "erased", "empty", "index", "stage", "root", "suffix",
    ] {
        let mut artifact = artifact.clone();
        let machine = &mut artifact["checkpoint"]["machine"];
        if damage == "suffix" {
            find_kind(machine, "composed_call").unwrap()["remaining"] = json!([]);
        } else {
            let site = &mut find_kind(machine, "program_return").unwrap()["call_site"];
            match damage {
                "missing" => {
                    site.as_object_mut().unwrap().remove("composition");
                }
                "erased" => site["composition"] = Value::Null,
                "empty" => site["composition"]["path"] = json!([]),
                "index" => site["composition"]["path"] = json!([2]),
                "stage" => site["composition"]["path"] = json!([1]),
                "root" => site["composition"]["root"] = json!({"kind": "composed", "targets": []}),
                _ => unreachable!(),
            }
        }
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &artifact,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        let expected = match damage {
            "missing" => "composition",
            "erased" => "checked nominal callee",
            "empty" => "empty",
            "index" | "root" => "out of bounds",
            "stage" => "selected composition stage",
            "suffix" => "remaining suffix",
            _ => unreachable!(),
        };
        assert!(error.message().contains(expected), "{damage}: {error}");
    }
}

#[test]
fn nested_compositions_keep_their_stage_path_for_calls_and_pipeline_stages() {
    for invocation in ["nested(19)", "19 ~> nested"] {
        let program = format!(
            r#"
module app;
import std.compiler.{{compile, CompileError}};
import std.runtime.checkpoint;
agent Worker(value: i32) -> i32 {{ checkpoint("native"); return value; }}
flow identity(value: i32) -> i32 ![] {{ return value; }}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let inner = identity | selected;
    let nested = inner | selected | identity;
    return {invocation};
}}
"#
        );
        for checkpoint in 0..2 {
            let (saved, artifact) = save_checkpoint_at(&program, &[None], 19, 2, checkpoint);
            let (checked, definitions, revisions) = restore_captured(&program, &saved);
            let result = resume_captured(&checked, definitions, revisions, &artifact);
            assert!(
                result.diagnostics.is_empty(),
                "{invocation}: {:?}",
                result.diagnostics
            );
            assert_eq!(result.value(), Some(&InterpValue::i32(19)));
            assert_eq!(result.checkpoints.len(), 1 - checkpoint);
        }
    }
}

#[test]
fn native_composition_stage_is_checked_against_its_intermediate_not_root_signature() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
agent Worker(value: string) -> string { checkpoint("native string"); return value; }
flow text(value: i32) -> string ![] { return "ok"; }
flow count(value: string) -> i32 ![] { return 19; }
flow incompatible(value: bool) -> i32 ![] { return 0; }
flow main() -> i32 {
    let checked_incompatible = incompatible;
    let selected = compile(^Worker);
    let composed = text | selected | count;
    return composed(7);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    let item_named = |name| {
        checked
            .hir
            .items
            .iter()
            .find_map(|(id, item)| match item {
                etas_hir::HirItem::Flow(flow)
                    if checked.symbols.get(flow.symbol).unwrap().name == name =>
                {
                    Some(id.0)
                }
                _ => None,
            })
            .unwrap()
    };
    let wrong = item_named("incompatible");
    let count = item_named("count");
    let mut damaged = artifact;
    let root = &mut find_kind(&mut damaged["checkpoint"]["machine"], "program_return").unwrap()["call_site"]
        ["composition"]["root"];
    let mut pending = vec![root];
    let mut replacements = 0;
    while let Some(node) = pending.pop() {
        if node.get("kind").and_then(Value::as_str) == Some("flow")
            && node.get("item").and_then(Value::as_u64) == Some(u64::from(count))
        {
            node["item"] = json!(wrong);
            replacements += 1;
        } else {
            match node {
                Value::Object(fields) => pending.extend(fields.values_mut()),
                Value::Array(values) => pending.extend(values),
                _ => {}
            }
        }
    }
    assert_eq!(replacements, 1);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &damaged,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.message().contains("composition stages"), "{error}");
}

#[test]
fn a_revision_returned_callable_can_checkpoint_as_a_composed_stage() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow pause(value: i32) -> i32 { checkpoint("scoped source"); return value; }
agent Worker() -> ((i32) -> i32) { return pause; }
flow identity(value: i32) -> i32 ![] { return value; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let callback = selected();
    let composed = callback | identity;
    return composed(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_source_stage_checkpoint_retains_the_following_native_stage() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
agent Worker(value: i32) -> i32 { checkpoint("native"); return value; }
flow pause(value: i32) -> i32 { checkpoint("source"); return value; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let composed = pause | selected;
    return composed(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 2, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert_eq!(result.checkpoints.len(), 1);
}
