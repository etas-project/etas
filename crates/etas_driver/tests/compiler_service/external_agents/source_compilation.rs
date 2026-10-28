use super::*;

#[test]
fn a_dependency_agent_with_a_checked_locked_source_environment_can_be_compiled() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Remote(value: bool) -> bool ![] { return value; } public flow lookup() -> i32 { return 7; }",
    );
    let metadata = origin_metadata::consumer(&workspace, "first");
    let root = metadata.package_root.unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"
module app.main;
import first.library.api.Remote;
import std.compiler.{compile, parse_body, CompileError};
flow main() -> bool ![Error<CompileError>] {
    let original = Remote(false);
    let body = match parse_body("return value;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let candidate = compile((^Remote).with_body(body));
    return original == false && candidate(true);
}
"#,
    )
    .unwrap();
    let prepared = etas_driver::compile_runnable_project_with_loaded(
        &origin_metadata::options(&root),
        etas_driver::LoadProjectRequest {
            paths: vec![root],
            all: false,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        etas_frontend::SnapshotDetailLevel::None,
    )
    .unwrap();
    assert!(
        !prepared
            .response
            .output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error),
        "{:?}",
        prepared.response.output.diagnostics
    );
    assert_eq!(prepared.loaded.loaded.input.sources.len(), 2);
    let checked = prepared.response.output.checked.unwrap();
    let definitions =
        prepare_native_definitions(&prepared.loaded, &checked, Default::default()).unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&prepared.loaded)
        .unwrap();
    let result =
        revisions::execute_using_service(checked, definitions, Default::default(), service, true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}
