use super::*;

#[test]
fn bounded_agent_step_controller_runs_native_tools_and_preserves_history() {
    let _guard = lock_model_cli_e2e();
    for (package_tool, package_agent) in [(false, false), (true, false), (true, true)] {
        for default_controller in [false, true] {
            run_controller(package_tool, package_agent, default_controller);
        }
    }
}

fn run_controller(package_tool: bool, package_agent: bool, default_controller: bool) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../etas_tests/fixtures/compiler/projects/positive/agent_step_controller");
    let project = copy_project_fixture("agent-step-controller", &fixture);
    if package_tool {
        let library = project.join("tools");
        fs::create_dir_all(library.join("src/library")).unwrap();
        fs::write(library.join("etas.toml"), "[package]\nname = \"controller-tools\"\nversion = \"0.1.0\"\n[source]\nroot = \"src\"\n").unwrap();
        fs::write(library.join("src/library/tools.es"), "module library.tools;\nimport std.agent.{ToolCall, ToolResult};\npublic tool uppercase(text: string) -> string ![] { return std.text.uppercase(text); }\npublic flow invoke_native<C>(call: ToolCall<C>) -> ToolResult<C> { return std.tool.invoke<C>(call); }\n").unwrap();
        let manifest = project.join("etas.toml");
        fs::write(
            &manifest,
            format!(
                "{}\n[dependencies]\nnative = {{ path = \"tools\", import = \"native\" }}\n",
                fs::read_to_string(&manifest).unwrap()
            ),
        )
        .unwrap();
        let source = project.join("src/app/main.es");
        fs::write(&source, fs::read_to_string(&source).unwrap()
            .replace("tool uppercase(text: string) -> string ![] {\n    return std.text.uppercase(text);\n}", "import native.library.tools.{uppercase, invoke_native};")
            .replace("flow invoke_native<C>(call: ToolCall<C>) -> ToolResult<C> {\n    return tool.invoke<C>(call);\n}\n", "")
        ).unwrap();
        if package_agent {
            let text = fs::read_to_string(&source).unwrap();
            let start = text.find("@model(\"configured-model\")").unwrap();
            let end = text.find("\nflow RunControlled").unwrap();
            let agent = &text[start..end];
            let library_source = library.join("src/library/tools.es");
            fs::write(
                &library_source,
                format!(
                    "{}\n{}",
                    fs::read_to_string(&library_source)
                        .unwrap()
                        .replace("public tool uppercase", "tool uppercase"),
                    agent.replace("agent Writer(", "public agent Writer(")
                ),
            )
            .unwrap();
            fs::write(
                &source,
                text.replace(agent, "import native.library.tools.Writer;\n")
                    .replace("{uppercase, invoke_native}", "{invoke_native}"),
            )
            .unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        let (code, stdout, stderr) =
            run_process_with_command(command, ["pkg", "lock", path(&project)], "");
        assert_eq!(code, 0, "{stdout}\n{stderr}");
    }
    if default_controller {
        let source = project.join("src/app/main.es");
        let text = fs::read_to_string(&source).unwrap();
        assert!(text.contains("println(RunControlled(\"hi\"))"));
        fs::write(
            source,
            text.replace(
                "println(RunControlled(\"hi\"))",
                "println(Writer.run(\"hi\"))",
            ),
        )
        .unwrap();
    }
    let (url, server) = typed_output::server(vec![
        serde_json::json!({
            "id":"turn-1", "choices":[{"finish_reason":"tool_calls","message":{
                "role":"assistant", "content":null, "reasoning_content":"preserve this turn",
                "tool_calls":[{"id":"call-1","type":"function","function":{
                    "name":"uppercase", "arguments":"{\"text\":\"hi\"}"
                }}]
            }}], "usage":{"prompt_tokens":2,"completion_tokens":1}
        }),
        serde_json::json!({
            "id":"turn-2", "choices":[{"finish_reason":"stop","message":{
                "role":"assistant", "content":"HI"
            }}], "usage":{"prompt_tokens":4,"completion_tokens":1}
        }),
    ]);
    fs::write(
        project.join("etas.local.toml"),
        format!("[runtime.profiles.local.model]\nbase_url = {url:?}\n"),
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(command, ["run", path(&project)], "");
    shutdown_mock_server(&url);
    let requests = server.join().unwrap();
    assert_eq!(code, 0, "{stdout}\n{stderr}\n{requests:#?}");
    assert_eq!(stdout, "HI\n");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(requests.len(), 2, "step performed a hidden generation");
    assert_eq!(requests[0]["tools"][0]["function"]["name"], "uppercase");
    let messages = requests[1]["messages"].as_array().unwrap();
    let assistant = messages
        .iter()
        .find(|message| message["role"] == "assistant")
        .unwrap();
    assert_eq!(assistant["reasoning_content"], "preserve this turn");
    assert_eq!(assistant["tool_calls"][0]["id"], "call-1");
    let feedback = messages
        .iter()
        .find(|message| message["role"] == "tool")
        .unwrap();
    assert_eq!(feedback["tool_call_id"], "call-1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(feedback["content"].as_str().unwrap()).unwrap(),
        serde_json::json!("HI")
    );
}
