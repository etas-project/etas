use super::*;

#[test]
fn metadata_replays_a_returned_capture_with_multiple_concrete_sources() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
private flow hold(callback: (i32) -> i32 ![]) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    return () => { return callback; };
}
public flow choose(flag: bool) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    let callback = if flag {
        (value: i32) => { println("first")?; return value; }
    } else {
        (value: i32) => { eprintln("second")?; return value; }
    };
    return hold(callback);
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    check_twice_returned_capture(&workspace, "choose(true)", "factory()");
}

#[test]
fn metadata_replays_a_nested_capture_without_executing_its_parent() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
private flow hold(callback: (i32) -> i32 ![]) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    return () => { return (value: i32) => { return callback(value); }; };
}
public flow choose(flag: bool) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    let callback = if flag {
        (value: i32) => { println("first")?; return value; }
    } else {
        (value: i32) => { eprintln("second")?; return value; }
    };
    return hold(callback);
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    check_twice_returned_capture(&workspace, "choose(true)", "factory()");
}

#[test]
fn metadata_preserves_a_nested_closures_own_future_callback_binding() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
private flow hold(first: (i32) -> i32 ![]) -> (((i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![]) ![] {
    return (second: (i32) -> i32 ![]) => {
        return (value: i32) => { return first(second(value)); };
    };
}
public flow choose(flag: bool) -> (((i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![]) ![] {
    let first = if flag {
        (value: i32) => { println("first")?; return value; }
    } else {
        (value: i32) => { eprintln("second")?; return value; }
    };
    return hold(first);
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    check_twice_returned_capture(
        &workspace,
        "choose(true)",
        "factory((value: i32) => { return value + 1; })",
    );
}

fn check_twice_returned_capture(
    workspace: &origin_metadata::Workspace,
    factory: &str,
    invocation: &str,
) {
    let mut loaded = origin_metadata::consumer(workspace, "first");
    loaded.loaded.input.sources[0].text = format!(
        r#"module app.main;
import first.library.api.choose;
flow main() -> i32 {{
    let factory = {factory};
    let callback = {invocation};
    callback(1);
    return callback(2);
}}
"#
    );
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    let mut counts = [0, 0];
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Seq(parts)
            | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Event(event) => {
                for (index, action) in [
                    etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
                    etas_effects::CONSOLE_STDERR_WRITE_ACTION,
                ]
                .iter()
                .enumerate()
                {
                    let effect = etas_effects::Effect::Action(etas_effects::ActionRef {
                        tag: etas_effects::CONSOLE_TAG,
                        action: *action,
                    });
                    counts[index] += usize::from(event.action == effect);
                    assert!(
                        summary.requested_actions.effects.contains(&effect),
                        "{summary:?}"
                    );
                    assert!(
                        summary.default_actions.effects.contains(&effect),
                        "{summary:?}"
                    );
                }
            }
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("returned capture left unresolved trace: {other:?}"),
        }
    }
    assert_eq!(counts, [2, 2], "{summary:?}");
}

#[test]
fn metadata_preserves_composite_type_and_effect_row_captures() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow wrap<T, effect E>(callback: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return (value: T) => { return callback(value); };
}
public flow make<U, effect F>(callback: (Array<U>) -> Array<U> ![Console.stdout_write, F])
    -> ((Array<U>) -> Array<U> ![Console.stdout_write, F]) ![] {
    return wrap<Array<U>, ![Console.stdout_write, F]>(callback);
}
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
        .unwrap()
        .returned_callables
        .as_ref()
        .unwrap();
    let [etas_package_metadata::CallableValueSource::Value(value)] = graph.sources.as_slice()
    else {
        panic!("missing captured callable root: {graph:?}");
    };
    let node = &graph.values[*value as usize];
    assert_eq!(node.params[0].kind, etas_package_metadata::TypeKind::Array);
    assert_eq!(
        node.params[0].children[0].kind,
        etas_package_metadata::TypeKind::Var
    );
    assert_eq!(node.params[0].children[0].name, "U");
    assert_eq!(node.output, node.params[0]);
    assert_eq!(node.public_effects.tail.as_deref(), Some("F"));
    assert_eq!(
        node.generic_bindings,
        vec![
            etas_package_metadata::CallableValueGenericBinding {
                label: "U".into(),
                scope: CallableValueScope::Root,
                position: 0
            },
            etas_package_metadata::CallableValueGenericBinding {
                label: "F".into(),
                scope: CallableValueScope::Root,
                position: 1
            },
        ]
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow echo(value: Array<i32>) -> Array<i32> ![Console.stdout_write] { perform Console.stdout_write("typed capture"); return value; }
flow main() -> Array<i32> ![Console.stdout_write] {
    let callback = make<i32, ![]>(echo);
    return callback([1, 2]);
}
"#.into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
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
    assert!(
        summary.action_trace.action_set().contains(&stdout),
        "{summary:?}"
    );
}

#[test]
fn metadata_replays_every_closed_capture_source_without_loading_dependency_bodies() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
private flow wrap(callback: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] {
    return (value: i32) => { return callback(value); };
}
public flow choose(flag: bool) -> ((i32) -> i32 ![]) ![] {
    let callback = if flag {
        (value: i32) => { println("first")?; return value; }
    } else {
        (value: i32) => { eprintln("second")?; return value; }
    };
    return wrap(callback);
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.choose;
flow main(flag: bool) -> i32 {
    let callback = choose(flag);
    callback(1);
    return callback(2);
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    let mut counts = [0, 0];
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Seq(parts)
            | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Event(event) => {
                for (index, action) in [
                    etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
                    etas_effects::CONSOLE_STDERR_WRITE_ACTION,
                ]
                .iter()
                .enumerate()
                {
                    let effect = etas_effects::Effect::Action(etas_effects::ActionRef {
                        tag: etas_effects::CONSOLE_TAG,
                        action: *action,
                    });
                    if event.action == effect {
                        counts[index] += 1;
                    }
                    assert!(
                        summary.requested_actions.effects.contains(&effect),
                        "{summary:?}"
                    );
                    assert!(
                        summary.default_actions.effects.contains(&effect),
                        "{summary:?}"
                    );
                }
            }
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("capture replay left unresolved trace: {other:?}"),
        }
    }
    assert_eq!(counts, [2, 2], "{summary:?}");
}
