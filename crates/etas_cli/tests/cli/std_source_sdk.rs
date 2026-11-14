use super::*;

#[test]
fn installed_std_executes_new_generic_flow_and_rebuilt_join_body_in_external_processes() {
    let root = temp_path(
        &format!("std-source-sdk-{}", std::process::id()),
        "workspace",
    );
    let source = root.join("std-source");
    fs::create_dir_all(source.join("library/std/text")).unwrap();
    fs::create_dir_all(source.join("library/std/extra")).unwrap();
    fs::create_dir_all(source.join("library/std/task")).unwrap();
    fs::write(
        source.join("library/std/task/mod.es"),
        r#"module std.task;
@std.role("std.task.Task")
public type Task<T, effect E>;
@std.role("std.task.Scheduler")
public type Scheduler;
@std.role("std.task.Continuation")
public type Continuation<T>;
@std.binding("std.task.run")
public flow run<T, effect E>(body: (Scheduler) -> T ![E]) -> T ![E] {}
@std.binding("std.task.run")
public flow execute<T, effect E>(body: (Scheduler) -> T ![E]) -> T ![E] {}
"#,
    )
    .unwrap();
    fs::create_dir_all(source.join("library/std/core")).unwrap();
    fs::write(
        source.join("library/std/core/mod.es"),
        r#"module std.core;
@std.role("std.core.Index")
public spec Index;
impl i8 ~ Index;
impl i16 ~ Index;
impl i32 ~ Index;
impl i64 ~ Index;
impl i128 ~ Index;
impl isize ~ Index;
impl u8 ~ Index;
impl u16 ~ Index;
impl u32 ~ Index;
impl u64 ~ Index;
impl u128 ~ Index;
impl usize ~ Index;
"#,
    )
    .unwrap();
    fs::create_dir_all(source.join("library/std/result")).unwrap();
    fs::write(source.join("library/std/result/mod.es"), r#"module std.result;
@std.role("std.result.Result")
public enum Result<T, E> { Ok(T), Err(E), }
public flow is_ok<T, E>(value: Result<T, E>) -> bool ![] {
    return match value { Result.Ok(_) => true, Result.Err(_) => false };
}
public flow is_err<T, E>(value: Result<T, E>) -> bool ![] {
    return match value { Result.Ok(_) => false, Result.Err(_) => true };
}

@std.role("std.result.Ok")
public flow Ok<T, E>(value: T) -> Result<T, E> ![] { return Result.Ok(value); }
@std.role("std.result.Err")
public flow Err<T, E>(error: E) -> Result<T, E> ![] { return Result.Err(error); }

impl Result<T, E> {
    public flow is_ok(self) -> bool ![] { return match self { Result.Ok(_) => true, Result.Err(_) => false }; }
    public flow is_err(self) -> bool ![] { return match self { Result.Ok(_) => false, Result.Err(_) => true }; }
}
"#).unwrap();
    fs::write(source.join("etas.toml"), "[package]\nname = 'std'\nversion = '0.1.0'\nedition = '2026'\n[source]\nroot = 'library'\n").unwrap();
    for (file, id) in [
        ("assembly.es", etas_std::StdLibrarySourceId::TextAssembly),
        ("format.es", etas_std::StdLibrarySourceId::TextFormat),
    ] {
        fs::write(
            source.join("library/std/text").join(file),
            etas_std::standard_library_source(id).source,
        )
        .unwrap();
    }
    fs::write(
        source.join("library/std/extra/mod.es"),
        r#"module std.extra;
import std.fs.IOError;
import std.core.Index;
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
    fs::create_dir_all(source.join("library/std/prelude")).unwrap();
    fs::write(
        source.join("library/std/prelude/mod.es"),
        "module std.prelude; public import std.extra.sdk_echo as bundled_echo; public import std.result.{Result, Ok, Err}; public import std.core.Index;",
    )
    .unwrap();
    fs::write(
        source.join("library/std/text/native.es"),
        r#"module std.text;
@std.binding("std.text.trim")
public flow trim(value: string) -> string ![] {}
@std.binding("std.collections.array_push")
public flow append<U>(values: Array<U>, value: U) -> Array<U> ![] {}
"#,
    )
    .unwrap();
    let app = root.join("app.es");
    fs::write(
        &app,
        r#"module app.main;
import std.text.{join, trim, append};
import std.extra.{sdk_echo, Parcel, Constructed, Reply, pack, read_reply, Reader, via_reader, reader, Transformer, transform, roundtrip};
import std.task.{run, execute, Scheduler};
import std.result.{is_ok, is_err};
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
flow main(args: Array<string>) -> i32 {
    let good: Result<i32, string> = Result.Ok(7);
    let bad: Result<i32, string> = Err("failed");
    if !is_ok(good) || is_err(good) || !is_err(bad) { return 79; }
    if !good.is_ok() || good.is_err() || !bad.is_err() { return 77; }
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
    let text = read_reply(Reply.Named { value = via_reader(parcel) });
    let constructed: Constructed<string> = Constructed { value = text };
    if constructed.read() != text { return 95; }
    if parcel.read() != text { return 96; }
    if read_reply(Reply.Item(text)) != text { return 98; }
    if read_reply(Reply.Empty) != "" { return 97; }
    if text == "a|b" { return 0; }
    if text == " A | B " { return 3; }
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
    let before = execute();
    assert_eq!(before["value"]["value"], "0");
    let native = source.join("library/std/text/native.es");
    let binding = fs::read_to_string(&native).unwrap();
    fs::write(
        &native,
        binding.replace("std.text.trim", "std.text.uppercase"),
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
