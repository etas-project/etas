use super::*;

#[cfg(unix)]
#[test]
fn command_result_fields_expose_real_nonzero_status_and_binary_output_after_resume() {
    let root = package_fixture(
        "command-result",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "command-result"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.1" }
[[bin]]
name = "command-result"
module = "app.main"
flow = "main"
[runtime]
default_profile = "test-command"
[runtime.profiles.test-command.command]
allow = ["/bin/sh"]
isolation = "trusted-unconfined"
[runtime.profiles.test-command.policy]
mode = "local-static"
rules = ["command=allow"]
"#,
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.host.command.{CommandResult, command, run};
import std.runtime.checkpoint;
flow exit_value(value: CommandResult) -> i32 { return value.exit_code; }
flow main(args: Array<string>) -> i32 {
    let value = run(command("/bin/sh", ["-c", "LC_ALL=C; export LC_ALL; awk -v fmt=%c%c 'BEGIN { printf fmt, 0, 255 }'; awk -v fmt=%c%c 'BEGIN { printf fmt, 128, 10 }' >&2; exit 7"]), DefaultCommandSandbox);
    let out = (value).stdout;
    let err = value.stderr;
    let observed = (exit_value(value), out, err);
    checkpoint("command-completed");
    if exit_value(value) == 7 { return 0; }
    return 1;
}
"#,
            ),
        ],
    );
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        command
    };
    let (code, stdout, stderr) =
        run_process_with_command(command(), ["pkg", "lock", path(&root)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let checkpoints = root.join("checkpoints");
    let trace = root.join("trace.json");
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "--format",
            "json",
            "run",
            path(&root),
            "--profile",
            "test-command",
            "--checkpoint-dir",
            path(&checkpoints),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let expected = serde_json::json!({"kind":"number","type":"i32","value":"0"});
    assert_eq!(report["value"], expected, "{stdout}\n{stderr}");
    let checkpoint = checkpoints.join("checkpoint-0.json");
    let original = fs::read(&checkpoint).unwrap();
    let artifact = read_checkpoint_file(&checkpoint);
    let mut pending = vec![&artifact];
    let mut results = 0;
    let observed = serde_json::json!({"kind":"tuple","values":[
        {"kind":"number","type":"i32","value":"7"},
        {"kind":"bytes","value":[0,255]},
        {"kind":"bytes","value":[128,10]},
    ]});
    let mut projections = 0;
    while let Some(value) = pending.pop() {
        if value == &observed {
            projections += 1;
        }
        match value {
            serde_json::Value::Object(fields) => {
                if fields.get("kind").and_then(serde_json::Value::as_str) == Some("command_result")
                {
                    assert_eq!(fields["exit_code"], 7);
                    assert_eq!(fields["stdout"], serde_json::json!([0, 255]));
                    assert_eq!(fields["stderr"], serde_json::json!([128, 10]));
                    results += 1;
                }
                pending.extend(fields.values());
            }
            serde_json::Value::Array(values) => pending.extend(values),
            _ => {}
        }
    }
    assert!(
        results > 0,
        "checkpoint must retain the native command result"
    );
    assert!(
        projections > 0,
        "checkpoint must retain the exact bytes extracted by source field reads"
    );
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "--format",
            "json",
            "resume",
            "0",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let resumed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(resumed["value"], expected);
    assert_eq!(fs::read(&checkpoint).unwrap(), original);
}
