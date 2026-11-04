use super::*;

const LIBRARY: &str = r#"
module library.api;
public spec Named { flow name(self) -> string ![Console.stdout_write]; }
public type Label = string;
impl Label ~ Named {
    flow name(self) -> string ![Console.stdout_write] {
        perform Console.stdout_write("label");
        return "label";
    }
}
public flow make() -> ? ~ Named ![] { return Label("value"); }
public flow pack<T ~ Named>(value: T) -> ? ~ Named ![] { return value; }
public flow lookup() -> i32 ![] { return 7; }
"#;

#[test]
fn object_methods_materialize_their_exact_impl_module_and_execute() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                r#"
module library.api;
import library.types.{Label, Named};
public flow make() -> ? ~ Named ![] { return Label("first"); }
public flow pack<T ~ Named>(value: T) -> ? ~ Named ![] { return value; }
public flow lookup() -> i32 ![] { return 7; }
"#,
            ),
            (
                "library/types.es",
                r#"
module library.types;
public spec Named { flow name(self) -> string ![]; }
public type Label = string;
"#,
            ),
            (
                "library/impls.es",
                r#"
module library.impls;
import library.types.{Label, Named};
impl Label ~ Named { flow name(self) -> string ![] { return "label"; } }
"#,
            ),
        ],
        "",
    );
    let loaded = origin_metadata::consumer(&workspace, "dep");
    let root = loaded.package_root.unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"
module app.main;
import dep.library.api.{make, pack};
import dep.library.types.{Label, Named};
flow main() -> bool ![] {
    let first = make();
    let second = pack(Label("second"));
    return first.name().len() == 5 && second.name() == "label";
}
"#,
    )
    .unwrap();
    let result = dependency_namespaces::execute_result(dependency_namespaces::compile(&root), 3);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

#[test]
fn object_witnesses_replay_through_binary_packages_and_reexports() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    for alias in ["objects", "renamed"] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        let source = format!(
            r#"
module app.main;
import {alias}.library.api.{{Named, Label, make, pack}};
public flow forward() -> ? ~ Named ![] {{ return make(); }}
flow main() -> usize ![Console.stdout_write] {{
    let first = forward();
    let second = pack(Label("second"));
    return first.name().len() + second.name().len();
}}
"#
        );
        let root = loaded.package_root.as_ref().unwrap();
        std::fs::write(root.join("src/app/main.es"), &source).unwrap();
        loaded.loaded.input.sources[0].text = source;
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1, "metadata-only consumer");
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert_eq!(summary.escaping_effects.effects.len(), 1);
        assert_eq!(summary.requested_actions.effects.len(), 1);
        let etas_effects::ActionTraceDomain::Seq(events) = &summary.action_trace else {
            panic!(
                "both selected method bodies must contribute a trace: {:?}",
                summary.action_trace
            );
        };
        assert_eq!(events.len(), 2);
        assert!(
            events
                .iter()
                .all(|event| matches!(event, etas_effects::ActionTraceDomain::Event(_)))
        );

        etas_driver::build_package_metadata(
            &origin_metadata::options(root),
            etas_driver::PackageMetadataRequest {
                package_root: root.clone(),
                bin: None,
            },
        )
        .unwrap();
        let metadata = etas_package::read_package_metadata_artifact(root)
            .unwrap()
            .unwrap();
        let graph = metadata
            .public_metadata
            .effect_summaries
            .iter()
            .find(|summary| summary.item == ["app", "main", "forward"])
            .unwrap()
            .returned_callables
            .as_ref()
            .unwrap();
        let [etas_package_metadata::CallableValueSource::Existential(object)] =
            graph.sources.as_slice()
        else {
            panic!("the forwarding wrapper must preserve its object source");
        };
        let mut object = *object;
        let mut visited = std::collections::BTreeSet::new();
        let (method, value) = loop {
            assert!(
                visited.insert(object),
                "object repack evidence must terminate"
            );
            let node = &graph.objects[object as usize];
            match &node.witnesses[0].target {
                etas_package_metadata::ExistentialWitnessTarget::Implementation {
                    method,
                    value,
                } => break (method, value),
                etas_package_metadata::ExistentialWitnessTarget::Bound => {
                    let [etas_package_metadata::CallableValueSource::Existential(payload)] =
                        node.payload_sources.as_slice()
                    else {
                        panic!("the repack must preserve its concrete producer evidence");
                    };
                    object = *payload;
                }
            }
        };
        assert_eq!(method.name(), "name");
        assert_eq!(
            graph.values[*value as usize]
                .requested_actions
                .effects
                .len(),
            1
        );
    }
}
