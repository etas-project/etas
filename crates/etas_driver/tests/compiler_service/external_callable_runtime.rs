use super::*;

const SOURCES: &[(&str, &str)] = &[
    (
        "library/api.es",
        r#"module library.api;
public flow apply(callback: (i32) -> i32 ![], value: i32) -> i32 ![] {
    return callback(value);
}
public flow lookup() -> i32 ![] { return 7; }
"#,
    ),
    (
        "library/extra.es",
        "module library.extra; public flow lookup() -> i32 ![] { return 7; }",
    ),
];

#[test]
fn an_uninvoked_source_callable_links_dependencies_without_requesting_their_actions() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                "module library.api; public flow lookup() -> i32 ![] { return 7; }",
            ),
            (
                "library/extra.es",
                "module library.extra; import std.io.println; public flow lookup() -> i32 ![] { println(\"not executed\")?; return 7; }",
            ),
        ],
        "",
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
flow callback() -> i32 ![] { return lookup(); }
flow main() -> i32 {
    let unused = callback;
    return 19;
}
"#,
    )
    .unwrap();
    let compiled = dependency_namespaces::compile(&root);
    let checked = compiled.response.output.checked.as_ref().unwrap();
    assert!(
        checked.effects.item_effects[&checked.entry.unwrap()]
            .requested_actions
            .is_empty()
    );
    let callback = checked
        .items
        .iter()
        .find_map(|(item, declaration)| {
            declaration
                .symbol()
                .and_then(|symbol| checked.symbols.get(symbol))
                .is_some_and(|symbol| symbol.name == "callback")
                .then_some(item)
        })
        .unwrap();
    assert!(checked.reachability.analysis_items.contains(&callback));
    assert!(!checked.reachability.reachable_items.contains(&callback));
    dependency_namespaces::execute(compiled, 2, 19);
}

#[test]
fn a_reflected_definition_local_is_not_an_executable_agent_reference() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
agent Worker() -> i32 ![] { return lookup(); }
flow main() -> i32 {
    let definition = ^Worker;
    if definition.body != None { return -1; }
    return 19;
}
"#,
    )
    .unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 19);
}

#[test]
fn higher_order_source_callbacks_materialize_their_dependency_implementations() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.apply;
import first.library.extra.lookup;
import std.compiler.{compile, CompileError};
flow callback(value: i32) -> i32 ![] { return lookup() + value; }
agent Worker(value: i32) -> i32 ![] { return apply(callback, value); }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return revised(12);
}
"#,
    )
    .unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 3, 19);
}

#[test]
fn compiling_a_dependency_calling_candidate_does_not_require_an_immediate_invocation() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return lookup(); }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return 19;
}
"#,
    )
    .unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 19);
}

#[test]
fn an_edited_candidate_can_call_a_locked_dependency_not_used_by_its_original_body() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 ![] { return 0; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return lookup();") {
        Ok(body) => body,
        Err(_) => { return -1; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised();
}
"#,
    )
    .unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 7);
}

#[test]
fn a_candidate_with_unlinked_runtime_sources_is_not_published_as_executable() {
    use etas_frontend::code::service::CandidatePublisher;
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
agent Worker() -> i32 ![] { return lookup(); }
flow main() -> i32 { let definition = ^Worker; return 0; }
"#,
    )
    .unwrap();
    let compiled = dependency_namespaces::compile(&root);
    assert!(compiled.response.output.diagnostics.is_empty());
    let checked = compiled.response.output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &compiled.loaded,
        &checked,
        Default::default(),
    )
    .unwrap();
    let reference = definitions
        .reflect_agent(*checked.types.reflections.keys().next().unwrap())
        .unwrap();
    let candidate = definitions
        .compile_definition(
            &reference,
            &mut etas_frontend::FrontendSession::new(),
            Default::default(),
        )
        .unwrap();
    assert!(
        !candidate
            .program()
            .reachability
            .runtime_source_requirements
            .dependencies
            .is_empty()
    );
    let error = etas_interpreter::api::code::InterpreterCandidatePublisher
        .publish(candidate)
        .unwrap_err();
    assert!(
        matches!(error, etas_interpreter::api::code::CandidatePublicationError::UnmaterializedRuntimeSources { packages } if packages.len() == 1)
    );
}

#[test]
fn candidate_source_completion_executes_private_dependency_helpers_in_its_own_program() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; flow helper() -> i32 ![] { return 7; } public flow lookup() -> i32 ![] { return helper(); }",
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 ![] { return 0; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return lookup();") { Ok(body) => body, Err(_) => { return -1; }, };
    return compile((^Worker).with_body(body))();
}
"#,
    )
    .unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 7);
}

#[test]
fn completed_original_analysis_does_not_widen_frozen_requested_action_bounds() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), r#"module app.main;
import first.library.extra.lookup;
import std.io.println;
import std.compiler.{check, parse_body, CompileError};
agent Worker() -> i32 ![] { return lookup(); }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("println(\"new action\")?; return lookup();") { Ok(body) => body, Err(_) => { return -1; }, };
    let report = check((^Worker).with_body(body));
    if report.status != "rejected" { return -2; }
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::AdditionalRequestedActions" { return 19; }
    }
    return -3;
}
"#).unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 19);
}

#[test]
fn dynamic_candidate_source_completion_rejects_dependency_bytes_changed_after_the_lock() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.extra.lookup;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 ![] { return 0; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return lookup();") { Ok(body) => body, Err(_) => { return -1; }, };
    return compile((^Worker).with_body(body))();
}
"#,
    )
    .unwrap();
    let compiled = dependency_namespaces::compile(&root);
    let dependency = &compiled
        .loaded
        .package_environment
        .as_ref()
        .unwrap()
        .dependencies[0];
    let path = match &dependency.source {
        etas_package::ResolvedDependencySource::Path { path, .. }
        | etas_package::ResolvedDependencySource::Vendor { path, .. } => path,
        source => panic!("fixture requires a materialized local dependency: {source:?}"),
    };
    std::fs::write(
        root.join(path).join("src/library/extra.es"),
        "module library.extra; public flow lookup() -> i32 ![] { return 999; }",
    )
    .unwrap();
    let result = dependency_namespaces::execute_result(compiled, 1);
    assert_ne!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(999))
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("source checksum mismatch")),
        "{:?}",
        result.diagnostics
    );
}
