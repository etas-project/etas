use super::*;

fn reexport_cli<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    run_process_with_command(command, args, "")
}

#[test]
fn source_flow_reexports_with_inferred_effects_check_run_and_resume() {
    let root = package_fixture(
        "source-reexports",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "source-reexports"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.1" }
[[bin]]
name = "source-reexports"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import lib.api.exposed as invoke;
flow main(args: Array<string>) -> i32 {
    let callback = invoke;
    let first = invoke();
    if first + callback() == 14 { return 0; }
    return 1;
}
"#,
            ),
            (
                "src/lib/leaf.es",
                r#"
module lib.leaf;
import std.io.println;
import std.runtime.checkpoint;
public flow emit() -> i32 {
    println("leaf");
    checkpoint("leaf");
    return 7;
}
"#,
            ),
            (
                "src/lib/facade.es",
                "module lib.facade; public import lib.leaf.emit as exposed;",
            ),
            (
                "src/lib/api.es",
                "module lib.api; public import lib.facade.*;",
            ),
            (
                "src/lib/unrelated.es",
                "module lib.unrelated; public flow emit() -> i32 { return 99; }",
            ),
        ],
    );
    let (code, stdout, stderr) = reexport_cli(["pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let (code, stdout, stderr) = reexport_cli(["check", "--all", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let checkpoints = temp_path("reexport-checkpoints", "dir");
    let trace = temp_path("reexport-trace", "json");
    let (code, stdout, stderr) = reexport_cli([
        "run",
        path(&root),
        "--checkpoint-dir",
        path(&checkpoints),
        "--trace-out",
        path(&trace),
    ]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert_eq!(stdout, "leaf\nleaf\n");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(&trace).unwrap()).unwrap();
    assert_eq!(
        report["value"],
        serde_json::json!({ "kind": "number", "type": "i32", "value": "0" })
    );
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let (code, stdout, stderr) =
        reexport_cli(["resume", "0", "--checkpoint-dir", path(&checkpoints)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(stdout.lines().filter(|line| *line == "leaf").count(), 1);
    assert_eq!(fs::read(&saved).unwrap(), before);
    let (code, stdout, stderr) = reexport_cli(["replay", path(&trace)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("replayed interpreter report"), "{stdout}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
}
