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
