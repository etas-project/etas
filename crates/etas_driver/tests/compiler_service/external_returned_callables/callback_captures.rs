use super::*;

#[test]
fn metadata_closures_preserve_creation_time_type_and_effect_row_bindings() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make<T, effect E>(f: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return (value: T) => { return f(value); };
}
public flow relay<U, effect F>(g: (U) -> U ![F]) -> ((U) -> U ![F]) ![] { return make<U, ![F]>(g); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    for factory in ["make", "relay", "local"] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        for metadata in &loaded.loaded.input.environment.external_public_metadata {
            for summary in &metadata.effect_summaries {
                if summary
                    .item
                    .last()
                    .is_some_and(|name| name == "make" || name == "relay")
                {
                    assert!(summary.public_effects.tail.is_none(), "{summary:?}");
                    assert!(summary.requested_actions.tail.is_none(), "{summary:?}");
                    assert!(
                        summary
                            .default_handler_effects
                            .as_ref()
                            .unwrap()
                            .direct_effects
                            .tail
                            .is_none(),
                        "{summary:?}"
                    );
                }
            }
        }
        loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.{make, relay};
flow local<V, effect G>(g: (V) -> V ![G]) -> ((V) -> V ![G]) ![] { return make<V, ![G]>(g); }
flow stdout(value: i32) -> i32 ![Console.stdout_write] {
    perform Console.stdout_write("first"); return value;
}
flow stderr(value: string) -> string ![Console.stderr_write] {
    perform Console.stderr_write(value); return value;
}
flow main() -> unit ![Console.stdout_write, Console.stderr_write] {
    let first = FACTORY<i32, ![Console.stdout_write]>(stdout);
    let second = FACTORY<string, ![Console.stderr_write]>(stderr);
    first(42);
    second("second");
    return;
}
"#
        .replace("FACTORY", factory);
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert_eq!(
            summary.escaping_effects.effects.iter().count(),
            2,
            "{summary:?}"
        );
        assert_eq!(
            summary.requested_actions.effects.iter().count(),
            2,
            "{summary:?}"
        );
        let mut actions = vec![];
        let mut pending = vec![&summary.action_trace];
        while let Some(trace) = pending.pop() {
            match trace {
                etas_effects::ActionTraceDomain::Seq(parts) => pending.extend(parts.iter().rev()),
                etas_effects::ActionTraceDomain::Event(event) => actions.push(event.action.clone()),
                etas_effects::ActionTraceDomain::Empty => {}
                other => panic!("generic capture was not instantiated: {other:?}"),
            }
        }
        assert_eq!(
            actions,
            [
                etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
                etas_effects::CONSOLE_STDERR_WRITE_ACTION
            ]
            .map(
                |action| etas_effects::Effect::Action(etas_effects::ActionRef {
                    tag: etas_effects::CONSOLE_TAG,
                    action
                })
            )
        );
    }
}

#[test]
fn callback_instantiation_does_not_erase_partial_arithmetic_from_the_frozen_contract() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] { return (value: i32) => { return f(value); }; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), r#"module app.main;
import first.library.api.make;
import std.compiler.{check, parse_body, CompileError};
agent Worker(offset: i32) -> i32 ![] { return offset; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("let saved = make((value: i32) => { return value + offset; }); return saved(20);") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let report = check((^Worker).with_body(body));
    if report.status != "rejected" { return -2; }
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" { return 19; }
    }
    return -3;
}
"#).unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 19);
}

#[test]
fn editing_a_metadata_only_agent_completes_the_locked_capture_factory_before_publication() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] { return (value: i32) => { return f(value); }; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), r#"module app.main;
import first.library.api.make;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(offset: i32) -> i32 ![] { return offset; }
flow main() -> i32 ![Error<CompileError>] {
    if (^make).body != None { return -1; }
    let body = match parse_body("let saved = make((value: i32) => { return offset; }); return saved(20);") {
        Ok(body) => body, Err(_) => { return -2; },
    };
    return compile((^Worker).with_body(body))(43);
}
"#).unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 43);
}

#[test]
fn a_revision_preserves_generic_captures_from_a_locked_dependency() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make<T, effect E>(f: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return (value: T) => { return f(value); };
}

public flow lookup() -> i32 { return 7; }
"#,
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), r#"module app.main;
import first.library.api.make;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(offset: i32) -> i32 ![] { return offset; }
flow main() -> i32 ![Error<CompileError>] {
    if (^make).body != None { return -1; }
    let body = match parse_body("let saved = make<i32, ![]>((value: i32) => { return offset; }); return saved(20);") {
        Ok(body) => body, Err(_) => { return -2; },
    };
    return compile((^Worker).with_body(body))(43);
}
"#).unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 1, 43);
}

#[test]
fn metadata_only_pure_generic_call_supplies_checked_candidate_outcomes() {
    use etas_code::{definition::CodeDefinitionOperations, syntax::CodeSyntaxOperations};
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow identity<T>(value: T) -> T ![] { return value; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, CompileError};
agent Worker(offset: i32) -> i32 ![] { return identity(offset); }
flow main() -> i32 ![Error<CompileError>] { return compile(^Worker)(43); }
"#
    .into();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    let checked = output.checked.expect("metadata-only project must check");
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let body = definitions
        .syntax_operations()
        .parse_body("return identity(offset);")
        .unwrap();
    let draft = definitions.with_body(&original, body).unwrap();
    let overlay = definitions
        .resolve(&draft)
        .unwrap()
        .body_overlay(definitions.syntax_operations())
        .unwrap();
    let analysis = etas_frontend::FrontendSession::new()
        .analyze_body_overlay(overlay)
        .unwrap();
    analysis.into_checked_candidate(Default::default()).unwrap();

    let mut missing = loaded.loaded.input.clone();
    let mut removed = false;
    for origin in &mut missing.environment.external_definition_origins {
        let Some(headers) = origin
            .descriptors
            .as_mut()
            .and_then(|set| set.headers.as_mut())
        else {
            continue;
        };
        for header in &mut headers.callables {
            if header
                .id
                .as_declaration()
                .is_some_and(|identity| identity.declaration() == ["identity"])
            {
                removed = header.analysis.outcomes.take().is_some();
            }
        }
    }
    assert!(
        removed,
        "fixture must contain a published outcome descriptor"
    );
    let output = etas_frontend::Frontend.check_project(missing);
    if let Some(checked) = output.checked {
        assert!(
            checked.effects.item_effects.values().any(|summary| {
                summary.exits.as_ref().is_some_and(|exits| {
                    exits
                        .gaps
                        .iter()
                        .any(|gap| matches!(gap, etas_effects::OutcomeAnalysisGap::ExternalCall(_)))
                })
            }),
            "missing outcome facts must not become a successful empty summary"
        );
    } else {
        assert!(!output.diagnostics.is_empty());
    }
}

#[test]
fn metadata_only_partial_pure_call_does_not_become_a_total_outcome() {
    use etas_code::{definition::CodeDefinitionOperations, syntax::CodeSyntaxOperations};
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow risky(value: i32) -> i32 ![] { return 42 / value; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.risky;
import std.compiler.{compile, CompileError};
agent Worker(offset: i32) -> i32 ![] { return risky(offset); }
flow main() -> i32 ![Error<CompileError>] { return compile(^Worker)(43); }
"#
    .into();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    let checked = output.checked.expect("metadata-only project must check");
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let body = definitions
        .syntax_operations()
        .parse_body("return risky(offset);")
        .unwrap();
    let draft = definitions.with_body(&original, body).unwrap();
    let overlay = definitions
        .resolve(&draft)
        .unwrap()
        .body_overlay(definitions.syntax_operations())
        .unwrap();
    let analysis = etas_frontend::FrontendSession::new()
        .analyze_body_overlay(overlay)
        .unwrap();
    let Err(etas_frontend::code::service::CandidateCompileError::Rejected(report)) =
        analysis.into_checked_candidate(Default::default())
    else {
        panic!("an unproved partial external call must be rejected");
    };
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "code::MissingOutcomeSensitiveSummary")
    );
}

#[test]
fn metadata_only_factory_outcome_is_replayed_but_captured_flow_remains_unproved() {
    use etas_code::{definition::CodeDefinitionOperations, syntax::CodeSyntaxOperations};
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow identity<T>(value: T) -> T ![] { return value; }
public flow make(f: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] {
    return (value: i32) => { return f(value); };
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let source = r#"module app.main;
import first.library.api.{identity, make};
import std.compiler.{compile, CompileError};
agent Worker(offset: i32) -> i32 ![] {
    let selected: (i32) -> i32 ![] = identity;
    let saved = make(selected);
    return saved(offset);
}
flow main() -> i32 ![Error<CompileError>] {
    return compile(^Worker)(43);
}
"#;
    loaded.loaded.input.sources[0].text = source.into();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    let checked = output.checked.expect("metadata-only project must check");
    assert_eq!(checked.sources.sources.len(), 1);
    assert!(
        checked
            .effects
            .callable_environments
            .values()
            .any(|environment| {
                environment.parameters.values().flatten().any(|origin| {
                    matches!(
                        origin,
                        etas_effects::facts::CallableValueOrigin::ExternalReference { .. }
                    )
                })
            })
    );
    let outcomes = checked
        .effects
        .item_effects
        .values()
        .filter_map(|summary| summary.exits.as_ref())
        .collect::<Vec<_>>();
    assert!(outcomes.iter().all(|exits| {
        !exits
            .gaps
            .iter()
            .any(|gap| matches!(gap, etas_effects::OutcomeAnalysisGap::ExternalCall(_)))
    }));
    assert!(outcomes.iter().any(|exits| {
        exits
            .gaps
            .iter()
            .any(|gap| matches!(gap, etas_effects::OutcomeAnalysisGap::DynamicCall(_)))
    }));
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let body = definitions.syntax_operations().parse_body(
        "let selected: (i32) -> i32 ![] = identity; let saved = make(selected); return saved(offset);",
    ).unwrap();
    let draft = definitions.with_body(&original, body).unwrap();
    let overlay = definitions
        .resolve(&draft)
        .unwrap()
        .body_overlay(definitions.syntax_operations())
        .unwrap();
    let analysis = etas_frontend::FrontendSession::new()
        .analyze_body_overlay(overlay)
        .unwrap();
    assert!(
        analysis.output().checked.is_some(),
        "{:?}",
        analysis.output().diagnostics
    );
    let Err(etas_frontend::code::service::CandidateCompileError::Rejected(report)) =
        analysis.into_checked_candidate(Default::default())
    else {
        panic!("an unproved captured callable outcome must be rejected");
    };
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "code::MissingOutcomeSensitiveSummary")
    );
    let root = loaded.package_root.unwrap();
    std::fs::write(root.join("src/app/main.es"), source).unwrap();
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 2, 43);
}

#[test]
fn two_capture_slots_can_refer_to_one_public_callback_without_duplicate_binding_labels() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow both(f: () -> unit ![], g: () -> unit ![]) -> (() -> unit ![]) ![] { return () => { f(); g(); return; }; }
public flow relay(h: () -> unit ![]) -> (() -> unit ![]) ![] { return both(h, h); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.relay;
import std.io.println;
flow main() -> unit { let later = relay(() => { println("twice")?; return; }); later(); return; }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let mut events = 0;
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Seq(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Event(_) => events += 1,
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("shared capture was not instantiated: {other:?}"),
        }
    }
    assert_eq!(events, 2, "{summary:?}");
}

#[test]
fn a_captured_callback_is_checked_by_trace_spec_only_when_the_closure_is_invoked() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: () -> unit ![]) -> (() -> unit ![]) ![] { return () => { f(); return; }; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    for invoked in [false, true] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.make;
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow main() -> unit ~ Quiet {{
    let later = make(() => {{ println("captured")?; return; }});
    {} return;
}}
"#,
            if invoked { "later();" } else { "" }
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
        if invoked {
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
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(summary.requested_actions.is_empty(), "{summary:?}");
            assert!(
                matches!(summary.action_trace, etas_effects::ActionTraceDomain::Empty),
                "{summary:?}"
            );
        }
    }
}

#[test]
fn two_metadata_closure_instances_keep_separate_captures_and_trace_order() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: () -> unit ![]) -> (() -> unit ![]) ![] { return () => { f(); f(); return; }; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
import std.io.{println, eprintln};
flow main() -> unit {
    var selected: () -> unit ![] = () => { println("first")?; return; };
    let first = make(selected);
    selected = () => { eprintln("second")?; return; };
    let second = make(selected);
    first(); second(); return;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let mut actions = vec![];
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Seq(parts) => pending.extend(parts.iter().rev()),
            etas_effects::ActionTraceDomain::Event(event) => actions.push(event.action.clone()),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("capture instances lost trace order: {other:?}"),
        }
    }
    let console = |action| {
        etas_effects::Effect::Action(etas_effects::ActionRef {
            tag: etas_effects::CONSOLE_TAG,
            action,
        })
    };
    assert_eq!(
        actions,
        vec![
            console(etas_effects::CONSOLE_STDOUT_WRITE_ACTION),
            console(etas_effects::CONSOLE_STDOUT_WRITE_ACTION),
            console(etas_effects::CONSOLE_STDERR_WRITE_ACTION),
            console(etas_effects::CONSOLE_STDERR_WRITE_ACTION)
        ]
    );
}

#[test]
fn a_revision_executes_a_dependency_closure_that_captures_its_callback() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] {
    return (value: i32) => { return f(value) + f(value + 1); };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let loaded = origin_metadata::consumer(&workspace, "first");
    let root = loaded.package_root.unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.make;
import std.compiler.{compile, CompileError};
agent Worker(offset: i32) -> i32 ![] {
    let saved = make((value: i32) => { return value + offset; });
    return saved(20);
}
flow main() -> i32 ![Error<CompileError>] { let revised = compile(^Worker); return revised(1); }
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
    assert_eq!(checked.sources.sources.len(), 2);
    assert!(!checked.effects.callable_environments.is_empty());
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
        Some(&etas_interpreter::api::InterpValue::i32(43))
    );
}

#[test]
fn metadata_closures_capture_callback_sources_at_creation_and_survive_a_local_wrapper() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make(f: () -> unit ![]) -> (() -> unit ![]) ![] {
    return () => { f(); f(); return; };
}
public flow relay(g: () -> unit ![]) -> (() -> unit ![]) ![] { return make(g); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    for (factory, wrapped) in [
        ("make", false),
        ("make", true),
        ("relay", false),
        ("relay", true),
    ] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.{{make, relay}};
import std.io.{{println, eprintln}};
flow wrapper(f: () -> unit ![]) -> (() -> unit ![]) ![] {{ return {factory}(f); }}
flow main() -> unit {{
    var selected: () -> unit ![] = () => {{ println("captured")?; return; }};
    let deferred = {}(selected);
    selected = () => {{ eprintln("later")?; return; }};
    deferred();
    return;
}}
"#,
            if wrapped { "wrapper" } else { factory }
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
            tag: etas_effects::CONSOLE_TAG,
            action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
        });
        let stderr = etas_effects::Effect::Action(etas_effects::ActionRef {
            tag: etas_effects::CONSOLE_TAG,
            action: etas_effects::CONSOLE_STDERR_WRITE_ACTION,
        });
        assert!(summary.escaping_effects.is_empty(), "{summary:?}");
        assert!(
            summary.requested_actions.effects.contains(&stdout),
            "{summary:?}"
        );
        assert!(
            !summary.requested_actions.effects.contains(&stderr),
            "{summary:?}"
        );
        let mut calls = 0;
        let mut pending = vec![&summary.action_trace];
        while let Some(trace) = pending.pop() {
            match trace {
                etas_effects::ActionTraceDomain::Event(event) => {
                    calls += usize::from(event.action == stdout)
                }
                etas_effects::ActionTraceDomain::Seq(parts) => pending.extend(parts),
                etas_effects::ActionTraceDomain::Empty => {}
                other => panic!("unresolved captured callback: {other:?}"),
            }
        }
        assert_eq!(calls, 2, "{summary:?}");
    }
}
