use super::*;

#[test]
fn hostless_run_preserves_cpu_worker_configuration() {
    let file = fixture(
        "cpu-workers",
        r#"
import std.task.{map_concurrent, run, Scheduler};
flow square(value: i32) -> i32 {
    var index = 0;
    while index < 2000 limit Iterations(2000) { index = index + 1; }
    return value * value;
}

flow main(args: Array<string>) -> i32 {
    let values = run((scheduler: Scheduler) => {
        return map_concurrent([2, 3, 4, 5], square);
    });
    if values == [4, 9, 16, 25] { return 0; }
    return 1;
}
"#,
    );
    let mut values = Vec::new();
    for threads in ["1", "4"] {
        let (code, stdout, stderr) = run([
            "etas",
            "run",
            path(&file),
            "--cpu-threads",
            threads,
            "--format",
            "json",
        ]);
        assert_eq!(code, 0, "{stderr}\n{stdout}");
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        values.push(result["value"].clone());
        let workers = result["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["kind"] == "task_cpu_completed")
            .map(|event| {
                assert_eq!(event["layer"], "cpu_scheduling");
                assert!(event.get("thread").is_none());
                event["worker"].as_u64().unwrap()
            })
            .collect::<std::collections::BTreeSet<_>>();
        if threads == "1" {
            assert!(workers.is_empty());
        } else {
            assert!(workers.len() >= 2 && workers.iter().all(|worker| *worker < 4));
        }
    }
    assert_eq!(values[0], values[1]);
}

#[test]
fn memory_task_trace_reports_resource_permits_without_exposing_keys() {
    let file = fixture(
        "memory-runtime-guards",
        r#"
import std.task.{map_concurrent, run, Scheduler};
alias Schema = MemoryRegion<{ Entries: Store<string, string> }>;
let Memory = std.memory.region<Schema>(stable_id = "runtime_guards", store = "demo");
flow write(key: string) -> string { Memory.Entries.put(key, key); return key; }
flow main(args: Array<string>) -> i32 {
    let values = run((scheduler: Scheduler) => { return map_concurrent(["a", "b", "a"], write); });
    if values == ["a", "b", "a"] { return 0; }
    return 1;
}
"#,
    );
    let config = temp_path("memory-runtime-guards", "toml");
    fs::write(
        &config,
        r#"
[runtime.profiles.guard.memory]
backend = "memory"
[runtime.profiles.guard.policy]
mode = "local-static"
rules = ["memory=allow"]
"#,
    )
    .unwrap();
    let project = package_fixture(
        "memory-runtime-guards",
        &[
            (
                "etas.toml",
                "[package]\nname = 'memory-runtime-guards'\nversion = '0.1.0'\n[source]\nroot = 'src'\n[[bin]]\nname = 'app'\nmodule = 'app.main'\nflow = 'main'\n",
            ),
            (
                "src/app/main.es",
                &format!("module app.main;\n{}", fs::read_to_string(&file).unwrap()),
            ),
        ],
    );
    let (code, stdout, stderr) = run([
        "etas",
        "--no-config",
        "--cache",
        "off",
        "--format",
        "json",
        "run",
        path(&project),
        "--runtime-config",
        path(&config),
        "--profile",
        "guard",
        "--allow-effects",
        "--cpu-threads",
        "4",
    ]);
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let events = report["events"].as_array().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| event["kind"] == "task_resources_acquired")
            .count(),
        3
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event["kind"] == "task_resources_released")
            .count(),
        3
    );
    assert!(
        events
            .iter()
            .any(|event| event["kind"] == "task_waiting_resource")
    );
    assert!(
        events
            .iter()
            .filter(|event| event["kind"] == "task_resources_acquired")
            .all(|event| event.get("key").is_none() && event["permits"] == 1)
    );
    assert!(
        !events
            .iter()
            .any(|event| event["kind"] == "task_dispatched")
    );
    assert_eq!(report["value"]["value"], "0");
}
