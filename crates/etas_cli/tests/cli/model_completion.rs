use super::*;

fn project(url: &str) -> PathBuf {
    package_fixture(
        "model-completion",
        &[
            (
                "etas.toml",
                &format!(
                    r#"
[package]
name = "model-completion"
version = "0.1.0"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
[runtime]
default_profile = "mock"
[runtime.profiles.mock.model]
adapter = "omlx-openai"
model = "mock"
base_url = {url:?}
[runtime.profiles.mock.policy]
mode = "local-static"
rules = ["model=allow", "tool=allow", "console=allow"]
"#
                ),
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.io.println;
tool echo(value: string) -> string ![] { return value; }
@tools([echo])
agent Planner() -> string ![] { return perform infer(Prompt.new()); }
agent Child(input: string) -> string ![] { return perform infer(Prompt.new().user(Public(input))); }
public flow main(args: Array<string>) -> i32 ![Agentic, Error<IOError>] {
    let plan = Planner.run();
    println("NEXT STAGE");
    println(Child.run(plan));
    return 0;
}
public flow single(args: Array<string>) -> i32 ![Agentic, Error<IOError>] {
    println(Planner.run());
    return 0;
}
"#,
            ),
        ],
    )
}

struct ReplyRun {
    code: i32,
    stdout: String,
    stderr: String,
    requests: Vec<serde_json::Value>,
    report: serde_json::Value,
}

fn run_reply(reply: serde_json::Value, flow: &str) -> ReplyRun {
    let (url, server) = typed_output::server(vec![
        reply,
        serde_json::json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"child ran"}}],"usage":{"prompt_tokens":1,"completion_tokens":1}}),
    ]);
    let root = project(&url);
    let trace = root.join("trace.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&root),
            "--flow",
            flow,
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    shutdown_mock_server(&url);
    let requests = server.join().unwrap();
    let report = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    ReplyRun {
        code,
        stdout,
        stderr,
        requests,
        report,
    }
}

#[test]
fn openai_null_optional_tool_fields_return_text_with_native_tools_enabled() {
    let _guard = lock_model_cli_e2e();
    let ReplyRun {
        code,
        stdout,
        stderr,
        requests,
        ..
    } = run_reply(
        serde_json::json!({
            "choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"finished","tool_calls":null,"function_call":null}}],
            "usage":{"prompt_tokens":10,"completion_tokens":4},
        }),
        "single",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert_eq!(stdout, "finished\n");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(requests.len(), 1);
}

#[test]
fn openai_incomplete_completion_never_executes_the_next_stage_or_repairs() {
    let _guard = lock_model_cli_e2e();
    for message in [
        serde_json::json!({"role":"assistant","content":"[\"delegate\",\"Read context.txt\"]","reasoning_content":"SYNTHETIC_PRIVATE_REASONING"}),
        serde_json::json!({"role":"assistant","content":""}),
        serde_json::json!({"role":"assistant","content":null,"tool_calls":[{"id":"partial","type":"function","function":{"name":"echo","arguments":"{\"value\":"}}]}),
    ] {
        let ReplyRun {
            code,
            stdout,
            stderr,
            requests,
            report,
        } = run_reply(
            serde_json::json!({
                "choices":[{"finish_reason":"length","message":message}],
                "usage":{"prompt_tokens":10,"completion_tokens":4,"total_tokens":14},
            }),
            "main",
        );
        assert_eq!(code, 3, "{stdout}\n{stderr}");
        assert!(stderr.contains("IncompleteResponse"), "{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert_eq!(requests.len(), 1, "incomplete response advanced/retried");
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains("SYNTHETIC_PRIVATE_REASONING")
        );
    }
}

#[test]
fn openai_missing_unknown_and_filtered_statuses_do_not_run_followup() {
    let _guard = lock_model_cli_e2e();
    for (reason, code_name) in [
        (None, "InvalidResponse"),
        (Some(serde_json::Value::Null), "InvalidResponse"),
        (
            Some(serde_json::json!("PRIVATE_UNKNOWN_REASON")),
            "InvalidResponse",
        ),
        (
            Some(serde_json::json!("content_filter")),
            "ProviderRejected",
        ),
    ] {
        let mut choice = serde_json::json!({"message":{"role":"assistant","content":"valid text"}});
        if let Some(reason) = reason {
            choice["finish_reason"] = reason;
        }
        let ReplyRun {
            code,
            stdout,
            stderr,
            requests,
            report,
        } = run_reply(
            serde_json::json!({
                "choices":[choice],"usage":{"prompt_tokens":10,"completion_tokens":4}
            }),
            "main",
        );
        assert_eq!(code, 3, "{stdout}\n{stderr}");
        assert!(stderr.contains(code_name), "{stderr}");
        assert!(stdout.is_empty());
        assert_eq!(requests.len(), 1);
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains("PRIVATE_UNKNOWN_REASON")
        );
    }
}
