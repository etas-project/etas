use super::*;

#[test]
fn binary_text_operands_preserve_effect_order_through_cli_file_checkpoint_resume() {
    let source = fixture(
        "binary-text-operands",
        r#"
module tests.cli.binary_text_operands;
import std.io.println;
import std.runtime.checkpoint;
effect Append { action suffix() -> string; }
type Row = { text: string }
flow build(prefix: string) -> string {
    var text = prefix;
    var i = 0;
    while i < 64 limit Iterations(128) {
        text = text + "中😀";
        i = i + 1;
    }
    return text;
}
flow main(args: Array<string>) -> i32 {
    var left = build("L");
    let right = build("R");
    let left_alias = left;
    let right_alias = right;
    var rows = [Row { text = left }];
    let rows_alias = rows;
    handle { left = left + perform Append.suffix(); } with {
        Append.suffix() => {
            println("right operand");
            checkpoint("pending-binary-text");
            if left != left_alias || right != right_alias { abort("operand changed before commit"); }
            left = "handler changed the slot";
            resume right;
        }
    };
    rows[0].text = rows[0].text + right;
    let combined = left + (right + "tail");
    var prepended = "tail";
    var index = 0;
    while index < 64 limit Iterations(128) {
        prepended = "中😀" + prepended;
        index = index + 1;
    }
    checkpoint("completed-binary-text");
    if left == left_alias + right_alias && rows[0].text == left
        && rows_alias[0].text == left_alias && left_alias == build("L")
        && right_alias == build("R") && combined == left + right_alias + "tail"
        && prepended == build("") + "tail" {
        return 0;
    }
    return 1;
}
"#,
    );
    let checkpoints = temp_path("binary-text-checkpoints", "dir");
    let trace = temp_path("binary-text-trace", "json");
    let keys = etas_host::TestWorkspace::create("binary-text-private-keys").unwrap();
    let key_dir = keys.path().join("private");
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command.env_clear().env("HOME", keys.path()).env(
            "PATH",
            std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
        );
        command
    };
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "run",
            path(&source),
            "--checkpoint-dir",
            path(&checkpoints),
            "--checkpoint-key-dir",
            path(&key_dir),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout.matches("right operand").count(), 1, "{stdout}");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(&trace).unwrap()).unwrap();
    assert_eq!(report["value"]["value"], "0", "{report}");
    let saved = checkpoints.join("checkpoint-0.json");
    let before = fs::read(&saved).unwrap();
    let artifact = read_checkpoint_file(&saved);
    assert!(!artifact.to_string().contains("Concat"));
    for id in ["0", "1"] {
        let resumed_trace = temp_path("binary-text-resumed-trace", "json");
        let (code, stdout, stderr) = run_process_with_command(
            command(),
            [
                "resume",
                id,
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&key_dir),
                "--trace-out",
                path(&resumed_trace),
            ],
            "",
        );
        assert_eq!(code, 0, "{id}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert!(
            !stdout.contains("right operand"),
            "repeated completed Console call: {stdout}"
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(resumed_trace).unwrap()).unwrap();
        assert_eq!(report["value"]["value"], "0", "{report}");
        assert_eq!(
            fs::read(&saved).unwrap(),
            before,
            "resume mutated its input checkpoint"
        );
    }
    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("replayed interpreter report"), "{stdout}");
}
