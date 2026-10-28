use super::*;

fn message_cli<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    run_process_with_command(command, args, "")
}

fn message_project(source: &str) -> PathBuf {
    let root = package_fixture(
        "message-cast",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "message-cast"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.1" }
[[bin]]
name = "message-cast"
module = "app.main"
flow = "main"
"#,
            ),
            ("src/app/main.es", source),
        ],
    );
    let (code, stdout, stderr) = message_cli(["pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    root
}

#[test]
fn message_constructor_cast_chains_check_and_run_with_checked_option_results() {
    let root = message_project(
        r#"
module app.main;
import std.agent.message.Message;
import std.json.JsonValue;
import std.crypto.sha256;
import std.bytes.len;
import std.codec.text.utf8_encode;
flow json(raw: bytes) -> Option<Message<JsonValue>> {
    return Message.new(raw).cast<JsonValue>();
}
flow main(args: Array<string>) -> i32 {
    let raw = utf8_encode("abc");
    let identity = Message.new(raw).cast<bytes>();
    let hashed = Message.new(sha256(raw)).cast<bytes>();
    let rejected = Message.new("not an integer").cast<i32>();
    let identity_ok = match identity { Some(message) => message.body == raw, None => false };
    let hash_ok = match hashed { Some(message) => len(message.body) == 32, None => false };
    let rejection_ok = match rejected { None => true, Some(_) => false };
    if identity_ok && hash_ok && rejection_ok { return 0; }
    return 1;
}
"#,
    );
    let (code, stdout, stderr) = message_cli(["check", "--all", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let (code, stdout, stderr) = message_cli(["run", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"0"}"#),
        "{stdout}\n{stderr}"
    );
}

#[test]
fn message_constructor_cast_chains_do_not_relax_the_declared_result_type() {
    let root = message_project(
        r#"
module app.main;
flow invalid(raw: bytes) -> Option<Message<bytes>> {
    return Message.new(raw).cast<i32>();
}
flow main(args: Array<string>) -> i32 { return 0; }
"#,
    );
    let (code, stdout, stderr) = message_cli(["check", "--all", path(&root)]);
    assert_eq!(code, 1, "{stdout}\n{stderr}");
    assert!(stderr.contains("type::TypeMismatch"), "{stdout}\n{stderr}");
}
