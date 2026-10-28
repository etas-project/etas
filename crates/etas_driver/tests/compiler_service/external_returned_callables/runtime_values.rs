use super::*;

#[test]
fn a_compiled_revision_executes_a_returned_callback_consumer_with_a_captured_input() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public flow make() -> (((i32) -> i32 ![]) -> i32 ![]) ![] {
    return (f: (i32) -> i32 ![]) => { return f(20) + f(21); };
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
    let invoke = make();
    return invoke((value: i32) => { return value + offset; });
}
flow main() -> i32 ![Error<CompileError>] {
    if (^make).body != None { return -1; }
    let revised = compile(^Worker);
    return revised(1);
}
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
fn concrete_returned_metadata_values_execute_only_after_locked_source_completion() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
private flow step(value: i32) -> i32 ![] { return value + 1; }
public flow make(flag: bool) -> ((i32) -> i32 ![]) ![] {
    if flag { return step; }
    return (value: i32) => { return value + 2; };
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
agent Worker(value: i32) -> i32 ![] { let selected = make(true); return selected(value); }
flow main() -> i32 ![Error<CompileError>] {
    if (^make).body != None { return -1; }
    let revised = compile(^Worker);
    return revised(18);
}
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
    assert_eq!(
        checked.sources.sources.len(),
        2,
        "only the locked callable's module should be materialized"
    );
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
        Some(&etas_interpreter::api::InterpValue::i32(19))
    );
}
