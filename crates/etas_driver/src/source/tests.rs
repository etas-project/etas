use super::*;

struct PackageFixture(PathBuf);

impl PackageFixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "etas-driver-reflection-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("src/app")).unwrap();
        std::fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "retained-source"
version = "0.1.0"
[reflection]
retain_bodies = ["app.extra.Worker"]
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            "module app.main; flow main(args: Array<string>) -> i32 { return 0; }",
        )
        .unwrap();
        std::fs::write(
            root.join("src/app/extra.es"),
            "module app.extra; public agent Worker() -> i32 { return 42; }",
        )
        .unwrap();
        Self(root)
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn package_reflection_configuration_reaches_frontend_without_entry_call_edges() {
    assert_retained_package_sources(false);
}

#[test]
fn single_file_package_input_keeps_package_reflection_roots() {
    assert_retained_package_sources(true);
}

fn assert_retained_package_sources(single_file: bool) {
    let fixture = PackageFixture::new();
    let options = DriverOptions {
        workspace: Some(fixture.0.clone()),
        cache_mode: crate::DriverCacheMode::Off,
        ..DriverOptions::default()
    };
    let loaded = load_project(
        &options,
        LoadProjectRequest {
            paths: vec![if single_file {
                fixture.0.join("src/app/main.es")
            } else {
                fixture.0.clone()
            }],
            all: false,
            entry_policy: EntryPolicy::Runnable,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(
        loaded.loaded.input.options.reflection.retain_bodies,
        ["app.extra.Worker"]
    );
    assert_eq!(loaded.loaded.input.sources.len(), 2);
    let result =
        compile_loaded_project(&options, loaded.loaded, SnapshotDetailLevel::Full).unwrap();
    assert!(
        result.response.output.diagnostics.is_empty(),
        "{:?}",
        result.response.output.diagnostics
    );
    let checked = result.response.output.checked.unwrap();
    let method = checked
        .types
        .agent_methods
        .values()
        .find(|method| method.name == "run")
        .unwrap();
    assert!(checked.definition_sources.exposure(method.body).retained);
    assert!(!checked.definition_sources.exposure(method.body).exported);
}
