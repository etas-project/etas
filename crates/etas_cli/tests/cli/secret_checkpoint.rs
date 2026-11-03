use super::*;

const MARKER: &str = "ETAS_AUDIT_ONLY_SECRET_FILE_PAYLOAD_20261031";

fn assert_completed(stage: &str, output: (i32, String, String)) {
    let (code, stdout, stderr) = output;
    assert_eq!(code, 0, "{stage}: {stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stage}: {stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["outcome"]["kind"], "completed", "{stage}");
    assert_eq!(
        report["value"],
        serde_json::json!({"kind":"number", "type":"i32", "value":"42"}),
        "{stage}: restored Secret must retain its original value"
    );
    assert!(!stdout.contains(MARKER), "{stage}: report leaked a secret");
}

fn assert_confidential_roundtrip(case: &str, declarations: &str, body: &str) {
    let root = temp_path(
        &format!("secret-checkpoint-{case}-{}", std::process::id()),
        "dir",
    );
    fs::create_dir(&root).unwrap();
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let source = root.join("main.es");
    let checkpoints = root.join("checkpoints");
    let keys = root.join("private-keys");
    let body = body.replace("__SECRET__", MARKER);
    fs::write(
        &source,
        format!(
            r#"
module tests.cli.secret_checkpoint;
import std.runtime.checkpoint;
flow consume(value: Secret<string>) -> i32 {{
    if value == Secret("{MARKER}") {{ return 42; }}
    return 41;
}}
{declarations}
flow main(args: Array<string>) -> i32 {{
{body}
}}
"#
        ),
    )
    .unwrap();
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command
            .env_clear()
            .env("HOME", &home)
            .env("PATH", "/usr/bin:/bin")
            .current_dir(&root);
        command
    };
    assert_completed(
        "run",
        run_process_with_command(
            command(),
            [
                "--format",
                "json",
                "run",
                path(&source),
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
            ],
            "",
        ),
    );
    let checkpoint = checkpoints.join("checkpoint-0.json");
    let bytes = fs::read(&checkpoint).unwrap();
    assert_completed(
        "resume",
        run_process_with_command(
            command(),
            [
                "--format",
                "json",
                "resume",
                "0",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
            ],
            "",
        ),
    );
    // Failed fake-only samples remain available for the privacy audit.
    assert!(
        !bytes
            .windows(MARKER.len())
            .any(|window| window == MARKER.as_bytes()),
        "plaintext live Secret persisted in {} ({case}); run and resume both retained the real value",
        checkpoint.display()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn file_checkpoint_keeps_a_direct_live_secret_confidential_and_restorable() {
    assert_confidential_roundtrip(
        "direct",
        "",
        r#"
    let value = Secret("__SECRET__");
    checkpoint("direct-secret");
    return consume(value);
"#,
    );
}

#[test]
fn file_checkpoint_keeps_a_nested_nominal_secret_confidential_and_restorable() {
    assert_confidential_roundtrip(
        "nominal",
        "type Bundle = { item: Secret<string> };",
        r#"
    let bundle = Bundle { item = Secret("__SECRET__") };
    checkpoint("nested-secret");
    return consume(bundle.item);
"#,
    );
}

#[test]
fn file_checkpoint_keeps_a_secret_in_an_escaped_capture_confidential_and_restorable() {
    assert_confidential_roundtrip(
        "capture",
        "",
        r#"
    let check = {
        let value = Secret("__SECRET__");
        () => consume(value)
    };
    checkpoint("captured-secret");
    return check();
"#,
    );
}

#[test]
fn file_checkpoint_keeps_a_pending_secret_argument_confidential_and_restorable() {
    assert_confidential_roundtrip(
        "pending-argument",
        r#"
flow mark() -> unit { checkpoint("pending-secret"); return; }
flow consume_after(value: Secret<string>, done: unit) -> i32 { return consume(value); }
"#,
        r#"
    return consume_after(Secret("__SECRET__"), mark());
"#,
    );
}

#[test]
fn sealed_file_rejects_missing_keys_tampering_and_plaintext_legacy_before_resume() {
    use etas_host::replay::{DirectoryReplayKeyProvider, ReplayKeyProvider};

    let workspace = etas_host::TestWorkspace::create("sealed-checkpoint-negative").unwrap();
    let root = workspace.path();
    let source = root.join("main.es");
    let checkpoints = root.join("checkpoints");
    let keys = root.join("private-keys");
    fs::write(
        &source,
        format!(
            r#"
module tests.cli.sealed_checkpoint;
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 {{
    let value = Secret("{MARKER}");
    checkpoint("saved");
    if value == Secret("{MARKER}") {{ return 42; }}
    return 41;
}}
"#
        ),
    )
    .unwrap();
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command
            .env_clear()
            .env("HOME", root)
            .env("PATH", "/usr/bin:/bin")
            .current_dir(root);
        command
    };
    assert_completed(
        "run",
        run_process_with_command(
            command(),
            [
                "--format",
                "json",
                "run",
                path(&source),
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
            ],
            "",
        ),
    );
    let file = checkpoints.join("checkpoint-0.json");
    let original = fs::read(&file).unwrap();
    assert!(original.starts_with(b"ETASCP01"));
    let reject = |key_dir: &Path, expected: &str| {
        let before = fs::read(&file).unwrap();
        let (code, stdout, stderr) = run_process_with_command(
            command(),
            [
                "--format",
                "json",
                "resume",
                "0",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(key_dir),
            ],
            "",
        );
        assert_ne!(code, 0, "{stdout}\n{stderr}");
        assert!(stdout.is_empty(), "rejected file executed source: {stdout}");
        assert!(stderr.contains(expected), "{stderr}");
        assert!(!stderr.contains(MARKER));
        assert_eq!(fs::read(&file).unwrap(), before, "resume rewrote its input");
    };
    reject(&root.join("missing-keys"), "unavailable");
    let provider = DirectoryReplayKeyProvider::open(&keys).unwrap();
    let wrong = provider.create().unwrap();
    provider.retain(&wrong).unwrap();
    let mut changed = original.clone();
    changed[8..72].copy_from_slice(wrong.id().to_hex().as_bytes());
    fs::write(&file, changed).unwrap();
    reject(&keys, "authentication failed");
    for offset in [72, 84, original.len() - 1] {
        let mut changed = original.clone();
        changed[offset] ^= 1;
        fs::write(&file, changed).unwrap();
        reject(&keys, "authentication failed");
    }
    fs::write(&file, &original[..90]).unwrap();
    reject(&keys, "truncated");
    fs::write(&file, &original).unwrap();
    let (document, _) =
        etas_cli::command::interpreter::checkpoint_file::read(&file, Some(&keys)).unwrap();
    let plaintext =
        etas_interpreter::api::codec::checkpoint_file_to_bytes(document, Default::default())
            .unwrap();
    fs::write(&file, plaintext).unwrap();
    reject(&keys, "legacy plaintext");
    fs::write(&file, &original).unwrap();
    assert_completed(
        "resume",
        run_process_with_command(
            command(),
            [
                "--format",
                "json",
                "resume",
                "0",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
            ],
            "",
        ),
    );
    assert_eq!(fs::read(&file).unwrap(), original);
}
