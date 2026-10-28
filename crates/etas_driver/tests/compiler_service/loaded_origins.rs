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
