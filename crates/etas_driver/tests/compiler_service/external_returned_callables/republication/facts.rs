use super::*;

#[test]
fn missing_checked_foreign_facts_cannot_be_published_as_an_empty_graph() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow wrap<T, effect E>(callback: (T) -> T ![E]) -> ((T) -> T ![E]) ![] {
    return (value: T) => { return callback(value); };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.wrap;
public flow make(callback: (i32) -> i32 ![]) -> ((i32) -> i32 ![]) ![] { return wrap<i32, ![]>(callback); }
flow main() -> unit { return; }
"#.into();
    let checked = origin_metadata::check(&loaded);
    assert!(!checked.effects.metadata_callables.is_empty());
    let publish = |checked: &etas_frontend::CheckedProgram| {
        etas_frontend::build_package_metadata_artifact_from_checked(
            etas_frontend::PackageMetadataBuildInput {
                definition_origin: None,
                package_id: "relay".into(),
                package_version: "0.1.0".into(),
                package_edition: "2026".into(),
                compiler_version: "test".into(),
                source_payload_hash: "blake3:test-source".into(),
                manifest_hash: "blake3:test-manifest".into(),
                dependency_lock_hash: "blake3:test-lock".into(),
                bins: vec![],
                dependencies: vec![],
                tool_bindings: vec![],
            },
            checked,
        )
    };
    publish(&checked).unwrap();
    for missing_graph in [false, true] {
        let mut tampered = checked.clone();
        let facts = Arc::make_mut(&mut Arc::make_mut(&mut tampered).effects);
        if missing_graph {
            facts.metadata_callable_graphs.clear();
        } else {
            facts.metadata_callables.clear();
        }
        let error = publish(&tampered).unwrap_err();
        assert!(
            error.to_string().contains(if missing_graph {
                "no checked graph fact"
            } else {
                "no materialized analysis"
            }),
            "{error}"
        );
    }
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
        assert!(
            body.facts.metadata_callable_graphs.is_empty(),
            "shared graph descriptors must not be copied into every body"
        );
        for (source, analysis) in &body.facts.metadata_callables {
            assert_eq!(
                checked.effects.callable_environments[&source.environment.unwrap()].owner,
                body.owner
            );
            assert!(projected.insert(*source));
            assert_eq!(analysis, &checked.effects.metadata_callables[source]);
        }
        let encoded = bincode::serde::encode_to_vec(body, bincode::config::standard()).unwrap();
        let (decoded, consumed): (etas_effects::BodyEffectOutput, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(
            decoded.facts.metadata_callables,
            body.facts.metadata_callables
        );
    }
    assert!(
        !projected.is_empty(),
        "captured metadata instances must be owned by their creation body"
    );
    assert_eq!(
        projected,
        checked
            .effects
            .metadata_callables
            .keys()
            .filter(|source| source.environment.is_some())
            .copied()
            .collect()
    );
}
