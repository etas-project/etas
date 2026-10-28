use super::*;

#[test]
fn nominal_record_equality_survives_real_cli_checkpoint_resume_and_replay() {
    let source = fixture(
        "record-equality",
        r#"
module tests.cli.record_equality;
import std.runtime.checkpoint;
type State = { ready: bool, label: string }
type Report<T> = { status: T, state: State }
flow main(args: Array<string>) -> i32 {
    let left = Report<i32> { status = 200, state = State { ready = true, label = "ok" } };
    let right = Report<i32> { state = State { label = "ok", ready = true }, status = 200 };
    let keys = #{left, right};
    let map: Map<Report<i32>, string> = { left => "found" };
    checkpoint("record-keys");
    var count = 0;
    for key in keys limit Iterations(4) { count = count + 1; }
    if left.status == right.status && left.state == right.state && left == right
        && count == 1 && map.get(right) == Some("found") { return 0; }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("record-equality-checkpoints", "dir");
    let trace = temp_path("record-equality-trace", "json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&source),
            "--checkpoint-dir",
            path(&checkpoints),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let artifact = read_checkpoint_file(&saved);
    assert!(artifact.to_string().contains("found"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
    assert_eq!(fs::read(&saved).unwrap(), before);
    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("replayed interpreter report"), "{stdout}");
    assert!(stdout.contains(r#""type":"i32","value":"0""#), "{stdout}");
}
