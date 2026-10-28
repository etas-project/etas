use super::*;

#[test]
fn shared_text_reassignment_survives_real_cli_checkpoint_resume_and_report() {
    let source = fixture(
        "shared-text-reassignment",
        r#"
module tests.cli.shared_text;
import std.text.len;
import std.runtime.checkpoint;
effect Append { action suffix() -> string; }
flow main(args: Array<string>) -> i32 {
    var text = "";
    var index = 0;
    while index < 256 limit Iterations(512) {
        text = text + "中😀";
        index = index + 1;
    }
    let alias = text;
    handle { text = text + perform Append.suffix(); } with {
        Append.suffix() => { checkpoint("pending-text-append"); resume "!"; }
    };
    if len(text) == 513 && len(alias) == 512 { return 0; }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("shared-text-checkpoints", "dir");
    let trace = temp_path("shared-text-trace", "json");
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
    assert!(artifact.to_string().contains("中😀"));
    assert!(
        !artifact.to_string().contains("Concat"),
        "internal text representation is not a wire format"
    );
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
