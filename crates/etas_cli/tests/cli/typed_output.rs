use super::*;
use serde_json::{Value, json};

#[path = "typed_output/credentials.rs"]
mod credentials;

fn project(base_url: &str, encoding: Option<&str>) -> PathBuf {
    let encoding = encoding.map_or(String::new(), |value| {
        format!("typed_output_encoding = {value:?}")
    });
    let root = package_fixture(
        "typed-output",
        &[
            (
                "etas.toml",
                &format!(
                    r#"
[package]
name = "typed-output"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = {{ version = "0.1" }}
[[bin]]
name = "typed-output"
module = "app.main"
flow = "main"
[runtime]
default_profile = "mock"
[runtime.profiles.mock]
boundary_policy = "typed-output-test"
[runtime.profiles.mock.model]
adapter = "omlx-openai"
model = "mock-model"
base_url = {base_url:?}
{encoding}
[runtime.profiles.mock.policy]
mode = "local-static"
rules = ["model=allow", "console=allow"]
"#
                ),
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.collections.Array;
import std.io.{println};
type Reply = { text: string };
agent Writer() -> Reply {
    return perform infer<Reply>(Prompt.new().user(Public("Return the requested record.")));
}
flow main(args: Array<string>) -> i32 { println(Writer().text); return 0; }
"#,
            ),
        ],
    );
    let (code, stdout, stderr) =
        run_process_with_command(command(), ["pkg", "lock", path(&root)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    root
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    command
}

fn server(responses: Vec<Value>) -> (String, thread::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base_url = mock_server_base_url(listener.local_addr().unwrap(), "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut requests = Vec::new();
        for response in responses {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if request.is_empty() {
                            continue;
                        }
                        if let Some(shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if shutdown {
                                return requests;
                            }
                            continue;
                        }
                        let (_, body) = request.split_once("\r\n\r\n").expect("HTTP request body");
                        requests.push(serde_json::from_str(body).unwrap());
                        write_http_json_response(&mut stream, &response.to_string());
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "model request timed out");
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("mock model accept: {error}"),
                }
            }
        }
        requests
    });
    (base_url, handle)
}

fn response(encoding: &str, payload: Value) -> Value {
    let message = if encoding == "output-tool" {
        json!({"role":"assistant", "tool_calls":[{
            "id":"typed-output", "type":"function",
            "function":{"name":"etas_emit_output", "arguments":payload.to_string()}
        }]})
    } else {
        json!({"role":"assistant", "content":payload.to_string()})
    };
    json!({"choices":[{"message":message}], "usage":{"prompt_tokens":1,"completion_tokens":1}})
}

fn assert_encoding(request: &Value, encoding: &str) {
    if encoding == "json-schema" {
        assert_eq!(
            request["response_format"]["type"], "json_schema",
            "{request}"
        );
        assert_eq!(request["response_format"]["json_schema"]["strict"], true);
        assert_eq!(
            request["response_format"]["json_schema"]["schema"]["properties"]["text"]["type"],
            "string"
        );
        assert!(
            request.get("tools").is_none(),
            "no application or private output tool: {request}"
        );
        assert!(request.get("tool_choice").is_none(), "{request}");
    } else {
        assert!(request.get("response_format").is_none(), "{request}");
        let tools = request["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1, "no application tools: {request}");
        assert_eq!(tools[0]["function"]["name"], "etas_emit_output");
        assert_eq!(
            tools[0]["function"]["parameters"]["properties"]["text"]["type"],
            "string"
        );
        assert_eq!(
            request["tool_choice"]["function"]["name"],
            "etas_emit_output"
        );
    }
}

#[test]
fn typed_output_default_uses_only_native_json_schema() {
    let _guard = lock_model_cli_e2e();
    let (url, handle) = server(vec![response(
        "json-schema",
        json!({"text":"native schema"}),
    )]);
    let project = project(&url, None);
    let (code, stdout, stderr) =
        run_process_with_command(command(), ["run", path(&project), "--allow-effects"], "");
    shutdown_mock_server(&url);
    let requests = handle.join().unwrap();
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("native schema"), "{stdout}");
    assert_eq!(requests.len(), 1);
    assert_encoding(&requests[0], "json-schema");
}

#[test]
fn typed_output_modes_keep_native_schema_and_bounded_typed_repair() {
    let _guard = lock_model_cli_e2e();
    for encoding in ["json-schema", "output-tool"] {
        for repaired in [true, false] {
            let payload = if repaired {
                json!({"text":"repaired record"})
            } else {
                json!({"text":42})
            };
            let attempts = if repaired {
                2
            } else {
                1 + etas_interpreter::api::ModelExecutionPolicy::default().max_tool_rounds
            };
            let responses = (0..attempts)
                .map(|index| {
                    response(
                        encoding,
                        if index + 1 == attempts {
                            payload.clone()
                        } else {
                            json!({"text":42})
                        },
                    )
                })
                .collect();
            let (url, handle) = server(responses);
            let project = project(&url, Some(encoding));
            let (code, stdout, stderr) =
                run_process_with_command(command(), ["run", path(&project), "--allow-effects"], "");
            shutdown_mock_server(&url);
            let requests = handle.join().unwrap();
            assert_eq!(
                code,
                if repaired { 0 } else { 3 },
                "{encoding}: {stdout}\n{stderr}"
            );
            if repaired {
                assert!(stdout.contains("repaired record"), "{stdout}");
            }
            assert_eq!(
                requests.len(),
                attempts,
                "typed repair must respect the configured round limit: {stderr}"
            );
            for request in &requests {
                assert_encoding(request, encoding);
            }
        }
    }
}

#[test]
fn typed_output_profile_local_env_cli_precedence_and_redacted_effective_policy() {
    let _guard = lock_model_cli_e2e();
    let project = project("http://127.0.0.1:8848/v1", Some("output-tool"));
    fs::write(
        project.join("etas.local.toml"),
        "[runtime.profiles.mock.model]\ntyped_output_encoding = 'json-schema'\n",
    )
    .unwrap();
    let print = |mut cmd: Command, args| {
        cmd.env("ETAS_HOST_MODEL_API_KEY", "do-not-print-this-secret");
        let (code, stdout, stderr) = run_process_with_command(cmd, args, "");
        assert_eq!(code, 0, "{stderr}");
        assert!(!stdout.contains("do-not-print-this-secret"));
        serde_json::from_str::<Value>(&stdout).unwrap()
    };
    let local = print(
        command(),
        ["run", path(&project), "--print-runtime-profile"],
    );
    assert_eq!(local["model"]["typed_output_encoding"], "json-schema");
    let mut cmd = command();
    cmd.env("ETAS_HOST_MODEL_OUTPUT_ENCODING", "output-tool");
    let env = print(cmd, ["run", path(&project), "--print-runtime-profile"]);
    assert_eq!(env["model"]["typed_output_encoding"], "output-tool");
    assert_ne!(local["profile_fingerprint"], env["profile_fingerprint"]);
    let mut cmd = command();
    cmd.env("ETAS_HOST_MODEL_OUTPUT_ENCODING", "invalid");
    let (code, stdout, stderr) = run_process_with_command(
        cmd,
        [
            "run",
            path(&project),
            "--print-runtime-profile",
            "--model-output-encoding",
            "output-tool",
        ],
        "",
    );
    assert_eq!(
        code, 0,
        "CLI overrides invalid lower-priority env: {stderr}"
    );
    let profile: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(profile["model"]["typed_output_encoding"], "output-tool");
    assert_eq!(profile["model"]["capabilities"]["forced_tool_output"], true);
    assert_eq!(
        profile["model"]["capabilities"]["json_schema_response_format"],
        false
    );
}

#[test]
fn typed_output_invalid_or_inapplicable_configuration_is_rejected() {
    let _guard = lock_model_cli_e2e();
    let project = project("http://127.0.0.1:8848/v1", Some("both"));
    let (code, _, stderr) = run_process_with_command(
        command(),
        ["run", path(&project), "--print-runtime-profile"],
        "",
    );
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("typed output encoding"), "{stderr}");
    let manifest = fs::read_to_string(project.join("etas.toml"))
        .unwrap()
        .replace("omlx-openai", "omlx-anthropic")
        .replace("\"both\"", "\"json-schema\"");
    fs::write(project.join("etas.toml"), manifest).unwrap();
    let (code, _, stderr) = run_process_with_command(
        command(),
        ["run", path(&project), "--print-runtime-profile"],
        "",
    );
    assert_eq!(code, 2, "{stderr}");
    assert!(
        stderr.contains("only supported by OpenAI-compatible"),
        "{stderr}"
    );
    let source = package_fixture(
        "no-model-output",
        &[("main.es", "module main; flow main() -> unit { return; }")],
    );
    let (code, _, stderr) = run_process_with_command(
        command(),
        [
            "run",
            path(&source.join("main.es")),
            "--model-output-encoding",
            "output-tool",
        ],
        "",
    );
    assert_eq!(code, 2, "{stderr}");
    assert!(
        stderr.contains("requires a configured model adapter"),
        "{stderr}"
    );
}

#[test]
fn typed_output_cli_override_is_preserved_when_resuming_checkpoint() {
    let _guard = lock_model_cli_e2e();
    let (url, handle) = server(vec![
        response("output-tool", json!({"text":"initial result"})),
        response("output-tool", json!({"text":"resumed result"})),
    ]);
    let project = project(&url, Some("json-schema"));
    let source_path = project.join("src/app/main.es");
    let source = fs::read_to_string(&source_path)
        .unwrap()
        .replace(
            "module app.main;",
            "module app.main;\nimport std.runtime.checkpoint;",
        )
        .replace(
            "flow main(args: Array<string>) -> i32 {",
            "flow main(args: Array<string>) -> i32 { checkpoint(\"before-model\");",
        );
    fs::write(source_path, source).unwrap();
    let checkpoints = project.join("checkpoints");
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "run",
            path(&project),
            "--allow-effects",
            "--model-output-encoding",
            "output-tool",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("initial result"), "{stdout}");
    assert!(checkpoints.join("checkpoint-0.json").exists());
    let (code, _, stderr) = run_process_with_command(
        command(),
        ["resume", "0", "--checkpoint-dir", path(&checkpoints)],
        "",
    );
    assert_eq!(
        code, 2,
        "changing the checkpoint encoding must be rejected: {stderr}"
    );
    assert!(
        stderr.contains("not compatible with the checkpoint"),
        "{stderr}"
    );
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "resume",
            "0",
            "--checkpoint-dir",
            path(&checkpoints),
            "--model-output-encoding",
            "output-tool",
        ],
        "",
    );
    shutdown_mock_server(&url);
    let requests = handle.join().unwrap();
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("resumed result"), "{stdout}");
    assert_eq!(requests.len(), 2);
    for request in &requests {
        assert_encoding(request, "output-tool");
    }
}
