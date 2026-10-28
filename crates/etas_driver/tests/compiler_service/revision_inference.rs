use super::revisions::execute;
use etas_interpreter::api::InterpValue;

fn assert_value(source: &str, expected: InterpValue) {
    let result = execute(source, Some(5_000));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&expected));
}

#[test]
fn compiled_inference_resumes_a_nominal_output_from_its_callers_handler() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> UserId { return perform infer<UserId>(Prompt.new()); }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let id = UserId("handled inference");
    let selected = handle { revised() } with {
        Agentic.infer<_, UserId>(_, _) => { resume id; }
    };
    return selected == id;
}
"#,
        Some(5_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn repeated_inference_resumes_each_native_continuation_without_a_model_request() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 {
    let first = perform infer<i32>(Prompt.new());
    let second = perform infer<i32>(Prompt.new());
    return first + second;
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Agentic.infer<_, i32>(_, _) => { resume 7; }
    };
}
"#,
        InterpValue::i32(14),
    );
}

#[test]
fn inference_finish_discards_the_native_program_suffix() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 {
    let selected = perform infer<i32>(Prompt.new());
    return abort("a finished inference must not resume the candidate");
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Agentic.infer<_, i32>(_, _) => { finish 23; }
    };
}
"#,
        InterpValue::i32(23),
    );
}

#[test]
fn inference_handler_matches_the_selected_callable_not_a_similar_agent() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> string { return perform infer<string>(Prompt.new()); }
agent Other() -> string { return perform infer<string>(Prompt.new()); }
flow main() -> string ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Agentic.infer<Other.run, string>(_, _) => { finish "wrong owner"; },
        Agentic.infer<Worker.run, string>(_, _) => { resume "selected method"; }
    };
}
"#,
        InterpValue::String("selected method".into()),
    );
}

#[test]
fn inference_crosses_two_independently_compiled_agents() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Worker() -> UserId { return perform infer<UserId>(Prompt.new()); }
agent Forwarder(worker: Worker) -> UserId { return worker(); }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let forwarder = compile(^Forwarder);
    let expected = UserId("nested native programs");
    let selected = handle { forwarder(revised) } with {
        Agentic.infer<_, UserId>(_, _) => { resume expected; }
    };
    return selected == expected;
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn a_candidate_handler_can_resume_inference_from_a_source_callback() {
    assert_value(&source_callback(" ![]"), InterpValue::Bool(true));
}

#[test]
fn compiled_implicit_callback_effect_row_retains_its_latent_behavior() {
    assert_value(&source_callback(""), InterpValue::Bool(true));
}

#[test]
fn a_changed_callback_cannot_hide_effects_behind_a_frozen_helper_summary() {
    let result = execute(
        r#"
module app;
import std.compiler.{check, parse_body, CompileError};
effect Probe extends Network { action ping() -> unit; }
flow invoke(callback: () -> unit) -> unit { callback(); return; }
agent Worker() -> unit { invoke(() => { return; }); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("invoke(() => { perform Probe.ping(); return; }); return;") {
        Ok(body) => body,
        Err(_) => { return "parse rejected"; },
    };
    let report = check((^Worker).with_body(body));
    var diagnostics = report.status;
    for diagnostic in report.diagnostics limit Iterations(64) {
        diagnostics = diagnostics + "|" + diagnostic.code + ": " + diagnostic.message;
    }
    return diagnostics;
}
"#,
        Some(5_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let Some(InterpValue::String(diagnostics)) = result.value() else {
        panic!("missing report: {result:?}")
    };
    assert!(diagnostics.starts_with("rejected|"), "{diagnostics}");
    assert!(diagnostics.contains("effect::IncompleteEffectFacts: callback behavior is incompatible with the checked inference used by the frozen callable summary"), "{diagnostics}");
}

fn source_callback(annotation: &str) -> String {
    r#"
module app;
import std.compiler.{compile, CompileError};
type UserId = string;
agent Source() -> UserId { return perform infer<UserId>(Prompt.new()); }
agent Worker(callback: () -> UserId@ROW@, expected: UserId) -> UserId {
    return handle { callback() } with {
        Agentic.infer<_, UserId>(_, _) => { resume expected; }
    };
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let expected = UserId("source callback");
    return revised(() => Source(), expected) == expected;
}
"#
    .replace("@ROW@", annotation)
}

#[test]
fn a_custom_inference_handler_does_not_require_a_provider_schema_for_a_recursive_adt() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
enum Tree<T> { Leaf(T), Branch(Tree<T>, Tree<T>), }
agent Worker() -> Tree<string> { return perform infer<Tree<string>>(Prompt.new()); }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let selected = handle { revised() } with {
        Agentic.infer<_, Tree<string>>(_, _) => {
            resume Tree.Branch(Tree.Leaf("left"), Tree.Leaf("right"));
        }
    };
    return match selected {
        Tree.Leaf(_) => false,
        Tree.Branch(left, _) => match left {
            Tree.Leaf(value) => value == "left",
            Tree.Branch(_, _) => false,
        },
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn an_unhandled_compiled_inference_still_requires_a_real_model_service() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> string { return perform infer<string>(Prompt.new()); }
flow main() -> string ![Error<CompileError>] {
    let revised = compile(^Worker);
    return revised();
}

"#,
        Some(5_000),
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("missing Phase 1 host service for Agentic")
        }),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn a_compiled_auxiliary_inference_preserves_its_selected_method_identity() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> string { return "default entry"; }
impl agent Worker {
    flow explain() -> string { return perform infer<string>(Prompt.new()); }
}
flow main() -> string ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised.explain() } with {
        Agentic.infer<Worker.run, string>(_, _) => { finish "wrong method"; },
        Agentic.infer<Worker.explain, string>(_, _) => { resume "selected auxiliary"; }
    };
}
"#,
        InterpValue::String("selected auxiliary".into()),
    );
}
