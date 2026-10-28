use super::*;

fn index_cli<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    run_process_with_command(command, args, "")
}

fn index_project(source: &str) -> PathBuf {
    let root = package_fixture(
        "index-inference",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "index-inference"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.1" }
[[bin]]
name = "index-inference"
module = "app.main"
flow = "main"
"#,
            ),
            ("src/app/main.es", source),
        ],
    );
    let (code, stdout, stderr) = index_cli(["pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    root
}

#[test]
fn deferred_index_literals_check_and_run_with_contextual_byte_types() {
    let file = index_project(
        r#"
module app.main;
import std.codec.text.utf8_encode;
flow identity<T>(value: T) -> T { return value; }
flow is_ascii_a(data: bytes) -> bool {
    let alias = identity(data);
    return alias[0] == 65;
}
flow main(args: Array<string>) -> i32 {
    if is_ascii_a(utf8_encode("A")) { return 0; }
    return 1;
}
"#,
    );
    let (code, stdout, stderr) = index_cli(["check", "--all", path(&file)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let (code, stdout, stderr) = index_cli(["run", path(&file)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"0"}"#),
        "{stdout}\n{stderr}"
    );
}

#[test]
fn deferred_index_literals_do_not_coerce_concrete_integers_or_overflowing_literals() {
    for (params, value, expected) in [
        ("data: bytes", "256", "numeric literal `256`"),
        (
            "data: bytes, other: i32",
            "other",
            "`i32` is not assignable to `u8`",
        ),
    ] {
        let file = index_project(&format!(
            "module app.main; flow identity<T>(value: T) -> T {{ return value; }} flow probe({params}) -> bool {{ return identity(data)[0] == {value}; }} flow main(args: Array<string>) -> i32 {{ return 0; }}"
        ));
        let (code, stdout, stderr) = index_cli(["check", "--all", path(&file)]);
        assert_eq!(code, 1, "{stdout}\n{stderr}");
        assert!(stderr.contains(expected), "{stdout}\n{stderr}");
    }
}
