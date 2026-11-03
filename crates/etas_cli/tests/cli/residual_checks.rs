use super::*;

const MANIFEST: &str = r#"
[package]
name = "residual-checks"
version = "0.1.0"
edition = "2026"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#;

const ORDERED: &str = r#"
module app.main;
import std.runtime.checkpoint;
effect Gate { action check() -> unit; action send() -> unit; }
spec Safe: trace = Gate.check >> Gate.send;
flow main(args: Array<string>) -> i32 ![] ~ Safe {
    var sent = 0;
    handle {
        perform Gate.check();
        checkpoint("after-guard");
        perform Gate.send();
    } with {
        Gate.check() => { resume (); }
        Gate.send() => { sent = sent + 1; resume (); }
    };
    return sent;
}
"#;

#[test]
fn residual_cli_check_run_and_checkpoint_resume_use_the_real_interpreter_target() {
    let root = package_fixture(
        "residual-resume",
        &[("etas.toml", MANIFEST), ("src/app/main.es", ORDERED)],
    );
    let (code, stdout, stderr) = run_process(["check", path(&root)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let checkpoints = temp_path("residual-checkpoints", "dir");
    let trace = temp_path("residual-trace", "json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(&root),
            "--checkpoint-dir",
            path(&checkpoints),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(&trace).unwrap()).unwrap();
    assert_eq!(
        report["value"],
        serde_json::json!({"kind": "number", "type": "i32", "value": "1"})
    );
    let checkpoint = checkpoints.join("checkpoint-0.json");
    let saved = read_checkpoint_file(&checkpoint);
    assert_eq!(saved["schema"], "etas.cli.interpreter-checkpoint.v67");
    assert_eq!(
        saved["checkpoint"]["machine"]["residual"]["scopes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for process in 0..8 {
        let (code, stdout, stderr) =
            run_process(["resume", "0", "--checkpoint-dir", path(&checkpoints)], "");
        assert_eq!(code, 0, "resume process {process}: {stdout}\n{stderr}");
        assert!(stdout.contains(r#""type":"i32","value":"1""#), "{stdout}");
    }
}

#[test]
fn residual_cli_failed_guard_is_rejected_before_target_dispatch() {
    let source = r#"
module app.main;
effect Gate { action check() -> unit; action send() -> unit; }
spec Safe: trace = Gate.check >> Gate.send;
flow main(args: Array<string>) -> i32 ![] ~ Safe {
    var sent = 0;
    let ignored = { handle { perform Gate.check(); } with {
        Gate.check() => { finish perform Error<i32>.raise(7); }
    } }?;
    handle { perform Gate.send(); } with {
        Gate.send() => { sent = sent + 1; resume (); }
    };
    return sent;
}
"#;
    let root = package_fixture(
        "residual-failed-guard",
        &[("etas.toml", MANIFEST), ("src/app/main.es", source)],
    );
    let (code, stdout, stderr) = run_process(["check", path(&root)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let (code, stdout, stderr) = run_process(["run", path(&root)], "");
    assert_eq!(code, 3, "{stdout}\n{stderr}");
    assert!(stderr.contains("TraceSpecResidualViolation"), "{stderr}");
    assert!(stderr.contains("Gate.send"), "{stderr}");
}
