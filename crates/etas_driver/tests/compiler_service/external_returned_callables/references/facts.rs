use super::*;

#[test]
fn checked_reference_facts_are_required_for_export_and_owned_by_the_creation_body() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
private flow fail<T>(value: T) -> T ![Error<T>] { return perform Error<T>.raise(value); }
private flow original<T>() -> ((T) -> T ![Error<T>]) ![] { return fail; }
public flow make<U>() -> ((Array<U>) -> Array<U> ![Error<Array<U>>]) ![] { return original<Array<U>>(); }
flow main() -> unit { return; }
"#.into();
    let checked = origin_metadata::check(&loaded);
    assert!(!checked.effects.callable_references.is_empty());
    let publish = |checked: &etas_frontend::CheckedProgram| {
        etas_frontend::build_package_metadata_artifact_from_checked(
            etas_frontend::PackageMetadataBuildInput {
                definition_origin: None,
                package_id: "references".into(),
                package_version: "0.1.0".into(),
                package_edition: "2026".into(),
                compiler_version: "test".into(),
                source_payload_hash: "blake3:source".into(),
                manifest_hash: "blake3:manifest".into(),
                dependency_lock_hash: "blake3:lock".into(),
                bins: vec![],
                dependencies: vec![],
                tool_bindings: vec![],
            },
            checked,
        )
    };
    publish(&checked).unwrap();
    let mut tampered = checked.clone();
    Arc::make_mut(&mut Arc::make_mut(&mut tampered).effects)
        .callable_references
        .clear();
    let error = publish(&tampered).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("no materialized checked analysis"),
        "{error}"
    );
    let bodies = etas_effects::BodyEffectOutput::project(
        &etas_effects::EffectOutput {
            facts: checked.effects.as_ref().clone(),
            diagnostics: vec![],
        },
        &checked.hir,
        checked.effects.item_effects.keys().copied(),
    )
    .unwrap();
    let mut projected = std::collections::BTreeSet::new();
    for body in bodies.values() {
        for reference in body.facts.callable_references.keys() {
            let owner = reference
                .environment
                .map_or(reference.owner, |environment| {
                    checked.effects.callable_environments[&environment].owner
                });
            assert_eq!(owner, body.owner);
            assert!(projected.insert(reference.clone()));
        }
        let encoded = bincode::serde::encode_to_vec(body, bincode::config::standard()).unwrap();
        let (decoded, consumed): (etas_effects::BodyEffectOutput, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(
            decoded.facts.callable_references,
            body.facts.callable_references
        );
    }
    assert_eq!(
        projected,
        checked
            .effects
            .callable_references
            .keys()
            .cloned()
            .collect()
    );
}
