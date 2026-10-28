use super::*;

#[test]
fn an_implicit_effect_row_in_a_checkpoint_cannot_be_removed_or_replaced() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] {
        checkpoint("checked implicit row");
        return callback(value);
    }
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("checkpoint row");
    if selected.apply((value: UserId) => value, id) == id { return 20; }
    return 0;
}
"#;
    let (sources, artifact) = save_captured(program, &[None], 20);
    let (checked, _, revisions) = restore_captured(program, &sources);
    for remove in [true, false] {
        let mut bad = artifact.clone();
        let boundary = generic_boundary(&mut bad).unwrap();
        assert_eq!(boundary["call"]["effect_row_bindings"][0][0], "E");
        if remove {
            boundary["call"]["effect_row_bindings"] = serde_json::json!([]);
        } else {
            boundary["call"]["effect_row_bindings"][0][1]["effects"] =
                serde_json::json!([{"name":"Console.stdout_write","args":[]}]);
        }
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &bad,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        assert!(
            error.message().contains("specialization") || error.message().contains("generic"),
            "{}",
            error.message()
        );
    }
}

#[test]
fn generic_program_boundary_rejects_changed_missing_and_wrong_kind_bindings() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow echo<T, effect E>(value: T) -> T ![E] { checkpoint("generic boundary"); return value; } }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("generic checkpoint");
    if selected.echo<UserId, ![]>(id) == id { return 20; }
    return 0;
}
"#;
    let (sources, artifact) = save_captured(program, &[None], 20);
    let (checked, _, revisions) = restore_captured(program, &sources);
    let string = checked
        .type_store
        .iter()
        .find_map(|(ty, value)| {
            matches!(
                value,
                etas_types::Type::Primitive(etas_types::PrimitiveType::String)
            )
            .then_some(ty)
        })
        .unwrap();
    for case in 0..5 {
        let mut bad = artifact.clone();
        let boundary = generic_boundary(&mut bad).expect("specialized program-return boundary");
        match case {
            0 => boundary["call"]["type_bindings"][0]["type"] = serde_json::json!(string.0),
            1 => boundary["call"]["type_bindings"] = serde_json::json!([]),
            2 => boundary["call"]["effect_row_bindings"] = serde_json::json!([]),
            3 => boundary["call"]["type_bindings"][0]["name"] = serde_json::json!("E"),
            4 => {
                boundary["call"]["effect_row_bindings"][0][1]["effects"] =
                    serde_json::json!([{"name":"Console.stdout_write","args":[]}])
            }
            _ => unreachable!(),
        }
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &bad,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        assert!(
            error.message().contains("specialization") || error.message().contains("generic"),
            "case {case}: {}",
            error.message()
        );
    }
}

fn generic_boundary(value: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    if value
        .get("call")
        .and_then(|call| call.get("kind"))
        .and_then(serde_json::Value::as_str)
        == Some("specialized")
        && value.get("handler_floor").is_some()
    {
        return Some(value);
    }
    match value {
        serde_json::Value::Object(fields) => fields.values_mut().find_map(generic_boundary),
        serde_json::Value::Array(values) => values.iter_mut().find_map(generic_boundary),
        _ => None,
    }
}

#[test]
fn a_specialized_callback_cannot_be_redirected_around_generic_spec_bounds() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
spec Allowed;
type UserId = string;
flow identity<T>(value: T) -> T { return value; }
flow guarded<T ~ Allowed>(value: T) -> T { return value; }
agent Worker(callback: (UserId) -> UserId ![], value: UserId) -> UserId {
    checkpoint("bounded callback");
    return callback(value);
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let value = UserId("generic");
    if selected(identity, value) == value { return 20; }
    return 0;
}
"#;
    let (sources, mut artifact) = save_captured(program, &[None], 20);
    let (checked, _, revisions) = restore_captured(program, &sources);
    let guarded = checked
        .hir
        .items
        .iter()
        .find_map(|(id, item)| {
            let etas_hir::HirItem::Flow(flow) = item else {
                return None;
            };
            (checked.symbols.get(flow.symbol)?.name == "guarded").then_some(id)
        })
        .unwrap();
    let callback = scoped(&mut artifact).unwrap();
    assert_eq!(callback["target"]["kind"], "specialized");
    callback["target"]["target"]["item"] = serde_json::json!(guarded.0);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.message().contains("spec bound"),
        "{}",
        error.message()
    );
}

const PROGRAM: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(value: i32) -> (() -> i32 ![]) { return () => value; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let callback = selected(19);
    checkpoint("returned closure");
    return callback() + 1;
}
"#;

fn scoped(value: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some("scoped_callable") {
        return Some(value);
    }
    match value {
        serde_json::Value::Object(fields) => fields.values_mut().find_map(scoped),
        serde_json::Value::Array(values) => values.iter_mut().find_map(scoped),
        _ => None,
    }
}

#[test]
fn callable_checkpoint_rejects_forged_projection_program_type_and_capture_facts() {
    let (sources, artifact) = save_captured(PROGRAM, &[None], 20);
    let (checked, _, revisions) = restore_captured(PROGRAM, &sources);
    for case in 0..6 {
        let mut bad = artifact.clone();
        let callable = scoped(&mut bad).expect("durable callback recipe");
        match case {
            0 => callable["metadata"]["native_type"] = serde_json::json!(u32::MAX),
            1 => callable["metadata"]["projections"][0]["from"] = serde_json::json!(u32::MAX),
            2 => callable["metadata"]["projections"][0]["to"] = serde_json::json!(u32::MAX),
            3 => callable["metadata"]["native_program"] = serde_json::json!({"kind":"source"}),
            4 => callable["target"]["expr"] = serde_json::json!(u32::MAX),
            5 => callable["target"]["captured"]["locals"] = serde_json::json!([]),
            _ => unreachable!(),
        }
        let result = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &bad,
            &checked,
            Some(&revisions),
        );
        assert!(result.is_err(), "tampered callable case {case} passed");
    }
    assert!(etas_interpreter::api::codec::checkpoint_from_json(&artifact, &checked).is_err());
}

#[test]
fn callable_checkpoint_cannot_substitute_a_target_with_broader_effects() {
    let program = PROGRAM.replace(
        "{ return () => value; }",
        "{ let evil: () -> i32 ![Console.stdout_write] = () => { perform Console.stdout_write(\"not allowed\"); return 19; }; return () => value; }",
    );
    let (sources, mut artifact) = save_captured(&program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(&program, &sources);
    let reflection = *checked.types.reflections.keys().next().unwrap();
    let revision = super::super::revision_checkpoint::compile(
        definitions.clone(),
        definitions.reflect_agent(reflection).unwrap(),
        true,
    )
    .unwrap();
    let native = revision.program();
    let evil = native
        .hir
        .stmts
        .iter()
        .find_map(|(_, stmt)| {
            let etas_hir::HirStmt::Let { pat, value, .. } = stmt else {
                return None;
            };
            let etas_hir::HirPat::Binding { symbol, .. } = &native.hir.pats[*pat] else {
                return None;
            };
            (native.symbols.get(*symbol)?.name == "evil").then_some(*value)
        })
        .unwrap();
    let callback = scoped(&mut artifact).unwrap();
    callback["target"]["expr"] = serde_json::json!(evil.0);
    callback["target"]["captured"]["locals"] = serde_json::json!([]);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap_err();
    assert!(
        error.message().contains("effects exceed"),
        "{}",
        error.message()
    );
}
