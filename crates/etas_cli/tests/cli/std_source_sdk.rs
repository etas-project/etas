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
        "module std.extra; public flow sdk_echo<T>(value: T) -> T { return value; }",
    )
    .unwrap();
    fs::create_dir_all(source.join("library/std/prelude")).unwrap();
    fs::write(
        source.join("library/std/prelude/mod.es"),
        "module std.prelude; public import std.extra.sdk_echo as bundled_echo;",
    )
    .unwrap();
    let app = root.join("app.es");
    fs::write(&app, "module app.main; import std.text.join; import std.extra.sdk_echo; flow main(args: Array<string>) -> i32 { let assemble = join; let text = sdk_echo(bundled_echo(assemble([\"a\", \"b\"], \"|\"))); if text == \"a|b\" { return 0; } if text == \"rebuilt:a|b\" { return 2; } return 99; }").unwrap();
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
    build();
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
    // A separate legacy session keeps its original join implementation.
    let legacy = root.join("legacy.es");
    fs::write(&legacy, "module app.main; import std.text.join; flow main(args: Array<string>) -> i32 { if join([\"a\", \"b\"], \"|\") == \"a|b\" { return 0; } return 99; }").unwrap();
    let output = invoke(&[
        "--cache",
        "off",
        "run",
        path(&legacy),
        "--std-provider",
        "legacy",
        "--format",
        "json",
    ]);
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["value"]["value"], "0");
    fs::remove_dir_all(root).unwrap();
}
