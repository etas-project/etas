use super::*;
use etas_package_metadata::{
    CallableValueGraph, CallableValueParameter, CallableValueScope, CallableValueSource,
};

fn parameter_sources(positions: &[u32]) -> CallableValueGraph {
    CallableValueGraph {
        sources: positions
            .iter()
            .map(|position| {
                CallableValueSource::Parameter(CallableValueParameter {
                    scope: CallableValueScope::Root,
                    position: *position,
                })
            })
            .collect(),
        values: vec![],
    }
}

const LIBRARY: &str = r#"module library.api;
public flow select<T, effect E>(callback: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return callback;
}
public flow lookup() -> i32 { return 7; }
"#;

#[test]
fn metadata_only_returned_private_and_anonymous_flows_keep_every_branch_and_invocation() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
private flow hidden(value: i32) -> i32 ![] { println("hidden")?; return value; }
public flow make(flag: bool) -> ((i32) -> i32 ![]) ![] {
    if flag { return hidden; }
    return (value: i32) => { eprintln("anonymous")?; return value; };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main(flag: bool) -> i32 {
    let callback = make(flag);
    callback(1);
    return callback(2);
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    let stderr = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDERR_WRITE_ACTION,
    });
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.requested_actions.effects.contains(&stderr),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stderr),
        "{summary:?}"
    );
    let mut counts = [0, 0];
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Event(event) => {
                counts[0] += usize::from(event.action == stdout);
                counts[1] += usize::from(event.action == stderr);
            }
            etas_effects::ActionTraceDomain::Seq(parts)
            | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("unresolved concrete return trace: {other:?}"),
        }
    }
    assert_eq!(counts, [2, 2], "{summary:?}");
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    assert!(
        !metadata
            .public_metadata
            .flows
            .iter()
            .any(|signature| signature.visibility == "public"
                && signature.path.last().is_some_and(|name| name == "hidden"))
    );
    let factory = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
        .unwrap();
    assert!(factory.requested_actions.is_empty());
    assert_eq!(factory.returned_callables.as_ref().unwrap().values.len(), 2);
    let mut forbidden = origin_metadata::consumer(&workspace, "first");
    forbidden.loaded.input.sources[0].text = "module app.main; import first.library.api.hidden; flow main() -> i32 { return hidden(1); }".into();
    let output = Frontend.check_project(forbidden.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("is private")),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn metadata_returned_callable_sources_survive_a_local_wrapper_and_trace_spec_checks() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.println;
private flow hidden(value: i32) -> i32 ![] { println("private")?; return value; }
public flow make() -> ((i32) -> i32 ![]) ![] { return hidden; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    for denied in [false, true] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.make;
spec Quiet: trace = -Console.stdout_write;
flow wrapper() -> ((i32) -> i32 ![]) ![] {{ return make(); }}
flow main() -> i32 {} {{ let callback = wrapper(); return callback(2); }}
"#,
            if denied { "~ Quiet" } else { "" }
        );
        let output = Frontend.check_project(loaded.loaded.input);
        assert!(
            !output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Effect(
                    etas_core::EffectDiagnosticCode::IncompleteEffectFacts
                )),
            "{:?}",
            output.diagnostics
        );
        if denied {
            assert!(output.checked.is_none());
            assert!(
                output.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(
                        etas_core::EffectDiagnosticCode::TraceSpecDenied
                    )),
                "{:?}",
                output.diagnostics
            );
        } else {
            let checked = output.checked.unwrap();
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(!summary.requested_actions.is_empty(), "{summary:?}");
        }
    }
}

#[test]
fn metadata_returned_callable_keeps_nonempty_escaping_action_rows() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow hidden() -> unit ![Console.stdout_write] { perform Console.stdout_write("direct"); }
public flow make() -> (() -> unit ![Console.stdout_write]) ![] { return hidden; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> unit ![] { make()(); return; }
"#
    .into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(etas_core::EffectDiagnosticCode::EscapedEffect)),
        "{:?}",
        output.diagnostics
    );
    assert!(
        !output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn forged_returned_callable_support_or_signature_never_becomes_a_pure_call() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.println;
private flow hidden(value: i32) -> i32 ![] { println("checked")?; return value; }
public flow make() -> ((i32) -> i32 ![]) ![] { return hidden; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    for forged_support in [false, true] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = "module app.main; import first.library.api.make; flow main() -> i32 { return make()(1); }".into();
        let factory = loaded
            .loaded
            .input
            .environment
            .external_public_metadata
            .iter_mut()
            .flat_map(|metadata| &mut metadata.effect_summaries)
            .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
            .unwrap();
        let node = &mut factory.returned_callables.as_mut().unwrap().values[0];
        if forged_support {
            node.behavior.host_requirements.clear();
            node.behavior.interpreter_features.clear();
            node.behavior.determinism =
                etas_package_metadata::CallableValueDeterminism::Deterministic;
        } else {
            node.params[0].name = "bool".into();
        }
        let output = Frontend.check_project(loaded.loaded.input);
        assert!(output.checked.is_none());
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| if forged_support {
                    diagnostic.code
                        == etas_core::DiagnosticCode::Type(
                            etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                        )
                        && diagnostic
                            .message
                            .contains("independent determinism exceeds its summary")
                } else {
                    diagnostic.code
                        == etas_core::DiagnosticCode::Effect(
                            etas_core::EffectDiagnosticCode::IncompleteEffectFacts,
                        )
                }),
            "{:?}",
            output.diagnostics
        );
    }
}

#[test]
fn constructing_a_returned_metadata_flow_does_not_invoke_it() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.println;
public flow make() -> (() -> unit ![]) ![] {
    return () => { println("later")?; return; };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> i32 { let callback = make(); return 0; }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.requested_actions.is_empty(), "{summary:?}");
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    assert_eq!(summary.action_trace, etas_effects::ActionTraceDomain::Empty);
}

#[test]
fn binary_metadata_preserves_returned_callback_sources_and_trace_multiplicity() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item == ["library", "api", "select"])
        .unwrap();
    assert_eq!(summary.returned_callables, Some(parameter_sources(&[0])));
    assert!(summary.requested_actions.is_empty());
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
import std.io.println;
flow write(value: i32) -> i32 ![] { println("callback")?; return value; }
flow main() -> i32 {
    let callback = select(write);
    callback(1);
    return callback(2);
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    let mut events = 0;
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Event(event) => {
                events += usize::from(event.action == stdout);
            }
            etas_effects::ActionTraceDomain::Seq(children) => pending.extend(children),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("unresolved returned callback trace: {other:?}"),
        }
    }
    assert_eq!(events, 2, "{summary:?}");
}

#[test]
fn returned_metadata_callbacks_cannot_hide_handled_actions_from_trace_specs() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow write(value: i32) -> i32 ![] { println("callback")?; return value; }
flow main() -> i32 ~ Quiet { return select(write)(2); }
"#
    .into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(etas_core::EffectDiagnosticCode::TraceSpecDenied)),
        "{:?}",
        output.diagnostics
    );
    assert!(
        !output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn candidate_check_uses_returned_metadata_callback_actions_in_the_frozen_contract() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
import std.io.println;
import std.compiler.{check, parse_body, CompileError};
flow write(value: i32) -> i32 ![] { println("candidate")?; return value; }
agent Worker() -> i32 ![] { return 1; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let callback = select(write); return callback(2);") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let report = check((^Worker).with_body(body));
    if report.status != "rejected" { return false; }
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::AdditionalRequestedActions" { return true; }
    }
    return false;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(2_000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    let host_calls = result
        .events
        .iter()
        .filter_map(|event| match event {
            etas_interpreter::api::WorkflowEvent::HostTrace(
                etas_host::TraceEvent::HostRequestStarted { kind, .. },
            ) => Some(*kind),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        host_calls,
        [etas_host::HostRequestKind::Compiler],
        "checking must not execute the callback"
    );
}

#[test]
fn missing_return_source_evidence_is_not_inferred_from_an_empty_escaping_row() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let original = loaded
        .loaded
        .input
        .environment
        .canonical_environment_fingerprint();
    let metadata = &mut loaded.loaded.input.environment.external_public_metadata[0];
    let summary = metadata
        .effect_summaries
        .iter_mut()
        .find(|summary| summary.item.last().is_some_and(|name| name == "select"))
        .unwrap();
    summary.returned_callables = None;
    assert_ne!(
        original,
        loaded
            .loaded
            .input
            .environment
            .canonical_environment_fingerprint(),
        "return-source changes must invalidate environment caches"
    );
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
flow idle(value: i32) -> i32 ![] { return value; }
flow main() -> i32 { return select(idle)(2); }
"#
    .into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn binary_return_sources_are_validated_against_the_imported_interface_and_keep_the_import_anchor() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let initial = origin_metadata::consumer(&workspace, "first");
    let consumer_root = initial.package_root.unwrap();
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
        &artifact,
        &std::fs::read(&artifact).unwrap(),
    )
    .unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter_mut()
        .find(|summary| summary.item == ["library", "api", "select"])
        .unwrap();
    summary.returned_callables = Some(parameter_sources(&[99]));
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
    etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
    etas_package::materialize_package(etas_package::MaterializePackageOptions {
        package_root: consumer_root.clone(),
        source_config: Some(
            etas_package::PackageSourceConfig::read_project(&consumer_root).unwrap(),
        ),
    })
    .unwrap();
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: consumer_root.clone(),
    })
    .unwrap();
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&consumer_root),
        etas_driver::LoadProjectRequest {
            paths: vec![consumer_root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    assert!(
        loaded
            .loaded
            .input
            .environment
            .external_public_metadata
            .iter()
            .flat_map(|metadata| &metadata.effect_summaries)
            .any(|summary| summary.returned_callables == Some(parameter_sources(&[99])))
    );
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("returned callable parameter reference is not declared")
        })
        .expect("malformed return metadata requires a precise diagnostic");
    assert_eq!(
        diagnostic.code,
        etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
    );
    let source = &loaded.loaded.input.sources[0];
    assert_eq!(diagnostic.primary.span.source, source.id);
    let span = diagnostic.primary.span.range;
    assert!(source.text[span.start.0 as usize..span.end.0 as usize].contains("first.library.api"));
}

#[test]
fn metadata_return_source_unions_preserve_every_possible_callback() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public flow select(first: () -> unit ![], second: () -> unit ![], flag: bool) -> (() -> unit ![]) ![] {
    if flag { return first; }
    return second;
}
public flow relay(f: () -> unit ![], g: () -> unit ![], flag: bool) -> (() -> unit ![]) ![] {
    return select(f, g, flag);
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    for alias in ["first", "second"] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        let relay = loaded.loaded.input.environment.external_public_metadata[0]
            .effect_summaries
            .iter()
            .find(|summary| summary.item.last().is_some_and(|name| name == "relay"))
            .unwrap();
        assert_eq!(relay.returned_callables, Some(parameter_sources(&[0, 1])));
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import {alias}.library.api.relay;
import std.io.{{println, eprintln}};
flow write() -> unit ![] {{ println("stdout")?; return; }}
flow warn() -> unit ![] {{ eprintln("stderr")?; return; }}
flow main() -> unit {{ let selected = relay(write, warn, true); selected(); }}
"#
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert!(summary.escaping_effects.is_empty());
        for action in [
            etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
            etas_effects::CONSOLE_STDERR_WRITE_ACTION,
        ] {
            let action = etas_effects::Effect::Action(etas_effects::ActionRef {
                tag: etas_effects::CONSOLE_TAG,
                action,
            });
            assert!(
                summary.requested_actions.effects.contains(&action),
                "{summary:?}"
            );
            assert!(
                summary.default_actions.effects.contains(&action),
                "{summary:?}"
            );
        }
    }
}

#[test]
fn binary_returned_callbacks_keep_their_nonempty_instantiated_effect_row() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
flow main() -> i32 ![Error<i32>] { return select(fail)(2); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let escaping = summary.escaping_effects.effects.iter().collect::<Vec<_>>();
    assert!(
        matches!(escaping.as_slice(), [etas_effects::Effect::Error(ty)] if checked.type_store.get(*ty) == Some(&etas_types::Type::Primitive(etas_types::PrimitiveType::I32))),
        "{summary:?}"
    );
    assert!(
        summary.requested_actions.is_empty(),
        "language Error is not a host action: {summary:?}"
    );
}

#[test]
fn returned_metadata_callbacks_cannot_escape_an_empty_declared_effect_bound() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.select;
flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
flow main() -> i32 ![] { return select(fail)(2); }
"#
    .into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::EffectOutsideDeclaredRow
            )),
        "{:?}",
        output.diagnostics
    );
    assert!(
        !output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn runnable_package_keeps_returned_callbacks_through_overlay_and_candidate_execution() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let loaded = origin_metadata::consumer(&workspace, "first");
    let root = loaded.package_root.unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"
module app.main;
import first.library.api.select;
import std.compiler.{compile, CompileError};
flow increment(value: i32) -> i32 ![] { return value + 1; }
agent Worker(value: i32) -> i32 ![] {
    let callback = select(increment);
    return callback(value);
}

flow main() -> i32 ![Error<CompileError>] {
    if (^select).body != None { return -1; }
    let revised = compile(^Worker);
    return revised(18);
}
"#,
    )
    .unwrap();
    let compiled = etas_driver::compile_runnable_project_with_loaded(
        &origin_metadata::options(&root),
        etas_driver::LoadProjectRequest {
            paths: vec![root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        etas_frontend::SnapshotDetailLevel::Full,
    )
    .unwrap();
    assert!(
        compiled.response.output.diagnostics.is_empty(),
        "{:?}",
        compiled.response.output.diagnostics
    );
    let checked = compiled.response.output.checked.unwrap();
    assert_eq!(
        checked.sources.sources.len(),
        2,
        "only runnable compilation materializes the called dependency module"
    );
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &compiled.loaded,
        &checked,
        Default::default(),
    )
    .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(2_000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(19))
    );
}

#[path = "external_returned_callables/runtime_values.rs"]
mod runtime_values;

#[path = "external_returned_callables/callback_parameters.rs"]
mod callback_parameters;

#[path = "external_returned_callables/callback_captures.rs"]
mod callback_captures;

#[path = "external_returned_callables/multi_captures.rs"]
mod multi_captures;

#[path = "external_returned_callables/republication.rs"]
mod republication;

#[path = "external_returned_callables/references.rs"]
mod references;
