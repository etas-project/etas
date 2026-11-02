use super::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn atomic_replace_preserves_unix_mode_through_the_real_cli() {
    let root = package_fixture(
        "atomic-replace-mode",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "atomic-replace-mode"
version = "0.1.0"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
[runtime]
default_profile = "write"
[runtime.profiles.write.filesystem.regions."app.main.Owned"]
root = "data"
write = true
"#,
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.fs.{Region, path, atomic_replace};
import std.fs.IOError as FileError;
import std.effects.Fs;
import std.codec.text.utf8_encode;
public type Owned;
impl Owned ~ Region;
public flow main(_args: Array<string>) -> i32
    ![Fs.atomic_replace<Owned>, Error<FileError>, Error<IndexError>] {
    let target = match path<Owned>("note.txt") {
        Ok(value) => value,
        Err(_) => { return 1; }
    };
    atomic_replace(target, utf8_encode("after\n"));
    return 0;
}
"#,
            ),
            ("data/note.txt", "before\n"),
        ],
    );
    let target = root.join("data/note.txt");
    let trace = root.join("trace.json");
    for mode in [0o600, 0o700, 0o444] {
        fs::remove_file(&target).unwrap();
        fs::write(&target, b"before\n").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(mode)).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        let (code, stdout, stderr) = run_process_with_command(
            command,
            ["run", path(&root), "--trace-out", path(&trace)],
            "",
        );
        assert_eq!(code, 0, "mode={mode:o}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert_eq!(fs::read(&target).unwrap(), b"after\n");
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o7777,
            mode
        );
        let report: serde_json::Value = serde_json::from_slice(&fs::read(&trace).unwrap()).unwrap();
        assert_eq!(report["outcome"]["kind"], "completed");
        assert_eq!(report["value"]["value"], "0");
        let requests: Vec<_> = report["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| {
                event["kind"] == "host_request_started" && event["request_kind"] == "filesystem"
            })
            .collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["qualified_action"], "Fs.atomic_replace");
        assert_eq!(fs::read_dir(root.join("data")).unwrap().count(), 1);
    }
}
