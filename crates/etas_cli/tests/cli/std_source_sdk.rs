use super::*;

#[test]
fn installed_std_executes_new_generic_flow_and_rebuilt_join_body_in_external_processes() {
    let root = temp_path(
        &format!("std-source-sdk-{}", std::process::id()),
        "workspace",
    );
    let source = root.join("std-source");
    let maintained = etas_std::library::source_checkout_manifest()
        .parent()
        .unwrap();
    copy_dir_contents(&maintained.join("library"), &source.join("library"));
    fs::create_dir_all(source.join("library/std/extra")).unwrap();
    // Check source ownership with binders independent of constructor/method binders.
    for (file, replacements) in [
        (
            "task/mod.es",
            vec![
                (
                    "public type Task<T, effect E>;",
                    "public type Task<Output, effect Remaining>;",
                ),
                (
                    "public type Continuation<T>;",
                    "public type Continuation<ActionOutput>;",
                ),
            ],
        ),
        (
            "fs/mod.es",
            vec![(
                "public type WorkspacePath<R ~ Region>;",
                "public type WorkspacePath<Authority ~ Region>;",
            )],
        ),
        (
            "option/mod.es",
            vec![(
                "public enum Option<T> { Some(T), None, }",
                "public enum Option<Payload> { Some(Payload), None, }",
            )],
        ),
        (
            "result/mod.es",
            vec![(
                "public enum Result<T, E> { Ok(T), Err(E), }",
                "public enum Result<Success, Failure> { Ok(Success), Err(Failure), }",
            )],
        ),
    ] {
        let path = source.join("library/std").join(file);
        let mut text = fs::read_to_string(&path).unwrap();
        for (from, to) in replacements {
            assert!(text.contains(from));
            text = text.replace(from, to);
        }
        fs::write(path, text).unwrap();
    }
    let task = source.join("library/std/task/mod.es");
    fs::write(
        &task,
        fs::read_to_string(&task).unwrap()
            + r#"
@std.binding("std.task.run")
public flow execute<T, effect E>(body: (Scheduler) -> T ![E]) -> T ![E] {}
"#,
    )
    .unwrap();
    fs::copy(maintained.join("etas.toml"), source.join("etas.toml")).unwrap();
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
public import std.extra.Reply.{Empty, Item, Named};
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
@std.binding("std.fs.read_bytes")
flow raw_read<R ~ Region>(target: WorkspacePath<R>) -> bytes ![Error<IOError>] {}
public flow load_checked(target: WorkspacePath<Sandbox>) -> Result<bytes, IOError> ![] {
    let CatchIo: ![Error<IOError> => [] for Result<bytes, IOError>] = handler {
        Error<IOError>.raise(error) => { finish Err(error); }
    };
    return handle { Ok(raw_read(target)) } with CatchIo;
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
    let native = source.join("library/std/text/native.es");
    fs::write(
        &native,
        fs::read_to_string(&native).unwrap()
            + r#"
@std.binding("std.text.uppercase")
public flow case_text(value: string) -> string ![] {}
@std.binding("std.collections.array_push")
public flow append<U>(values: Array<U>, value: U) -> Array<U> ![] {}
"#,
    )
    .unwrap();
    let app = root.join("app.es");
    fs::write(
        &app,
        r#"module app.main;
import std.text.{join, trim, append, case_text};
import std.text.{TextError, ParseFailure, NumericOverflow};
import std.extra.{sdk_echo, Parcel, Constructed, Reply, pack, pack_reply, read_reply, Reader, via_reader, reader, Transformer, transform, roundtrip};
import std.task.{run, execute, Scheduler};
import std.result.{is_ok, is_err};
import std.collections.{array_extend, list_push, list_extend};
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
flow scheduled(scheduler: Scheduler) -> i32 ![] { return 7; }
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
    if std.collections.list_get([1; 2; 3], ordinal) != None || [1; 2; 3].get(1) != Some(2) { return 20; }
    return 0;
}

flow option_checks() -> bool ![] {
    let callable = std.option.unwrap(Some(increment));
    if callable(4) != 5 { return false; }
    let Handle: ![Wrapped => [] for i32] = handler {
        Wrapped.read() => { resume 13; }
    };
    let answer = handle { std.option.unwrap(Some(deferred))() } with Handle;
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
    if std.collections.array_pop([4]) != ([], Some(4)) { return false; }
    if std.collections.list_head([5;]) != Some(5) { return false; }
    let Handle: ![Wrapped => [] for i32] = handler {
        Wrapped.read() => { resume 13; }
    };
    let callbacks = list_push([], deferred).extend([deferred;]);
    let head_answer = handle { callbacks.head().unwrap()() } with Handle;
    if head_answer != 13 { return false; }
    let list_index: u8 = 1;
    let list_answer = handle { callbacks.get(list_index).unwrap()() } with Handle;
    if list_answer != 13 { return false; }
    let array_callbacks = array_extend([], [deferred, deferred]);
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
flow main(args: Array<string>) -> i32 {
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
    if array_extend([1, 2], [3, 4]) != [1, 2, 3, 4] { return 71; }
    if list_push([1; 2], 0) != [0; 1; 2] { return 72; }
    if list_extend([1; 2], [3; 4]) != [3; 4; 1; 2] { return 73; }
    if trim("　 a \t b \n　") != "a \t b" || trim("　 \n\t") != "" { return 74; }
    let good: Result<i32, string> = Result.Ok(7);
    let bad: Result<i32, string> = Err("failed");
    if !is_ok(good) || is_err(good) || !is_err(bad) { return 79; }
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
    let outcome = execute(scheduled);
    if outcome != 7 { return 94; }
    if run(scheduled) != 7 { return 93; }
    let assemble = join;
    let strip = trim;
    let values = append([strip(" a ")], strip(" b "));
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
        let output = Command::new(env!("CARGO_BIN_EXE_etas"))
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
std = { version = "0.2" }
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
    invoke(&["pkg", "lock", path(&bridge)]);
    let old_lock = fs::read(bridge.join("etas.lock")).unwrap();
    let needs_upgrade = Command::new(env!("CARGO_BIN_EXE_etas"))
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
        .replace("    let caller = sdk_echo", "    let bridged = bridge_summary(\"package\");\n    if !is_ok(bridged) { return 76; }\n    let caller = sdk_echo");
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
    let native = source.join("library/std/text/native.es");
    let binding = fs::read_to_string(&native).unwrap();
    fs::write(
        &native,
        binding.replace("std.text.uppercase", "std.text.lowercase"),
    )
    .unwrap();
    build();
    assert_eq!(execute()["value"]["value"], "3");
    fs::write(native, binding).unwrap();
    let assembly = source.join("library/std/text/assembly.es");
    let text = fs::read_to_string(&assembly).unwrap();
    let start = text.find("public flow join(").unwrap();
    let end = text.find("public flow repeat(").unwrap();
    fs::write(
        assembly,
        format!(
            "{}{}{}",
            &text[..start],
            text[start..end].replace("return output;", "return \"rebuilt:\" + output;"),
            &text[end..]
        ),
    )
    .unwrap();
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
    for value in [1, 2, 3] limit Iterations(4) { total = total + value; }
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
    // Remove only the checked bodies, leaving valid public metadata and digests.
    // Artifact mode must report the missing implementation despite source and
    // legacy implementations remaining available.
    let manifest_path = sdk.join("std/manifest.json");
    let mut manifest: etas_driver::std_library::sdk::StdSdkManifest =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let artifact = sdk.join("std").join(&manifest.artifact);
    let decoded =
        etas_package_metadata::decode_metadata_artifact(&artifact, &fs::read(&artifact).unwrap())
            .unwrap();
    let sections = decoded
        .sections
        .into_iter()
        .filter(|(kind, _)| *kind != etas_package_metadata::MetadataSectionKind::LibraryBodies)
        .map(|(kind, payload)| etas_package_metadata::EncodedMetadataSection { kind, payload })
        .collect();
    let bodyless =
        etas_package_metadata::encode_metadata_artifact(&decoded.header, sections).unwrap();
    manifest.artifact_hash = etas_package_metadata::blake3_hash(&bodyless);
    fs::write(artifact, bodyless).unwrap();
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    let missing = Command::new(env!("CARGO_BIN_EXE_etas"))
        .args([
            "--cache",
            "off",
            "run",
            path(&common),
            "--std-provider",
            "artifact",
            "--sysroot",
            path(&sdk),
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr)
            .contains("std artifact is missing checked library bodies"),
        "{}",
        String::from_utf8_lossy(&missing.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}
