use super::implicit_self::event_count;
use super::*;

const SOURCE: &str = r#"
module library.api;
import std.io.println;
public spec Encoder { flow encode(self) -> i32 ![]; }
public type Quiet = { number: i32 };
public type Loud = { number: i32 };
impl Quiet ~ Encoder { flow encode(self) -> i32 ![] { return self.number; } }
impl Loud ~ Encoder { flow encode(self) -> i32 ![] { println("encoded")?; return self.number; } }
public flow twice<T ~ Encoder>(value: T) -> i32 ![] { return value::Encoder.encode() + value::Encoder.encode(); }
public flow lookup() -> i32 ![] { return 7; }
"#;

#[test]
fn behavioral_self_metadata_only_direct_impl_replay_preserves_exact_requests_and_trace() {
    let workspace = origin_metadata::workspace_with_source(SOURCE);
    for chosen in ["Quiet", "Loud"] {
        for call in [
            "value::Encoder.encode() + value::Encoder.encode()",
            "twice(value)",
        ] {
            let mut loaded = origin_metadata::consumer(&workspace, "dep");
            loaded.loaded.input.sources[0].text = format!(
                r#"
module app.main;
import dep.library.api.{{Encoder, Quiet, Loud, twice}};
flow main() -> i32 ![] {{ let value = {chosen} {{ number = 21 }}; return {call}; }}
"#
            );
            let checked = origin_metadata::check(&loaded);
            assert_eq!(checked.sources.sources.len(), 1, "metadata-only acceptance");
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(
                summary.escaping_effects.is_empty(),
                "{chosen}, {call}: {summary:?}"
            );
            assert_eq!(
                summary.requested_actions.effects.iter().count(),
                usize::from(chosen == "Loud"),
                "{chosen}, {call}: {summary:?}"
            );
            assert_eq!(
                event_count(&summary.action_trace),
                if chosen == "Loud" { 2 } else { 0 },
                "{chosen}, {call}: {summary:?}"
            );
        }
    }
}

#[test]
fn behavioral_self_metadata_only_impl_replay_retains_callback_row_and_multiplicity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Runner { flow run<effect E>(self, f: () -> unit ![E]) -> unit ![E]; }
public type Worker = { id: i32 };
impl Worker ~ Runner { flow run<effect F>(self, callback: () -> unit ![F]) -> unit ![F] { callback(); callback(); } }
public flow apply<T ~ Runner, effect E>(value: T, f: () -> unit ![E]) -> unit ![E] { value::Runner.run(f); }
public flow lookup() -> i32 ![] { return 7; }
"#,
    );
    let package = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let root = package.public_metadata.spec_impls[0].methods[0]
        .root()
        .unwrap();
    assert_eq!(root.public_effects.tail.as_deref(), Some("F"), "{root:?}");
    assert_eq!(
        root.default_handler_effects.direct_effects.tail.as_deref(),
        Some("F"),
        "{root:?}"
    );
    for (body, row, escaping) in [
        ("println(\"handled\")?; return;", "", 0),
        (
            "perform Console.stdout_write(\"escaping\"); return;",
            "Console.stdout_write",
            1,
        ),
    ] {
        for call in [
            "worker::Runner.run(f)".to_owned(),
            "apply(worker, f)".to_owned(),
            format!("worker::Runner.run<![{row}]>(f)"),
        ] {
            let mut loaded = origin_metadata::consumer(&workspace, "dep");
            loaded.loaded.input.sources[0].text = format!(
                r#"
module app.main;
import dep.library.api.{{Runner, Worker, apply}};
import std.io.println;
flow main() -> unit ![{row}] {{
    let worker = Worker {{ id = 1 }};
    let f = () => {{ {body} }};
    {call};
}}
"#
            );
            let checked = origin_metadata::check(&loaded);
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert_eq!(
                summary.escaping_effects.effects.iter().count(),
                escaping,
                "{call}: {summary:?}; bindings {:?}; rows {:?}; latent {:?}",
                checked.types.generic_instantiations,
                checked.effects.generic_effect_rows,
                checked
                    .effects
                    .unit_effects
                    .iter()
                    .filter(|(unit, _)| !matches!(unit, etas_effects::EffectUnit::Item(_)))
                    .collect::<Vec<_>>()
            );
            assert!(
                summary.escaping_effects.open.is_none(),
                "{call}: {summary:?}"
            );
            assert_eq!(
                summary.requested_actions.effects.iter().count(),
                1,
                "{call}: {summary:?}"
            );
            assert_eq!(event_count(&summary.action_trace), 2, "{call}: {summary:?}");
        }
    }
}

#[test]
fn behavioral_self_metadata_only_impl_requests_are_checked_against_trace_specs() {
    let workspace = origin_metadata::workspace_with_source(SOURCE);
    for call in ["value::Encoder.encode()", "twice(value)"] {
        let mut loaded = origin_metadata::consumer(&workspace, "dep");
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import dep.library.api.{{Encoder, Loud, twice}};
spec Silent: trace = -Console.stdout_write;
flow main() -> i32 ![] ~ Silent {{ let value = Loud {{ number = 21 }}; return {call}; }}
"#
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none());
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Effect(
                    etas_core::EffectDiagnosticCode::TraceSpecDenied
                )),
            "{call}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn behavioral_self_metadata_only_impl_preserves_generic_type_arguments() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Echo { flow echo<T>(self, value: T) -> T ![]; }
public type Packet = { id: i32 };
impl Packet ~ Echo { flow echo<U>(self, value: U) -> U ![] { return value; } }
public flow apply<T ~ Echo, U>(receiver: T, value: U) -> U ![] { return receiver::Echo.echo(value); }
public flow lookup() -> i32 ![] { return 7; }
"#,
    );
    for call in [
        "value::Echo.echo(42)",
        "value::Echo.echo<i32>(42)",
        "apply(value, 42)",
    ] {
        let mut loaded = origin_metadata::consumer(&workspace, "dep");
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import dep.library.api.{{Echo, Packet, apply}};
flow main() -> i32 ![] {{ let value = Packet {{ id = 1 }}; return {call}; }}
"#
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert!(
            summary.escaping_effects.is_empty() && summary.requested_actions.is_empty(),
            "{call}: {summary:?}"
        );
        assert_eq!(event_count(&summary.action_trace), 0);
    }
    for call in ["value::Echo.echo<string>(42)", "value::Echo.echo<![]>(42)"] {
        let mut loaded = origin_metadata::consumer(&workspace, "dep");
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import dep.library.api.{{Echo, Packet}}; flow main() -> i32 ![] {{ let value = Packet {{ id = 1 }}; return {call}; }}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(
            output.checked.is_none(),
            "accepted invalid method arguments: {call}"
        );
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::TypeMismatch)),
            "{call}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn behavioral_self_impl_with_a_marker_spec_replays_its_method_once() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec First { flow encode(self) -> i32 ![]; }
public spec Second;
public type Packet = { number: i32 };
impl Packet ~ First + Second { flow encode(self) -> i32 ![] { return self.number; } }
public flow lookup() -> i32 ![] { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{First, Second, Packet};
flow main() -> i32 ![] { let value = Packet { number = 21 }; return value::First.encode(); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.types.external_specs.methods.len(), 1);
    let identities = checked
        .types
        .external_specs
        .methods
        .iter()
        .map(|method| &method.identity)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(identities.len(), 1);
    assert!(
        checked.effects.item_effects[&checked.entry.unwrap()]
            .requested_actions
            .is_empty()
    );
}
