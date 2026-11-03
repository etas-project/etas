use super::*;

#[test]
fn runtime_profile_filesystem_io_errors_are_catchable_without_enabling_writes() {
    let project = package_fixture(
        "filesystem-ioerror-control",
        &[
            (
                "etas.toml",
                r#"[package]
name = "filesystem-ioerror-control"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "filesystem-ioerror-control"
module = "app.main"
flow = "main"
profile = "readonly"
[runtime.profiles.readonly.filesystem.regions."app.main.ProjectRoot"]
root = "."
read = true
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;
import std.fs.{Region, IOError as FileError, path, stat, write_bytes};
import std.io.println;
import std.codec.text.utf8_encode;
type ProjectRoot;
impl ProjectRoot ~ Region;

flow main(args: Array<string>) -> i32 {
    let target = match path<ProjectRoot>("missing.txt") {
        Ok(value) => value,
        Err(_) => { return 2; }
    };
    return handle {
        let value = stat(target);
        println("UNEXPECTED");
        1
    } with {
        Error<FileError>.raise(_) => {
            println("CAUGHT direct IOError");
            finish 0;
        }
    };
}

flow denied(args: Array<string>) -> i32 {
    let target = match path<ProjectRoot>("must-not-exist.txt") {
        Ok(value) => value,
        Err(_) => { return 2; }
    };
    return handle {
        write_bytes(target, utf8_encode("must not be written"));
        println("UNEXPECTED");
        1
    } with {
        Error<FileError>.raise(_) => {
            println("CAUGHT direct IOError");
            finish 0;
        }
    };
}
"#,
            ),
        ],
    );
    for (flow, expected_code) in [("main", 0), ("denied", 3)] {
        let trace = project.join(format!("{flow}.trace.json"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        command.env_clear().env("HOME", &project).env(
            "PATH",
            std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
        );
        let (code, stdout, stderr) = run_process_with_command(
            command,
            [
                "run",
                path(&project),
                "--flow",
                flow,
                "--trace-out",
                path(&trace),
            ],
            "",
        );
        assert_eq!(code, expected_code, "{flow}: {stdout}\n{stderr}");
        assert!(!stdout.contains("UNEXPECTED"), "{stdout}");
        if flow == "main" {
            assert_eq!(
                stdout.matches("CAUGHT direct IOError").count(),
                1,
                "{stdout}"
            );
            assert!(stderr.is_empty(), "{stderr}");
        } else {
            assert!(!stdout.contains("CAUGHT direct IOError"), "{stdout}");
            assert!(!project.join("must-not-exist.txt").exists());
        }
        let report: serde_json::Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
        if flow == "main" {
            assert_eq!(report["outcome"]["kind"], "completed", "{report}");
            assert_eq!(report["value"]["value"], "0", "{report}");
        } else {
            assert_eq!(report["outcome"]["kind"], "failed", "{report}");
            assert_eq!(
                report["outcome"]["failure"]["kind"], "preparation_rejected",
                "{report}"
            );
            assert_eq!(
                report["diagnostics"][0]["code"], "Analysis(MissingHostHandler)",
                "{report}"
            );
        }
        let events = report["events"].as_array().unwrap();
        let filesystem_requests = events
            .iter()
            .filter(|event| {
                event["kind"] == "host_request_started" && event["request_kind"] == "filesystem"
            })
            .collect::<Vec<_>>();
        if flow == "denied" {
            assert!(
                filesystem_requests.is_empty(),
                "unauthorized write was dispatched: {report}"
            );
            continue;
        }
        assert_eq!(filesystem_requests.len(), 1, "{flow}: {report}");
        assert_eq!(filesystem_requests[0]["qualified_action"], "Fs.stat");
        let id = &filesystem_requests[0]["id"];
        assert!(
            events
                .iter()
                .any(|event| event["kind"] == "host_request_finished"
                    && &event["id"] == id
                    && event["outcome"]["code"] == "IoFailure"),
            "{flow}: {report}"
        );
    }
}
