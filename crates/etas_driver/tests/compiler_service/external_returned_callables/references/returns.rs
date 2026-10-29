use super::*;

#[test]
fn a_local_returned_generic_reference_executes_with_composed_type_arguments() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
private flow identity<T>(value: T) -> T ![] { return value; }
private flow original<T>() -> ((T) -> T ![]) ![] { return identity; }
private flow make<U>() -> ((Array<U>) -> Array<U> ![]) ![] { return original<Array<U>>(); }
flow main() -> Array<i32> ![] { let callback = make<i32>(); return callback([2]); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            Default::default(),
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let Some(etas_interpreter::api::InterpValue::Array(values)) = result.value() else {
        panic!("unexpected result: {:?}", result.value());
    };
    let values = values.borrow();
    assert_eq!(values.len(), 1);
    assert_eq!(
        etas_interpreter::api::codec::value_json(&values[0]).unwrap(),
        serde_json::json!({"kind": "number", "type": "i32", "value": "2"})
    );
    assert_eq!(host.calls(), 0);
}

#[test]
fn an_instantiated_reference_can_return_another_generic_closure() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow original<T>() -> ((T) -> T ![Error<T>]) ![] {
    return (value: T) => { return perform Error<T>.raise(value); };
}
public flow make<U>() -> (() -> ((U) -> U ![Error<U>]) ![]) ![] { return original; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> i32 ![Error<i32>] {
    let factory = make<i32>();
    let callback = factory();
    return callback(2);
}
"#
    .into();
    assert_i32_error(&origin_metadata::check(&loaded));
}

#[test]
fn a_returned_reference_composes_effect_row_arguments_and_callback_traces() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow twice<effect E>(callback: () -> unit ![E]) -> unit ![E] { callback(); callback(); }
private flow original<effect E>() -> ((() -> unit ![E]) -> unit ![E]) ![] { return twice; }
public flow make<effect F>() -> ((() -> unit ![F]) -> unit ![F]) ![] { return original<![F]>(); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let artifact = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let graph = artifact
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
        .and_then(|summary| summary.returned_callables.as_ref())
        .expect("generic returned callable must survive binary metadata projection");
    assert!(
        graph.values.iter().any(|node| {
            let mut pending = vec![&node.action_trace];
            while let Some(trace) = pending.pop() {
                match trace {
                    etas_package_metadata::ActionTrace::ParameterCall { .. } => {
                        return node.behavior.independent_determinism
                            == Some(
                                etas_package_metadata::CallableValueDeterminism::Deterministic,
                            );
                    }
                    etas_package_metadata::ActionTrace::Seq(parts)
                    | etas_package_metadata::ActionTrace::Choice(parts) => pending.extend(parts),
                    etas_package_metadata::ActionTrace::Repeat(part) => pending.push(part),
                    etas_package_metadata::ActionTrace::Widened {
                        parameter_calls, ..
                    } if !parameter_calls.is_empty() => {
                        return node.behavior.independent_determinism
                            == Some(
                                etas_package_metadata::CallableValueDeterminism::Deterministic,
                            );
                    }
                    _ => {}
                }
            }
            false
        }),
        "metadata must preserve callback-dependent determinism: {graph:?}"
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> unit ![Console.stdout_write] {
    let callback = make<![Console.stdout_write]>();
    callback(() => { perform Console.stdout_write("test"); });
}

"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert_eq!(
        summary.determinism,
        etas_effects::Determinism::RuntimeMediated
    );
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(
        summary.escaping_effects.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    let mut count = 0;
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Event(event) => {
                count += usize::from(event.action == stdout)
            }
            etas_effects::ActionTraceDomain::Seq(parts)
            | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("unresolved reference trace: {other:?}"),
        }
    }
    assert_eq!(count, 2, "{summary:?}");
    assert_eq!(checked.sources.sources.len(), 1);
}

#[test]
fn external_generic_flow_preserves_parametric_determinism_through_binary_metadata() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow relay<effect E>(callback: () -> unit ![E]) -> unit ![E] { callback(); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let artifact = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let summary = artifact
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "relay"))
        .unwrap();
    let behavior = summary.behavior.as_ref().expect("published flow behavior");
    assert_eq!(
        behavior.independent_determinism,
        Some(etas_package_metadata::CallableValueDeterminism::Deterministic)
    );

    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.relay;
flow main() -> unit ![Console.stdout_write] {
    let definition = ^relay;
    relay<![Console.stdout_write]>(() => { perform Console.stdout_write("test"); });
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expr = *checked.types.reflections.keys().next().unwrap();
    let etas_frontend::code::definition::StaticReflectionView::Callable(header) =
        definitions.catalog().static_reflection(expr).unwrap()
    else {
        panic!("expected external flow descriptor");
    };
    assert_eq!(
        header.analysis.independent_determinism,
        Some(etas_code::descriptor::DeterminismInfo::Deterministic)
    );
    assert!(header.analysis.determinism.is_none());
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert_eq!(
        summary.determinism,
        etas_effects::Determinism::RuntimeMediated
    );
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(summary.requested_actions.effects.contains(&stdout));

    let summary = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter_mut()
        .flat_map(|metadata| &mut metadata.effect_summaries)
        .find(|summary| summary.item.last().is_some_and(|name| name == "relay"))
        .unwrap();
    summary.behavior.as_mut().unwrap().independent_determinism =
        Some(etas_package_metadata::CallableValueDeterminism::NonDeterministic);
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("independent determinism exceeds its summary")),
        "{:?}",
        output.diagnostics
    );
}
