use super::*;

#[test]
fn native_tool_outer_effect_handler_completes_both_openai_model_rounds() {
    let _guard = lock_model_cli_e2e();
    let (url, server) = typed_output::server(vec![
        serde_json::json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{
            "id":"lookup-1","type":"function","function":{"name":"lookup","arguments":"{}"}
        }]}}],"usage":{"prompt_tokens":1,"completion_tokens":1}}),
        serde_json::json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"finished"}}],"usage":{"prompt_tokens":2,"completion_tokens":1}}),
    ]);
    let root = package_fixture(
        "native-tool-handler",
        &[
            (
                "etas.toml",
                &format!(
                    r#"
[package]
name = "native-tool-handler"
version = "0.1.0"
[source]
root = "src"
[dependencies]
std = {{ version = "0.2" }}
[[bin]]
name = "app"
module = "app.main"
flow = "main"
[runtime]
default_profile = "mock"
[runtime.execution]
max_steps = 2000
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
effect Evidence { action read() -> string; }
tool lookup() -> string ![Evidence.read] { return perform Evidence.read(); }
@tools([lookup])
agent Reader(question: string) -> string ![Evidence.read] {
    return perform infer(Prompt.new().user(Public(question)));
}
public flow main(args: Array<string>) -> i32 ![Agentic, Error<IOError>] {
    let evidence: ![Evidence => [] for string] = handler {
        Evidence.read() => { resume "local evidence"; }
    };
    let answer = Reader.run("read evidence") with evidence;
    println(answer);
    return 0;
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
    let (lock_code, lock_out, lock_err) =
        run_process_with_command(command(), ["pkg", "lock", path(&root)], "");
    if lock_code != 0 {
        shutdown_mock_server(&url);
        let _ = server.join();
        panic!("{lock_out}\n{lock_err}");
    }
    let trace = root.join("run-report.json");
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        ["run", path(&root), "--trace-out", path(&trace)],
        "",
    );
    shutdown_mock_server(&url);
    let requests = server.join().unwrap();
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "finished\n");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    assert_eq!(
        report["value"],
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"})
    );
    assert_eq!(report["outcome"]["kind"], "completed");
    assert_eq!(requests.len(), 2);
    let messages = requests[1]["messages"].as_array().unwrap();
    let results = messages
        .iter()
        .filter(|message| message["role"] == "tool")
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["tool_call_id"], "lookup-1");
    let tool_value: serde_json::Value =
        serde_json::from_str(results[0]["content"].as_str().unwrap()).unwrap();
    assert_eq!(tool_value, serde_json::json!("local evidence"));
}
