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
        assert!(text.contains("etas.cli.interpreter-report.v1"));
        assert!(text.contains("\"command\":\"run\""));
        assert!(
            text.matches("\"kind\":\"variant\"").count() >= 8002,
            "{format}"
        );

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
