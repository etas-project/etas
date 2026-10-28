use super::*;

#[test]
fn priority_queue_comparison_replays_a_generic_metadata_factory_then_runs() {
    let dependency = package_fixture(
        "priority-factory-library",
        &[
            (
                "etas.toml",
                "[package]\nname = \"priority-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = { version = \"0.1\" }\n",
            ),
            (
                "src/repro/priority.es",
                r#"
module repro.priority;
public flow make<T, P>(compare: (P, P) -> i32 ![]) -> PriorityQueue<T, P> {
    return PriorityQueue.new<T, P>(compare);
}
"#,
            ),
        ],
    );
    let consumer = package_fixture(
        "priority-factory-consumer",
        &[
            (
                "etas.toml",
                &format!(
                    "[package]\nname = \"priority-consumer\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = {{ version = \"0.1\" }}\npriority = {{ path = \"{}\", import = \"repro.priority\" }}\n[[bin]]\nname = \"main\"\nmodule = \"repro.consumer\"\nflow = \"main\"\n",
                    dependency.display()
                ),
            ),
            (
                "src/repro/consumer.es",
                r#"
module repro.consumer;
import repro.priority.make;
type Rank = { score: i32 };
flow compare(a: Rank, b: Rank) -> i32 ![] {
    if a.score < b.score { return 1; }
    if a.score > b.score { return -1; }
    return 0;
}
flow main(args: Array<string>) -> i32 {
    let queue = make<string, Rank>(compare)
        .push("second", Rank { score = 2 }).push("first", Rank { score = 1 });
    let (_, next) = queue.pop();
    if next == Some("first") { return 0; }
    return 1;
}
"#,
            ),
        ],
    );
    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&consumer)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(dependency.join(".etas/package.etasmeta").is_file());
    let (code, stdout, stderr) = run(["etas", "check", path(&consumer), "--all"]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    let (code, stdout, stderr) = run(["etas", "run", path(&consumer)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"0"}"#),
        "{stdout}"
    );
}
