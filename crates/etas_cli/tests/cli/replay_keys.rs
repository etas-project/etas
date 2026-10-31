use super::*;

pub(super) fn is_checked_source_tool_boundary(boundary: &serde_json::Value) -> bool {
    boundary["kind"] == "tool"
        && boundary["occurrence"]["kind"] == "source_tool_call"
        && boundary["occurrence"]["model_request_id"]
            .as_u64()
            .is_some()
        && boundary["occurrence"]["call_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
        && boundary["key"]
            .as_str()
            .is_some_and(|key| key.parse::<etas_host::replay::ReplayFingerprint>().is_ok())
}

#[test]
fn checkpoint_private_keys_survive_process_restart_without_persisting_approval_reason() {
    let project = etas_host::TestWorkspace::create("cli-replay-key-project").unwrap();
    let keys = etas_host::TestWorkspace::create("cli-replay-key-store").unwrap();
    let key_dir = keys.path().join("private");
    let absent_dir = keys.path().join("absent");
    let source = project.path().join("main.es");
    let checkpoints = project.path().join("checkpoints");
    let trace = project.path().join("resumed-trace.json");
    fs::write(
        &source,
        r#"
module app.main;
import std.runtime.checkpoint;
effect Approval { action request(reason: string) -> bool; }
flow ask() -> bool { return perform Approval.request("cli-private-replay-marker-9074"); }
flow main(args: Array<string>) -> i32 { let allowed = ask(); checkpoint("saved"); if allowed { return 0; } return 1; }
"#,
    )
    .unwrap();
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        command.env("ETAS_HOST_APPROVAL", "auto");
        command
    };
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "run",
            path(&source),
            "--allow-effects",
            "--checkpoint-dir",
            path(&checkpoints),
            "--checkpoint-key-dir",
            path(&key_dir),
        ],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let file = checkpoints.join("checkpoint-0.json");
    let bytes = fs::read(&file).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("cli-private-replay-marker-9074"));
    let artifact = read_checkpoint_file_with_keys(&file, Some(&key_dir));
    assert_eq!(artifact["schema"], "etas.cli.interpreter-checkpoint.v60");
    let id = artifact["checkpoint"]["replay_key_id"].as_str().unwrap();
    assert_eq!(id.len(), 64);
    assert!(key_dir.join(id).is_file());
    assert!(!checkpoints.join(id).exists());
    assert_eq!(
        artifact["checkpoint"]["completed_host_boundaries"][0]["key"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "resume",
            "0",
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
    assert!(stdout.contains("resumed checkpoint 0"), "{stdout}");
    assert!(
        !fs::read_to_string(&trace)
            .unwrap()
            .contains("cli-private-replay-marker-9074")
    );
    let (code, stdout, stderr) = run_process_with_command(
        command(),
        [
            "resume",
            "0",
            "--checkpoint-dir",
            path(&checkpoints),
            "--checkpoint-key-dir",
            path(&absent_dir),
        ],
        "",
    );
    assert_eq!(code, 3, "{stdout}\n{stderr}");
    assert!(
        stderr.contains("replay key") && stderr.contains("unavailable"),
        "{stderr}"
    );
    assert_eq!(
        fs::read_dir(&absent_dir).unwrap().count(),
        0,
        "resume must not generate substitute credentials"
    );
}
