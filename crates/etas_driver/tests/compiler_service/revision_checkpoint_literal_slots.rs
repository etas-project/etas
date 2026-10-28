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
fn a_partially_evaluated_tuple_preserves_its_compiled_agent_element() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow pause() -> i32 { checkpoint("tuple field"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let pair = (selected, pause());
    let (worker, number) = pair;
    return worker() + number;
}
"#,
        20,
    );
}

#[test]
fn partially_evaluated_records_and_named_variants_keep_typed_agent_fields() {
    for construction in [
        "Holder { worker = selected, number = pause() }",
        "Holder { selected, number = pause() }",
        "Envelope.Full { worker = selected, number = pause() }",
    ] {
        let (declaration, result) = if construction.starts_with("Envelope") {
            (
                "enum Envelope { Full { worker: Worker, number: i32 }, Empty, }",
                "match holder { Envelope.Full { worker, number } => worker() + number, Envelope.Empty => 0 }",
            )
        } else if construction.contains("worker =") {
            (
                "type Holder = { worker: Worker, number: i32 };",
                "{ let worker = holder.worker; worker() + holder.number }",
            )
        } else {
            (
                "type Holder = { selected: Worker, number: i32 };",
                "{ let worker = holder.selected; worker() + holder.number }",
            )
        };
        roundtrip(
            &format!(
                r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
{declaration}
flow pause() -> i32 {{ checkpoint("record field"); return 1; }}
flow main() -> i32 ![Error<CompileError>] {{
    let selected = compile(^Worker);
    let holder = {construction};
    return {result};
}}
"#
            ),
            20,
        );
    }
}

#[test]
fn partially_evaluated_sequences_keep_agent_elements_and_their_collection_kind() {
    for (construction, expected) in [
        ("[selected, pause(selected)]", 38),
        ("[selected; pause(selected)]", 38),
        ("#{selected, pause(selected)}", 19),
    ] {
        roundtrip(
            &format!(
                r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
flow pause(worker: Worker) -> Worker {{ checkpoint("sequence element"); return worker; }}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let values = {construction};
    var result = 0;
    for worker in values limit Iterations(4) {{ result = result + worker(); }}
    return result;
}}
"#
            ),
            expected,
        );
    }
}

#[test]
fn suspended_map_keys_and_values_keep_completed_native_entries() {
    for construction in [
        "{ \"first\" => selected, pause_key() => selected }",
        "{ \"first\" => selected, \"second\" => pause_value(selected) }",
    ] {
        roundtrip(
            &format!(
                r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 {{ return 19; }}
flow pause_key() -> string {{ checkpoint("map key"); return "second"; }}
flow pause_value(worker: Worker) -> Worker {{ checkpoint("map value"); return worker; }}
flow main() -> i32 ![Error<CompileError>] {{
    let selected = compile(^Worker);
    let values = {construction};
    return match values.get("first") {{ Some(worker) => worker(), None => 0 }};
}}
"#
            ),
            19,
        );
    }
}

#[test]
fn a_pending_map_value_keeps_the_native_identity_of_its_evaluated_key() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow pause() -> i32 { checkpoint("map value"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let values = { selected => pause() };
    return match values.get(selected) { Some(number) => selected() + number, None => 0 };
}
"#,
        20,
    );
}

#[test]
fn a_record_under_construction_keeps_its_scoped_native_closure() {
    roundtrip(SCOPED_RECORD, 20);
}

const SCOPED_RECORD: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(seed: i32) -> (() -> i32 ![]) { return () => seed; }
type Holder = { callback: () -> i32 ![], number: i32 };
flow pause() -> i32 { checkpoint("closure field"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = selected(19);
    let holder = Holder { callback, number = pause() };
    let restored = holder.callback;
    return restored() + holder.number;
}
"#;

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
fn a_construction_field_cannot_replace_a_function_with_a_nominal_agent() {
    let (saved, mut artifact) = save_captured(SCOPED_RECORD, &[None], 20);
    let (checked, _, revisions) = restore_captured(SCOPED_RECORD, &saved);
    let agent = find_kind(&mut artifact["checkpoint"]["machine"], "agent_revision")
        .unwrap()
        .clone();
    let record = find_kind(&mut artifact["checkpoint"]["machine"], "record_field").unwrap();
    record["values"][0]["value"] = serde_json::json!({ "kind": "callable", "target": agent });
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(error.message().contains("checked slot type"), "{error}");
}

#[test]
fn a_construction_field_cannot_erase_a_required_callable() {
    let (saved, mut artifact) = save_captured(SCOPED_RECORD, &[None], 20);
    let (checked, _, revisions) = restore_captured(SCOPED_RECORD, &saved);
    let record = find_kind(&mut artifact["checkpoint"]["machine"], "record_field").unwrap();
    record["values"][0]["value"] = serde_json::json!({ "kind": "unit" });
    let result = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    );
    assert!(
        result.is_err(),
        "checkpoint erased a required callable field and still passed validation"
    );
}

#[test]
fn checkpoint_type_validation_keeps_empty_optional_callable_fields_in_recursive_values() {
    roundtrip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
type Node = { callback: Option<() -> i32 ![]>, next: Option<Node> };
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let empty = Node { callback = None, next = None };
    let root = Node { callback = None, next = Some(empty) };
    checkpoint("optional recursive function fields");
    return selected();
}
"#,
        19,
    );
}

#[test]
fn a_checked_local_cannot_erase_its_callable_value_or_its_enclosing_container() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(seed: i32) -> (() -> i32 ![]) { return () => seed; }
type Holder = { callback: () -> i32 ![], number: i32 };
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = selected(19);
    let holder = Holder { callback, number = 1 };
    checkpoint("completed native holder");
    let restored = holder.callback;
    return restored() + holder.number;
}
"#;
    let (saved, artifact) = save_captured(program, &[None], 20);
    let (checked, _, revisions) = restore_captured(program, &saved);
    fn erase_local(value: &mut serde_json::Value, symbol: u32) -> usize {
        if value.get("symbol").and_then(serde_json::Value::as_u64) == Some(u64::from(symbol))
            && value.get("value").is_some()
        {
            value["value"] = serde_json::json!({"kind": "unit"});
            return 1;
        }
        match value {
            serde_json::Value::Object(fields) => fields
                .values_mut()
                .map(|value| erase_local(value, symbol))
                .sum(),
            serde_json::Value::Array(values) => values
                .iter_mut()
                .map(|value| erase_local(value, symbol))
                .sum(),
            _ => 0,
        }
    }
    for name in ["selected", "callback", "holder"] {
        let symbol = checked
            .symbols
            .iter()
            .find(|symbol| symbol.name == name && symbol.kind == etas_hir::SymbolKind::Local)
            .unwrap()
            .id
            .0;
        let mut damaged = artifact.clone();
        assert!(
            erase_local(&mut damaged["checkpoint"]["machine"], symbol) > 0,
            "missing saved local: {name}"
        );
        let result = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &damaged,
            &checked,
            Some(&revisions),
        );
        assert!(
            result.is_err(),
            "checkpoint erased checked {name} and passed restore validation"
        );
    }
}

#[test]
fn erased_generic_values_cannot_hide_behind_missing_frame_type_bindings() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow keep<T>(value: T) -> i32 { checkpoint("generic value"); return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return keep(selected) + selected();
}
"#;
    let (saved, mut artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    let parameter = checked
        .symbols
        .iter()
        .find(|symbol| symbol.name == "value" && symbol.kind == etas_hir::SymbolKind::Param)
        .unwrap()
        .id
        .0;
    fn erase(value: &mut serde_json::Value, parameter: u32) -> usize {
        match value {
            serde_json::Value::Object(fields) => {
                if let Some(serde_json::Value::Array(bindings)) = fields.get_mut("type_bindings") {
                    bindings.retain(|binding| {
                        binding.get("name").and_then(serde_json::Value::as_str) != Some("T")
                    });
                }
                if fields.get("symbol").and_then(serde_json::Value::as_u64)
                    == Some(u64::from(parameter))
                    && fields.contains_key("value")
                {
                    fields.insert("value".into(), serde_json::json!({"kind":"unit"}));
                    return 1;
                }
                fields
                    .values_mut()
                    .map(|value| erase(value, parameter))
                    .sum()
            }
            serde_json::Value::Array(values) => {
                values.iter_mut().map(|value| erase(value, parameter)).sum()
            }
            _ => 0,
        }
    }
    assert!(erase(&mut artifact["checkpoint"]["machine"], parameter) > 0);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.message().contains("unresolved generic value type"),
        "{error}"
    );
}
