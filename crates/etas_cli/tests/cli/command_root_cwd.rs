use super::*;

#[test]
fn command_run_accepts_workspace_region_root_as_cwd() {
    let root = package_fixture(
        "command-root-cwd",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "command-root-cwd"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.2" }
[runtime]
default_profile = "command"
[runtime.profiles.command.filesystem.regions."app.main.ProjectRoot"]
root = "/tmp"
read = true
write = false
[runtime.profiles.command.command]
allow = ["/bin/pwd"]
isolation = "trusted-unconfined"
[runtime.profiles.command.policy]
mode = "local-static"
rules = ["command=allow", "console=allow"]
[[bin]]
name = "command-root-cwd"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"
module app.main;
import std.fs.{Region, path};
import std.host.command.{command, with_cwd, run};
type ProjectRoot;
impl ProjectRoot ~ Region;
flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>] {
    let root = match path<ProjectRoot>(".") {
        Ok(value) => value,
        Err(_) => { abort("root path construction failed"); }
    };
    let result = run(with_cwd(command("/bin/pwd", []), root), DefaultCommandSandbox);
    return result.exit_code;
}
"#,
            ),
        ],
    );
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
        remove_legacy_runtime_host_env(&mut command);
        command
    };

    let (code, stdout, stderr) =
        run_process_with_command(command(), ["pkg", "lock", path(&root)], "");
    assert_eq!(code, 0, "{stdout}\n{stderr}");

    let (code, stdout, stderr) = run_process_with_command(
        command(),
        ["--format", "json", "run", path(&root), "--allow-effects"],
        "",
    );
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        report["value"],
        serde_json::json!({"kind":"number","type":"i32","value":"0"}),
        "{stdout}\n{stderr}"
    );
}
