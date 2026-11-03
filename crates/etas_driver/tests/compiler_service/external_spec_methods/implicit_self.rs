use super::*;

pub(super) fn event_count(trace: &etas_effects::ActionTraceDomain) -> usize {
    let mut pending = vec![trace];
    let mut events = 0;
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Event(_) => events += 1,
            etas_effects::ActionTraceDomain::Seq(parts)
            | etas_effects::ActionTraceDomain::Choice(parts) => pending.extend(parts),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("closed entry has unresolved or widened evidence: {other:?}"),
        }
    }
    events
}

#[test]
fn behavioral_self_evidence_replays_real_binary_package_and_selects_only_the_called_implementation()
{
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Encoder { flow encode(self) -> i32 ![]; }
public flow twice<T ~ Encoder>(value: T) -> i32 ![] {
    let first = value::Encoder.encode();
    return first + value::Encoder.encode();
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    for (alias, chosen, calls) in [("quiet", "Quiet", 0), ("loud", "Loud", 2)] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import {alias}.library.api.{{Encoder, twice}};
import std.io.println;
type Quiet = {{ number: i32 }};
type Loud = {{ number: i32 }};
impl Quiet ~ Encoder {{ flow encode(self) -> i32 ![] {{ return self.number; }} }}
impl Loud ~ Encoder {{ flow encode(self) -> i32 ![] {{ println("encoded")?; return self.number; }} }}
flow main() -> i32 ![] {{ return twice({chosen} {{ number = 21 }}); }}
"#
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(
            checked.sources.sources.len(),
            1,
            "metadata-only check must not load the producer body"
        );
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert!(summary.escaping_effects.effects.is_empty());
        assert!(summary.escaping_effects.open.is_none());
        let events = event_count(&summary.action_trace);
        assert_eq!(events, calls, "{chosen}: {summary:?}");
        assert_eq!(
            summary.requested_actions.effects.is_empty(),
            calls == 0,
            "unselected impl effects must not leak into {chosen}"
        );
        let method = checked.types.spec_impls.iter().find(|implementation| {
            matches!(checked.type_store.get(implementation.self_type), Some(etas_types::Type::Nominal(nominal)) if nominal.name.ends_with(chosen))
        }).unwrap().methods.first().unwrap();
        let etas_hir::SymbolDef::Item { item } =
            checked.hir.symbols.get(method.symbol).unwrap().def
        else {
            panic!("normalized method")
        };
        assert_eq!(
            summary.requested_actions,
            checked.effects.item_effects[&item].requested_actions
        );
    }
}

#[test]
fn behavioral_self_binary_replay_preserves_callback_origins_and_invocation_multiplicity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public spec Runner { flow run(self, callback: () -> i32 ![]) -> i32 ![]; }
public flow apply<T ~ Runner>(value: T, callback: () -> i32 ![]) -> i32 ![] {
    return value::Runner.run(callback);
}

public flow embedded<T ~ Runner>(value: T) -> i32 ![] {
    return value::Runner.run(() => { println("embedded")?; return 7; });
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    for (alias, invocation) in [
        (
            "parameter",
            "apply(value, () => { println(\"consumer\")?; return 7; })",
        ),
        ("embedded", "embedded(value)"),
    ] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import std.io.println;
import {alias}.library.api.{{Runner, apply, embedded}};
type Worker = {{ id: i32 }};
impl Worker ~ Runner {{ flow run(self, callback: () -> i32 ![]) -> i32 ![] {{ return callback() + callback(); }} }}
flow main() -> i32 ![] {{ let value = Worker {{ id = 1 }}; return {invocation}; }}
"#
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert!(summary.escaping_effects.is_empty(), "{summary:?}");
        assert!(!summary.requested_actions.is_empty(), "{summary:?}");
        assert_eq!(
            event_count(&summary.action_trace),
            2,
            "{alias}: {summary:?}"
        );
    }
}

#[test]
fn behavioral_self_binary_replay_preserves_inferred_method_rows_and_handled_requests() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public spec Runner {
    flow run<effect E>(self, f: () -> unit ![E], g: () -> unit ![E]) -> unit ![E];
}
public flow apply<T ~ Runner>(value: T) -> unit ![] {
    value::Runner.run(() => { println("handled")?; return; }, () => { return; });
}
public flow lookup() -> i32 ![] { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Runner, apply};
type Worker = { id: i32 };
impl Worker ~ Runner {
    flow run<effect F>(self, f: () -> unit ![F], g: () -> unit ![F]) -> unit ![F] {
        f(); g(); f();
    }
}
flow main() -> unit ![] { apply(Worker { id = 1 }); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(
        checked.sources.sources.len(),
        1,
        "consumer must use binary metadata only"
    );
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    assert_eq!(summary.requested_actions.effects.iter().count(), 1);
    assert_eq!(event_count(&summary.action_trace), 2, "{summary:?}");
}
