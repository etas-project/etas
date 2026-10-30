use super::*;

#[test]
fn deep_checkpoint_run_resume_and_trace_are_stack_safe_in_every_output_format() {
    let file = fixture(
        &format!("deep-runtime-report-{}", std::process::id()),
        r#"
module tests.cli.deep_runtime_report;
import std.runtime.checkpoint;
enum Link { End, Next(Link) }
flow main(args: Array<string>) -> i32 {
    var chain = Link.End;
    var count: i32 = 0;
    while count < 4000 limit Iterations(5000) {
        chain = Link.Next(chain);
        count = count + 1;
    }
    checkpoint("deep-result");
    checkpoint("after-resume");
    let first = Message.new(chain);
    let second = Message.new(chain);
    return match chain { Link.Next(_) => 0, Link.End => 1 };
}
"#,
    );
    for format in ["json", "jsonl", "human"] {
        let trace = file.with_extension(format!("{format}.trace.json"));
        let checkpoints = file.with_extension(format!("{format}.checkpoints"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command.env("ETAS_HOST_MEMORY", "memory");
        let (code, stdout, stderr) = run_process_with_command(
            command,
            [
                "--format",
                format,
                "run",
                path(&file),
                "--allow-effects",
                "--trace-out",
                path(&trace),
                "--checkpoint-dir",
                path(&checkpoints),
            ],
            "",
        );
        assert_eq!(code, 0, "{format}: {stderr}\n{stdout}");
        assert!(stderr.is_empty(), "{format}: {stderr}");
        if format == "json" {
            assert!(
                stdout.matches("\"kind\":\"variant\"").count() >= 8002,
                "{format}"
            );
        }
        assert!(stdout.contains("\"value\":\"0\""), "{format}");
        let text = fs::read_to_string(&trace).unwrap();
        assert!(text.contains("etas.cli.interpreter-report.v2"));
        assert!(text.contains("\"command\":\"run\""));
        assert!(
            text.matches("\"kind\":\"variant\"").count() >= 8002,
            "{format}"
        );
        assert_eq!(text.matches("\"kind\":\"checkpoint_summary\"").count(), 2);
        assert_eq!(text.matches("\"resumable\":false").count(), 2);
        assert!(!text.contains("\"machine\":"));

        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command.env("ETAS_HOST_MEMORY", "memory");
        let (code, stdout, stderr) = run_process_with_command(
            command,
            [
                "--format",
                format,
                "resume",
                "0",
                "--checkpoint-dir",
                path(&checkpoints),
                "--trace-out",
                path(&trace),
            ],
            "",
        );
        assert_eq!(code, 0, "resume {format}: {stderr}\n{stdout}");
        assert!(stderr.is_empty(), "resume {format}: {stderr}");
        if format == "json" {
            assert!(
                stdout.matches("\"kind\":\"variant\"").count() >= 4001,
                "{format}"
            );
        }
        assert!(stdout.contains("\"value\":\"0\""), "{format}");
        let text = fs::read_to_string(&trace).unwrap();
        assert!(text.contains("\"command\":\"resume\""));
        assert!(
            text.matches("\"kind\":\"variant\"").count() >= 4001,
            "{format}"
        );
        fs::remove_file(&trace).unwrap();
        fs::remove_dir_all(&checkpoints).unwrap();
    }
}

#[test]
fn secret_message_and_checkpoint_state_do_not_enter_cli_reports_or_replay() {
    let source = fixture(
        "secret-report",
        r#"
module tests.cli.secret_report;
import std.runtime.checkpoint;
flow consume(value: Secret<string>) -> i32 { return 0; }
flow main(args: Array<string>) -> i32 {
    let value = Secret("AUDIT_ONLY_CLI_REPORT_SECRET");
    let message = Message.new(value);
    checkpoint("secret-live");
    return consume(value);
}
"#,
    );
    let home = etas_host::TestWorkspace::create("secret-report-home").unwrap();
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command
            .env_clear()
            .env("HOME", home.path())
            .env("PATH", "/usr/bin:/bin");
        command
    };
    let trace = temp_path("secret-report", "json");
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "--format",
            "json",
            "run",
            path(&source),
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("AUDIT_ONLY_CLI_REPORT_SECRET"));
    assert!(!stderr.contains("AUDIT_ONLY_CLI_REPORT_SECRET"));
    let bytes = fs::read(&trace).unwrap();
    let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(report["schema"], "etas.cli.interpreter-report.v2");
    assert!(
        !String::from_utf8(bytes)
            .unwrap()
            .contains("AUDIT_ONLY_CLI_REPORT_SECRET")
    );
    assert!(report["events"].as_array().unwrap().iter().any(|event| {
        event["kind"] == "message_created" && event["payload"]["kind"] == "redacted"
    }));
    assert_eq!(report["checkpoints"][0]["kind"], "checkpoint_summary");
    assert_eq!(report["checkpoints"][0]["resumable"], false);
    assert!(report["checkpoints"][0].get("machine").is_none());
    let (code, stdout, stderr) = run_process_with_command(command(), ["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("AUDIT_ONLY_CLI_REPORT_SECRET"));
    assert!(!stderr.contains("AUDIT_ONLY_CLI_REPORT_SECRET"));
}

#[test]
fn replay_rejects_legacy_reports_without_printing_their_payloads() {
    let trace = temp_path("legacy-secret-report", "json");
    fs::write(
        &trace,
        r#"{"schema":"etas.cli.interpreter-report.v1","value":"AUDIT_ONLY_LEGACY_REPORT_SECRET"}"#,
    )
    .unwrap();
    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 2, "{stdout}\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(!stderr.contains("AUDIT_ONLY_LEGACY_REPORT_SECRET"));
}
