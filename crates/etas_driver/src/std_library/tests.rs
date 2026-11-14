use super::*;
use std::fs;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("etas-std-driver-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("library/std/text")).unwrap();
        fs::write(root.join("etas.toml"), "[package]\nname = 'std'\nversion = '0.2.2'\nedition = '2026'\n[source]\nroot = 'library'\n").unwrap();
        fs::write(root.join("library/std/text/mod.es"), "module std.text;").unwrap();
        fs::write(
            root.join("library/std/text/base.es"),
            "module std.text; public flow identity(value: string) -> string { return value; }",
        )
        .unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn manifest_source_root_discovers_new_std_parts_with_canonical_package_and_source_owners() {
    let fixture = Fixture::new();
    let options = DriverOptions {
        cache_mode: crate::DriverCacheMode::Off,
        ..Default::default()
    };
    let check = || {
        check_standard_library_source(
            &options,
            StdLibraryCheckRequest {
                manifest_path: fixture.0.join("etas.toml"),
            },
        )
        .unwrap()
    };
    let before = check();
    assert!(
        before.response.output.checked.is_some(),
        "{:?}",
        before.response.output.diagnostics
    );
    assert_eq!(
        before.response.output.sources.as_ref().unwrap().files.len(),
        2
    );
    fs::write(fixture.0.join("library/std/text/extra.es"), "module std.text; import std.text.char_count; public flow width(value: string) -> usize { return char_count(identity(value)); }").unwrap();
    let after = check();
    assert!(
        after.response.output.checked.is_some(),
        "{:?}",
        after.response.output.diagnostics
    );
    assert_eq!(after.manifest.source.root, PathBuf::from("library"));
    assert_eq!(before.definition_origin, after.definition_origin);
    let sources = after.response.output.sources.as_ref().unwrap();
    assert_eq!(sources.files.len(), 3);
    assert_eq!(sources.stable_source_keys.len(), 3);
    let old_sources = before.response.output.sources.as_ref().unwrap();
    for old_file in &old_sources.files {
        let new_file = sources
            .files
            .iter()
            .find(|file| file.path == old_file.path)
            .unwrap();
        let old_key = old_sources
            .stable_source_keys
            .iter()
            .find(|entry| entry.source == old_file.id)
            .unwrap();
        let new_key = sources
            .stable_source_keys
            .iter()
            .find(|entry| entry.source == new_file.id)
            .unwrap();
        assert_eq!(old_key.key, new_key.key);
    }
    let modules = after.response.output.modules.as_ref().unwrap();
    let text = modules
        .modules
        .get(
            modules.by_path[&etas_frontend::ModulePath {
                segments: vec!["std".into(), "text".into()],
            }],
        )
        .unwrap();
    assert!(text.visibility_exports.items.contains_key("width"));
    assert!(
        after
            .response
            .output
            .entry
            .as_ref()
            .unwrap()
            .resolved
            .is_none()
    );
}
