use super::*;
use etas_driver::{
    LoadProjectRequest, PackageMetadataRequest, PackageUpdateRequest, SourceDependencyMode,
};
use std::path::{Path, PathBuf};

#[path = "republication/facts.rs"]
mod facts;

fn package(root: &Path, name: &str, dependency: &str, source_path: &str, source: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::create_dir_all(path.join("src")).unwrap();
    std::fs::write(
        path.join("etas.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n{dependency}"
        ),
    )
    .unwrap();
    let source_path = path.join("src").join(source_path);
    std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    std::fs::write(source_path, source).unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&path),
        PackageUpdateRequest {
            package_root: path.clone(),
        },
    )
    .unwrap();
    path
}

fn republished(base: &str, relay: &str, consumer: &str) -> etas_frontend::CheckedProgram {
    let workspace = origin_metadata::workspace_with_source(base);
    let relay = package(
        &workspace.0,
        "relay-library",
        r#"[dependencies]
base = { package = "origin-library", version = "0.1.0", import = "base", path = "../library" }
"#,
        "relay/api.es",
        relay,
    );
    etas_driver::build_package_metadata(
        &origin_metadata::options(&relay),
        PackageMetadataRequest {
            package_root: relay.clone(),
            bin: None,
        },
    )
    .unwrap();
    let metadata = etas_package::read_package_metadata_artifact(&relay)
        .unwrap()
        .unwrap();
    assert!(
        metadata
            .public_metadata
            .effect_summaries
            .iter()
            .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
            .unwrap()
            .returned_callables
            .is_some(),
        "republishing cannot discard the dependency's checked callable graph"
    );
    let consumer = package(
        &workspace.0,
        "final-consumer",
        r#"[dependencies]
relay = { package = "relay-library", version = "0.1.0", import = "middle", path = "../relay-library" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
        "app/main.es",
        consumer,
    );
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&consumer),
        LoadProjectRequest {
            paths: vec![consumer],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    checked
}

fn assert_actions(
    checked: &etas_frontend::CheckedProgram,
    expected: &[(etas_effects::EffectActionId, usize)],
) {
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    for (action, count) in expected {
        let action = etas_effects::Effect::Action(etas_effects::ActionRef {
            tag: etas_effects::CONSOLE_TAG,
            action: *action,
        });
        assert!(
            summary.requested_actions.effects.contains(&action),
            "{summary:?}"
        );
        let mut pending = vec![&summary.action_trace];
        let mut actual = 0;
        while let Some(trace) = pending.pop() {
            match trace {
                etas_effects::ActionTraceDomain::Seq(parts)
                | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
                etas_effects::ActionTraceDomain::Event(event) => {
                    actual += usize::from(event.action == action)
                }
                etas_effects::ActionTraceDomain::Empty => {}
                other => panic!("unresolved republished callback trace: {other:?}"),
            }
        }
        assert_eq!(actual, *count, "{summary:?}");
    }
}

#[test]
fn a_republished_generic_closure_keeps_its_checked_capture_graph() {
    let checked = republished(
        r#"module library.api;
public flow wrap<T, effect E>(callback: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return (value: T) => { return callback(value); };
}
public flow lookup() -> i32 { return 7; }
"#,
        r#"module relay.api;
import base.library.api.wrap;
public flow make<U, effect F>(callback: (Array<U>) -> Array<U> ![F])
    -> ((Array<U>) -> Array<U> ![F]) ![] {
    return wrap<Array<U>, ![F]>(callback);
}
"#,
        r#"module app.main;
import middle.relay.api.make;
flow echo(value: Array<i32>) -> Array<i32> ![Console.stdout_write] {
    perform Console.stdout_write("replayed"); return value;
}
flow main() -> Array<i32> ![Console.stdout_write] {
    let callback = make<i32, ![Console.stdout_write]>(echo);
    callback([1]); return callback([2]);
}
"#,
    );
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(
        summary.escaping_effects.effects.contains(&stdout),
        "{summary:?}"
    );
    assert_actions(&checked, &[(etas_effects::CONSOLE_STDOUT_WRITE_ACTION, 2)]);
}

#[test]
fn republishing_rebases_the_callback_scope_of_a_future_parent_invocation() {
    let checked = republished(
        r#"module library.api;
public flow make(first: (i32) -> i32 ![]) -> (((i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![]) ![] {
    return (second: (i32) -> i32 ![]) => { return (value: i32) => { return first(second(value)); }; };
}
public flow lookup() -> i32 { return 7; }
"#,
        r#"module relay.api;
import base.library.api.make as original;
public flow make(first: (i32) -> i32 ![]) -> (((i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![]) ![] { return original(first); }
"#,
        r#"module app.main;
import middle.relay.api.make;
import std.io.{println, eprintln};
flow first(value: i32) -> i32 ![] { println("first")?; return value; }
flow second(value: i32) -> i32 ![] { eprintln("second")?; return value; }
flow main() -> i32 {
    let parent = make(first); let callback = parent(second);
    callback(1); return callback(2);
}
"#,
    );
    assert!(
        checked.effects.item_effects[&checked.entry.unwrap()]
            .escaping_effects
            .is_empty()
    );
    assert_actions(
        &checked,
        &[
            (etas_effects::CONSOLE_STDOUT_WRITE_ACTION, 2),
            (etas_effects::CONSOLE_STDERR_WRITE_ACTION, 2),
        ],
    );
}

#[test]
fn republishing_preserves_every_returned_concrete_capture_source() {
    let checked = republished(
        r#"module library.api;
public flow make(first: (i32) -> i32 ![], second: (i32) -> i32 ![], flag: bool) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    return () => { if flag { return first; } return second; };
}

public flow lookup() -> i32 { return 7; }
"#,
        r#"module relay.api;
import base.library.api.make as original;
import std.io.{println, eprintln};
public flow make(flag: bool) -> (() -> ((i32) -> i32 ![]) ![]) ![] {
    let first = (value: i32) => { println("first")?; return value; };
    let second = (value: i32) => { eprintln("second")?; return value; };
    return original(first, second, flag);
}
"#,
        r#"module app.main;
import middle.relay.api.make;
flow main(flag: bool) -> i32 {
    let factory = make(flag); let callback = factory();
    callback(1); return callback(2);
}
"#,
    );
    assert!(
        checked.effects.item_effects[&checked.entry.unwrap()]
            .escaping_effects
            .is_empty()
    );
    assert_actions(
        &checked,
        &[
            (etas_effects::CONSOLE_STDOUT_WRITE_ACTION, 2),
            (etas_effects::CONSOLE_STDERR_WRITE_ACTION, 2),
        ],
    );
}
