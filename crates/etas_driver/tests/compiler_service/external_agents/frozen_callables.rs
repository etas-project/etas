use super::*;

#[test]
fn a_dependency_returning_a_nominal_closure_keeps_its_frozen_contract_across_import_aliases() {
    let workspace = origin_metadata::workspace_with_source_config(
        r#"module library.api;
public type UserId = string;
public agent Remote(value: UserId) -> (() -> UserId ![]) { return () => value; }
public flow lookup() -> i32 { return 7; }
"#,
        "[reflection]\nexport_bodies = [\"library.api.Remote\"]\n",
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let published = metadata
        .public_metadata
        .definition_descriptors
        .as_ref()
        .unwrap()
        .headers
        .as_ref()
        .unwrap()
        .snapshots
        .as_ref()
        .unwrap()
        .agents[0]
        .revision
        .contract;
    for (alias, namespace) in [
        ("first", "first.library"),
        ("second", "second.library"),
        ("library", "library"),
    ] {
        let root = origin_metadata::consumer(&workspace, alias)
            .package_root
            .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
import {namespace}.api.{{Remote, UserId}};
import std.compiler.{{compile, CompileError}};
flow main() -> bool ![Error<CompileError>] {{
    let value = UserId("canonical frozen region");
    let original = Remote(value);
    let revised = compile(^Remote);
    return original() == value && revised(value)() == value;
}}
"#
            ),
        )
        .unwrap();
        let compiled = dependency_namespaces::compile(&root);
        assert!(
            compiled.response.output.diagnostics.is_empty(),
            "{}: {:?}",
            alias,
            compiled.response.output.diagnostics
        );
        let checked = compiled.response.output.checked.unwrap();
        assert_eq!(checked.sources.sources.len(), 2);
        let definitions =
            prepare_native_definitions(&compiled.loaded, &checked, Default::default()).unwrap();
        let reflection = *checked.types.reflections.keys().next().unwrap();
        let frozen = definitions.catalog().freeze_agent(reflection).unwrap();
        assert_eq!(
            frozen.contract().fingerprint(),
            published,
            "alias {alias} changed the canonical frozen contract"
        );
        let result = revisions::execute_prepared(
            (checked, definitions),
            Some(2_000),
            Default::default(),
            true,
        );
        assert!(
            result.diagnostics.is_empty(),
            "{}: {:?}",
            alias,
            result.diagnostics
        );
        assert_eq!(
            result.value(),
            Some(&etas_interpreter::api::InterpValue::Bool(true))
        );
    }
}
