use std::{path::Path, process::Command};

use super::{
    package_fixture, path, remove_legacy_runtime_host_env, run_process_with_command, temp_path,
};

#[test]
fn runtime_profile_storage_limits_reach_sqlite_memory_backend() {
    let database = temp_path("runtime-storage-limits", "sqlite");
    let manifest = runtime_manifest(&database, None);
    let payload = "x".repeat(1_100_000);
    let source = format!(
        r#"module app.main;

alias ProjectMemorySchema = MemoryRegion<{{
  Values: Store<string, string>
}}>;

let ProjectMemory =
  std.memory.region<ProjectMemorySchema>(
    stable_id = "runtime_storage_limits",
    store = "runtime-storage-limits"
  );

flow main(args: Array<string>) -> i32 ![Memory.write<ProjectMemory>] {{
  let payload = "{payload}";
  ProjectMemory.Values.put("large", payload);
  return 0;
}}
"#
    );
    let project = package_fixture(
        "runtime-storage-limits",
        &[("etas.toml", &manifest), ("src/app/main.es", &source)],
    );

    let (code, stdout, stderr) = run_project(&project);
    assert_ne!(code, 0, "default 1 MiB value limit must reject the payload");
    assert!(
        format!("{stdout}\n{stderr}").contains("storage resource limit exceeded"),
        "{stderr}"
    );

    std::fs::write(
        project.join("etas.toml"),
        runtime_manifest(&database, Some(2 * 1024 * 1024)),
    )
    .expect("update runtime storage limits");
    let (code, stdout, stderr) = run_project(&project);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(r#""value":"0""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

fn runtime_manifest(database: &Path, max_value_bytes: Option<usize>) -> String {
    let storage = max_value_bytes.map_or_else(String::new, |max_value_bytes| {
        format!("\n[runtime.profiles.local.storage]\nmax_value_bytes = {max_value_bytes}\n")
    });
    format!(
        r#"[package]
name = "runtime-storage-limits"
version = "0.1.0"

[runtime]
default_profile = "local"

[runtime.profiles.local.memory]
backend = "sqlite"
path = '{}'
{storage}
[runtime.profiles.local.policy]
mode = "local-static"
rules = ["memory=allow"]

[[bin]]
name = "app"
module = "app.main"
flow = "main"
profile = "local"
"#,
        path(database)
    )
}

fn run_project(project: &Path) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    run_process_with_command(
        command,
        [
            "run",
            path(project),
            "--profile",
            "local",
            "--allow-effects",
        ],
        "",
    )
}
