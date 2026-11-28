use super::*;

fn rewrite_join_result(assembly: &Path, rewrite: impl FnOnce(&str) -> String) {
    let mut source = fs::read_to_string(assembly).unwrap();
    let start = source.find("public flow join(").unwrap();
    let end = start + source[start..].find("\n}").unwrap();
    let result = start + source[start..end].rfind("return ").unwrap() + "return ".len();
    let semicolon = result + source[result..end].find(';').unwrap();
    let replacement = rewrite(&source[result..semicolon]);
    source.replace_range(result..semicolon, &replacement);
    fs::write(assembly, source).unwrap();
}

#[test]
fn installed_std_executes_new_generic_flow_and_rebuilt_join_body_in_external_processes() {
    let root = temp_path(
        &format!(
            "std-source-sdk-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
        "workspace",
    );
    let binary = std::env::var_os("ETAS_STD_DELIVERY_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_etas")));
    let source = root.join("std-source");
    let maintained = etas_std::library::source_checkout_manifest()
        .parent()
        .unwrap();
    copy_dir_contents(&maintained.join("library"), &source.join("library"));
    fs::create_dir_all(source.join("library/std/extra")).unwrap();
    fs::write(
        source.join("library/std/io/sdk.es"),
        r#"module std.io;
public flow sdk_try_line() -> Result<string, IOError> ![] { return read_line()?; }
"#,
    )
    .unwrap();
    let limits = source.join("library/std/runtime/limits/mod.es");
    fs::write(
        &limits,
        fs::read_to_string(&limits)
            .unwrap()
            .replace("public enum Limit {", "public enum Limit { SdkOnly,"),
    )
    .unwrap();
    let filesystem = source.join("library/std/fs/mod.es");
    fs::write(
        &filesystem,
        fs::read_to_string(&filesystem).unwrap().replace(
            "public enum ReadLimitError {",
            "public enum ReadLimitError { SdkOnly,",
        ),
    )
    .unwrap();
    fs::copy(maintained.join("etas.toml"), source.join("etas.toml")).unwrap();
    fs::write(
        source.join("library/std/runtime/sdk.es"),
        r#"module std.runtime;
import std.runtime.checkpoint.checkpoint;
type Pending<T> = { value: T };
flow checkpoint_private<T>(value: T) -> T {
    let pending = Pending { value = value };
    checkpoint("inside-std-private");
    return pending.value;
}
public flow checkpointed<T>(value: T) -> T { return checkpoint_private(value); }
public flow checkpoint_apply<T>(action: (T) -> T ![], value: T) -> T {
    let first = action(value);
    checkpoint("inside-std-callback");
    return action(first);
}
"#,
    )
    .unwrap();
    // Ordinary error declarations belong to source. An extra constructor needs
    // no compiler role or native registry entry, and native parse errors keep
    // their source-owned nominal type.
    let errors = source.join("library/std/text/errors.es");
    fs::write(
        &errors,
        fs::read_to_string(&errors).unwrap().replace(
            "public enum TextError {",
            "public enum TextError { SdkOnly,",
        ),
    )
    .unwrap();
    fs::write(
        source.join("library/std/extra/mod.es"),
        r#"module std.extra;
import std.fs.IOError;
import std.core.Index;
import std.compiler.{RevisionInfo, TypeInfo, CallableId, SyntaxBlock, CallableDef, AgentDef};
public import std.extra.Reply.{Empty, Item, Named};
public effect Delivery extends std.effects.Network;
public effect Echo {
    action<Payload> echo(value: Payload) -> Payload;
    action<Payload> select<Key>(key: Key, value: Payload) -> Payload;
}
public flow request_echo<Value>(value: Value) -> Value ![Echo.echo] {
    return perform Echo.echo(value);
}
public flow delivery_value() -> i32 ![Delivery] { return 41; }
public flow primitive_value(value: std.core.i32) -> i32 ![] { return value; }
public flow money_value(value: f64) -> std.runtime.budget.Money ![] { return std.runtime.budget.usd(value); }
public flow duration_value(value: std.runtime.time.Duration) -> std.runtime.time.Duration ![] { return value; }
public flow limited_value(value: usize) -> std.runtime.limits.Limit ![] { return std.runtime.limits.Iterations(value); }
public flow syntax_text(value: SyntaxBlock) -> string ![] { return value.text; }
public flow callable_revision<Function>(value: CallableDef<Function>) -> RevisionInfo ![] { return value.revision; }
public flow agent_revision<Actor>(value: AgentDef<Actor>) -> RevisionInfo ![] { return value.revision; }
public flow preserve_type(value: TypeInfo) -> TypeInfo ![] { return value; }
public flow preserve_callable(value: CallableId) -> CallableId ![] { return value; }
public flow revision_key(value: RevisionInfo) -> string ![] { return value.key; }
public flow copy_revision(value: RevisionInfo) -> RevisionInfo ![] {
    return RevisionInfo { key = value.key, content = value.content, parent = value.parent };
}
public flow index_identity<I ~ Index>(index: I) -> I ![] { return index; }
public flow sdk_echo<T>(value: T) -> T { return value; }
public type Parcel<T> = { value: T };
public spec Reader { flow read(self) -> string ![]; }
public type Constructed<T> = { value: T };
impl Constructed<string> ~ Reader {
    flow read(self) -> string ![] { return self.value; }
}
impl Parcel<string> ~ Reader {
    flow read(self) -> string ![] { return self.value; }
}
public flow via_reader<T ~ Reader>(value: T) -> string ![] { return value.read(); }
public flow reader<T ~ Reader>(value: T) -> () -> string ![] {
    return () => { return value.read(); };
}
public spec Transformer { flow apply<T>(self, value: T) -> T ![]; }
public flow transform<R ~ Transformer, T>(receiver: R, value: T) -> T ![] {
    return receiver.apply<T>(value);
}
type Hidden<T> = { value: T };
public flow roundtrip<R ~ Transformer>(receiver: R) -> string ![] {
    let value = transform(receiver, Hidden { value = "hidden" });
    return value.value;
}
public spec Loader { flow load(self) -> bytes ![Error<IOError>]; }
public flow load_through<L ~ Loader>(loader: L) -> bytes ![Error<IOError>] {
    return loader.load();
}
public enum Reply<T> { Empty, Item(T), Named { value: T }, }
public flow pack_reply(value: string) -> Reply<string> {
    let first: Reply<string> = Item(value);
    return match first { Reply.Item(text) => Named { value = text }, _ => Empty };
}
public flow pack(value: string) -> Parcel<string> { return Parcel { value = value }; }
public flow read_reply(value: Reply<string>) -> string {
    return match value { Reply.Empty => "", Reply.Item(text) => text, Reply.Named { value } => value };
}
"#,
    )
    .unwrap();
    fs::write(
        source.join("library/std/extra/protected.es"),
        r#"module std.extra;
import std.fs.{Region, WorkspacePath, IOError};
public type Sandbox;
impl Sandbox ~ Region;
public flow load_checked(target: WorkspacePath<Sandbox>) -> Result<bytes, IOError> ![] {
    let CatchIo: ![Error<IOError> => [] for Result<bytes, IOError>] = handler {
        Error<IOError>.raise(error) => { finish Err(error); }
    };
    return handle { Ok((perform std.effects.Fs.read<Sandbox>(target, None)).unwrap()) } with CatchIo;
}
"#,
    )
    .unwrap();
    let prelude = source.join("library/std/prelude/mod.es");
    fs::write(
        &prelude,
        fs::read_to_string(&prelude).unwrap()
            + "\npublic import std.extra.sdk_echo as bundled_echo;\n",
    )
    .unwrap();
    let app = root.join("app.es");
    fs::write(
        &app,
        r#"module app.main;
import std.text.{join, trim, uppercase as case_text};
import std.text.{TextError, ParseFailure, NumericOverflow};
import std.extra.{sdk_echo, Parcel, Constructed, Reply, pack, pack_reply, read_reply, Reader, via_reader, reader, Transformer, transform, roundtrip};
import std.task.{run, Scheduler, Async};
import std.extra.index_identity;
type CallerParcel<T> = { value: T };
impl CallerParcel<T> ~ Reader {
    flow read(self) -> string ![] { return "caller"; }
}
type CallerTransformer<A> = { active: A };
type PrivateBox<T> = { value: T };
impl CallerTransformer<A> ~ Transformer {
    flow apply<U>(self, value: U) -> U ![] {
        let box: PrivateBox<U> = PrivateBox { value = sdk_echo(value) };
        return sdk_echo(box.value);
    }
}
flow scheduled(scheduler: Scheduler) -> i32 ![] {
    let Cooperative = handler { Async.yield() => { resume (); } };
    return handle { perform Async.yield(); 7 } with Cooperative;
}
effect Wrapped { action read() -> i32; }
flow deferred() -> i32 ![Wrapped] { return perform Wrapped.read(); }
flow increment(value: i32) -> i32 ![] { return value + 1; }
flow text_policy_checks() -> i32 ![] {
    let text = "aé界😀a";
    if std.text.len(text) != 11 || std.text.char_count(text) != 5 { return 1; }
    if !std.text.is_ascii("Etas\n") || std.text.is_ascii(text) { return 2; }
    let ordinal: u8 = 3;
    if std.text.char_get(text, ordinal) != Some('😀') { return 3; }
    let boundary: u16 = 6;
    if !std.text.is_char_boundary(text, boundary) || std.text.is_char_boundary(text, 4) { return 4; }
    if std.text.find(text, "界😀") != Some(3) || std.text.rfind(text, "a") != Some(10) { return 5; }
    if std.text.find(text, "") != Some(0) || std.text.rfind(text, "") != Some(11) { return 6; }
    if std.text.count("aaaaa", "aa") != 2 || std.text.count("é😀", "") != 3 { return 7; }
    if std.text.split("a界b界", "界") != ["a", "b", ""] { return 8; }
    if std.text.splitn("a界b界c", "界", 2) != ["a", "b界c"] { return 9; }
    if std.text.split("é😀", "") != ["", "é", "😀", ""] { return 10; }
    if std.text.splitn("é😀界", "", 3) != ["", "é", "😀界"] { return 11; }
    if std.text.split_once("a界b界c", "界") != Some(("a", "b界c")) { return 12; }
    if std.text.rsplit_once("a界b界c", "界") != Some(("a界b", "c")) { return 13; }
    if std.text.split_once("é😀", "") != Some(("", "é😀")) || std.text.rsplit_once("é😀", "") != Some(("é😀", "")) { return 14; }
    if std.text.replace("a界a界", "界", "😀") != "a😀a😀" { return 15; }
    if std.text.replace_n("a界a界", "界", "😀", 1) != "a😀a界" { return 16; }
    if std.text.replace("é😀", "", "|") != "|é|😀|" || std.text.replace_n("é😀", "", "|", 2) != "|é|😀" { return 17; }
    if std.text.ascii_lowercase("ÉAZ界") != "Éaz界" || std.text.ascii_uppercase("éaz界") != "éAZ界" { return 18; }
    if !std.text.eq_ignore_ascii_case("Etas界", "eTAS界") || std.text.eq_ignore_ascii_case("É", "é") { return 19; }
    if [1; 2; 3].get(ordinal) != None || [1; 2; 3].get(1) != Some(2) { return 20; }
    return 0;
}

flow option_checks() -> bool ![] {
    let callable = Some(increment).unwrap();
    if callable(4) != 5 { return false; }
    let Handle: ![Wrapped => [] for i32] = handler {
        Wrapped.read() => { resume 13; }
    };
    let answer = handle { Some(deferred).unwrap()() } with Handle;
    if answer != 13 { return false; }
    let some: Option<i32> = Some(7);
    let none: Option<i32> = None;
    let empty: Option<i32> = None<i32>();
    let variant = Option.Some;
    let copied: Option<i32> = variant(9);
    let constructor = Some;
    let wrapped: Option<i32> = constructor(11);
    let payload = match copied { Option.Some(value) => value, Option.None => 0 };
    let old = match wrapped { Some(value) => value, None => 0 };
    return some.is_some() && !some.is_none() && none.is_none() && empty.is_none()
        && some.unwrap() == 7 && payload == 9 && old == 11;
}
flow collection_checks() -> bool ![] {
    let values = [1, 2];
    let (array, last) = values.pop();
    if array != [1] || last != Some(2) || values != [1, 2] { return false; }
    let index: u8 = 1;
    if values.get(index) != Some(2) || values.push(3) != [1, 2, 3] { return false; }
    if values.extend([3]) != [1, 2, 3] { return false; }
    let linked = [1; 2];
    let (list, first) = linked.pop();
    if list != [2;] || first != Some(1) || linked != [1; 2] { return false; }
    if linked.head() != Some(1) || linked.tail() != Some([2;]) { return false; }
    if linked.push(0).extend([3; 4]) != [3; 4; 0; 1; 2] { return false; }
    let empty: List<i32> = [];
    if empty.pop() != (empty, None) || empty.tail() != None { return false; }
    if [4].pop() != ([], Some(4)) { return false; }
    if [5;].head() != Some(5) { return false; }
    let Handle: ![Wrapped => [] for i32] = handler {
        Wrapped.read() => { resume 13; }
    };
    let callbacks = [deferred;].extend([deferred;]);
    let head_answer = handle { callbacks.head().unwrap()() } with Handle;
    if head_answer != 13 { return false; }
    let list_index: u8 = 1;
    let list_answer = handle { callbacks.get(list_index).unwrap()() } with Handle;
    if list_answer != 13 { return false; }
    let array_callbacks = [deferred, deferred];
    let (earlier, final_callback) = array_callbacks.pop();
    let final_answer = handle { final_callback.unwrap()() } with Handle;
    if final_answer != 13 { return false; }
    let position: usize = 0;
    let earlier_answer = handle { earlier.get(position).unwrap()() } with Handle;
    if earlier_answer != 13 { return false; }
    std.core.assert(true);
    if std.text.char_count("Á😀") != 3 || !std.text.is_empty("") { return false; }
    if std.text.trim_start("　 a 　") != "a 　" { return false; }
    if std.text.trim_end("　 a 　") != "　 a" { return false; }
    return true;
}
flow runtime_limit_checks() -> bool ![] {
    let count = match std.extra.limited_value(3) { Limit.Iterations(value) => value, _ => { return false; } };
    if count != 3 || Tokens(2) != Limit.Tokens(2) || Attempts(2) != Limit.Attempts(2) { return false; }
    if ContextTokens(4) != Limit.ContextTokens(4) || Concurrency(2) != Limit.Concurrency(2) { return false; }
    let cost = Cost(std.extra.money_value(1.25));
    let money = match cost { Limit.Cost(value) => value, _ => { return false; } };
    let again = Cost(money);
    var total = 0;
    for value in [1, 2, 3] { total = total + value; }
    let custom = match Limit.SdkOnly { Limit.SdkOnly => true, _ => false };
    return total == 6 && custom && match again { Limit.Cost(_) => true, _ => false };
}
flow main(args: Array<string>) -> i32 {
    if !runtime_limit_checks() { return 89; }
    if std.extra.primitive_value(42) != 42 { return 88; }
    if std.extra.delivery_value() != 41 { return 87; }
    if !collection_checks() { return 86; }
    let policy = text_policy_checks();
    if policy != 0 { return 100 + policy; }
    if !option_checks() { return 76; }
    let custom: TextError = TextError.SdkOnly;
    if match custom { TextError.SdkOnly => false, _ => true } { return 57; }
    let parsed: Result<usize, TextError> = std.text.parse_usize("42");
    if parsed != Ok(42) { return 58; }
    if std.text.parse_usize("words") != Err(ParseFailure) { return 59; }
    if std.text.parse_bool("TRUE") != Err(ParseFailure) { return 54; }
    if std.text.parse_usize("999999999999999999999999999999999999999") != Err(NumericOverflow) { return 60; }
    if std.text.parse_bool(std.text.from_bool(true)) != Ok(true) { return 61; }
    if std.text.parse_bool(std.text.from_bool(false)) != Ok(false) { return 62; }
    if !std.text.starts_with("世界😀", "世界") { return 63; }
    if !std.text.ends_with("世界😀", "😀") { return 64; }
    if !std.text.contains("aba世界😀", "世界") { return 65; }
    if std.text.strip_prefix("世界😀", "世界") != Some("😀") { return 66; }
    if std.text.strip_suffix("世界😀", "😀") != Some("世界") { return 67; }
    if [1, 2].extend([3, 4]) != [1, 2, 3, 4] { return 71; }
    if [1; 2].push(0) != [0; 1; 2] { return 72; }
    if [1; 2].extend([3; 4]) != [3; 4; 1; 2] { return 73; }
    if trim("　 a \t b \n　") != "a \t b" || trim("　 \n\t") != "" { return 74; }
    let good: Result<i32, string> = Result.Ok(7);
    let bad: Result<i32, string> = Err("failed");
    if !good.is_ok() || good.is_err() || !bad.is_err() { return 79; }
    if !good.is_ok() || good.is_err() || !bad.is_err() || good.unwrap() != 7 { return 77; }
    let constructor = Ok;
    let copied: Result<i32, string> = constructor(9);
    let payload = match copied { Result.Ok(value) => value, Result.Err(_) => 0 };
    if payload != 9 || index_identity<i32>(7) != 7 { return 78; }
    let caller = sdk_echo(CallerParcel { value = "caller" });
    if caller.value != "caller" { return 92; }
    if via_reader(caller) != "caller" { return 83; }
    if reader(caller)() != "caller" { return 87; }
    let transformer = CallerTransformer { active = true };
    if transform(transformer, 27) != 27 { return 84; }
    if transform(transformer, "caller") != "caller" { return 85; }
    if roundtrip(transformer) != "hidden" { return 86; }
    let outcome = run(scheduled);
    if outcome != 7 { return 94; }
    if run(scheduled) != 7 { return 93; }
    let assemble = join;
    let strip = trim;
    let values = [strip(" a ")].push(strip(" b "));
    let parcel: Parcel<string> = sdk_echo(pack(bundled_echo(assemble(values, "|"))));
    let text = read_reply(pack_reply(via_reader(parcel)));
    let constructed: Constructed<string> = Constructed { value = text };
    if constructed.read() != text { return 95; }
    if parcel.read() != text { return 96; }
    if read_reply(Reply.Item(text)) != text { return 98; }
    if read_reply(Reply.Empty) != "" { return 97; }
    if text == "a|b" {
        if case_text("x") == "x" { return 3; }
        return 0;
    }
    if text == "rebuilt:a|b" { return 2; }
    return 99;
}
"#,
    )
    .unwrap();
    let sdk = root.join("sdk");
    let invoke = |args: &[&str]| {
        let output = Command::new(&binary)
            .args(args)
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let build = || {
        invoke(&[
            "--cache",
            "off",
            "std",
            "build",
            "--manifest-path",
            path(&source.join("etas.toml")),
            "--out-dir",
            path(&sdk.join("std")),
        ]);
    };
    let cache = root.join("cache");
    let execute = || {
        serde_json::from_str::<serde_json::Value>(&invoke(&[
            "--cache",
            "read-write",
            "--cache-root",
            path(&cache),
            "run",
            path(&app),
            "--std-provider",
            "artifact",
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]))
        .unwrap()
    };
    build();
    let methods_only = root.join("methods-only.es");
    fs::write(
        &methods_only,
        r#"module app.main;
effect Wrapped { action read() -> i32; }
flow deferred() -> i32 ![Wrapped] { return perform Wrapped.read(); }
flow main(args: Array<string>) -> i32 {
    let Handle: ![Wrapped => [] for i32] = handler {
        Wrapped.read() => { resume 13; }
    };
    let value = Option.Some(deferred);
    let answer = handle { value.unwrap()() } with Handle;
    if answer == 13 { return 0; }
    return 1;
}
"#,
    )
    .unwrap();
    let methods_report: serde_json::Value = serde_json::from_str(&invoke(&[
        "--cache",
        "off",
        "run",
        path(&methods_only),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
        "--format",
        "json",
    ]))
    .unwrap();
    assert_eq!(methods_report["value"]["value"], "0");
    let before = execute();
    assert_eq!(before["value"]["value"], "0");
    let first_manifest = fs::read(sdk.join("std/manifest.json")).unwrap();
    let sdk_manifest: serde_json::Value = serde_json::from_slice(&first_manifest).unwrap();
    build();
    assert_eq!(
        first_manifest,
        fs::read(sdk.join("std/manifest.json")).unwrap()
    );
    // Package preparation and projection must use the selected SDK as well.
    let packaged = root.join("package-app");
    fs::create_dir_all(packaged.join("src/app")).unwrap();
    fs::write(
        packaged.join("etas.toml"),
        r#"[package]
name = "source-sdk-consumer"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = "0.2" }
bridge = { path = "../bridge" }
[[bin]]
name = "source-sdk-consumer"
module = "app.main"
flow = "main"
"#
        .replace(
            "std = { version = \"0.2\" }",
            &format!(
                "std = {{ version = \"={}\" }}",
                sdk_manifest["version"].as_str().unwrap(),
            ),
        ),
    )
    .unwrap();
    let bridge = root.join("bridge");
    fs::create_dir_all(bridge.join("src")).unwrap();
    fs::write(
        bridge.join("etas.toml"),
        r#"[package]
name = "source-sdk-bridge"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[dependencies]
std = { version = ">=0.2" }
"#,
    )
    .unwrap();
    fs::write(
        bridge.join("src/bridge.es"),
        r#"module bridge;
public flow bridge_summary(value: string) -> Result<string, string> ![] { return Ok(value); }
"#,
    )
    .unwrap();
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: bridge.clone(),
    })
    .unwrap();
    let old_lock = fs::read(bridge.join("etas.lock")).unwrap();
    let needs_upgrade = Command::new(&binary)
        .args([
            "pkg",
            "metadata",
            path(&bridge),
            "--std-provider",
            "artifact",
            "--sysroot",
            path(&sdk),
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!needs_upgrade.status.success());
    assert!(
        String::from_utf8_lossy(&needs_upgrade.stderr)
            .contains("explicitly upgrade the builtin std lock")
    );
    assert_eq!(fs::read(bridge.join("etas.lock")).unwrap(), old_lock);
    invoke(&[
        "pkg",
        "lock",
        path(&bridge),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
    ]);
    let upgraded = etas_package::lockfile::read_lockfile(&bridge)
        .unwrap()
        .unwrap();
    let standard = upgraded
        .packages
        .iter()
        .find(|package| package.name == "std")
        .unwrap();
    assert_eq!(standard.version, sdk_manifest["version"].as_str().unwrap());
    assert!(
        standard
            .checksum
            .contains(sdk_manifest["native_binding_abi"].as_str().unwrap())
    );
    invoke(&[
        "pkg",
        "metadata",
        path(&bridge),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
    ]);
    let package_source = fs::read_to_string(&app).unwrap()
        .replace("type CallerParcel<T>", "import bridge.bridge_summary;\ntype CallerParcel<T>")
        .replace("    let caller = sdk_echo", "    let bridged = bridge_summary(\"package\");\n    if !bridged.is_ok() { return 76; }\n    let caller = sdk_echo");
    fs::write(packaged.join("src/app/main.es"), package_source).unwrap();
    invoke(&[
        "pkg",
        "update",
        path(&packaged),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
    ]);
    invoke(&[
        "check",
        path(&packaged),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
    ]);
    let package_run: serde_json::Value = serde_json::from_str(&invoke(&[
        "run",
        path(&packaged),
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
        "--format",
        "json",
    ]))
    .unwrap();
    assert_eq!(package_run["value"]["value"], "0");
    let assembly = source.join("library/std/text/assembly.es");
    rewrite_join_result(&assembly, |result| format!("\"rebuilt:\" + ({result})"));
    invoke(&[
        "--cache",
        "off",
        "--std-provider",
        "artifact",
        "--sysroot",
        path(&sdk),
        "std",
        "build",
        "--manifest-path",
        path(&source.join("etas.toml")),
        "--out-dir",
        path(&sdk.join("std")),
    ]);
    fs::remove_dir_all(source).unwrap();
    let after = execute();
    assert_eq!(after["value"]["value"], "2");
    // Install the actual CLI next to the SDK, then move the whole toolchain.
    // The new process discovers std from its executable, outside the repository.
    let prefix = root.join("prefix");
    invoke(&[
        "--cache",
        "off",
        "std",
        "install",
        "--manifest-path",
        path(&sdk.join("std/etas.toml")),
        "--prefix",
        path(&prefix),
    ]);
    let moved = root.join("moved-prefix");
    fs::rename(prefix, &moved).unwrap();
    for command in ["check", "run"] {
        let output = Command::new(moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)))
            .args([
                "--cache",
                "off",
                command,
                path(&app),
                "--std-provider",
                "artifact",
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "run" {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["value"]["value"], "2");
        }
    }
    let protected = package_fixture(
        "std-sdk-protected-read",
        &[
            (
                "etas.toml",
                r#"[package]
name = "std-sdk-protected-read"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "std-sdk-protected-read"
module = "app.main"
flow = "main"
profile = "readonly"
[runtime.profiles.readonly.filesystem.regions."std.extra.Sandbox"]
root = "."
read = true
[runtime.profiles.readonly.filesystem.regions."app.main.CallerSandbox"]
root = "."
read = true
"#,
            ),
            ("data.txt", "sdk read"),
            (
                "src/app/main.es",
                r#"module app.main;
import std.extra.{Sandbox, load_checked, pack, Reader, via_reader, Reply, read_reply, Loader, load_through};
import std.fs.{Region, IOError, WorkspacePath, path, read_bytes};
import std.codec.text.utf8_encode;
type CallerSandbox;
impl CallerSandbox ~ Region;
type File = { target: WorkspacePath<CallerSandbox> };
impl File ~ Loader {
    flow load(self) -> bytes ![Error<IOError>] { return read_bytes(self.target); }
}
flow main(args: Array<string>) -> i32 {
    let Catch: ![Error<IOError> => [] for i32] = handler {
        Error<IOError>.raise(error) => { finish 7; }
    };
    let caller_target = match path<CallerSandbox>("data.txt") {
        Ok(value) => value, Err(_) => { return 8; }
    };
    let caller_loaded = handle {
        if load_through(File { target = caller_target }) == utf8_encode("sdk read") { 0 } else { 9 }
    } with Catch;
    if caller_loaded != 0 { return caller_loaded; }
    let caller_absent = match path<CallerSandbox>("missing.txt") {
        Ok(value) => value, Err(_) => { return 8; }
    };
    let caller_missing = handle { load_through(File { target = caller_absent }); 99 } with Catch;
    if caller_missing != 7 { return 10; }
    let parcel = pack("sdk read");
    if parcel.read() != via_reader(parcel) { return 4; }
    let expected = read_reply(Reply.Named { value = parcel.read() });
    let target = match path<Sandbox>("data.txt") {
        Ok(value) => value, Err(_) => { return 2; }
    };
    let loaded = match load_checked(target) {
        Ok(value) => value, Err(_) => { return 3; }
    };
    if loaded != utf8_encode(expected) { return 5; }
    let missing = match path<Sandbox>("missing.txt") {
        Ok(value) => value, Err(_) => { return 2; }
    };
    return match load_checked(missing) { Err(_) => 0, Ok(_) => 6 };
}
"#,
            ),
        ],
    );
    let output = Command::new(moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)))
        .args([
            "--cache",
            "off",
            "run",
            path(&protected),
            "--std-provider",
            "artifact",
            "--format",
            "json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["value"]["value"], "0", "{report}");
    assert_eq!(
        report["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["kind"] == "host_request_started"
                && event["qualified_action"] == "Fs.read")
            .count(),
        4,
        "{report}"
    );
    verify_source_filesystem_sdk(&moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)));
    verify_source_checkpoint_sdk(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_source_action_sdk(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_task_provider_parity(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_caller_task_handler(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_source_console_sdk(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_message_json_map(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    verify_source_codec_sdk(&moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)));
    verify_source_compiler_report_sdk(
        &moved.join(format!("bin/etas{}", std::env::consts::EXE_SUFFIX)),
        &root,
    );
    fs::remove_dir_all(protected).unwrap();
    // The exact same application runs in two independent provider sessions.
    let common = root.join("common.es");
    fs::write(
        &common,
        r#"module app.main;
import std.text.join;
flow main(args: Array<string>) -> i32 {
    let text = join(["a", "b"], "|");
    if text == "a|b" { return 0; }
    if text == "rebuilt:a|b" { return 2; }
    return 99;
}"#,
    )
    .unwrap();
    // The acceptance toolchain compiles the complete legacy factory out.
    // Ordinary builds additionally retain the independent legacy comparison.
    let providers = if cfg!(feature = "forbid-legacy-std") {
        &[("artifact", "2")][..]
    } else {
        &[("artifact", "2"), ("legacy", "0")][..]
    };
    for &(provider, expected) in providers {
        let output = invoke(&[
            "--cache",
            "off",
            "run",
            path(&common),
            "--std-provider",
            provider,
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]);
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["value"]["value"], expected, "{provider}");
    }
    // The same callback crosses source std, requests a protected host action,
    // and raises a typed error back to a handler owned by its caller.
    let task_app = package_fixture(
        "std-sdk-task-callback",
        &[
            (
                "etas.toml",
                r#"[package]
name = "task-run-parity"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "parity"
module = "app"
flow = "main"
profile = "read"
[runtime.profiles.read.filesystem.regions."app.Sandbox"]
root = "."
read = true
"#,
            ),
            ("data.txt", "task read"),
            (
                "src/app.es",
                r#"module app;
import std.task.{run, Scheduler};
import std.fs.{Region, IOError, path, read_bytes};
import std.codec.text.utf8_encode;
type Sandbox;
impl Sandbox ~ Region;
flow job(scheduler: Scheduler) -> bytes ![Error<IOError>] {
    var total = 0;
    for value in [1, 2, 3] { total = total + value; }
    let target = match path<Sandbox>("data.txt") {
        Ok(value) => value,
        Err(error) => { return perform Error<IOError>.raise(error); }
    };
    return read_bytes(target);
}
flow missing(scheduler: Scheduler) -> bytes ![Error<IOError>] {
    let target = match path<Sandbox>("missing.txt") {
        Ok(value) => value,
        Err(error) => { return perform Error<IOError>.raise(error); }
    };
    return read_bytes(target);
}
flow main(args: Array<string>) -> i32 {
    let Catch: ![Error<IOError> => [] for i32] = handler {
        Error<IOError>.raise(error) => { finish 2; }
    };
    let first = handle {
        let data = run(job);
        if data == utf8_encode("task read") { 0 } else { 3 }
    } with Catch;
    if first != 0 { return first; }
    let Missing: ![Error<IOError> => [] for i32] = handler {
        Error<IOError>.raise(error) => { finish 7; }
    };
    let absent = handle { run(missing); 99 } with Missing;
    if absent == 7 { return 0; }
    return 4;
}
"#,
            ),
        ],
    );
    let mut reference_effects = None;
    for &(provider, _) in providers {
        let output = invoke(&[
            "--cache",
            "off",
            "effects",
            path(&task_app),
            "--std-provider",
            provider,
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]);
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        let effects = report["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                serde_json::json!({
                    "name": item["name"],
                    "requested": item["requested_actions"],
                    "escaping": item["escaping_effects"],
                    "determinism": item["determinism"],
                    "support": item["support"],
                })
            })
            .collect::<Vec<_>>();
        let main = effects.iter().find(|item| item["name"] == "main").unwrap();
        assert_eq!(main["requested"], "[Fs.read[app.Sandbox]]", "{provider}");
        assert_eq!(main["escaping"], "[]", "{provider}");
        if let Some(reference) = &reference_effects {
            assert_eq!(&effects, reference, "{provider}");
        } else {
            reference_effects = Some(effects);
        }
        let output = invoke(&[
            "--cache",
            "off",
            "run",
            path(&task_app),
            "--std-provider",
            provider,
            "--sysroot",
            path(&sdk),
            "--format",
            "json",
        ]);
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(report["value"]["value"], "0", "{provider}: {report}");
        assert_eq!(
            report["events"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| {
                    event["kind"] == "host_request_started"
                        && event["qualified_action"] == "Fs.read"
                })
                .count(),
            2,
            "{provider}: {report}"
        );
    }
    fs::remove_dir_all(task_app).unwrap();
    fs::remove_dir_all(root).unwrap();
}

fn verify_source_checkpoint_sdk(binary: &Path, root: &Path) {
    let app = root.join("checkpoint-app.es");
    let checkpoints = root.join("std-checkpoints");
    let keys = root.join("checkpoint-keys");
    fs::write(
        &app,
        r#"module app.main;
import std.runtime.{checkpointed, checkpoint_apply};
import std.runtime.checkpoint.checkpoint as save;
flow main(args: Array<string>) -> i32 {
    let Complete: ![Error<i32> => [] for i32] = handler {
        Error<i32>.raise(value) => { finish value; }
    };
    return handle {
    let (first, money) = checkpointed((40, std.runtime.budget.usd(1.25)));
    let retained = match Cost(money) { Limit.Cost(_) => true, _ => false };
    if !retained { perform Error<i32>.raise(90); }
    let add = (value: i32) => value + 1;
    let result = checkpoint_apply(add, first);
    save("app-alias");
    perform Error<i32>.raise(result)
    } with Complete;
}
"#,
    )
    .unwrap();
    let execute = |resume: Option<usize>| {
        let mut command = Command::new(binary);
        command.args(["--cache", "off"]);
        if let Some(index) = resume {
            command.args(["resume", &index.to_string()]);
        } else {
            command.args(["run", path(&app)]);
        }
        let output = command
            .args([
                "--std-provider",
                "artifact",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "42", "{report}");
        assert!(
            report["diagnostics"].as_array().unwrap().is_empty(),
            "{report}"
        );
        report["checkpoints"].as_array().unwrap().len()
    };
    assert_eq!(execute(None), 3);
    for index in 0..3 {
        assert_eq!(execute(Some(index)), 2 - index);
    }
}

fn verify_source_compiler_report_sdk(binary: &Path, root: &Path) {
    let application = root.join("compiler-reports.es");
    fs::write(&application, r#"module compiler_reports;
import std.compiler.{RevisionInfo, parse_body, format};
import std.extra.{revision_key, copy_revision, preserve_type, preserve_callable, syntax_text, callable_revision, agent_revision};
flow original(value: string) -> string ![] { return value; }
agent Worker = {};
impl Worker {
    public flow run(self) -> string { return "worker"; }
}
flow main(args: Array<string>) -> i32 {
    let definition = ^original;
    let revision: RevisionInfo = definition.revision;
    let copied = copy_revision(revision);
    if revision_key(copied) != revision.key || copied.content != revision.content || copied.parent != revision.parent { return 1; }
    if copied.key == "" || copied.content == "" { return 2; }
    var parameters: usize = 0;
    for parameter in definition.signature.parameters {
        if parameter.name != Some("value") { return 3; }
        parameters = parameters + 1;
    }
    if parameters != 1 { return 4; }
    let syntax = match parse_body("return 1;") { Ok(body) => body, Err(_) => { return 5; } };
    if format(syntax) == "" { return 6; }
    let output = definition.signature.output;
    if preserve_type(output) != output { return 7; }
    let identity = definition.id;
    if preserve_callable(identity) != identity { return 8; }
    if syntax_text(syntax) != syntax.text { return 9; }
    if callable_revision(definition).key != revision.key { return 10; }
    if agent_revision(^Worker).key != (^Worker).revision.key { return 11; }
    return 0;
}
"#).unwrap();
    let output = Command::new(binary)
        .args([
            "--cache",
            "off",
            "run",
            path(&application),
            "--std-provider",
            "artifact",
            "--format",
            "json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["value"]["value"], "0", "{report}");
}

fn verify_message_json_map(binary: &Path, root: &Path) {
    let app = root.join("message-json.es");
    fs::write(&app, r#"module app.main;
import std.agent.message.Message;
import std.json.{JsonValue, stringify};
type Item = { text: string, count: i64 };
flow main(args: Array<string>) -> i32 {
    let message = Message.new(Item { text = "sample", count = 7 }).cast<JsonValue>().unwrap();
    let fields = match message.body { JsonValue.Object(fields) => fields, _ => { return 1; } };
    if fields.get("text") != Some(JsonValue.String("sample")) || !fields.contains_key("count") { return 3; }
    let updated = fields.insert("copy", fields.get("text").unwrap()).remove("count");
    if stringify(JsonValue.Object(updated)) == Ok("{\"copy\":\"sample\",\"text\":\"sample\"}") { return 0; }
    return 2;
}
"#).unwrap();
    let output = Command::new(binary)
        .args([
            "--cache",
            "off",
            "run",
            path(&app),
            "--std-provider",
            "artifact",
            "--format",
            "json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["value"]["value"], "0", "{report}");
}

fn verify_source_codec_sdk(binary: &Path) {
    let project = package_fixture(
        "std-source-codec",
        &[
            (
                "etas.toml",
                r#"[package]
name = "std-source-codec"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
profile = "files"
[runtime.profiles.files.filesystem.regions."app.main.Corpus"]
root = "workspace"
read = true
"#,
            ),
            ("workspace/malformed.bin", ""),
            (
                "src/app/main.es",
                r#"module app.main;
import std.fs.{Region, path, read_bytes};
import std.codec.text.{utf8_encode, utf8_decode, Strict, Replace, MalformedInput, TextCodecError, InvalidUtf8};
import std.bytes.len as byte_length;
type Corpus; impl Corpus ~ Region;
flow main(args: Array<string>) -> i32 {
    let text = "A界😀";
    let encoded = utf8_encode(text);
    if byte_length(encoded) != 8 { return 1; }
    let mode: MalformedInput = MalformedInput.Strict;
    if utf8_decode(encoded, mode) != Ok(text) || utf8_decode(encoded, Replace) != Ok(text) { return 2; }
    // CHARACTER_CHECKS
    let file = match path<Corpus>("malformed.bin") { Ok(value) => value, Err(_) => { return 6; } };
    let malformed = read_bytes(file);
    let failure: TextCodecError = TextCodecError.InvalidUtf8;
    if utf8_decode(malformed, Strict) != Err(failure) || failure != InvalidUtf8 { return 7; }
    if utf8_decode(malformed, Replace) != Ok("a�b") { return 8; }
    if std.text.lowercase("ÉAZ界") != "éaz界" || std.text.uppercase("Straße") != "STRASSE" { return 9; }
    return 0;
}
"#,
            ),
        ],
    );
    fs::write(project.join("workspace/malformed.bin"), [b'a', 0xff, b'b']).unwrap();
    let application = project.join("src/app/main.es");
    let template = fs::read_to_string(&application).unwrap();
    let providers = if cfg!(feature = "forbid-legacy-std") {
        vec!["artifact"]
    } else {
        vec!["legacy", "artifact"]
    };
    for provider in providers {
        let character_checks = match provider {
            "legacy" => {
                r#"
    if !std.char.is_alphabetic('界') || !std.char.is_alphanumeric('٣') || !std.char.is_ascii_digit('7') { return 3; }
    if !std.char.is_control('\n') || !std.char.is_control(std.char.from_u32(133).unwrap()) { return 4; }
    if std.char.to_u32('😀') != 128512 || std.char.from_u32(128512) != Some('😀') { return 5; }"#
            }
            _ => {
                r#"
    if !'界'.is_alphabetic() || !'٣'.is_alphanumeric() || !'7'.is_ascii_digit() { return 3; }
    if !'\n'.is_control() || !char.from_u32(133).unwrap().is_control() { return 4; }
    if '😀'.to_u32() != 128512 || char.from_u32(128512) != Some('😀') { return 5; }"#
            }
        };
        fs::write(
            &application,
            template.replace("// CHARACTER_CHECKS", character_checks),
        )
        .unwrap();
        let output = Command::new(binary)
            .args([
                "--cache",
                "off",
                "run",
                path(&project),
                "--std-provider",
                provider,
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{provider}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "0", "{provider}: {report}");
        assert_eq!(
            report["events"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["kind"] == "host_request_started"
                    && event["qualified_action"] == "Fs.read")
                .count(),
            1,
            "{provider}: {report}"
        );
    }
}

fn verify_source_filesystem_sdk(binary: &Path) {
    let project = package_fixture(
        "std-source-filesystem",
        &[
            (
                "etas.toml",
                r#"[package]
name = "std-source-filesystem"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
profile = "files"
[runtime.profiles.files.filesystem.regions."app.main.Project"]
root = "workspace"
read = true
write = true
[runtime.profiles.readonly.filesystem.regions."app.main.Project"]
root = "workspace"
read = true
"#,
            ),
            ("workspace/item.txt", "source fs"),
            ("workspace/sub/other.txt", "more"),
            (
                "src/app/main.es",
                r#"module app.main;
import std.fs.{Region, WorkspacePath, IOError, FsStat, ReadLimitError,
    root, relative_name, path, read_bytes, read_bytes_limited, write_bytes, list, list_limited, stat, atomic_replace};
import std.codec.text.utf8_encode;
type Project; impl Project ~ Region;
flow selected<R ~ Region>() -> WorkspacePath<R> ![] { return root<R>(); }
flow denied(args: Array<string>) -> i32 {
    let target = match path<Project>("created.txt") { Ok(value) => value, Err(_) => { return 1; } };
    write_bytes(target, utf8_encode("forbidden"));
    return 0;
}
flow denied_retry(args: Array<string>) -> i32 {
    let target = match path<Project>("created.txt") { Ok(value) => value, Err(_) => { return 1; } };
    retry { write_bytes(target, utf8_encode("forbidden")); }
    return 0;
}
flow main(args: Array<string>) -> i32 {
    let directory: WorkspacePath<Project> = selected();
    if relative_name(directory) != "." { return 1; }
    let item = match path<Project>("item.txt") { Ok(value) => value, Err(_) => { return 2; } };
    let metadata: FsStat = stat(item);
    if !metadata.is_file || metadata.is_dir || metadata.len != 9 { return 3; }
    if read_bytes_limited(item, 9) != utf8_encode("source fs") { return 4; }
    var names = "";
    for entry in list(directory) { names = names + relative_name(entry) + ";"; }
    if names != "item.txt;sub;" { return 5; }
    var limited_names = "";
    for entry in list_limited(directory, 2, 11) { limited_names = limited_names + relative_name(entry) + ";"; }
    if limited_names != names { return 6; }
    let exceeded: ![Error<IOError> => [] for bool] = handler {
        Error<IOError>.raise(IOError.LimitExceeded) => { finish true; }
    };
    let read_limit = handle { read_bytes_limited(item, 8); false } with exceeded;
    if !read_limit { return 7; }
    let list_limit = handle { list_limited(directory, 1, 11); false } with exceeded;
    if !list_limit { return 8; }
    let custom = match ReadLimitError.SdkOnly { ReadLimitError.SdkOnly => true, _ => false };
    if !custom { return 11; }
    let destination = match path<Project>("created.txt") { Ok(value) => value, Err(_) => { return 9; } };
    write_bytes(destination, utf8_encode("before"));
    atomic_replace(destination, utf8_encode("after"));
    if read_bytes(destination) != utf8_encode("after") { return 10; }
    return 0;
}
"#,
            ),
        ],
    );
    let output = Command::new(binary)
        .args([
            "--cache",
            "off",
            "run",
            path(&project),
            "--std-provider",
            "artifact",
            "--format",
            "json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["value"]["value"], "0", "{report}");
    let actions = report["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "host_request_started")
        .map(|event| event["qualified_action"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        [
            "Fs.stat",
            "Fs.read",
            "Fs.list",
            "Fs.list",
            "Fs.read",
            "Fs.list",
            "Fs.write",
            "Fs.atomic_replace",
            "Fs.read"
        ],
        "{report}"
    );
    assert_eq!(
        fs::read(project.join("workspace/created.txt")).unwrap(),
        b"after"
    );
    let denied = Command::new(binary)
        .args([
            "--cache",
            "off",
            "run",
            path(&project),
            "--std-provider",
            "artifact",
            "--profile",
            "readonly",
            "--flow",
            "denied",
            "--format",
            "json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(
        denied.status.code(),
        Some(3),
        "{}\n{}",
        String::from_utf8_lossy(&denied.stdout),
        String::from_utf8_lossy(&denied.stderr)
    );
    let denied_report: serde_json::Value = serde_json::from_slice(&denied.stdout).unwrap();
    assert_eq!(
        denied_report["diagnostics"][0]["code"], "Analysis(UnhandledRuntimeError)",
        "{denied_report}"
    );
    assert!(
        denied_report["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("missing its region-scoped action grant"),
        "{denied_report}"
    );
    assert_eq!(
        fs::read(project.join("workspace/created.txt")).unwrap(),
        b"after"
    );
    let providers = if cfg!(feature = "forbid-legacy-std") {
        &["artifact"][..]
    } else {
        &["artifact", "legacy"][..]
    };
    let mut previous = None;
    for provider in providers {
        let output = Command::new(binary)
            .args([
                "--cache",
                "off",
                "run",
                path(&project),
                "--std-provider",
                provider,
                "--profile",
                "readonly",
                "--flow",
                "denied_retry",
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(3),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        // Both entry points must preserve the original region grant denial.
        // Retrying with the same read-only authority cannot admit the write.
        assert_eq!(
            report["diagnostics"], denied_report["diagnostics"],
            "{report}"
        );
        let events = report["events"].as_array().unwrap();
        let count = |kind| events.iter().filter(|event| event["kind"] == kind).count();
        assert_eq!(count("retry_attempt_started"), 1, "{report}");
        assert_eq!(count("host_request_started"), 1, "{report}");
        assert_eq!(count("retry_exhausted"), 0, "{report}");
        let signature = (
            report["diagnostics"].clone(),
            events
                .iter()
                .map(|event| event["kind"].clone())
                .collect::<Vec<_>>(),
        );
        if let Some(previous) = previous {
            assert_eq!(previous, signature);
        }
        previous = Some(signature);
        assert_eq!(
            fs::read(project.join("workspace/created.txt")).unwrap(),
            b"after"
        );
    }
    fs::remove_dir_all(project).unwrap();
}

// The custom action and its forwarding body were compiled into a different
// process's std bundle. Capture while source await is parked and resume using
// a fresh process, preserving its inferred type/row and continuation ownership.
fn verify_source_console_sdk(binary: &Path, root: &Path) {
    let app = root.join("console-source.es");
    let checkpoints = root.join("console-checkpoints");
    let keys = root.join("console-checkpoint-keys");
    fs::write(
        &app,
        r#"module app.main;
import std.effects.Console;
import std.io.{print, println, eprintln, read_line, read_all};
import std.runtime.checkpoint.checkpoint;
import std.task.{Scheduler, run, start, await};
flow main(args: Array<string>) -> i32 ![] {
    var captured = "";
    let answer = handle {
        let write = println;
        write("first");
        run((scheduler: Scheduler) => {
            let child = start(() => { println(read_line()); return 7; });
            return await(child);
        })
    } with {
        Console.stdout_write(text) => {
            captured = captured + text;
            checkpoint("console-handler");
            resume ();
        }
        Console.stdin_read_line() => { resume "second"; }
        Error<IOError>.raise(error) => { finish 91; }
    };
    if answer != 7 || captured != "first\nsecond\n" { return 1; }
    let tail = handle { eprintln("error"); print(read_all()); 0 } with {
        Console.stderr_write(text) => { if text != "error\n" { finish 2; } resume (); }
        Console.stdin_read_all() => { resume "remaining"; }
        Console.stdout_write(text) => { if text != "remaining" { finish 3; } resume (); }
        Error<IOError>.raise(error) => { finish 92; }
    };
    return tail;
}
"#,
    )
    .unwrap();
    for resume in [None, Some(0), Some(1)] {
        let mut command = Command::new(binary);
        command.args(["--cache", "off"]);
        if let Some(index) = resume {
            command.args(["resume", &index.to_string()]);
        } else {
            command.args(["run", path(&app)]);
        }
        let output = command
            .args([
                "--std-provider",
                "artifact",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "0", "{report}");
        assert!(
            report["diagnostics"].as_array().unwrap().is_empty(),
            "{report}"
        );
        assert_eq!(
            report["checkpoints"].as_array().unwrap().len(),
            resume.map_or(2, |index| 1 - index),
            "{report}"
        );
    }
    let project = package_fixture(
        "std-source-console",
        &[
            (
                "etas.toml",
                r#"[package]
name = "std-source-console"
version = "0.1.0"
edition = "2026"
[source]
root = "src"
[[bin]]
name = "app"
module = "app.main"
flow = "main"
profile = "stdio"
[runtime.profiles.stdio.policy]
mode = "local-static"
rules = ["console=allow"]
[runtime.profiles.denied.policy]
mode = "local-static"
rules = ["model=allow"]
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;
import std.effects.Console;
import std.io.{sdk_try_line, read_all, print, println, eprintln};
spec Gate: trace = +Console;
flow main(args: Array<string>) -> i32 ![] ~ Gate {
    let first = match sdk_try_line() { Ok(text) => text, Err(error) => { return 9; } };
    return handle {
        print("<" + first + ">"); println(read_all()); eprintln("done"); 0
    } with { Error<IOError>.raise(error) => { finish 91; } };
}
"#,
            ),
        ],
    );
    for (profile, input, expected) in [
        ("stdio", b"alpha\nbeta".as_slice(), "0"),
        ("stdio", b"\xff\n".as_slice(), "9"),
        ("denied", b"alpha\nbeta".as_slice(), "denied"),
    ] {
        let trace = root.join(format!("console-{expected}.json"));
        let mut child = Command::new(binary)
            .args([
                "--cache",
                "off",
                "run",
                path(&project),
                "--profile",
                profile,
                "--std-provider",
                "artifact",
                "--trace-out",
                path(&trace),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(std::env::temp_dir())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        let report: serde_json::Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
        if expected == "denied" {
            assert_eq!(output.status.code(), Some(3), "{report}");
            assert!(output.stdout.is_empty());
            assert!(
                report["diagnostics"][0]["message"]
                    .as_str()
                    .unwrap()
                    .contains("console policy denied request"),
                "{report}"
            );
        } else {
            assert!(
                output.status.success(),
                "{report}\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(report["value"]["value"], expected, "{report}");
            assert!(
                report["diagnostics"].as_array().unwrap().is_empty(),
                "{report}"
            );
            assert_eq!(
                output.stdout,
                if expected == "0" {
                    b"<alpha\n>beta\n".as_slice()
                } else {
                    b""
                }
            );
            assert_eq!(
                output.stderr,
                if expected == "0" {
                    b"done\n".as_slice()
                } else {
                    b""
                }
            );
        }
    }
}

fn verify_source_action_sdk(binary: &Path, root: &Path) {
    let app = root.join("generic-action-app.es");
    let checkpoints = root.join("action-checkpoints");
    let keys = root.join("action-checkpoint-keys");
    fs::write(
        &app,
        r#"module app.main;
import std.extra.{Echo, request_echo};
import std.runtime.checkpoint.checkpoint;
import std.task.{Async, Scheduler, run, start, await};
flow greeting() -> string ![] { return "hello"; }
flow main(args: Array<string>) -> i32 ![] {
    let Uniform = handler {
        Echo.echo(value) => { resume value; }
        Echo.select(key, value) => { resume value; }
    };
    let values = handle {
        (request_echo(7), request_echo("hello"), perform Echo.select<string>("key", 11))
    } with Uniform;
    if values != (7, "hello", 11) { return 2; }
    return run((scheduler: Scheduler) => {
        let first = start(() => {
            return handle { perform Async.yield(); checkpoint("generic-task"); 7 }
                with { Async.yield() as continuation => { scheduler.requeue(continuation, ()); } };
        });
        let second = start(greeting);
        return handle {
            let n: i32 = await(first);
            let text: string = await(second);
            if n == 7 && text == "hello" { 0 } else { 1 }
        } with {
            Async.await(task) as continuation => { scheduler.park(task, continuation); }
            Async.yield() as continuation => { scheduler.requeue(continuation, ()); }
        };
    });
}

"#,
    )
    .unwrap();
    for resume in [false, true] {
        let mut command = Command::new(binary);
        command.args(["--cache", "off"]);
        if resume {
            command.args(["resume", "0"]);
        } else {
            command.args(["run", path(&app), "--cpu-threads", "4"]);
        }
        let output = command
            .args([
                "--std-provider",
                "artifact",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "0", "{report}");
        assert!(
            report["diagnostics"].as_array().unwrap().is_empty(),
            "{report}"
        );
        assert_eq!(
            report["checkpoints"].as_array().unwrap().len(),
            usize::from(!resume)
        );
        if !resume {
            let events = report["events"].as_array().unwrap();
            let parked = events
                .iter()
                .position(|event| event["kind"] == "task_parked")
                .unwrap();
            let saved = events
                .iter()
                .position(|event| event["kind"] == "checkpoint_created")
                .unwrap();
            assert!(
                parked < saved,
                "checkpoint must retain an already parked source await: {report}"
            );
        }
    }
}

fn verify_task_provider_parity(binary: &Path, root: &Path) {
    let app = root.join("task-provider-parity.es");
    fs::write(
        &app,
        r#"module app.main;
import std.task.{Async, Scheduler, run, start, await};
flow number() -> i32 ![] { return 7; }
flow greeting() -> string ![] { return "hello"; }
flow main(args: Array<string>) -> i32 {
    return run((scheduler: Scheduler) => {
        let first = start(number);
        let second = start(greeting);
        return handle {
            let n: i32 = await(first);
            let text: string = await(second);
            if n == 7 && text == "hello" { 0 } else { 1 }
        } with {
            Async.await(task) as continuation => { scheduler.park(task, continuation); }
            Async.yield() as continuation => { scheduler.requeue(continuation, ()); }
        };
    });
}
"#,
    )
    .unwrap();
    let providers: &[&str] = if cfg!(feature = "forbid-legacy-std") {
        &["artifact"]
    } else {
        &["legacy", "artifact"]
    };
    for provider in providers {
        let output = Command::new(binary)
            .args([
                "--cache",
                "off",
                "run",
                path(&app),
                "--std-provider",
                provider,
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "0", "{provider}: {report}");
    }
}

// A source std wrapper carries a caller's generic action and static selector.
// Handler arms stay with their lexical task, including a nested scheduled owner.
fn verify_caller_task_handler(binary: &Path, root: &Path) {
    let app = root.join("caller-task-handler.es");
    let checkpoints = root.join("caller-handler-checkpoints");
    let keys = root.join("caller-handler-checkpoint-keys");
    fs::write(
        &app,
        r#"module app.main;
import std.task.{Scheduler, run, start, await};
import std.runtime.checkpoint.checkpoint;
import std.extra.sdk_echo;
effect Probe { action<T> value<Key>(key: Key, value: T) -> T; }
flow scheduled(scheduler: Scheduler) -> i32 ![Probe.value<string>] {
    let value = perform Probe.value<string>("key", 40);
    return value + 2;
}
flow main(args: Array<string>) -> i32 ![] {
    let answer = handle { run(scheduled) } with {
        Probe.value<string>(key, value) => { checkpoint("caller-handler"); resume sdk_echo(value); }
    };
    if answer + 1 != 43 { return 1; }
    let nested = run((scheduler: Scheduler) => {
        let inner = handle {
            let ready = start(() => { return 1; });
            let requesting = start(() => {
                let value = perform Probe.value<string>("key", 40);
                return value + 1;
            });
            let first = await(ready);
            first + await(requesting)
        } with {
            Probe.value<string>(key, value) => { checkpoint("nested-handler"); resume sdk_echo(value); }
        };
        return inner + 1;
    });
    if nested != 43 { return 2; }
    let finished = run((scheduler: Scheduler) => {
        let answer = handle { run(scheduled); "unreached" } with {
            Probe.value<string>(key, value) => { checkpoint("finish-handler"); finish "finished"; }
        };
        if answer == "finished" { return "continued"; }
        return "unreached";
    });
    if finished != "continued" { return 3; }
    return 0;
}
"#,
    )
    .unwrap();
    for resume in [None, Some(0), Some(1), Some(2)] {
        let mut command = Command::new(binary);
        command.args(["--cache", "off"]);
        if let Some(index) = resume {
            command.args(["resume", &index.to_string()]);
        } else {
            command.args(["run", path(&app)]);
        }
        let output = command
            .args([
                "--std-provider",
                "artifact",
                "--checkpoint-dir",
                path(&checkpoints),
                "--checkpoint-key-dir",
                path(&keys),
                "--format",
                "json",
            ])
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], "0", "{report}");
        assert!(
            report["diagnostics"].as_array().unwrap().is_empty(),
            "{report}"
        );
        assert_eq!(
            report["checkpoints"].as_array().unwrap().len(),
            resume.map_or(3, |index| 2 - index),
            "{report}"
        );
    }
}

#[test]
fn source_sdk_join_and_added_generic_api_run_after_install_and_relocation() {
    let root = temp_path(
        &format!(
            "source-sdk-declaration-closure-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
        "workspace",
    );
    let source = root.join("source");
    let consumer = root.join("consumer");
    let library = source.join("library/std");
    fs::create_dir_all(&consumer).unwrap();
    let maintained = etas_std::library::source_checkout_manifest()
        .parent()
        .unwrap();
    copy_dir_contents(&maintained.join("library"), &source.join("library"));
    fs::copy(maintained.join("etas.toml"), source.join("etas.toml")).unwrap();
    let binary = std::env::var_os("ETAS_STD_DELIVERY_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_etas")));
    let invoke = |binary: &Path, args: &[&str]| {
        let output = Command::new(binary)
            .current_dir(&consumer)
            .args([
                "--no-config",
                "--std-provider",
                "artifact",
                "--cache",
                "off",
                "--color",
                "never",
                "--format",
                "json",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{:?}\n{}\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    let manifest = source.join("etas.toml");
    let base = root.join("base/std");
    invoke(
        &binary,
        &[
            "std",
            "build",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--out-dir",
            base.to_str().unwrap(),
        ],
    );
    let app = consumer.join("app.es");
    fs::write(&app, "module app; import std.text.join; flow main(args: Array<string>) -> i32 ![] { return if join([\"source\", \"sdk\"], \":\") == \"source:sdk\" { 42 } else { 0 }; }").unwrap();
    let result = invoke(
        &binary,
        &[
            "--sysroot",
            base.parent().unwrap().to_str().unwrap(),
            "run",
            "app.es",
        ],
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"value\":\"42\""));
    // The only API change is this new Etas file. Its private type/method body
    // must remain available to the application in another process.
    fs::write(
        library.join("text/added.es"),
        r#"module std.text;
type Envelope<T> = { value: T };
impl Envelope<T> {
    flow unpack(self) -> T ![] { return self.value; }
}
public flow sdk_echo<T>(value: T) -> T ![] {
    let envelope = Envelope { value };
    return envelope.unpack();
}
"#,
    )
    .unwrap();
    let prefix = root.join("prefix");
    invoke(
        &binary,
        &[
            "std",
            "install",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--prefix",
            prefix.to_str().unwrap(),
        ],
    );
    let installed_std = fs::read_dir(prefix.join("lib/etas"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("std");
    let installed: serde_json::Value =
        serde_json::from_slice(&fs::read(installed_std.join("manifest.json")).unwrap()).unwrap();
    let repeat = root.join("repeat/std");
    invoke(
        &binary,
        &[
            "std",
            "build",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--out-dir",
            repeat.to_str().unwrap(),
        ],
    );
    // Installation adds toolchain-local publication, query and layout data.
    // The canonical checked objects and public package must be identical.
    let mut canonical = installed.clone();
    for field in [
        "effect_analysis_hash",
        "interpreter_layout_hash",
        "prepared_queries_hash",
    ] {
        canonical.as_object_mut().unwrap().remove(field);
    }
    let rebuilt: serde_json::Value =
        serde_json::from_slice(&fs::read(repeat.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(canonical, rebuilt);
    let artifact = installed["artifact"].as_str().unwrap();
    assert_eq!(
        fs::read(installed_std.join(artifact)).unwrap(),
        fs::read(repeat.join(artifact)).unwrap()
    );
    // The SDK calls back into an implementation owned by the application.
    // The restored method proof must share the live caller's type arena.
    fs::write(
        &app,
        r#"module app;
import std.text.{join, sdk_echo};
import std.collections.{Order, lower_bound};
type Ranked = { key: i32 };
impl Ranked ~ Order {
    public flow compare(self, other: Ranked) -> i32 ![] { return other.key.compare(self.key); }
}
flow main(args: Array<string>) -> i32 ![] {
    let values = [Ranked{key= 9}, Ranked{key= 3}];
    if lower_bound(values, Ranked{key= 3}) != 1 { return 1; }
    let text = sdk_echo(join(["source", "sdk"], ":"));
    return if text == "source:sdk" { 42 } else { if text == "source:sdk!" { 43 } else { 0 } };
}"#,
    )
    .unwrap();
    let moved = root.join("moved-prefix");
    fs::rename(&prefix, &moved).unwrap();
    fs::remove_dir_all(source.join("library")).unwrap();
    let relocated_binary = moved
        .join("bin")
        .join(format!("etas{}", std::env::consts::EXE_SUFFIX));
    // Discovery must use the moved executable's install prefix.
    invoke(&relocated_binary, &["check", "app.es"]);
    let result = invoke(&relocated_binary, &["run", "app.es"]);
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"value\":\"42\""));
    let cache = root.join("consumer-cache");
    let cached_run = |label: &str, expected: &str| {
        let profile = root.join(format!("{label}-profile.json"));
        let output = Command::new(&relocated_binary)
            .current_dir(&consumer)
            .env("XDG_CACHE_HOME", root.join("xdg"))
            .args([
                "--no-config",
                "--std-provider",
                "artifact",
                "--cache",
                "read-write",
                "--cache-root",
            ])
            .arg(&cache)
            .arg("--profile-out")
            .arg(&profile)
            .args(["--format", "json", "run", "app.es"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{label}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["value"]["value"], expected, "{label}");
        let profile: serde_json::Value =
            serde_json::from_slice(&fs::read(profile).unwrap()).unwrap();
        let counter = |name: &str| {
            profile["counters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|counter| counter["name"] == name)
                .unwrap()["value"]
                .as_u64()
                .unwrap()
        };
        assert_eq!(counter("frontend.retained_revisions"), 0, "{label}");
        #[cfg(target_os = "linux")]
        if label == "cold" || label == "warm" {
            for name in [
                "std.consumer.link_cache_hit",
                "std.runtime.plan_cache_hit",
                "interpreter.entry_plan_cache_hit",
            ] {
                assert_eq!(counter(name), u64::from(label == "warm"), "{label}: {name}");
            }
        }
        let reused = counter("frontend.reused_parsed_sources");
        eprintln!(
            "installed SDK cached {label}: duration_ns={}, reused_parsed_sources={reused}, value={expected}",
            profile["total_duration_ns"]
        );
        reused
    };
    cached_run("cold", "42");
    assert!(fs::read_dir(&cache).unwrap().next().is_some());
    cached_run("warm", "42");

    let bridge = root.join("bridge");
    fs::create_dir_all(bridge.join("src")).unwrap();
    fs::write(
        bridge.join("etas.toml"),
        "[package]\nname = 'sdk-bridge'\nversion = '0.1.0'\nedition = '2026'\n[source]\nroot = 'src'\n[dependencies]\nstd = { version = '>=0.2' }\n",
    )
    .unwrap();
    fs::write(
        bridge.join("src/bridge.es"),
        "module bridge; public flow advance(value: i32) -> i32 ![] { return value + 1; }\n",
    )
    .unwrap();
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: bridge.clone(),
    })
    .unwrap();
    let old = etas_package::lockfile::read_lockfile(&bridge)
        .unwrap()
        .unwrap();
    assert_eq!(old.packages[0].checksum, "builtin:std");
    invoke(
        &relocated_binary,
        &["pkg", "lock", bridge.to_str().unwrap()],
    );
    let upgraded = etas_package::lockfile::read_lockfile(&bridge)
        .unwrap()
        .unwrap();
    assert_eq!(upgraded.packages[0].version, installed["version"]);
    assert_eq!(
        upgraded.packages[0].checksum,
        format!(
            "builtin:std:artifact:{}:{}",
            installed["version"].as_str().unwrap(),
            installed["native_binding_abi"].as_str().unwrap()
        )
    );
    let package = root.join("package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(
        package.join("etas.toml"),
        "[package]\nname = 'sdk-package-consumer'\nversion = '0.1.0'\nedition = '2026'\n[source]\nroot = 'src'\n[dependencies]\nbridge = { path = '../bridge' }\n[[bin]]\nname = 'app'\nmodule = 'app'\nflow = 'main'\n",
    )
    .unwrap();
    fs::write(
        package.join("src/app.es"),
        "module app; import bridge.advance; import std.text.sdk_echo; flow main(args: Array<string>) -> i32 ![] { return sdk_echo(advance(41)); }\n",
    )
    .unwrap();
    invoke(
        &relocated_binary,
        &["pkg", "update", package.to_str().unwrap()],
    );
    let result = invoke(&relocated_binary, &["run", package.to_str().unwrap()]);
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["value"]["value"], "42");

    let moved_std = fs::read_dir(moved.join("lib/etas"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("std");
    let text = moved_std.join("library/std/text/assembly.es");
    rewrite_join_result(&text, |result| format!("({result}) + \"!\""));
    let rebuilt = root.join("rebuilt/std");
    invoke(
        &relocated_binary,
        &[
            "std",
            "build",
            "--manifest-path",
            moved_std.join("etas.toml").to_str().unwrap(),
            "--out-dir",
            rebuilt.to_str().unwrap(),
        ],
    );
    let changed: serde_json::Value =
        serde_json::from_slice(&fs::read(rebuilt.join("manifest.json")).unwrap()).unwrap();
    assert_ne!(installed["artifact_hash"], changed["artifact_hash"]);
    fs::rename(&moved_std, root.join("previous-installed-std")).unwrap();
    fs::rename(&rebuilt, &moved_std).unwrap();
    let result = invoke(&relocated_binary, &["run", "app.es"]);
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"value\":\"43\""));
    // Rebuilding only a std body must invalidate the same application's cache.
    cached_run("changed-body", "43");
    fs::remove_dir_all(root).unwrap();
}
