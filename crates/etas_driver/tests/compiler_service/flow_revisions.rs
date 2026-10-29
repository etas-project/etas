use super::*;
use etas_interpreter::api::InterpValue;

#[test]
fn flow_compile_contracts_survive_source_package_binary_projection_and_metadata_only_replay() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{compile, CompileError, CallableDef};
import std.io.println;
private flow hidden(value: i32) -> i32 ![] { println("frozen request")?; return value; }
public flow make() -> ((i32) -> i32 ![]) ![Error<CompileError>] { return compile(^hidden); }
public flow revise<F>(draft: CallableDef<F>) -> F ![Error<CompileError>] { let candidate = compile(draft); return candidate; }
public flow choose<F>(draft: CallableDef<F>, replacement: CallableDef<F>, replace: bool) -> F ![Error<CompileError>] {
    let other = compile(replacement);
    return handle { compile(draft) } with {
        Code.compile(_) => { if replace { finish other; } resume compile(draft); }
    };
}
public flow choose_value<F>(draft: CallableDef<F>, replacement: F) -> F {
    return handle { compile(draft) } with { Code.compile(_) => { finish replacement; } };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let producer = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
        .unwrap();
    let graph = producer.returned_callables.as_ref().unwrap();
    assert!(matches!(
        graph.sources.as_slice(),
        [etas_package_metadata::CallableValueSource::FrozenContract(
            _
        )]
    ));
    let chosen = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| {
            summary
                .item
                .last()
                .is_some_and(|name| name == "choose_value")
        })
        .unwrap();
    assert!(
        matches!(chosen.returned_callables.as_ref().unwrap().sources.as_slice(), [etas_package_metadata::CallableValueSource::Parameter(parameter)] if parameter.scope == etas_package_metadata::CallableValueScope::Root && parameter.position == 1)
    );
    let request = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "revise"))
        .unwrap()
        .handler_requests
        .as_ref()
        .unwrap()
        .actions[0]
        .signature
        .as_ref()
        .unwrap();
    assert_eq!(request.params.len(), 1);
    assert!(
        matches!(&request.output, etas_package::PackageTypeMetadata::Var { name } if name == "F")
    );
    for (expression, capture, console, denied) in [
        ("make()", true, true),
        ("revise(^local)", true, true),
        ("choose(^pure, ^local, true)", true, true),
        ("choose_value(^pure, local)", false, true),
        ("choose_value(^local, pure)", false, false),
        (
            "handle { revise(^local) } with { Code.compile(_) => { resume local; } }",
            false,
            true,
        ),
        (
            "handle { revise(^pure) } with { Code.compile(_) => { resume pure; } }",
            false,
            false,
        ),
    ]
    .into_iter()
    .flat_map(|(expression, capture, console)| {
        [false, true].map(move |denied| (expression, capture, console, denied))
    }) {
        let invocation = if capture {
            format!(
                "let result = {expression}?; return match result {{ Ok(candidate) => candidate(4), Err(_) => 0, }};"
            )
        } else {
            format!("let candidate = {expression}; return candidate(4);")
        };
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.{{make, revise, choose, choose_value}};
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow local(value: i32) -> i32 ![] {{ println("local frozen request")?; return value; }}
flow pure(value: i32) -> i32 ![] {{ return value; }}
flow main() -> i32 {} {{
    {invocation}
}}
"#,
            if denied { "~ Quiet" } else { "" }
        );
        let output = Frontend.check_project(loaded.loaded.input);
        if denied && console {
            assert!(
                output.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(
                        etas_core::EffectDiagnosticCode::TraceSpecDenied
                    )),
                "{:?}",
                output.diagnostics
            );
        } else {
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            let checked = output.checked.unwrap();
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(summary.escaping_effects.is_empty());
            assert_eq!(
                summary.requested_actions.effects.iter().count(),
                if console { 2 } else { 1 },
                "{summary:#?}"
            );
            assert_eq!(summary.default_actions.is_empty(), !console);
        }
    }
}

#[test]
fn ordinary_flow_direct_actions_use_real_reports_and_reject_invalid_candidates() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{parse_body, CompileError};
flow normalize(text: string) -> string ![] { return text; }
flow main() -> bool ![Error<CompileError>] {
    let original = ^normalize;
    let accepted = perform Code.check(original);
    let syntax = match parse_body("return 42;") { Ok(body) => body, Err(_) => { return false; }, };
    let draft = original.with_body(syntax);
    let rejected = perform Code.check(draft);
    let result = (perform Code.compile(draft))?;
    return accepted.status == "accepted" && rejected.status == "rejected"
        && match result { Err(error) => error.kind == "rejected", Ok(_) => false, };
}
"#,
        &["app.normalize"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_compile_publishes_without_invoking_the_candidate() {
    for expression in ["compile(^target)", "perform Code.compile(^target)"] {
        let prepared = source::prepared_with_bodies(
            &format!(
                r#"
module app;
import std.compiler.{{compile, CompileError}};
flow target() -> i32 ![] {{ return abort("compilation must not invoke this body"); }}
flow main() -> bool ![Error<CompileError>] {{ let candidate = {expression}; return true; }}
"#
            ),
            &["app.target"],
        );
        let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
        assert!(
            result.diagnostics.is_empty(),
            "{expression}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    }
}

#[test]
fn successive_flow_revisions_keep_the_original_contract_and_pinned_aliases() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, check, parse_body, CompileError};
flow target(flag: bool) -> bool ![] { return flag; }
flow main() -> bool ![Error<CompileError>] {
    var current = target;
    let original = current;
    let first_body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return false; }, };
    current = compile((^current).with_body(first_body));
    let first = current;
    let first_view = ^current;
    let second_body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return false; }, };
    let second_draft = first_view.with_body(second_body);
    if check(second_draft).status != "accepted" { return false; }
    current = compile(second_draft);
    let second_view = ^current;
    let first_text = match first_view.body { Some(body) => body.text, None => { return false; }, };
    let second_text = match second_view.body { Some(body) => body.text, None => { return false; }, };
    return original(true) && !first(true) && current(false)
        && first_view.id == second_view.id
        && first_view.revision.key != second_view.revision.key
        && first_view.dependencies == second_view.dependencies
        && first_text != second_text;
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_source_compile_handler_can_resume_a_pinned_compatible_flow_revision() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
flow target(flag: bool) -> bool ![] { return flag; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return false; }, };
    let pinned = compile((^target).with_body(body));
    let current = handle { compile(^target) } with { Code.compile(_) => { resume pinned; } };
    return !current(true) && target(true);
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_source_edit_check_compile_and_call_preserve_old_bindings() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, check, parse_body, CompileError};
flow normalize(text: string) -> string ![] { return text; }
flow main() -> bool ![Error<CompileError>] {
    var current = normalize;
    let old = current;
    let syntax = match parse_body("return \"changed\";") { Ok(body) => body, Err(_) => { return false; }, };
    let draft = (^normalize).with_body(syntax);
    let report = check(draft);
    if report.status != "accepted" { return false; }
    current = compile(draft);
    return current("input") == "changed" && old("input") == "input" && normalize("input") == "input";
}
"#,
        &["app.normalize"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn compiled_flow_source_preserves_nominal_arguments_and_results() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
flow identity(value: UserId) -> UserId ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let syntax = match parse_body("return value;") { Ok(body) => body, Err(_) => { return false; }, };
    let candidate = compile((^identity).with_body(syntax));
    let value = UserId("native");
    return candidate(value) == value;
}
"#,
        &["app.identity"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}
