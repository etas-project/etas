use super::revisions::execute;
use etas_interpreter::api::InterpValue;

fn assert_success(source: &str) {
    let result = execute(source, Some(2_000));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_compiled_revision_executes_a_recursive_captured_callable_chain() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
flow wrap(callback: () -> i32 ![]) -> (() -> i32 ![]) { return () => callback(); }
flow chain(depth: i32) -> (() -> i32 ![]) {
    if depth == 0 { return () => 19; }
    return wrap(chain(depth - 1));
}
agent Worker(depth: i32) -> i32 ![] { return chain(depth)(); }
flow main() -> bool ![Error<CompileError>] { return compile(^Worker)(20) == 19; }
"#,
    );
}

#[test]
fn compiled_auxiliary_generic_method_preserves_nominal_type_bindings() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow echo<T>(value: T) -> T ![] { return value; } }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("revision");
    return revised.echo<UserId>(id) == id && revised.echo<string>("second") == "second";
}
"#,
    );
}

#[test]
fn compiled_generic_method_preserves_its_effect_row_and_typed_error() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow fail<T, effect E>(value: T) -> T ![Error<T>, E] { perform Error<T>.raise(value); }
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("typed generic error");
    return handle { revised.fail<UserId, ![]>(id); false } with {
        Error<UserId>.raise(error) => { finish error == id; }
    };
}
"#,
    );
}

#[test]
fn a_candidate_can_invoke_a_frozen_generic_method() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
agent Worker(value: UserId) -> UserId ![] { return value; }
impl agent Worker { flow echo<T>(value: T) -> T ![] { return value; } }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return Worker.echo<UserId>(value);") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let id = UserId("frozen generic");
    return revised(id) == id;
}
"#,
    );
}

#[test]
fn compiled_generic_method_projects_structured_results_and_keeps_spec_evidence() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
spec Allowed;
type UserId = string;
impl UserId ~ Allowed;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow wrap<T ~ Allowed>(value: T) -> Option<T> ![] { return Some(value); }
}

flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("structured generic result");
    return match revised.wrap<UserId>(id) { Some(value) => value == id, None => false, };
}
"#,
    );
}

#[test]
fn compiled_generic_method_relocates_nested_aggregate_type_arguments() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T>(value: T) -> Option<T> ![] { return Some(value); } }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let values = [UserId("nested")];
    return match revised.wrap<Array<UserId>>(values) { Some(actual) => actual == values, None => false, };
}
"#,
    );
}

#[test]
fn compiled_generic_method_infers_nominal_bindings_without_explicit_arguments() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
spec Allowed;
type UserId = string;
impl UserId ~ Allowed;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T ~ Allowed>(value: T) -> Option<T> ![] { return Some(value); } }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("inferred nominal");
    let result = revised.wrap(id);
    return match result { Some(value) => value == id, None => false, };
}
"#,
    );
}

#[test]
fn compiled_generic_method_inherits_wrapper_instantiation_bindings() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T>(value: T) -> Option<T> ![] { return Some(value); } }
flow forward<T>(worker: Worker, value: T) -> Option<T> ![] { return worker.wrap<T>(value); }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("generic wrapper");
    return match forward<UserId>(revised, id) { Some(value) => value == id, None => false, };
}
"#,
    );
}

#[test]
fn compiled_generic_method_materializes_internal_types_through_multiple_wrappers() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T>(value: T) -> Option<T> ![] { return Some(value); } }
flow leaf<U>(worker: Worker, value: U) -> bool ![] {
    let values = [value];
    return match worker.wrap<Array<U>>(values) { Some(actual) => actual == values, None => false, };
}
flow forward<T>(worker: Worker, value: T) -> bool ![] { return leaf<T>(worker, value); }
flow main() -> bool ![Error<CompileError>] {
    return forward<UserId>(compile(^Worker), UserId("internal aggregate"));
}
"#,
    );
}

#[test]
fn compiled_generic_method_substitutes_wrapper_effect_binders_by_scope() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T, effect E>(value: T) -> Option<T> ![E] { return Some(value); } }
flow forward<U, effect F>(worker: Worker, value: U) -> Option<U> ![F] {
    return worker.wrap<U, ![F]>(value);
}
flow main() -> bool ![Error<CompileError>] {
    let id = UserId("scoped row");
    return match forward<UserId, ![]>(compile(^Worker), id) { Some(value) => value == id, None => false, };
}
"#,
    );
}

#[test]
fn compilation_retains_deferred_effect_row_instantiations() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
flow twice<effect E>(callback: () -> unit ![E]) -> unit ![E] { callback(); callback(); }
agent Worker() -> bool ![] { return true; }
flow main() -> bool ![Error<CompileError>] {
    twice(() => { return; });
    return compile(^Worker)();
}
"#,
    );
}

#[test]
fn compiled_generic_method_inherits_a_wrapper_selected_through_a_callable_alias() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow wrap<T>(value: T) -> Option<T> ![] { return Some(value); } }
flow forward<T>(worker: Worker, value: T) -> Option<T> ![] { return worker.wrap<T>(value); }
flow main() -> bool ![Error<CompileError>] {
    let forwarder = forward;
    let id = UserId("aliased wrapper");
    return match forwarder(compile(^Worker), id) { Some(value) => value == id, None => false, };
}
"#,
    );
}

#[test]
fn compiled_generic_method_preserves_the_callers_same_named_effect_row() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return 1; }
impl agent Worker { flow factory<effect E>() -> (() -> unit ![E]) ![] { return () => { return; }; } }
flow outer<effect E>(worker: Worker) -> (() -> unit ![Console.stdout_write, E]) ![] {
    return worker.factory<![Console.stdout_write, E]>();
}
flow main() -> bool ![Error<CompileError>] {
    let callback = outer<![]>(compile(^Worker));
    callback();
    return true;
}
"#,
    );
}

#[test]
fn compiled_generic_method_infers_an_implicit_effect_row_from_a_callback() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] { return callback(value); }
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("implicit row");
    return revised.apply((value: UserId) => value, id) == id;
}
"#,
    );
}

#[test]
fn compiled_generic_method_infers_a_nonempty_callback_row_with_a_scoped_handler() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Echo { action send(value: UserId) -> UserId; }
agent Worker() -> i32 ![] { return 1; }
impl agent Worker {
    flow apply<T, effect E>(callback: (T) -> T ![E], value: T) -> T ![E] {
        return callback(value);
    }
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("nonempty implicit row");
    return handle {
        revised.apply((value: UserId) => perform Echo.send(value), id) == id
    } with {
        Echo.send(value) => { resume value; }
    };
}
"#,
    );
}

#[test]
fn revision_callback_keeps_its_source_capture_and_nominal_identity() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker(callback: () -> UserId ![]) -> UserId { return callback(); }
flow main() -> bool ![Error<CompileError>] {
    let captured = UserId("source");
    let callback = () => captured;
    let revised = compile(^Worker);
    return revised(callback) == captured;
}
"#,
    );
}

#[test]
fn revision_returned_closure_keeps_its_candidate_capture_and_nominal_identity() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker(value: UserId) -> (() -> UserId ![]) { return () => value; }
flow main() -> bool ![Error<CompileError>] {
    let captured = UserId("candidate");
    let revised = compile(^Worker);
    let callback = revised(captured);
    return callback() == captured;
}
"#,
    );
}

#[test]
fn revision_returned_closure_can_cross_a_second_revision() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Factory(value: UserId) -> (() -> UserId ![]) { return () => value; }
agent Worker(callback: () -> UserId ![]) -> UserId { return callback(); }
flow main() -> bool ![Error<CompileError>] {
    let captured = UserId("nested");
    let factory = compile(^Factory);
    let callback = factory(captured);
    let revised = compile(^Worker);
    return revised(callback) == captured;
}
"#,
    );
}

#[test]
fn revision_accepts_a_specialized_generic_source_callable() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow identity<T>(value: T) -> T { return value; }
agent Worker(callback: (UserId) -> UserId ![], value: UserId) -> UserId {
    return callback(value);
}
flow main() -> bool ![Error<CompileError>] {
    let captured = UserId("generic");
    let revised = compile(^Worker);
    return revised(identity, captured) == captured;
}
"#,
    );
}

#[test]
fn revision_keeps_independent_instantiations_of_a_generic_callable_alias() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow identity<T>(value: T) -> T { return value; }
agent Worker(callback: (UserId) -> UserId ![], value: UserId) -> UserId {
    return callback(value);
}
flow main() -> bool ![Error<CompileError>] {
    let nominal_identity = identity;
    let scalar_identity = identity;
    let revised = compile(^Worker);
    let captured = UserId("alias");
    return revised(nominal_identity, captured) == captured && scalar_identity("scalar") == "scalar";
}
"#,
    );
}

#[test]
fn revision_generic_callable_keeps_its_instantiated_error_selector() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow fail<T>(value: T) -> unit ![Error<T>] { perform Error<T>.raise(value); }
agent Worker(callback: (UserId) -> unit ![Error<UserId>], value: UserId) -> unit ![Error<UserId>] {
    callback(value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("typed-error");
    return handle { revised(fail, id); false } with {
        Error<UserId>.raise(error) => { finish error == id; }
    };
}
"#,
    );
}

#[test]
fn revision_catches_callback_error_before_the_source_handler() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow fail<T>(value: T) -> unit ![Error<T>] { perform Error<T>.raise(value); }
agent Worker(callback: (UserId) -> unit ![Error<UserId>], value: UserId) -> bool {
    return handle { callback(value); false } with {
        Error<UserId>.raise(error) => { finish error == value; }
    };
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("inner-handler");
    return handle { revised(fail, id) } with {
        Error<UserId>.raise(_) => { finish false; }
    };
}
"#,
    );
}

#[test]
fn revision_error_is_captured_by_the_callers_try_boundary() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow fail<T>(value: T) -> unit ![Error<T>] { perform Error<T>.raise(value); }
agent Worker(callback: (UserId) -> unit ![Error<UserId>], value: UserId) -> unit ![Error<UserId>] {
    callback(value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("try-boundary");
    let result = revised(fail, id)?;
    return match result { Ok(_) => false, Err(error) => error == id, };
}
"#,
    );
}

#[test]
fn revision_error_keeps_nominal_identity_across_two_compiled_agents() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
type OtherId = string;
agent Inner(value: UserId) -> unit ![Error<UserId>] { perform Error<UserId>.raise(value); }
agent Worker(inner: Inner, value: UserId) -> unit ![Error<UserId>] { inner(value); }
flow main() -> bool ![Error<CompileError>] {
    let first = compile(^Inner);
    let second = compile(^Worker);
    let id = UserId("two-revisions");
    return handle { second(first, id); false } with {
        Error<OtherId>.raise(_) => { finish false; },
        Error<UserId>.raise(error) => { finish error == id; }
    };
}
"#,
    );
}

#[test]
fn revision_generic_handler_uses_its_lexical_type_binding() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
flow fail<T>(value: T) -> unit ![Error<T>] { perform Error<T>.raise(value); }
flow recover<T>(callback: (T) -> unit ![Error<T>], value: T) -> bool {
    return handle { callback(value); false } with {
        Error<T>.raise(_) => { finish true; }
    };
}
agent Worker(callback: (UserId) -> unit ![Error<UserId>], value: UserId) -> bool {
    return recover(callback, value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    return revised(fail, UserId("generic-handler"));
}
"#,
    );
}

#[test]
fn forwarded_agent_values_keep_independently_pinned_implementations() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
agent Inner(value: UserId) -> UserId { return value; }
agent Worker(inner: Inner, value: UserId) -> UserId { return inner(value); }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return UserId(\"edited\");") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let edited = compile((^Inner).with_body(body));
    let original = compile(^Inner);
    let revised = compile(^Worker);
    let id = UserId("original");
    return revised(edited, id) == UserId("edited")
        && revised(original, id) == id
        && edited(id) == UserId("edited");
}
"#,
    );
}

#[test]
fn returned_agent_values_keep_their_executable_revision() {
    assert_success(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Inner(value: UserId) -> UserId { return value; }
agent Worker(inner: Inner) -> Inner { return inner; }
flow main() -> bool ![Error<CompileError>] {
    let inner = compile(^Inner);
    let revised = compile(^Worker);
    let returned = revised(inner);
    let id = UserId("returned-agent");
    return returned(id) == id && inner(id) == id;
}
"#,
    );
}
