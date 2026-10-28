use super::*;

#[test]
fn collection_functions_replay_generic_bounds_and_all_collection_families() {
    let library = package_fixture(
        "collection-api-library",
        &[
            (
                "etas.toml",
                "[package]\nname = \"collection-api\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = { version = \"0.1\" }\n",
            ),
            (
                "src/repro/api.es",
                r#"
module repro.api;
import std.collections;
public flow array<T, I ~ Index>(xs: Array<T>, i: I) -> Option<T> { return std.collections.array_get(xs, i); }
public flow list<T, I ~ Index>(xs: List<T>, i: I) -> Option<T> { return std.collections.list_get(xs, i); }
public flow slice<T, I ~ Index>(xs: Slice<T>, i: I) -> Option<T> { return std.collections.slice_get(xs, i); }
public flow map<K, V>(xs: Map<K, V>, k: K) -> Option<V> { return std.collections.map_get(xs, k); }
public flow range<I ~ Index>(start: I, end: I) -> Range<I> { return std.collections.closed<I>(start, end); }
public flow deque<T>(xs: Deque<T>, value: T) -> Deque<T> { return std.collections.deque_push_back(xs, value); }
public flow queue<T>(xs: Queue<T>, value: T) -> Queue<T> { return std.collections.queue_push(xs, value); }
public flow stack<T>(xs: Stack<T>, value: T) -> Stack<T> { return std.collections.stack_push(xs, value); }
public flow priority<T, P>(xs: PriorityQueue<T, P>, value: T, p: P) -> PriorityQueue<T, P> { return std.collections.priority_queue_push(xs, value, p); }
public flow ordered<K, V>(xs: OrderedMap<K, V>, k: K, v: V) -> OrderedMap<K, V> { return std.collections.ordered_map_insert(xs, k, v); }
public flow unique<T>(xs: OrderedSet<T>, value: T) -> OrderedSet<T> { return std.collections.ordered_set_insert(xs, value); }
public flow set<T>(xs: Set<T>) -> Set<T> { return xs; }
public flow ordered_new<K, V>(compare: (K, K) -> i32 ![]) -> OrderedMap<K, V> { return OrderedMap.new<K, V>(compare); }
public flow unique_new<T>(compare: (T, T) -> i32 ![]) -> OrderedSet<T> { return OrderedSet.new<T>(compare); }
"#,
            ),
        ],
    );
    let consumer = package_fixture(
        "collection-api-consumer",
        &[
            (
                "etas.toml",
                &format!(
                    "[package]\nname = \"collection-consumer\"\nversion = \"0.1.0\"\nedition = \"2026\"\n[source]\nroot = \"src\"\n[dependencies]\nstd = {{ version = \"0.1\" }}\napi = {{ path = \"{}\", import = \"repro.api\" }}\n[[bin]]\nname = \"main\"\nmodule = \"repro.consumer\"\nflow = \"main\"\n",
                    library.display()
                ),
            ),
            (
                "src/repro/consumer.es",
                r#"
module repro.consumer;
import repro.api;
type Key = i32;
type Rank = { value: i32 };
flow reverse(a: Rank, b: Rank) -> i32 ![] {
    if a.value > b.value { return -1; }
    if a.value < b.value { return 1; }
    return 0;
}
flow main(args: Array<string>) -> i32 {
    let index: u8 = 1;
    let a = [4, 5];
    let l = [4; 5];
    let s = a[0, 2);
    let m = {Key(4) => "four"};
    let checked_array: Option<i32> = api.array(a, index);
    let checked_list: Option<i32> = api.list(l, index);
    let checked_slice: Option<i32> = api.slice(s, index);
    let checked_map: Option<string> = api.map(m, Key(4));
    let d = api.deque(Deque.new<i32>().push_back(1), 2);
    let q = api.queue(Queue.new<i32>().push(1), 2);
    let st = api.stack(Stack.new<i32>().push(1), 2);
    let pq = api.priority(PriorityQueue.new<string, i32>().push("low", 1), "high", 2);
    let om = api.ordered(OrderedMap.new<i32, string>().insert(2, "two"), 1, "one");
    let os = api.unique(OrderedSet.new<i32>().insert(2), 1);
    var r = 0; for i in api.range(1, 3) limit Iterations(4) { r = r * 10 + i; }
    var dc = 0; for i in d limit Iterations(3) { dc = dc * 10 + i; }
    var qc = 0; for i in q limit Iterations(3) { qc = qc * 10 + i; }
    var sc = 0; for i in st limit Iterations(3) { sc = sc * 10 + i; }
    var pc = ""; for (_, value) in pq limit Iterations(3) { pc = pc + value; }
    var mc = 0; for (key, _) in om limit Iterations(3) { mc = mc * 10 + key; }
    var oc = 0; for key in os limit Iterations(3) { oc = oc * 10 + key; }
    var uc = 0; for i in api.set(#{1, 2, 1}) limit Iterations(3) { uc = uc * 10 + i; }
    if checked_array != Some(5) { return 11; }
    if checked_list != Some(5) { return 12; }
    if checked_slice != Some(5) { return 13; }
    if checked_map != Some("four") { return 14; }
    if r != 123 { return 15; }
    if dc != 12 { return 16; }
    if qc != 12 { return 17; }
    if sc != 12 { return 18; }
    if pc != "lowhigh" { return 19; }
    if mc != 12 { return 20; }
    if oc != 12 { return 21; }
    if uc != 12 { return 22; }
    let (_, top) = pq.pop();
    if top != Some("high") { return 23; }
    let cm = api.ordered_new<Rank, string>(reverse).insert(Rank { value = 1 }, "one").insert(Rank { value = 2 }, "two");
    let cs = api.unique_new<Rank>(reverse).insert(Rank { value = 1 }).insert(Rank { value = 2 }).insert(Rank { value = 1 });
    var cmc = 0; for (key, _) in cm limit Iterations(3) { cmc = cmc * 10 + key.value; }
    var csc = 0; for key in cs limit Iterations(3) { csc = csc * 10 + key.value; }
    if cmc != 21 || csc != 21 { return 24; }
    return 0;
}
"#,
            ),
        ],
    );
    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&consumer)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert_collection_binary_signatures(&library);
    let (code, stdout, stderr) = run(["etas", "check", path(&consumer), "--all"]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    let (code, stdout, stderr) = run(["etas", "run", path(&consumer)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"0"}"#),
        "{stdout}"
    );

    let file = consumer.join("src/repro/consumer.es");
    let valid = fs::read_to_string(&file).unwrap();
    for (from, to) in [
        ("api.array(a, index)", "api.array(a, true)"),
        ("api.list(l, index)", "api.list(l, Key(1))"),
        ("api.map(m, Key(4))", "api.map(m, 4)"),
        (
            "let checked_map:",
            "let wrong: Option<i32> = api.map(m, Key(4)); let checked_map:",
        ),
        (
            "api.queue(Queue.new<i32>().push(1), 2)",
            "api.queue(Queue.new<i32>().push(1), \"two\")",
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
            fs::canonicalize(&file).unwrap()
        );
        let line = valid[..valid.find(from).unwrap()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        let diagnostics = files[0]["diagnostics"].as_array().unwrap();
        assert!(!diagnostics.is_empty(), "{output}");
        for diagnostic in diagnostics {
            assert!(
                diagnostic["code"].as_str().unwrap().starts_with("type::"),
                "{output}"
            );
            assert_eq!(diagnostic["primary"]["span"]["startLine"], line, "{output}");
        }
    }
    fs::remove_dir_all(consumer).unwrap();
    fs::remove_dir_all(library).unwrap();
}

fn assert_collection_binary_signatures(library: &std::path::Path) {
    use etas_package_metadata::{GenericParamKind, TypeKind, package_metadata_from_artifact};

    let artifact = library.join(".etas/package.etasmeta");
    let (_, metadata) = package_metadata_from_artifact(&artifact, &fs::read(&artifact).unwrap())
        .expect("the producer must publish a valid binary artifact");
    let flows = &metadata.public_metadata.flows;
    assert_eq!(flows.len(), 14);
    for (name, params, index) in [
        ("array", &["T", "I"][..], Some("I")),
        ("list", &["T", "I"][..], Some("I")),
        ("slice", &["T", "I"][..], Some("I")),
        ("map", &["K", "V"][..], None),
        ("range", &["I"][..], Some("I")),
        ("deque", &["T"][..], None),
        ("queue", &["T"][..], None),
        ("stack", &["T"][..], None),
        ("priority", &["T", "P"][..], None),
        ("ordered", &["K", "V"][..], None),
        ("unique", &["T"][..], None),
        ("set", &["T"][..], None),
        ("ordered_new", &["K", "V"][..], None),
        ("unique_new", &["T"][..], None),
    ] {
        let expected_path = vec!["repro".to_owned(), "api".to_owned(), name.to_owned()];
        let flow = flows
            .iter()
            .find(|flow| flow.path == expected_path)
            .unwrap();
        assert_eq!(
            flow.generic_params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            params,
            "{name}"
        );
        for param in &flow.generic_params {
            assert_eq!(param.kind, GenericParamKind::Type, "{name}");
            if index == Some(param.name.as_str()) {
                assert_eq!(param.bounds.len(), 1, "{name}");
                assert_eq!(param.bounds[0].spec, ["std", "core", "Index"], "{name}");
                assert!(param.bounds[0].args.is_empty(), "{name}");
            } else {
                assert!(param.bounds.is_empty(), "{name}");
            }
        }
        if matches!(name, "ordered_new" | "unique_new") {
            let compare = &flow.input[0];
            assert_eq!(compare.kind, TypeKind::Function, "{name}");
            assert_eq!(compare.children.len(), 3, "{name}");
            assert!(
                compare
                    .effects
                    .as_ref()
                    .is_some_and(|row| row.effects.is_empty() && row.tail.is_none()),
                "{name}"
            );
        }
    }
}

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
