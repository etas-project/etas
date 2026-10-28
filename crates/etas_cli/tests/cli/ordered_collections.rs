use super::*;

#[test]
fn ordered_comparisons_replay_generic_metadata_factories_then_run() {
    let dependency = package_fixture(
        "ordered-factory-library",
        &[
            (
                "etas.toml",
                "[package]\nname = \"ordered-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = { version = \"0.1\" }\n",
            ),
            (
                "src/repro/ordered.es",
                r#"
module repro.ordered;
public flow map<K, V>(compare: (K, K) -> i32 ![]) -> OrderedMap<K, V> {
    return OrderedMap.new<K, V>(compare);
}
public flow set<T>(compare: (T, T) -> i32 ![]) -> OrderedSet<T> {
    return OrderedSet.new<T>(compare);
}
"#,
            ),
        ],
    );
    let consumer = package_fixture(
        "ordered-factory-consumer",
        &[
            (
                "etas.toml",
                &format!(
                    "[package]\nname = \"ordered-consumer\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = {{ version = \"0.1\" }}\nordered = {{ path = \"{}\", import = \"repro.ordered\" }}\n[[bin]]\nname = \"main\"\nmodule = \"repro.consumer\"\nflow = \"main\"\n",
                    dependency.display()
                ),
            ),
            (
                "src/repro/consumer.es",
                r#"
module repro.consumer;
import repro.ordered.{map, set};
type Key = { parts: (i32, string) };
flow compare(a: Key, b: Key) -> i32 ![] {
    let (left, _) = a.parts;
    let (right, _) = b.parts;
    if left < right { return -1; }
    if left > right { return 1; }
    return 0;
}
flow main(args: Array<string>) -> i32 {
    let one = Key { parts = (1, "one") };
    let two = Key { parts = (2, "two") };
    let old = map<Key, string>(compare).insert(two, "old").insert(one, "first");
    let changed = old.insert(Key { parts = (2, "equivalent") }, "new");
    let keys = set<Key>(compare).insert(two).insert(one).insert(two);
    var map_order = 0;
    var set_order = 0;
    for (key, value) in changed limit Iterations(8) {
        let (number, _) = key.parts;
        map_order = map_order * 10 + number;
    }
    for key in keys limit Iterations(8) {
        let (number, _) = key.parts;
        set_order = set_order * 10 + number;
    }
    if map_order == 12 && set_order == 12 && old.get(two) == Some("old")
        && changed.get(two) == Some("new") && keys.contains(one) { return 0; }
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
