use super::revision_checkpoint_values::{restore_captured, resume_captured, save_captured};
use etas_interpreter::api::InterpValue;

#[path = "revision_checkpoint_callbacks/tamper.rs"]
mod tamper;

#[test]
fn an_active_compiled_generic_method_restores_its_nominal_bindings() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow echo<T>(value: T) -> T { checkpoint("generic method"); return value; }
}

flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("checkpoint revision");
    if selected.echo<UserId>(id) == id { return 20; }
    return 0;
}
"#,
    );
}

#[test]
fn a_compiled_generic_method_restores_materialized_structured_result_types() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow wrap<T>(value: T) -> Option<T> { checkpoint("structured generic result"); return Some(value); }
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("restored aggregate");
    return match selected.wrap<UserId>(id) { Some(value) => { if value == id { return 20; } return 0; }, None => 0, };
}
"#,
    );
}

#[test]
fn a_generic_wrapper_restores_its_instantiation_before_a_compiled_method_returns() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow wrap<T>(value: T) -> Option<T> { checkpoint("wrapper instantiation"); return Some(value); }
}

flow forward<T>(worker: Worker, value: T) -> bool ![] {
    let values = [value];
    return match worker.wrap<Array<T>>(values) { Some(actual) => actual == values, None => false, };
}
flow main() -> i32 ![Error<CompileError>] {
    if forward<UserId>(compile(^Worker), UserId("wrapper restore")) { return 20; }
    return 0;
}
"#,
    );
}

#[test]
fn an_active_callback_restores_its_source_nominal_capture_and_result() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker(callback: () -> UserId ![]) -> UserId { return callback(); }
flow main() -> i32 ![Error<CompileError>] {
    let captured = UserId("source nominal");
    let callback: () -> UserId ![] = () => { checkpoint("nominal callback"); return captured; };
    let selected = compile(^Worker);
    if selected(callback) == captured { return 20; }
    return 0;
}
"#,
    );
}

#[test]
fn a_candidate_checkpoint_preserves_a_specialized_generic_source_callback() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
flow identity<T>(value: T) -> T { return value; }
agent Worker(callback: (UserId) -> UserId ![], value: UserId) -> UserId {
    checkpoint("generic callback");
    return callback(value);
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let value = UserId("generic nominal");
    if selected(identity, value) == value { return 20; }
    return 0;
}
"#,
    );
}

#[test]
fn a_compiled_generic_callback_restores_its_implicit_effect_row_binding() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] {
        checkpoint("implicit callback row");
        return callback(value);
    }
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let id = UserId("restored implicit row");
    if selected.apply((value: UserId) => value, id) == id { return 20; }
    return 0;
}
"#,
    );
}

fn round_trip(program: &str) {
    let (sources, artifact) = save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = restore_captured(program, &sources);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
    assert!(result.checkpoints.is_empty());
}

#[test]
fn a_candidate_checkpoint_retains_a_source_callback_and_its_capture() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(callback: () -> i32 ![]) -> i32 {
    checkpoint("callback local");
    return callback();
}
flow main() -> i32 ![Error<CompileError>] {
    let captured: i32 = 19;
    let callback = () => captured;
    let selected = compile(^Worker);
    return selected(callback) + 1;
}
"#,
    );
}

#[test]
fn a_checkpoint_retains_a_returned_candidate_closure() {
    round_trip(
        r#"
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
"#,
    );
}

#[test]
fn a_callback_returned_to_its_original_program_needs_no_obsolete_revision() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(callback: () -> i32 ![]) -> (() -> i32 ![]) { return callback; }
flow forward(callback: () -> i32 ![]) -> (() -> i32 ![]) ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected(callback);
}
flow main() -> i32 ![Error<CompileError>] {
    let captured: i32 = 19;
    let callback = forward(() => captured);
    checkpoint("back in source");
    return callback() + 1;
}
"#,
    );
}

#[test]
fn an_active_source_callback_checkpoints_across_both_program_boundaries() {
    round_trip(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(callback: () -> i32 ![]) -> i32 { return callback(); }
flow main() -> i32 ![Error<CompileError>] {
    let captured: i32 = 19;
    let callback: () -> i32 ![] = () => { checkpoint("inside callback"); return captured; };
    let selected = compile(^Worker);
    return selected(callback) + 1;
}
"#,
    );
}
