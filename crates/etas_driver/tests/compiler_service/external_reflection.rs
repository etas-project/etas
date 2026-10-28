use super::*;
use etas_code::descriptor::{GenericOwner, TypeInfo};
use etas_driver::compiler::prepare_native_definitions;
use etas_frontend::code::definition::StaticReflectionView;

#[test]
fn metadata_only_agent_reflection_preserves_revisions_without_inventing_native_environments() {
    use etas_code::{definition::CodeDefinitionOperations, syntax::CodeSyntaxOperations};
    use etas_frontend::code::definition::NativeDefinitionError;

    for bare in [false, true] {
        let declaration = if bare {
            "public agent Worker;"
        } else {
            "public agent Worker(value: i32) -> i32 ![] { return value; }"
        };
        let workspace = origin_metadata::workspace_with_source(&format!(
            "module library.api; {declaration} impl agent Worker {{ public flow explain(value: i32) -> i32 {{ return value; }} private flow hidden() -> i32 {{ return 42; }} }} public flow lookup() -> i32 {{ return 7; }}"
        ));
        let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
            .unwrap()
            .unwrap();
        let snapshot = &metadata
            .public_metadata
            .definition_descriptors
            .as_ref()
            .unwrap()
            .headers
            .as_ref()
            .unwrap()
            .snapshots
            .as_ref()
            .unwrap()
            .agents[0];
        for alias in ["first", "second"] {
            let mut loaded = origin_metadata::consumer(&workspace, alias);
            loaded.loaded.input.sources[0].text = format!(
                r#"
module app.main;
import {alias}.library.api.Worker as Remote;
flow main() -> bool {{
    let definition = ^Remote;
    let method = match definition.methods.get(0) {{
        Some(method) => method, None => {{ return false; }},
    }};
    return definition.methods.len() == 1 && definition.body == None
        && (definition.signature == None) == {bare}
        && method.body == None && method.effects.requested != None;
}}
"#
            );
            assert_eq!(loaded.loaded.input.sources.len(), 1);
            let checked = origin_metadata::check(&loaded);
            let definitions =
                prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
            let expression = *checked.types.reflections.keys().next().unwrap();
            let reflected = definitions.reflect_agent(expression).unwrap();
            let view = definitions.resolve(&reflected).unwrap();
            assert_eq!(view.revision(), &snapshot.revision_info());
            assert_eq!(view.dependency_info().unwrap(), snapshot.dependencies);
            assert!(matches!(
                view.environment(),
                Err(NativeDefinitionError::MissingSourceEnvironment)
            ));
            assert!(matches!(
                definitions.body_overlay(&reflected),
                Err(NativeDefinitionError::MissingSourceEnvironment)
            ));
            let methods = definitions.methods(&reflected).unwrap();
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].header.id.as_method().unwrap().name(), "explain");
            let body = definitions
                .syntax_operations()
                .parse_body("return value;")
                .unwrap();
            let draft = definitions.with_body(&reflected, body).unwrap();
            let edited = definitions.resolve(&draft).unwrap();
            assert_eq!(
                edited.revision().inputs().parent,
                Some(reflected.revision().key())
            );
            assert_eq!(edited.dependency_info().unwrap(), snapshot.dependencies);
            assert!(edited.current_analysis().is_err());
            assert!(matches!(
                edited.environment(),
                Err(NativeDefinitionError::MissingSourceEnvironment)
            ));
            assert!(view.body().is_none());

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
fn metadata_only_generic_callable_reflection_uses_its_own_graph_and_never_executes_the_body() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup<T>(value: T) -> T ![] { return value; }",
    );
    let mut identity = None;
    for alias in ["first", "second"] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import {alias}.library.api.lookup as renamed;
flow main() -> bool {{
    let definition = ^renamed;
    let parameter = match definition.signature.parameters.get(0) {{
        Some(parameter) => parameter, None => {{ return false; }},
    }};
    return definition.signature.generics.len() == 1
        && parameter.name == Some("value")
        && definition.signature.output == parameter.ty
        && definition.signature.declared_output == Some(parameter.ty)
        && definition.body == None
        && definition.effects.requested != None
        && definition.effects.default_handled != None
        && definition.determinism != None
        && definition.trace != None;
}}
"#
        );
        assert_eq!(loaded.loaded.input.sources.len(), 1);
        let checked = origin_metadata::check(&loaded);
        let definitions =
            prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
        let expr = *checked.types.reflections.keys().next().unwrap();
        let StaticReflectionView::Callable(header) =
            definitions.catalog().static_reflection(expr).unwrap()
        else {
            panic!("external callable descriptor");
        };
        let graph = definitions
            .catalog()
            .callable_descriptors(&header.id)
            .unwrap();
        let TypeInfo::Function(signature) = graph.type_info(header.signature).unwrap() else {
            panic!("external callable signature");
        };
        assert_eq!(signature.generics.len(), 1);
        assert_eq!(
            graph.binder(signature.generics[0]).unwrap().owner,
            GenericOwner::Declaration(header.id.as_declaration().unwrap().clone())
        );
        if let Some(identity) = &identity {
            assert_eq!(&header.id, identity);
        }
        identity = Some(header.id.clone());
        assert!(definitions.catalog().item_owner(&header.id).is_none());
        assert!(
            definitions
                .catalog()
                .retained_body(expr, definitions.syntax_operations())
                .unwrap()
                .is_none()
        );
        assert_ne!(graph, definitions.catalog().descriptors());

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
        assert!(
            matches!(
                result.events.as_slice(),
                [
                    etas_interpreter::api::WorkflowEvent::StepStarted(_),
                    etas_interpreter::api::WorkflowEvent::StepCompleted(_),
                ]
            ),
            "{:?}",
            result.events
        );
    }
}

#[test]
fn a_newly_compiled_body_can_reflect_a_frozen_external_callable_without_source_overlay() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup<T>(value: T) -> T ![] { return value; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.lookup as renamed;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: bool) -> bool ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let d = ^renamed; return d.body == None && d.signature.generics.len() == 1 && d.signature.parameters.len() == 1;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let next = compile((^Worker).with_body(body));
    return next(true);
}

"#.into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                etas_interpreter::api::WorkflowEvent::HostTrace(
                    etas_host::TraceEvent::HostRequestStarted {
                        kind: etas_host::HostRequestKind::Compiler,
                        ..
                    }
                )
            ))
            .count(),
        1
    );
}
