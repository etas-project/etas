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

#[test]
fn sdk_keeps_public_metadata_checked_bodies_and_sources_together_after_relocation() {
    let options = DriverOptions {
        cache_mode: crate::DriverCacheMode::Off,
        ..Default::default()
    };
    const CHILD_ROOT: &str = "ETAS_STD_SDK_BUILD_TEST_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        sdk::build_standard_library(
            &options,
            sdk::StdLibraryBuildRequest {
                manifest_path: root.join("etas.toml"),
                out_dir: root.join("second/std"),
            },
        )
        .unwrap();
        return;
    }
    let fixture = Fixture::new();
    fs::write(fixture.0.join("library/std/text/extra.es"), "module std.text; import std.text.char_count; public flow width(value: string) -> usize { return char_count(value); } public flow decorate(value: string) -> string { return value + \"!\"; }").unwrap();
    let build = |name| {
        sdk::build_standard_library(
            &options,
            sdk::StdLibraryBuildRequest {
                manifest_path: fixture.0.join("etas.toml"),
                out_dir: fixture.0.join(name),
            },
        )
        .unwrap()
    };
    let first = build("sdk/std");
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            std::thread::current().name().unwrap(),
            "--nocapture",
        ])
        .env(CHILD_ROOT, &fixture.0)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let second = fixture.0.join("second/std");
    let second_manifest: sdk::StdSdkManifest =
        serde_json::from_slice(&fs::read(second.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(first.manifest, second_manifest);
    assert_eq!(
        fs::read(first.out_dir.join(&first.manifest.artifact)).unwrap(),
        fs::read(second.join(&second_manifest.artifact)).unwrap()
    );
    let moved = fixture.0.join("relocated/std");
    fs::create_dir_all(moved.parent().unwrap()).unwrap();
    fs::rename(first.out_dir, &moved).unwrap();
    fs::remove_dir_all(fixture.0.join("library")).unwrap();
    let bytes = fs::read(moved.join(&first.manifest.artifact)).unwrap();
    assert_eq!(
        etas_package_metadata::blake3_hash(&bytes),
        first.manifest.artifact_hash
    );
    assert_eq!(
        etas_package::package_source_payload_hash(&moved).unwrap(),
        first.manifest.source_payload_hash
    );
    let path = moved.join(&first.manifest.artifact);
    let (_, public) = etas_package_metadata::package_metadata_from_artifact(&path, &bytes).unwrap();
    assert_eq!(public.package.name, "std");
    let decoded = etas_package_metadata::decode_metadata_artifact(&path, &bytes).unwrap();
    assert!(
        decoded
            .sections
            .contains_key(&etas_package_metadata::MetadataSectionKind::LibraryBodies)
    );
    let sdk = sdk::load_standard_library_sdk(moved.parent().unwrap()).unwrap();
    assert_eq!(sdk.manifest, first.manifest);
    let checked = sdk.image.into_checked_from_image(&sdk.root).unwrap();
    assert_eq!(checked.hir.items.len(), 3);
    assert!(
        checked.sources.sources.iter().all(|source| source
            .path
            .as_ref()
            .unwrap()
            .starts_with(&moved))
    );
}
