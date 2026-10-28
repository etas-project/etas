use super::revisions::execute;

#[test]
fn revised_body_uses_every_anonymous_return_of_an_unchanged_source_factory() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
flow choose(flag: bool, value: UserId) -> (() -> UserId ![]) {
    if flag { let selected = () => value; return selected; }
    return () => UserId("other");
}
agent Worker(value: UserId) -> UserId ![] { return choose(false, value)(); }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let selected = choose(true, value); return selected();") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let revised = compile((^Worker).with_body(body));
    let value = UserId("captured");
    return revised(value) == value && Worker(value) == UserId("other");
}
"#,
        Some(2_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

#[test]
fn unedited_agent_rebinds_anonymous_returns_by_native_body_identity() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker(flag: bool, value: UserId) -> (() -> UserId ![]) {
    if flag { let selected = () => value; return selected; }
    return () => UserId("other");
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let value = UserId("captured");
    let first = revised(true, value);
    let second = revised(false, value);
    return first() == value && second() == UserId("other");
}
"#,
        Some(2_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}
