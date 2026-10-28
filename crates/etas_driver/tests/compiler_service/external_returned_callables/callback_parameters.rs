use super::*;

#[test]
fn metadata_returned_anonymous_callable_instantiates_its_own_callback_by_position() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make() -> ((() -> unit ![]) -> unit ![]) ![] {
    return (f: () -> unit ![]) => { f(); f(); return; };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    for denied in [false, true] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.make;
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow main(f: () -> unit ![]) -> unit {} {{
    let invoke = make();
    invoke(() => {{ println("inner")?; return; }});
    return;
}}
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
            let checked = output.checked.expect("checked metadata-only consumer");
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(summary.escaping_effects.is_empty(), "{summary:?}");
            assert_eq!(stdout_count(summary), 2, "{summary:?}");
        }
    }
}

#[test]
fn metadata_returned_anonymous_callable_can_return_its_own_callback_parameter() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make() -> ((() -> unit ![]) -> (() -> unit ![]) ![]) ![] {
    return (f: () -> unit ![]) => { return f; };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
import std.io.println;
flow main() -> unit {
    let forward = make();
    let callback = forward(() => { println("on invocation")?; return; });
    callback();
    return;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert_eq!(stdout_count(summary), 1, "{summary:?}");
}

fn stdout_count(summary: &etas_effects::EffectSummary) -> usize {
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stdout),
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
            other => panic!("uninstantiated callback trace: {other:?}"),
        }
    }
    count
}
