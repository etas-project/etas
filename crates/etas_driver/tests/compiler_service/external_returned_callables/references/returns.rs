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
