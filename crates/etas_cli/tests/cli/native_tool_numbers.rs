use super::*;

enum Expected {
    Value(f64),
    SchemaRepair,
    TypeError,
}

#[test]
fn native_tool_json_numbers_follow_checked_parameter_types() {
    let _guard = lock_model_cli_e2e();
    for (ty, literal, expected) in [
        ("f64", "90", Expected::Value(90.0)),
        ("f64", "90.0", Expected::Value(90.0)),
        ("f64", "90.5", Expected::Value(90.5)),
        ("f64", "-0.0", Expected::Value(-0.0)),
        ("f64", "1e300", Expected::Value(1e300)),
        ("f32", "90", Expected::Value(90.0)),
        ("f32", "90.0", Expected::Value(90.0)),
        ("i32", "90.5", Expected::SchemaRepair),
        ("u8", "256", Expected::TypeError),
        ("f32", "1e300", Expected::TypeError),
    ] {
        let arguments = format!(r#"{{"value":{literal}}}"#);
        let (url, server) = typed_output::server(vec![
            serde_json::json!({"choices":[{"message":{"role":"assistant","content":null,"tool_calls":[{
                "id":"echo-1","type":"function","function":{"name":"echo","arguments":arguments}
            }]}}],"usage":{"prompt_tokens":1,"completion_tokens":1}}),
            serde_json::json!({"choices":[{"message":{"role":"assistant","content":"finished"}}],"usage":{"prompt_tokens":2,"completion_tokens":1}}),
        ]);
        let root = package_fixture(
            "native-tool-json-number",
            &[
                (
                    "etas.toml",
                    &format!(
                        r#"
[package]
name = "native-tool-json-number"
version = "0.1.0"
[source]
root = "src"
[dependencies]
std = {{ version = "0.1" }}
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
                    &format!(
                        r#"
module app.main;
import std.io.println;
tool echo(value: {ty}) -> {ty} ![] {{ return value; }}
@tools([echo])
agent Reader() -> string ![] {{ return perform infer(Prompt.new()); }}
public flow main(args: Array<string>) -> i32 ![Agentic, Error<IOError>] {{
    println(Reader.run());
    return 0;
}}
"#
                    ),
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
        let (code, stdout, stderr) = run_process_with_command(command(), ["run", path(&root)], "");
        shutdown_mock_server(&url);
        let requests = server.join().unwrap();
        if !matches!(expected, Expected::TypeError) {
            assert_eq!(code, 0, "{ty}/{literal}: {stdout}\n{stderr}");
            assert!(stderr.is_empty(), "{stderr}");
            assert_eq!(stdout, "finished\n");
            assert_eq!(requests.len(), 2);
            let messages = requests[1]["messages"].as_array().unwrap();
            let results: Vec<_> = messages
                .iter()
                .filter(|message| message["role"] == "tool")
                .collect();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0]["tool_call_id"], "echo-1");
            let value: serde_json::Value =
                serde_json::from_str(results[0]["content"].as_str().unwrap()).unwrap();
            match expected {
                Expected::Value(expected) => assert_eq!(
                    value.as_f64().unwrap().to_bits(),
                    expected.to_bits(),
                    "{ty}/{literal}"
                ),
                Expected::SchemaRepair => assert_eq!(value["kind"], "InvalidToolArguments"),
                Expected::TypeError => unreachable!(),
            }
        } else {
            assert_eq!(code, 3, "{ty}/{literal}: {stdout}\n{stderr}");
            assert!(
                stderr.contains("did not match the source tool parameter type"),
                "{stderr}"
            );
            assert!(stdout.is_empty(), "{stdout}");
            assert_eq!(requests.len(), 1);
        }
    }
}
