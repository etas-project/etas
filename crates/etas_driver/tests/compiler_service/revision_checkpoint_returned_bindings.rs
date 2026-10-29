use super::revision_checkpoint_values::{restore_captured, resume_captured, save_checkpoint_at};
use etas_interpreter::api::InterpValue;

#[test]
fn a_returned_higher_order_callable_preserves_a_nonempty_effect_row_after_restore() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
effect Probe { action touch() -> unit; }
flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] { return callback(value); }
flow pause(value: i32) -> i32 ![Probe.touch] {
    checkpoint("effectful higher order callback");
    perform Probe.touch();
    return value;
}
agent Worker() -> (((i32) -> i32 ![Probe.touch], i32) -> i32 ![Probe.touch]) { return apply; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let invoke = selected();
    return handle {
        return invoke(pause, 19);
    } with {
        Probe.touch() => { resume; }
    };
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

const HIGHER_ORDER: &str = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] { return callback(value); }
flow pause(value: i32) -> i32 ![] { checkpoint("instantiated higher order callback"); return value; }
agent Worker() -> (((i32) -> i32 ![], i32) -> i32 ![]) { return apply; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let invoke = selected();
    return invoke(pause, 19);
}
"#;

#[test]
fn a_returned_source_tool_preserves_its_generic_frame_after_restore() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
tool apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] { return callback(value); }
flow pause(value: i32) -> i32 ![] { checkpoint("instantiated source tool callback"); return value; }
agent Worker() -> (((i32) -> i32 ![], i32) -> i32 ![]) { return apply; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let invoke = selected();
    return invoke(pause, 19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_returned_global_tool_binding_preserves_its_generic_frame_after_restore() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
tool apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] { return callback(value); }
let selected_tool: ((i32) -> i32 ![], i32) -> i32 ![] = apply;
flow pause(value: i32) -> i32 ![] { checkpoint("global source tool callback"); return value; }
agent Worker() -> (((i32) -> i32 ![], i32) -> i32 ![]) { return selected_tool; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let invoke = selected();
    return invoke(pause, 19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_returned_higher_order_callable_preserves_type_and_effect_row_instantiation() {
    let program = HIGHER_ORDER;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn restoring_a_higher_order_callable_rejects_missing_or_forged_effect_row_bindings() {
    fn alter(value: &mut serde_json::Value, case: usize) -> bool {
        if value.get("kind").and_then(serde_json::Value::as_str) == Some("specialized")
            && value
                .get("effect_row_bindings")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|rows| !rows.is_empty())
        {
            match case {
                0 => {
                    value.as_object_mut().unwrap().remove("effect_row_bindings");
                }
                1 => value["effect_row_bindings"] = serde_json::json!([]),
                2 => {
                    let first = value["effect_row_bindings"][0].clone();
                    value["effect_row_bindings"]
                        .as_array_mut()
                        .unwrap()
                        .push(first);
                }
                3 => value["effect_row_bindings"][0][1]["tail"] = serde_json::json!("E"),
                4 => value["effect_row_bindings"][0][0] = serde_json::json!("T"),
                5 => {
                    value["effect_row_bindings"][0][1]["effects"] =
                        serde_json::json!([{"name":"Console.stdout_write", "args":[]}])
                }
                _ => unreachable!(),
            }
            return true;
        }
        match value {
            serde_json::Value::Object(fields) => {
                fields.values_mut().any(|value| alter(value, case))
            }
            serde_json::Value::Array(values) => values.iter_mut().any(|value| alter(value, case)),
            _ => false,
        }
    }
    let (saved, artifact) = save_checkpoint_at(HIGHER_ORDER, &[None], 19, 1, 0);
    let (checked, _, revisions) = restore_captured(HIGHER_ORDER, &saved);
    for case in 0..6 {
        let mut forged = artifact.clone();
        assert!(
            alter(&mut forged, case),
            "specialization must be present in the saved checkpoint"
        );
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &forged,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        let message = error.message();
        assert!(
            message.contains("effect")
                || message.contains("signature")
                || message.contains("interface")
                || (case == 5 && message.contains("native application was not prepared")),
            "case {case}: {error}"
        );
    }
}

#[test]
fn a_frozen_factory_preserves_the_instantiation_of_its_returned_callable() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow identity<T>(value: T) -> T ![] { return value; }
flow factory() -> ((i32) -> i32 ![]) { return identity; }
agent Worker() -> ((i32) -> i32 ![]) { return factory(); }
flow main() -> i32 {
    let selected = compile(^Worker);
    let convert = selected();
    checkpoint("frozen instantiated source callback");
    return convert(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_can_return_an_instantiated_source_callable_and_restore_its_binding() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow identity<T>(value: T) -> T ![] { return value; }
agent Worker() -> ((i32) -> i32 ![]) { return identity; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let convert = selected();
    checkpoint("instantiated source callback");
    return convert(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_preserves_distinct_instantiations_of_a_returned_standard_callable() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
import std.option.unwrap;
let as_int: (Option<i32>) -> i32 = unwrap;
let as_text: (Option<string>) -> string = unwrap;
type Callbacks = {
    number: (Option<i32>) -> i32,
    text: (Option<string>) -> string,
};
agent Worker() -> Callbacks { return Callbacks { number = as_int, text = as_text }; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let callbacks = selected();
    checkpoint("instantiated standard callbacks");
    let number = callbacks.number;
    let text = callbacks.text;
    if text(Some("ok")) == "ok" { return number(Some(19)); }
    return 0;
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_can_return_a_top_level_standard_callable_and_restore_its_checked_abi() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
import std.text.lowercase;
let convert = lowercase;
agent Worker() -> ((string) -> string ![]) { return convert; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let convert = selected();
    checkpoint("standard callback");
    if convert("OK") == "ok" { return 19; }
    return 0;
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn returning_a_callback_does_not_require_its_host_adapter_until_invocation() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
import std.io.println;
flow write(value: i32) -> i32 ![] { println("not invoked")?; return value; }
let callback = write;
agent Worker() -> ((i32) -> i32 ![]) { return callback; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let unused = selected();
    checkpoint("uncalled value");
    return 19;
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_can_return_a_top_level_lambda_and_resume_its_body() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
let callback: (i32) -> i32 = (value: i32) => { checkpoint("returned lambda"); return value; };
agent Worker() -> ((i32) -> i32) { return callback; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let restored = selected();
    return restored(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_can_return_a_top_level_callable_binding_and_resume_its_body() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow pause(value: i32) -> i32 { checkpoint("returned binding"); return value; }
let callback = pause;
agent Worker() -> ((i32) -> i32) { return callback; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let restored = selected();
    return restored(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_revision_can_return_a_top_level_record_containing_a_callable() {
    let program = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
flow pause(value: i32) -> i32 { checkpoint("returned record"); return value; }
type Callbacks = { next: (i32) -> i32 };
let callbacks = Callbacks { next = pause };
agent Worker() -> Callbacks { return callbacks; }
flow main() -> i32 {
    let selected = compile(^Worker);
    let callbacks = selected();
    let next = callbacks.next;
    return next(19);
}
"#;
    let (saved, artifact) = save_checkpoint_at(program, &[None], 19, 1, 0);
    let (checked, definitions, revisions) = restore_captured(program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}
