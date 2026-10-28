use super::revisions::execute;
use etas_interpreter::api::InterpValue;

fn assert_value(source: &str, expected: InterpValue) {
    let result = execute(source, Some(5_000));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&expected));
}

#[test]
fn local_console_handler_does_not_require_or_execute_the_default_host_boundary() {
    assert_value(
        r#"
module app;
import std.io.{println, IOError};
agent Worker() -> unit { return; }
flow main() -> bool ![Error<IOError>] {
    let result = handle { println("local"); 7 } with {
        Console.stdout_write(_) => { resume; }
    };
    return result == 7;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn same_named_actions_keep_their_checked_owner_identity_across_revisions() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
effect Other { action choose(value: UserId) -> UserId; }
agent Worker(value: UserId) -> UserId ![Query.choose] {
    return perform Query.choose(value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("action-owner");
    return handle { revised(id) == id } with {
        Other.choose(_) => { finish false; },
        Query.choose(value) => { resume value; },
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn compiler_action_forwarder_can_resume_with_a_report_from_the_parent_program() {
    assert_value(
        r#"
module app;
import std.compiler.{check, compile, CheckReport, CompileError};
agent Worker() -> i32 { return 1; }
agent Checker() -> CheckReport ![Error<CompileError>] { return check(^Worker); }
flow main() -> bool ![Error<CompileError>] {
    let report = check(^Worker);
    let revised = compile(^Checker);
    let selected = handle { revised() } with {
        Code.check(_) => { resume report; }
    };
    return selected.status == "accepted";
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn suspended_program_frames_remain_charged_to_the_shared_call_depth_limit() {
    let result = super::revisions::execute_with_limits(
        r#"
module app;
import std.compiler.{compile, CompileError};
effect Query { action next(value: i32) -> i32; }
agent Worker(value: i32) -> i32 ![Query.next] {
    return perform Query.next(value);
}
flow main() -> i32 ![Error<CompileError>, Query.next] {
    let revised = compile(^Worker);
    return handle { revised(0) } with {
        Query.next(value) => {
            let child = revised(value + 1);
            resume child;
        }
    };
}
"#,
        etas_interpreter::api::ExecutionLimits::new(
            std::num::NonZeroU32::new(32).unwrap(),
            std::num::NonZeroU64::new(20_000),
        )
        .unwrap(),
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("maximum interpreter call depth")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn standard_default_payload_is_preserved_when_an_outer_handler_finishes() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.io.{println, IOError};
agent Worker() -> bool ![Console.stdout_write, Error<IOError>] {
    println("payload");
    return false;
}
flow main() -> bool ![Error<CompileError>, Error<IOError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Console.stdout_write(text) => { finish text == "payload\n"; }
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn generic_lexical_handler_uses_its_runtime_type_binding_for_a_revision_action() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose<T>(value: T) -> T; }
agent Worker(value: UserId) -> UserId ![Query.choose<UserId>] {
    return perform Query.choose<UserId>(value);
}
flow handle_query<T>(callback: () -> T ![Query.choose<T>]) -> T {
    return handle { callback() } with {
        Query.choose<T>(value) => { resume value; }
    };
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("lexical-selector");
    return handle_query<UserId>(() => revised(id)) == id;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn action_resumes_across_two_independently_compiled_agent_values() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
agent Worker(value: UserId) -> UserId ![Query.choose] {
    return perform Query.choose(value);
}
agent Forwarder(worker: Worker, value: UserId) -> UserId ![Query.choose] {
    return worker(value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let forwarder = compile(^Forwarder);
    let id = UserId("two-agents");
    let selected = handle { forwarder(revised, id) } with {
        Query.choose(value) => { resume value; }
    };
    return selected == id;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn console_default_action_can_be_handled_by_the_calling_program_without_a_host() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.io.{println, IOError};
agent Worker() -> unit ![Console.stdout_write, Error<IOError>] { println("handled outside"); }
flow main() -> bool ![Error<CompileError>, Error<IOError>] {
    let revised = compile(^Worker);
    return handle { revised(); true } with {
        Console.stdout_write(_) => { resume; }
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn compiled_action_resumes_in_its_native_program_with_a_relocated_nominal_result() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
agent Worker(value: UserId) -> UserId ![Query.choose] {
    let selected = perform Query.choose(value);
    return selected;
}

flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("native-result");
    let selected = handle { revised(id) } with {
        Query.choose(value) => { resume value; }
    };
    return selected == id;
}
"#,
        Some(2_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn finish_discards_the_suspended_revision_suffix() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
effect Query { action choose() -> i32; action suffix() -> unit; }
agent Worker() -> i32 ![Query.choose, Query.suffix] {
    let result = perform Query.choose();
    perform Query.suffix();
    return result;
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Query.choose() => { finish 7; },
        Query.suffix() => { finish 99; },
    };
}
"#,
        InterpValue::i32(7),
    );
}

#[test]
fn repeated_actions_resume_the_same_native_frame_and_caller_continuation() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
effect Query { action next(value: i32) -> i32; }
agent Worker() -> i32 ![Query.next] {
    let first = perform Query.next(0);
    let second = perform Query.next(first);
    return first * 10 + second;
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    let result = handle { revised() } with {
        Query.next(value) => { resume value + 1; }
    };
    return result + 100;
}
"#,
        InterpValue::i32(112),
    );
}

#[test]
fn native_handler_precedes_a_matching_outer_program_handler() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
effect Query { action next(value: i32) -> i32; }
agent Worker() -> i32 {
    return handle { perform Query.next(1) } with {
        Query.next(value) => { resume value + 1; }
    };
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Query.next(_) => { finish 99; }
    };
}
"#,
        InterpValue::i32(2),
    );
}

#[test]
fn source_callback_action_resumes_through_candidate_and_root_programs() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
flow source(value: UserId) -> UserId ![Query.choose] {
    return perform Query.choose(value);
}
agent Worker(callback: (UserId) -> UserId ![Query.choose], value: UserId) -> UserId ![Query.choose] {
    let selected = callback(value);
    return selected;
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("roundtrip");
    let selected = handle { revised(source, id) } with {
        Query.choose(value) => { resume value; }
    };
    return selected == id;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn candidate_handler_can_resume_an_action_from_a_source_callback() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose(value: UserId) -> UserId; }
flow source(value: UserId) -> UserId ![Query.choose] {
    return perform Query.choose(value);
}
agent Worker(callback: (UserId) -> UserId ![Query.choose], value: UserId) -> UserId {
    return handle { callback(value) } with {
        Query.choose(selected) => { resume selected; }
    };
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("candidate-handler");
    return revised(source, id) == id;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn generic_action_preserves_the_checked_selector_and_payload_instantiation() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
effect Query { action choose<T>(value: T) -> T; }
agent Worker(value: UserId) -> UserId ![Query.choose<UserId>] {
    return perform Query.choose<UserId>(value);
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("generic-selector");
    let selected = handle { revised(id) } with {
        Query.choose<UserId>(value) => { resume value; }
    };
    return selected == id;
}
"#,
        InterpValue::Bool(true),
    );
}
