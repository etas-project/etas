use super::*;

const PROFILE_KEY_ENV: &str = "ETAS_TEST_MODEL_PROFILE_CREDENTIAL";
const PROFILE_KEY: &str = "profile-test-key-not-for-output";

struct Request {
    path: String,
    authorization: Option<String>,
    body: Value,
}

fn authenticated_project(url: &str) -> PathBuf {
    let root = project(url, Some("json-schema"));
    let manifest_path = root.join("etas.toml");
    let mut manifest: toml::Value =
        toml::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["runtime"]["profiles"]["mock"]["model"]
        .as_table_mut()
        .unwrap()
        .insert(
            "api_key_env".into(),
            toml::Value::String(PROFILE_KEY_ENV.into()),
        );
    fs::write(manifest_path, toml::to_string(&manifest).unwrap()).unwrap();
    root
}

fn authenticated_command(key: &str) -> Command {
    let mut cmd = command();
    cmd.env_remove("ETAS_HOST_MODEL_API_KEY");
    cmd.env_remove("ETAS_HOST_OMLX_API_KEY");
    cmd.env(PROFILE_KEY_ENV, key);
    cmd
}

fn capturing_server(responses: Vec<Value>) -> (String, thread::JoinHandle<Vec<Request>>) {
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
                        let (head, body) = request.split_once("\r\n\r\n").unwrap();
                        let path = head.lines().next().unwrap().to_owned();
                        let authorization = head.lines().skip(1).find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("authorization")
                                .then(|| value.trim().to_owned())
                        });
                        requests.push(Request {
                            path,
                            authorization,
                            body: serde_json::from_str(body).unwrap(),
                        });
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

fn assert_request(request: &Request, key: &str, encoding: &str) {
    assert!(request.path.starts_with("POST "));
    assert!(request.path.ends_with("/v1/chat/completions HTTP/1.1"));
    assert_eq!(
        request.authorization.as_deref(),
        Some(format!("Bearer {key}").as_str())
    );
    assert_eq!(request.body["model"], "mock-model");
    assert_encoding(&request.body, encoding);
}

#[test]
fn model_profile_credentials_survive_cli_and_env_output_encoding_overrides() {
    let _guard = lock_model_cli_e2e();
    for (cli, env, encoding) in [
        (None, None, "json-schema"),
        (Some("json-schema"), None, "json-schema"),
        (Some("output-tool"), None, "output-tool"),
        (None, Some("json-schema"), "json-schema"),
        (None, Some("output-tool"), "output-tool"),
        (Some("json-schema"), Some("output-tool"), "json-schema"),
    ] {
        let (url, handle) =
            capturing_server(vec![response(encoding, json!({"text":"authenticated"}))]);
        let root = authenticated_project(&url);
        let mut cmd = authenticated_command(PROFILE_KEY);
        if let Some(encoding) = env {
            cmd.env("ETAS_HOST_MODEL_OUTPUT_ENCODING", encoding);
        }
        let mut args = vec!["run", path(&root), "--allow-effects"];
        if let Some(encoding) = cli {
            args.extend(["--model-output-encoding", encoding]);
        }
        cmd.args(args);
        let (code, stdout, stderr) = run_process_with_command(cmd, [], "");
        shutdown_mock_server(&url);
        let requests = handle.join().unwrap();
        assert_eq!(code, 0, "{stdout}\n{stderr}");
        assert!(stdout.contains("authenticated"));
        assert!(!stdout.contains(PROFILE_KEY));
        assert!(!stderr.contains(PROFILE_KEY));
        assert_eq!(requests.len(), 1);
        assert_request(&requests[0], PROFILE_KEY, encoding);

        let mut cmd = authenticated_command(PROFILE_KEY);
        if let Some(encoding) = env {
            cmd.env("ETAS_HOST_MODEL_OUTPUT_ENCODING", encoding);
        }
        let mut args = vec!["run", path(&root), "--print-runtime-profile"];
        if let Some(encoding) = cli {
            args.extend(["--model-output-encoding", encoding]);
        }
        cmd.args(args);
        let (code, stdout, stderr) = run_process_with_command(cmd, [], "");
        assert_eq!(code, 0, "{stderr}");
        assert!(!stdout.contains(PROFILE_KEY));
        assert!(!stderr.contains(PROFILE_KEY));
        let profile: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(profile["model"]["typed_output_encoding"], encoding);
    }
}

#[test]
fn model_profile_credentials_keep_explicit_generic_key_precedence() {
    let _guard = lock_model_cli_e2e();
    for (model_key, omlx_key, expected) in [
        (None, None, PROFILE_KEY),
        (Some("explicit-model-key"), None, "explicit-model-key"),
        (None, Some("explicit-omlx-key"), "explicit-omlx-key"),
        (
            Some("explicit-model-key"),
            Some("explicit-omlx-key"),
            "explicit-model-key",
        ),
    ] {
        let (url, handle) = capturing_server(vec![response(
            "json-schema",
            json!({"text":"authenticated"}),
        )]);
        let root = authenticated_project(&url);
        let mut cmd = authenticated_command(PROFILE_KEY);
        if let Some(key) = model_key {
            cmd.env("ETAS_HOST_MODEL_API_KEY", key);
        }
        if let Some(key) = omlx_key {
            cmd.env("ETAS_HOST_OMLX_API_KEY", key);
        }
        let (code, stdout, stderr) = run_process_with_command(
            cmd,
            [
                "run",
                path(&root),
                "--allow-effects",
                "--model-output-encoding",
                "json-schema",
            ],
            "",
        );
        shutdown_mock_server(&url);
        let requests = handle.join().unwrap();
        assert_eq!(code, 0, "{stdout}\n{stderr}");
        assert_eq!(requests.len(), 1);
        assert_request(&requests[0], expected, "json-schema");
        for key in [PROFILE_KEY, "explicit-model-key", "explicit-omlx-key"] {
            assert!(!stdout.contains(key));
            assert!(!stderr.contains(key));
        }
    }
}

#[test]
fn model_profile_credentials_are_reassembled_on_resume_and_never_saved() {
    let _guard = lock_model_cli_e2e();
    let rotated = "rotated-profile-key-not-for-checkpoint";
    let (url, handle) = capturing_server(vec![
        response("json-schema", json!({"text":"initial response"})),
        response("json-schema", json!({"text":"resumed response"})),
    ]);
    let root = authenticated_project(&url);
    let source_path = root.join("src/app/main.es");
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
    let checkpoints = root.join("checkpoints");
    let (code, stdout, stderr) = run_process_with_command(
        authenticated_command(PROFILE_KEY),
        [
            "run",
            path(&root),
            "--allow-effects",
            "--model-output-encoding",
            "json-schema",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("initial response"));
    let saved = fs::read_to_string(checkpoints.join("checkpoint-0.json")).unwrap();
    for key in [PROFILE_KEY, rotated] {
        assert!(!saved.contains(key));
        assert!(!stdout.contains(key));
        assert!(!stderr.contains(key));
    }
    let (code, stdout, stderr) = run_process_with_command(
        authenticated_command(rotated),
        [
            "resume",
            "0",
            "--checkpoint-dir",
            path(&checkpoints),
            "--model-output-encoding",
            "json-schema",
        ],
        "",
    );
    shutdown_mock_server(&url);
    let requests = handle.join().unwrap();
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("resumed response"));
    assert_eq!(requests.len(), 2);
    assert_request(&requests[0], PROFILE_KEY, "json-schema");
    assert_request(&requests[1], rotated, "json-schema");
    for key in [PROFILE_KEY, rotated] {
        assert!(!stdout.contains(key));
        assert!(!stderr.contains(key));
    }
    assert_eq!(
        saved,
        fs::read_to_string(checkpoints.join("checkpoint-0.json")).unwrap()
    );
}
