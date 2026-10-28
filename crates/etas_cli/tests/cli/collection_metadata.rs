use super::*;

#[test]
fn collection_entry_iteration_replays_generic_dependency_signatures() {
    let dependency = package_fixture(
        "collection-entry-library",
        &[
            (
                "etas.toml",
                "[package]\nname = \"entry-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = { version = \"0.1\" }\n",
            ),
            (
                "src/repro/library.es",
                r#"
module repro.library;
public flow ordered<K, V>(entries: OrderedMap<K, V>) -> OrderedMap<K, V> { return entries; }
public flow priorities<T, P>(entries: PriorityQueue<T, P>) -> PriorityQueue<T, P> { return entries; }
public flow unique<T>(entries: OrderedSet<T>) -> OrderedSet<T> { return entries; }
"#,
            ),
        ],
    );
    let consumer = package_fixture(
        "collection-entry-consumer",
        &[
            (
                "etas.toml",
                &format!(
                    "[package]\nname = \"entry-consumer\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = {{ version = \"0.1\" }}\nentries = {{ path = \"{}\", import = \"repro.library\" }}\n[[bin]]\nname = \"main\"\nmodule = \"repro.consumer\"\nflow = \"main\"\n",
                    dependency.display(),
                ),
            ),
            (
                "src/repro/consumer.es",
                r#"
module repro.consumer;
import repro.library.{ordered, priorities, unique};
flow main(args: Array<string>) -> i32 {
    let map = ordered(OrderedMap.new<string, i32>().insert("entry", 7));
    for (key, value) in map limit Iterations(2) {
        let checked_key: string = key;
        let checked_value: i32 = value;
    }
    let queue = priorities(PriorityQueue.new<string, i32>().push("work", 2));
    for (priority, value) in queue limit Iterations(2) {
        let checked_priority: i32 = priority;
        let checked_value: string = value;
    }
    for value in unique(OrderedSet.new<string>().insert("entry")) limit Iterations(2) {
        let checked_value: string = value;
    }
    return 0;
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

    let file = consumer.join("src/repro/consumer.es");
    let valid = fs::read_to_string(&file).unwrap();
    for (from, to) in [
        (
            "let checked_key: string = key;",
            "let checked_key: i32 = key;",
        ),
        (
            "let checked_priority: i32 = priority;",
            "let checked_priority: string = priority;",
        ),
    ] {
        fs::write(&file, valid.replace(from, to)).unwrap();
        let (code, stdout, stderr) = run([
            "etas",
            "check",
            path(&consumer),
            "--all",
            "--format",
            "json",
        ]);
        assert_eq!(code, 1, "{stdout}\n{stderr}");
        let output: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let files = output["files"].as_array().unwrap();
        assert_eq!(files.len(), 1, "{output}");
        assert_eq!(
            fs::canonicalize(files[0]["path"].as_str().unwrap()).unwrap(),
            fs::canonicalize(&file).unwrap(),
        );
        let diagnostics = files[0]["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1, "{output}");
        assert!(
            matches!(
                diagnostics[0]["code"].as_str(),
                Some("type::Mismatch" | "type::TypeMismatch")
            ),
            "{output}"
        );
        let expected_line = valid[..valid.find(from).unwrap()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        assert_eq!(
            diagnostics[0]["primary"]["span"]["startLine"], expected_line,
            "{output}"
        );
    }
    fs::remove_dir_all(consumer).unwrap();
    fs::remove_dir_all(dependency).unwrap();
}
