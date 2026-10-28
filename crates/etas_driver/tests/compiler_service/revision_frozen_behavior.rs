use super::revisions::execute;
use etas_interpreter::api::InterpValue;

fn succeeds(source: &str) {
    let result = execute(source, Some(2_000));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn an_edited_factory_can_return_its_frozen_original_closure_without_the_old_body() {
    succeeds(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
agent Worker(value: UserId) -> (() -> UserId ![]) { return () => value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return Worker(value);") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let value = UserId("frozen closure");
    let callback = revised(value);
    return callback() == value;
}
"#,
    );
}

#[test]
fn a_local_alias_preserves_the_frozen_revision_of_a_returned_callable() {
    succeeds(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: i32) -> (() -> i32 ![]) { return () => value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let original = Worker; return original(value);") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    return revised(19)() == 19;
}
"#,
    );
}

#[test]
fn frozen_nested_closures_keep_their_original_callback_binding() {
    succeeds(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
agent Worker(callback: () -> UserId ![]) -> (() -> (() -> UserId ![]) ![]) {
    return () => { return () => callback(); };
}
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return Worker(callback);") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let value = UserId("nested frozen callback");
    return revised(() => value)()() == value;
}
"#,
    );
}

#[test]
fn replacing_the_callback_does_not_preserve_the_frozen_origin_binding() {
    succeeds(
        r#"
module app;
import std.compiler.{check, parse_body};
agent Worker(callback: () -> i32 ![]) -> (() -> i32 ![]) { return () => callback(); }
flow main() -> bool {
    let body = match parse_body("return Worker(() => { return 9; });") { Ok(body) => body, Err(_) => { return false; }, };
    let report = check((^Worker).with_body(body));
    return report.status == "rejected";
}
"#,
    );
}

#[test]
fn equal_effect_rows_do_not_prove_a_fresh_diverging_closure_is_compatible() {
    succeeds(
        r#"
module app;
import std.compiler.{check, parse_body};
agent Worker(value: i32) -> (() -> i32 ![]) { return () => value; }
flow main() -> bool {
    let body = match parse_body("return () => { return abort(\"diverging replacement\"); };") { Ok(body) => body, Err(_) => { return false; }, };
    let report = check((^Worker).with_body(body));
    return report.status == "rejected";
}
"#,
    );
}
