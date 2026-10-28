use super::revision_checkpoint_values::{restore_captured, resume_captured, save_captured};

fn replace_flow(value: &mut serde_json::Value, from: u32, to: u32) -> usize {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some("flow")
        && value.get("item").and_then(serde_json::Value::as_u64) == Some(u64::from(from))
    {
        value["item"] = to.into();
        return 1;
    }
    match value {
        serde_json::Value::Object(fields) => fields
            .values_mut()
            .map(|value| replace_flow(value, from, to))
            .sum(),
        serde_json::Value::Array(values) => values
            .iter_mut()
            .map(|value| replace_flow(value, from, to))
            .sum(),
        _ => 0,
    }
}

fn flow(checked: &etas_frontend::CheckedProgram, name: &str) -> u32 {
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
}

#[test]
fn checkpoint_rejects_a_source_callable_with_a_different_checked_slot_signature() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow good() -> i32 { return 1; }
flow bad() -> string { return "wrong ABI"; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = good;
    checkpoint("callable slot");
    return callback() + selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(20))
    );
    assert_eq!(
        replace_flow(
            &mut artifact["checkpoint"]["machine"],
            flow(&checked, "good"),
            flow(&checked, "bad")
        ),
        1
    );
    let result = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    );
    assert!(
        result.is_err(),
        "checkpoint accepted a source callable returning the wrong type"
    );
}

#[test]
fn checkpoint_rejects_a_source_callable_exceeding_the_checked_effect_interface() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow good() -> unit ![] { return; }
flow bad() -> unit ![Console.stdout_write] { perform Console.stdout_write("forged"); }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback: () -> unit ![] = good;
    checkpoint("callable effects");
    callback();
    return selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 19);
    let (checked, _, revisions) = restore_captured(program, &saved);
    assert_eq!(
        replace_flow(
            &mut artifact["checkpoint"]["machine"],
            flow(&checked, "good"),
            flow(&checked, "bad")
        ),
        1
    );
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.to_string().contains("effects exceed"), "{error}");
}

#[test]
fn checkpoint_composition_checks_intermediate_interfaces_and_keeps_valid_conversions() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow text(value: i32) -> string { return "ok"; }
flow count(value: string) -> i32 { return 2; }
flow incompatible(value: bool) -> i32 { return 3; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = text | count;
    checkpoint("composed interface");
    return callback(1) + selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 21);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(21))
    );
    assert_eq!(
        replace_flow(
            &mut artifact["checkpoint"]["machine"],
            flow(&checked, "count"),
            flow(&checked, "incompatible")
        ),
        1
    );
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.to_string().contains("composition stages"), "{error}");
}

#[test]
fn checkpoint_checks_the_abi_of_a_source_call_waiting_on_an_argument() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow good(value: i32) -> i32 { return value; }
flow bad(value: i32) -> string { return "wrong ABI"; }
flow pause() -> i32 { checkpoint("pending source call"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return good(pause()) + selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(20))
    );
    assert_eq!(
        replace_flow(
            &mut artifact["checkpoint"]["machine"],
            flow(&checked, "good"),
            flow(&checked, "bad")
        ),
        1
    );
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.to_string().contains("checked slot type"), "{error}");
}

#[test]
fn checkpoint_source_call_uses_the_checked_generic_call_instantiation() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow echo<T>(value: T) -> T { return value; }
flow pause() -> i32 { checkpoint("generic source call"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return echo<i32>(pause()) + selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(20))
    );
    fn change_binding(value: &mut serde_json::Value) -> usize {
        if value.get("kind").and_then(serde_json::Value::as_str) == Some("specialized") {
            value["type_bindings"][0]["name"] = "unrelated".into();
            return 1;
        }
        match value {
            serde_json::Value::Object(fields) => fields.values_mut().map(change_binding).sum(),
            serde_json::Value::Array(values) => values.iter_mut().map(change_binding).sum(),
            _ => 0,
        }
    }
    assert_eq!(change_binding(&mut artifact["checkpoint"]["machine"]), 1);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("specialization differs"),
        "{error}"
    );
}
