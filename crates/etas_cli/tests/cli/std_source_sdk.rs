use super::*;

#[test]
fn installed_std_executes_new_generic_flow_and_rebuilt_join_body_in_external_processes() {
    let root = temp_path("std-source-sdk", "workspace");
    let source = root.join("std-source");
    fs::create_dir_all(source.join("library/std/text")).unwrap();
    fs::create_dir_all(source.join("library/std/extra")).unwrap();
    fs::write(source.join("etas.toml"), "[package]\nname = 'std'\nversion = '0.1.0'\nedition = '2026'\n[source]\nroot = 'library'\n").unwrap();
    for (file, id) in [
        ("assembly.es", etas_std::StdLibrarySourceId::TextAssembly),
        ("format.es", etas_std::StdLibrarySourceId::TextFormat),
    ] {
        fs::write(
            source.join("library/std/text").join(file),
            etas_std::standard_library_source(id).source,
        )
        .unwrap();
    }
    fs::write(
        source.join("library/std/extra/mod.es"),
        r#"module std.extra;
public flow sdk_echo<T>(value: T) -> T { return value; }
public type Parcel<T> = { value: T };
public enum Reply<T> { Empty, Item(T), Named { value: T }, }
public flow pack(value: string) -> Parcel<string> { return Parcel { value = value }; }
public flow read_reply(value: Reply<string>) -> string {
    return match value { Reply.Empty => "", Reply.Item(text) => text, Reply.Named { value } => value };
}
"#,
    )
    .unwrap();
    fs::create_dir_all(source.join("library/std/prelude")).unwrap();
    fs::write(
        source.join("library/std/prelude/mod.es"),
        "module std.prelude; public import std.extra.sdk_echo as bundled_echo;",
    )
    .unwrap();
    fs::write(
        source.join("library/std/text/native.es"),
        r#"module std.text;
@std.binding("std.text.trim")
public flow trim(value: string) -> string ![] {}
@std.binding("std.collections.array_push")
public flow append<U>(values: Array<U>, value: U) -> Array<U> ![] {}
"#,
    )
    .unwrap();
    let app = root.join("app.es");
    fs::write(
        &app,
        r#"module app.main;
import std.text.{join, trim, append};
import std.extra.{sdk_echo, Parcel, Reply, pack, read_reply};
flow main(args: Array<string>) -> i32 {
    let assemble = join;
    let strip = trim;
    let values = append([strip(" a ")], strip(" b "));
    let parcel: Parcel<string> = sdk_echo(pack(bundled_echo(assemble(values, "|"))));
    let text = read_reply(Reply.Named { value = parcel.value });
    if read_reply(Reply.Item(text)) != text { return 98; }
    if read_reply(Reply.Empty) != "" { return 97; }
    if text == "a|b" { return 0; }
    if text == " A | B " { return 3; }
    if text == "rebuilt:a|b" { return 2; }
    return 99;
}
"#,
    )
    .unwrap();
    let sdk = root.join("sdk");
    let invoke = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_etas"))
            .args(args)
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let build = || {
        invoke(&[
            "--cache",
            "off",
            "std",
            "build",
            "--manifest-path",
            path(&source.join("etas.toml")),
            "--out-dir",
            path(&sdk.join("std")),
        ]);
    };
    let cache = root.join("cache");
    let execute = || {
        serde_json::from_str::<serde_json::Value>(&invoke(&[
            "--cache",
            "read-write",
            "--cache-root",
            path(&cache),
            "run",
            path(&app),
            "--std-provider",
            "artifact",
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]))
        .unwrap()
    };
    build();
    let before = execute();
    assert_eq!(before["value"]["value"], "0");
    let native = source.join("library/std/text/native.es");
    let binding = fs::read_to_string(&native).unwrap();
    fs::write(
        &native,
        binding.replace("std.text.trim", "std.text.uppercase"),
    )
    .unwrap();
    build();
    assert_eq!(execute()["value"]["value"], "3");
    fs::write(native, binding).unwrap();
    let assembly = source.join("library/std/text/assembly.es");
    let text = fs::read_to_string(&assembly).unwrap();
    let start = text.find("public flow join(").unwrap();
    let end = text.find("public flow repeat(").unwrap();
    fs::write(
        assembly,
        format!(
            "{}{}{}",
            &text[..start],
            text[start..end].replace("return output;", "return \"rebuilt:\" + output;"),
            &text[end..]
        ),
    )
    .unwrap();
    invoke(&[
        "--cache",
        "off",
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
        "std",
        "build",
        "--manifest-path",
        path(&source.join("etas.toml")),
        "--out-dir",
        path(&sdk.join("std")),
    ]);
    fs::remove_dir_all(source).unwrap();
    let after = execute();
    assert_eq!(after["value"]["value"], "2");
    // Install the actual CLI next to the SDK, then move the whole toolchain.
    // The new process discovers std from its executable, outside the repository.
    let prefix = root.join("prefix");
    invoke(&[
        "--cache",
        "off",
        "std",
        "install",
        "--manifest-path",
        path(&sdk.join("std/etas.toml")),
        "--prefix",
        path(&prefix),
    ]);
    let moved = root.join("moved-prefix");
    fs::rename(prefix, &moved).unwrap();
    for command in ["check", "run"] {
        let output = Command::new(moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)))
            .args([
                "--cache",
                "off",
                command,
                path(&app),
                "--std-provider",
                "artifact",
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "run" {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["value"]["value"], "2");
        }
    }
    // The exact same application runs in two independent provider sessions.
    let common = root.join("common.es");
    fs::write(
        &common,
        r#"module app.main;
import std.text.join;
flow main(args: Array<string>) -> i32 {
    let text = join(["a", "b"], "|");
    if text == "a|b" { return 0; }
    if text == "rebuilt:a|b" { return 2; }
    return 99;
}"#,
    )
    .unwrap();
    for (provider, expected) in [("artifact", "2"), ("legacy", "0")] {
        let output = invoke(&[
            "--cache",
            "off",
            "run",
            path(&common),
            "--std-provider",
            provider,
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]);
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["value"]["value"], expected, "{provider}");
    }
    // Remove only the checked bodies, leaving valid public metadata and digests.
    // Artifact mode must report the missing implementation despite source and
    // legacy implementations remaining available.
    let manifest_path = sdk.join("std/manifest.json");
    let mut manifest: etas_driver::std_library::sdk::StdSdkManifest =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let artifact = sdk.join("std").join(&manifest.artifact);
    let decoded =
        etas_package_metadata::decode_metadata_artifact(&artifact, &fs::read(&artifact).unwrap())
            .unwrap();
    let sections = decoded
        .sections
        .into_iter()
        .filter(|(kind, _)| *kind != etas_package_metadata::MetadataSectionKind::LibraryBodies)
        .map(|(kind, payload)| etas_package_metadata::EncodedMetadataSection { kind, payload })
        .collect();
    let bodyless =
        etas_package_metadata::encode_metadata_artifact(&decoded.header, sections).unwrap();
    manifest.artifact_hash = etas_package_metadata::blake3_hash(&bodyless);
    fs::write(artifact, bodyless).unwrap();
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    let missing = Command::new(env!("CARGO_BIN_EXE_etas"))
        .args([
            "--cache",
            "off",
            "run",
            path(&common),
            "--std-provider",
            "artifact",
            "--sysroot",
            path(&sdk),
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr)
            .contains("std artifact is missing checked library bodies"),
        "{}",
        String::from_utf8_lossy(&missing.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}
