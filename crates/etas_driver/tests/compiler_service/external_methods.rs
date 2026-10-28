use super::*;
use etas_driver::compiler::prepare_native_definitions;
use etas_frontend::code::definition::StaticReflectionView;

#[test]
fn external_default_and_additional_method_reflection_uses_published_identities() {
    let workspace = origin_metadata::workspace_with_source_config(
        "module library.api; public agent Worker(value: bool) -> bool ![] { return value; } impl agent Worker { public flow explain(value: bool) -> bool { return !value; } private flow hidden() -> bool { return false; } } public flow lookup() -> i32 { return 7; }",
        "[reflection]\nexport_bodies = [\"library.api.Worker\", \"library.api.Worker.explain\"]\n",
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let headers = metadata
        .public_metadata
        .definition_descriptors
        .as_ref()
        .unwrap()
        .headers
        .as_ref()
        .unwrap();
    let header = &headers.agents[0];
    for alias in ["first", "second"] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        for (import, receiver) in [
            (
                format!("import {alias}.library.api.Worker as Remote;"),
                "Remote",
            ),
            (format!("import {alias}.library.api as Lib;"), "Lib.Worker"),
            (format!("import {alias}.library.api.*;"), "Worker"),
            (
                format!("import {alias}.library.api.{{Worker as Grouped}};"),
                "Grouped",
            ),
        ] {
            loaded.loaded.input.sources[0].text = format!(
                r#"
module app.main;
{import}
flow main() -> bool {{
    let definition = ^{receiver};
    let run = ^{receiver}.run;
    let explain = ^{receiver}.explain;
    let signature = match definition.signature {{ Some(signature) => signature, None => {{ return false; }}, }};
    let method = match definition.methods.get(0) {{ Some(method) => method, None => {{ return false; }}, }};
    return run.signature == signature && run.body == definition.body
        && explain.id == method.id && explain.body == method.body;
}}
"#
            );
            let checked = origin_metadata::check(&loaded);
            let definitions =
                prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
            let mut ids = Vec::new();
            for expression in checked.types.reflections.keys() {
                if let StaticReflectionView::Callable(callable) = definitions
                    .catalog()
                    .static_reflection(*expression)
                    .unwrap()
                {
                    ids.push(callable.id.clone());
                    assert!(definitions.catalog().item_owner(&callable.id).is_none());
                }
            }
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(header.default_entry.as_ref().unwrap()));
            assert!(ids.contains(&header.methods[0]));
            let host = no_host::NoHost::default();
            let result = runtime()
                .block_on(etas_interpreter::Interpreter.run_checked(
                    &checked,
                    etas_interpreter::api::EntryPoint {
                        item: checked.entry.unwrap(),
                    },
                    vec![],
                    &host,
                    etas_interpreter::api::RunOptions {
                        code_definitions: Some(Arc::new(
                            etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions),
                        )),
                        ..Default::default()
                    },
                ))
                .unwrap();
            assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
            assert_eq!(
                result.value(),
                Some(&etas_interpreter::api::InterpValue::Bool(true))
            );
            assert_eq!(host.calls(), 0);
        }
    }
}

#[test]
fn external_method_reflection_rejects_private_missing_and_unchecked_targets() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Worker; impl agent Worker { public flow explain() -> i32 { return 1; } private flow hidden() -> i32 { return 2; } } public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    for member in ["hidden", "missing", "run"] {
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import dep.library.api.Worker as Remote; flow main() -> unit {{ let method = ^Remote.{member}; return; }}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none(), "{member}");
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::TypeMismatch)),
            "{member}: {:?}",
            output.diagnostics
        );
    }
    loaded.loaded.input.sources[0].text = "module app.main; import dep.library.api.Worker as Remote; flow main() -> unit { let method = ^Remote.explain; return; }".into();
    for origin in &mut loaded.loaded.input.environment.external_definition_origins {
        if let Some(headers) = origin
            .descriptors
            .as_mut()
            .and_then(|set| set.headers.as_mut())
        {
            for agent in &mut headers.agents {
                agent.method_info = None;
            }
        }
    }
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
            && diagnostic.message.contains("method selection")),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn external_method_selection_preserves_inherent_precedence_and_spec_ambiguity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Left { flow explain() -> i32; flow echo() -> i32; }
public spec Right { flow echo() -> i32; }
public agent Worker;
impl agent Worker ~ Left {
    public flow explain() -> i32 { return 1; }
    public flow echo() -> i32 { return 2; }
}
impl agent Worker ~ Right { public flow echo() -> i32 { return 3; } }
impl agent Worker { public flow explain() -> bool { return true; } }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = "module app.main; import dep.library.api.Worker as Remote; flow main() -> unit { let method = ^Remote.explain; return; }".into();
    let checked = origin_metadata::check(&loaded);
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let header = &metadata
        .public_metadata
        .definition_descriptors
        .as_ref()
        .unwrap()
        .headers
        .as_ref()
        .unwrap()
        .agents[0];
    let selected = header
        .method_info
        .as_ref()
        .unwrap()
        .iter()
        .find(|method| method.inherent)
        .unwrap();
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let StaticReflectionView::Callable(callable) =
        definitions.catalog().static_reflection(expression).unwrap()
    else {
        panic!("callable");
    };
    assert_eq!(callable.id, selected.callable);
    loaded.loaded.input.sources[0].text = "module app.main; import dep.library.api.Worker as Remote; flow main() -> unit { let method = ^Remote.echo; return; }".into();
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("uniquely selected")),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn a_newly_compiled_local_body_can_reflect_an_external_method_without_source_overlay() {
    let workspace = origin_metadata::workspace_with_source_config(
        "module library.api; public agent Worker; impl agent Worker { public flow explain(value: bool) -> bool { return !value; } } public flow lookup() -> i32 { return 7; }",
        "[reflection]\nexport_bodies = [\"library.api.Worker.explain\"]\n",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.Worker as Remote;
import std.compiler.{compile, parse_body, CompileError};
agent Target(value: bool) -> bool ![] { return value && true; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let method = ^Remote.explain; match method.body { Some(_) => { return !value; }, None => { return value; }, }") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let next = compile((^Target).with_body(body));
    return next(false);
}

"#
    .into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.types.reflections.len(), 1);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

#[test]
fn external_method_reflection_follows_public_reexport_identity() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Worker; impl agent Worker { public flow explain() -> i32 { return 1; } } public flow lookup() -> i32 { return 7; }",
    );
    let library = workspace.0.join("library");
    std::fs::write(
        library.join("src/library/forward.es"),
        "module library.forward; public import library.api.Worker as Forwarded;",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &origin_metadata::options(&library),
        etas_driver::PackageMetadataRequest {
            package_root: library.clone(),
            bin: None,
        },
    )
    .unwrap();
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = "module app.main; import dep.library.forward.Forwarded as Remote; import dep.library.api.Worker as Original; flow main() -> unit { let method = ^Remote.explain; let original = ^Original.explain; return; }".into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let StaticReflectionView::Callable(callable) =
        definitions.catalog().static_reflection(expression).unwrap()
    else {
        panic!("callable");
    };
    let owner = callable.id.as_method().unwrap().implementation().subject();
    assert_eq!(owner.module(), ["library", "api"]);
    assert_eq!(owner.declaration(), ["Worker"]);
    for expression in checked.types.reflections.keys() {
        let StaticReflectionView::Callable(other) = definitions
            .catalog()
            .static_reflection(*expression)
            .unwrap()
        else {
            panic!("callable");
        };
        assert_eq!(other.id, callable.id);
    }
    assert_eq!(loaded.loaded.input.sources.len(), 1);
}

#[test]
fn external_default_method_visibility_is_not_inferred_from_agent_visibility() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Worker; impl agent Worker { private flow run() -> i32 { return 1; } } public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = "module app.main; import dep.library.api.Worker; flow main() -> unit { let method = ^Worker.run; return; }".into();
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("private")),
        "{:?}",
        output.diagnostics
    );
}
