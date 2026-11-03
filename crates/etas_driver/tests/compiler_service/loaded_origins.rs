use super::*;
use etas_driver::{
    DriverCacheMode, DriverOptions, LoadProjectRequest, SourceDependencyMode,
    compiler::prepare_native_definitions, load_project,
};

struct Package(std::path::PathBuf);

impl Drop for Package {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn loaded_standalone_flow_origin_survives_candidate_compilation() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = Package(std::env::temp_dir().join(format!(
        "etas-standalone-compiler-{}-{nonce}",
        std::process::id()
    )));
    std::fs::create_dir_all(&directory.0).unwrap();
    let path = directory.0.join("main.es");
    std::fs::write(
        &path,
        r#"
module app.main;
import std.compiler.{compile, parse_body, CompileError};
flow identity(value: i32) -> i32 { return value; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return value;") {
        Ok(body) => body,
        Err(_) => { return -1; },
    };
    let candidate = compile((^identity).with_body(body));
    return candidate(42);
}

"#,
    )
    .unwrap();
    let mut loaded = load_project(
        &DriverOptions {
            workspace: Some(directory.0.clone()),
            cache_mode: DriverCacheMode::Off,
            ..Default::default()
        },
        LoadProjectRequest {
            paths: vec![path.clone()],
            all: false,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.mode, etas_driver::DriverProjectMode::SingleFile);
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.identity".into());
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(
        !output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error),
        "{:?}",
        output.diagnostics
    );
    let checked = output.checked.unwrap();
    let package = loaded
        .loaded
        .input
        .environment
        .source_package(&loaded.loaded.input.sources[0])
        .unwrap();
    let origins = etas_driver::compiler::source_definition_origins(&loaded, &checked).unwrap();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins.values().next().unwrap().package(), package);
    std::fs::write(&path, "this is not the checked source").unwrap();
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(42))
    );
}

#[test]
fn loaded_dependency_overlay_carries_owner_without_definition_metadata() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 { return 7; }",
    );
    let consumer = origin_metadata::consumer(&workspace, "first");
    let root = consumer.package_root.unwrap();
    let loaded = load_project(
        &origin_metadata::options(&root),
        LoadProjectRequest {
            paths: vec![root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::All,
        },
    )
    .unwrap();
    let source = loaded
        .loaded
        .input
        .sources
        .iter()
        .find(|source| {
            matches!(
                source.kind,
                etas_frontend::SourceKind::DependencySourceOverlay { .. }
            )
        })
        .expect("dependency overlay source");
    let mut environment = loaded.loaded.input.environment.clone();
    let owner = environment
        .source_package(source)
        .expect("locked source owner");
    environment.external_definition_origins.clear();
    assert_eq!(environment.source_package(source), Ok(owner));
    let library = workspace.0.join("library");
    let owner_project = load_project(
        &origin_metadata::options(&library),
        LoadProjectRequest {
            paths: vec![library],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let checked = origin_metadata::check(&owner_project);
    let origins =
        etas_driver::compiler::source_definition_origins(&owner_project, &checked).unwrap();
    assert!(origins.values().any(|origin| origin.package() == owner));
}

#[test]
fn checked_flow_origin_rejects_changed_external_type_metadata() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public type Marker = string; public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let checked = origin_metadata::check(&loaded);
    let signature = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter_mut()
        .flat_map(|metadata| &mut metadata.types)
        .find(|signature| signature.path.last().is_some_and(|name| name == "Marker"))
        .unwrap();
    signature.ty = Some(etas_frontend::ProjectExternalTypeInput::Primitive(
        "i32".into(),
    ));
    assert!(!checked.uses_environment(&loaded.loaded.input.environment));
    assert!(
        etas_driver::compiler::source_definition_origins(&loaded, &checked)
            .unwrap_err()
            .to_string()
            .contains("checked environment differs from the prepared driver input")
    );
}

#[test]
fn loaded_package_origins_reach_real_compiler_publication_without_rereading_mutable_source() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = Package(std::env::temp_dir().join(format!(
        "etas-loaded-compiler-{}-{nonce}",
        std::process::id()
    )));
    std::fs::create_dir_all(package.0.join("src/app")).unwrap();
    std::fs::write(
        package.0.join("etas.toml"),
        r#"
[package]
name = "immutable-native-compiler"
version = "0.1.0"
[reflection]
retain_bodies = ["app.main.Worker"]
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
    )
    .unwrap();
    let path = package.0.join("src/app/main.es");
    std::fs::write(
        &path,
        r#"
module app.main;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return Worker();") {
        Ok(body) => body,
        Err(_) => { return -1; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised();
}
"#,
    )
    .unwrap();
    let loaded = load_project(
        &DriverOptions {
            workspace: Some(package.0.clone()),
            cache_mode: DriverCacheMode::Off,
            ..Default::default()
        },
        LoadProjectRequest {
            paths: vec![package.0.clone()],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    std::fs::write(&path, "this is not the frozen program").unwrap();
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(500), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(1))
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                etas_interpreter::api::WorkflowEvent::HostTrace(
                    etas_host::TraceEvent::HostRequestStarted {
                        kind: etas_host::HostRequestKind::Compiler,
                        ..
                    }
                )
            ))
            .count(),
        1,
    );
}
