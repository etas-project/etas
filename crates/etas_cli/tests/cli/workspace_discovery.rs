use super::*;

fn project() -> PathBuf {
    package_fixture(
        "bounded-workspace-discovery",
        &[
            (
                "etas.toml",
                r#"[package]
name = "bounded-workspace-discovery"
version = "0.1.0"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
profile = "readonly"
[runtime]
default_profile = "readonly"
[runtime.profiles.readonly.filesystem.regions."app.main.Root"]
root = "workspace"
read = true
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;
import std.fs.{Region, WorkspacePath, IOError, ListLimitError, root, relative_name, list_limited, path, read_bytes_limited, write_bytes};
import std.effects.Fs;
import std.io.println;
import std.codec.text.utf8_encode;
type Root; impl Root ~ Region;
flow selected<R ~ Region>() -> WorkspacePath<R> { return root<R>(); }
public flow main(args: Array<string>) -> i32 {
    let directory: WorkspacePath<Root> = selected();
    let entries = list_limited(directory, 3, 13);
    for entry in entries limit Iterations(4) { println(relative_name(entry)); }
    return 0;
}
public flow sub(args: Array<string>) -> i32 {
    let directory = std.result.unwrap(path<Root>("sub"));
    for entry in list_limited(directory, 1, 9) limit Iterations(2) {
        println(relative_name(entry));
        let body = read_bytes_limited(entry, 3);
        if body != utf8_encode("ccc") { return 7; }
    }
    return 0;
}
public flow count_overflow(args: Array<string>) -> i32 {
    return handle { let entries = list_limited(root<Root>(), 2, 13); println("UNEXPECTED"); 7 } with {
        Error<ListLimitError>.raise(ListLimitError.LimitExceeded(count, bytes)) => { println("LIMIT"); finish 0; }
    };
}
public flow byte_overflow(args: Array<string>) -> i32 {
    return handle { let entries = list_limited(root<Root>(), 3, 12); println("UNEXPECTED"); 7 } with {
        Error<ListLimitError>.raise(_) => { println("LIMIT"); finish 0; }
    };
}
public flow denied(args: Array<string>) -> i32 ![Fs.write<Root>, Error<IOError>] {
    write_bytes(std.result.unwrap(path<Root>("blocked")), utf8_encode("not allowed")); return 0;
}
public flow empty(args: Array<string>) -> i32 {
    let entries = list_limited(std.result.unwrap(path<Root>("empty")), 0, 0);
    if entries.len() != 0 { return 7; }
    println("EMPTY"); return 0;
}
"#,
            ),
            ("workspace/a.txt", "aaa"),
            ("workspace/b.txt", "bbb"),
            ("workspace/sub/c.txt", "ccc"),
        ],
    )
}

#[test]
fn readonly_profile_discovers_root_and_subdirectory_with_relative_provenance_and_bounds() {
    let root = project();
    for (flow, expected) in [
        ("main", vec!["a.txt", "b.txt", "sub"]),
        ("sub", vec!["sub/c.txt"]),
        ("count_overflow", vec!["LIMIT"]),
        ("byte_overflow", vec!["LIMIT"]),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        let (code, stdout, stderr) =
            run_process_with_command(command, ["run", path(&root), "--flow", flow], "");
        assert_eq!(code, 0, "{flow}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        let lines = stdout
            .lines()
            .filter(|line| !line.starts_with("run value:") && !line.starts_with("events:"))
            .collect::<Vec<_>>();
        assert_eq!(lines, expected, "{flow}: {stdout}");
        assert!(!stdout.contains("UNEXPECTED"));
        assert!(!stdout.contains(path(&root)));
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", path(&root), "--flow", "denied"], "");
    assert_eq!(code, 3, "{stdout}\n{stderr}");
    assert!(!root.join("workspace/blocked").exists());
    fs::create_dir(root.join("workspace/empty")).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", path(&root), "--flow", "empty"], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.lines().any(|line| line == "EMPTY"), "{stdout}");
}

#[test]
fn workspace_discovery_retains_region_evidence_and_opaque_path_types() {
    let root = project();
    for source in [
        "module app.main; type NotRegion; public flow main(args: Array<string>) -> i32 { let value = std.fs.root<NotRegion>(); return 0; }",
        "module app.main; public flow main(args: Array<string>) -> i32 { let value = std.fs.relative_name(\"not a path\"); return 0; }",
        "module app.main; public flow main(args: Array<string>) -> i32 { let value = std.fs.list_limited(\"not a path\", 1, 10); return 0; }",
    ] {
        fs::write(root.join("src/app/main.es"), source).unwrap();
        let (code, stdout, stderr) = run_process(["check", path(&root)], "");
        assert_ne!(
            code, 0,
            "invalid discovery source accepted: {stdout}\n{stderr}"
        );
        assert!(
            stderr.contains("type::"),
            "expected a type-check rejection, not an invalid entry or missing Host: {stdout}\n{stderr}"
        );
    }
}
