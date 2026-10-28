use super::*;

#[path = "references/facts.rs"]
mod facts;
#[path = "references/returns.rs"]
mod returns;

#[test]
fn a_returned_instantiated_private_flow_keeps_its_error_type() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
public flow make() -> ((i32) -> i32 ![Error<i32>]) ![] { return fail; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> i32 ![Error<i32>] { let callback = make(); return callback(2); }
"#
    .into();
    assert_i32_error(&origin_metadata::check(&loaded));
}

#[test]
fn a_returned_standard_flow_reference_preserves_its_checked_abi() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.text.trim;
public flow make() -> ((string) -> string ![]) ![] { return trim; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> string ![] { let callback = make(); return callback(" trimmed "); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    assert!(summary.requested_actions.is_empty(), "{summary:?}");
}

#[test]
fn generic_wrappers_compose_the_environment_of_a_returned_reference() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
private flow original<T>() -> ((T) -> T ![Error<T>]) ![] { return fail; }
public flow make<U>() -> ((Array<U>) -> Array<U> ![Error<Array<U>>]) ![] { return original<Array<U>>(); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> Array<i32> ![Error<Array<i32>>] { let callback = make<i32>(); return callback([2]); }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let [etas_effects::Effect::Error(ty)] = summary
        .escaping_effects
        .effects
        .iter()
        .collect::<Vec<_>>()
        .as_slice()
    else {
        panic!("{summary:?}");
    };
    assert!(
        matches!(checked.type_store.get(*ty), Some(etas_types::Type::Array(element)) if checked.type_store.get(*element) == Some(&etas_types::Type::Primitive(etas_types::PrimitiveType::I32))),
        "{summary:?}"
    );
}

#[test]
fn a_nested_generic_closure_retains_the_creation_environment_of_its_reference() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
private flow original<T>() -> (() -> ((T) -> T ![Error<T>]) ![]) ![] {
    return () => { return fail; };
}
public flow make<U>() -> (() -> ((U) -> U ![Error<U>]) ![]) ![] { return original<U>(); }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.make;
flow main() -> i32 ![Error<i32>] { let factory = make<i32>(); let callback = factory(); return callback(2); }
"#.into();
    assert_i32_error(&origin_metadata::check(&loaded));
}

fn assert_i32_error(checked: &etas_frontend::CheckedProgram) {
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let errors = summary.escaping_effects.effects.iter().collect::<Vec<_>>();
    assert!(
        matches!(errors.as_slice(), [etas_effects::Effect::Error(ty)] if checked.type_store.get(*ty) == Some(&etas_types::Type::Primitive(etas_types::PrimitiveType::I32))),
        "{summary:?}"
    );
    assert!(
        summary.requested_actions.is_empty(),
        "Error is not a host action: {summary:?}"
    );
}
