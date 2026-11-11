use super::*;

#[test]
fn configured_tool_round_limit_allows_more_than_eight_native_rounds() {
    let _guard = lock_model_cli_e2e();
    let mut responses = (0..9)
        .map(|index| {
            serde_json::json!({
                "choices": [{
                    "finish_reason": "tool_calls",
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": format!("tick-{index}"),
                            "type": "function",
                            "function": {
                                "name": "tick",
                                "arguments": serde_json::json!({"value": index}).to_string()
                            }
                        }]
                    }
                }],
                "usage": {"prompt_tokens": 100, "completion_tokens": 20}
            })
        })
        .collect::<Vec<_>>();
    responses.push(serde_json::json!({
        "choices": [{
            "finish_reason": "tool_calls",
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "answer",
                    "type": "function",
                    "function": {
                        "name": "etas_emit_output",
                        "arguments": "{\"text\":\"All nine tools completed.\"}"
                    }
                }]
            }
        }],
        "usage": {"prompt_tokens": 100, "completion_tokens": 20}
    }));
    let (url, server) = typed_output::server(responses);
    let root = package_fixture(
        "configured-tool-rounds",
        &[
            (
                "etas.toml",
                &format!(
                    r#"
[package]
name = "configured-tool-rounds"
version = "0.1.0"
edition = "2026"
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
max_tool_rounds = 16
[runtime.profiles.mock.model]
adapter = "omlx-openai"
model = "mock"
base_url = {url:?}
typed_output_encoding = "output-tool"
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

type Answer = { text: string };
public tool tick(value: i32) -> i32 ![] { return value + 1; }

@tools([tick])
@limits([Tokens(131072), ContextTokens(1048576)])
agent Worker(input: string) -> Answer {
    return perform infer<Answer>(Prompt.new().system(Trusted(input)));
}

flow main(args: Array<string>) -> i32 ![Agentic, Console, Error<IOError>] {
    let answer = Worker.run("Call tick nine times, then answer.");
    println(answer.text);
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
    assert_eq!(lock_code, 0, "{lock_out}\n{lock_err}");

    let (code, stdout, stderr) = run_process_with_command(command(), ["run", path(&root)], "");
    shutdown_mock_server(&url);
    let requests = server.join().unwrap();

    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert_eq!(stdout, "All nine tools completed.\n");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(requests.len(), 10);
}

#[test]
fn builtin_model_response_uses_raw_decode_with_source_tools() {
    let _guard = lock_model_cli_e2e();
    let responses = vec![serde_json::json!({
        "choices": [{
            "finish_reason": "tool_calls",
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "ping-1",
                    "type": "function",
                    "function": {
                        "name": "ping",
                        "arguments": "{\"value\":7}"
                    }
                }]
            }
        }],
        "usage": {"prompt_tokens": 8, "completion_tokens": 4}
    })];
    let (url, server) = typed_output::server(responses);
    let root = package_fixture(
        "builtin-model-response",
        &[
            (
                "etas.toml",
                &format!(
                    r#"
[package]
name = "builtin-model-response"
version = "0.1.0"
edition = "2026"
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
[runtime.profiles.mock.model]
adapter = "omlx-openai"
model = "mock"
base_url = {url:?}
typed_output_encoding = "output-tool"
[runtime.profiles.mock.policy]
mode = "local-static"
rules = ["model=allow", "tool=allow"]
"#
                ),
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.agent.prompt.Prompt;
import std.agent.schema.ModelResponse;

public tool ping(value: i32) -> i32 ![] { return value + 1; }

@tools([ping])
agent Worker(input: string) -> ModelResponse {
    return perform infer<ModelResponse>(Prompt.new().system(Trusted(input)));
}

flow main(args: Array<string>) -> i32 ![Agentic] {
    let response = Worker.run("Call ping with value 7.");
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
    assert_eq!(lock_code, 0, "{lock_out}\n{lock_err}");

    let (code, stdout, stderr) = run_process_with_command(command(), ["run", path(&root)], "");
    shutdown_mock_server(&url);
    let requests = server.join().unwrap();

    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains("run value: {\"kind\":\"number\",\"type\":\"i32\",\"value\":\"0\"}"),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert!(request.get("response_format").is_none(), "{request}");
    let tools = request["tools"].as_array().expect("source tool schema");
    assert_eq!(tools.len(), 1, "{request}");
    assert_eq!(tools[0]["function"]["name"], "ping", "{request}");
}
