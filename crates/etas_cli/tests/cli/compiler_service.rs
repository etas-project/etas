use super::{fs, path, read_checkpoint_file, run, run_process};

fn project(body: &str, profile: &str) -> etas_host::TestWorkspace {
    project_retaining(body, profile, "app.main.Worker")
}

fn project_retaining(body: &str, profile: &str, retained: &str) -> etas_host::TestWorkspace {
    project_with_imports(body, profile, retained, "")
}

fn project_with_imports(
    body: &str,
    profile: &str,
    retained: &str,
    imports: &str,
) -> etas_host::TestWorkspace {
    let workspace = etas_host::TestWorkspace::create(&format!(
        "cli-native-compiler-{}",
        super::NEXT_FILE.fetch_add(1, super::Ordering::Relaxed)
    ))
    .unwrap();
    fs::create_dir_all(workspace.path().join("src/app")).unwrap();
    fs::write(workspace.path().join("etas.toml"), format!(
        "[package]\nname = 'native-compiler-app'\nversion = '0.1.0'\n[reflection]\nretain_bodies = ['{retained}']\n[[bin]]\nname = 'app'\nmodule = 'app.main'\nflow = 'main'\n[runtime.profiles.local.compiler]\n{profile}\n"
    )).unwrap();
    fs::write(workspace.path().join("src/app/main.es"), format!(
        "module app.main; import std.compiler.{{check, compile, parse_body, CompileError}}; import std.runtime.checkpoint; {imports} agent Worker() -> i32 {{ return 1; }} {body}"
    )).unwrap();
    workspace
}

fn assert_metadata_only_project_checked(
    workspace: &etas_host::TestWorkspace,
    options: &etas_driver::DriverOptions,
) {
    assert_metadata_only_project_checked_with_sources(workspace, options, 1);
}

fn assert_metadata_only_project_checked_with_sources(
    workspace: &etas_host::TestWorkspace,
    options: &etas_driver::DriverOptions,
    source_count: usize,
) {
    let loaded = etas_driver::load_project(
        options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), source_count);
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_some(), "{:?}", output.diagnostics);
}

#[test]
fn native_compiler_profile_executes_a_real_revised_agent_without_other_adapters() {
    let workspace = project(
        r#"
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("return 42;") { Ok(body) => body, Err(_) => { return -1; }, };
    let original = Worker;
    let revised = compile((^Worker).with_body(body));
    return revised() + original();
}

"#,
        "backend = 'native'\nallow_compile = true",
    );
    let trace = workspace.path().join("trace.json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"43""#), "{stdout}");
    let trace: serde_json::Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    let serialized = serde_json::to_string(&trace).unwrap();
    assert!(serialized.contains("compiler"), "{serialized}");
}

#[test]
fn native_compiler_profile_executes_a_real_revised_flow() {
    let workspace = project_retaining(
        r#"
flow normalize(value: i32) -> i32 ![] { return value + 0; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("return value * 1;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^normalize).with_body(body));
    return revised(21) + normalize(1);
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"22""#), "{stdout}");
}

#[test]
fn native_compiler_profile_checks_and_installs_guarded_flow_and_callee_plans() {
    for reflected in ["guarded", "wrapper"] {
        let body = format!(
            r#"
effect Gate {{ action check() -> unit; action send() -> unit; }}
spec Safe: trace = Gate.check >> Gate.send;
flow guarded() -> i32 ![] ~ Safe {{
    handle {{ perform Gate.check(); perform Gate.send(); }} with {{
        Gate.check() => {{ resume (); }}
        Gate.send() => {{ resume (); }}
    }};
    return 7;
}}
flow wrapper() -> i32 ![] {{ return guarded(); }}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {{
    let definition = ^{reflected};
    let report = check(definition);
    if report.status != "guarded" {{ return -1; }}
    if report.runtime_checks.len() == 0 {{ return -2; }}
    let revised = compile(definition);
    return revised();
}}
"#
        );
        let workspace = project_retaining(
            &body,
            "backend = 'native'\nallow_check = true\nallow_compile = true",
            &format!("app.main.{reflected}"),
        );
        let (code, stdout, stderr) =
            run_process(["run", path(workspace.path()), "--profile", "local"], "");
        assert_eq!(code, 0, "{reflected}: {stderr}\n{stdout}");
        assert!(stdout.contains(r#""value":"7""#), "{reflected}: {stdout}");
    }
}

#[test]
fn native_compiler_profile_checks_and_invokes_edited_guarded_flow_and_callee_plans() {
    for reflected in ["guarded", "wrapper"] {
        let replacement = if reflected == "guarded" {
            "let edited = true; handle { perform Gate.check(); perform Gate.send(); } with { Gate.check() => { let value = (); resume value; } Gate.send() => { resume (); } }; return 9;"
        } else {
            "let edited = true; return guarded();"
        };
        let body = format!(
            r#"
effect Gate {{ action check() -> unit; action send() -> unit; }}
spec Safe: trace = Gate.check >> Gate.send;
flow guarded() -> i32 ![] ~ Safe {{
    handle {{ perform Gate.check(); perform Gate.send(); }} with {{
        Gate.check() => {{ resume (); }}
        Gate.send() => {{ resume (); }}
    }};
    return 7;
}}
flow wrapper() -> i32 ![] {{ return guarded(); }}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {{
    let body = match parse_body("{replacement}") {{ Ok(body) => body, Err(_) => {{ return -1; }}, }};
    let definition = (^{reflected}).with_body(body);
    let report = check(definition);
    if report.status != "guarded" {{ return -2; }}
    if report.runtime_checks.len() == 0 {{ return -3; }}
    let revised = compile(definition);
    return revised();
}}
"#
        );
        let workspace = project_retaining(
            &body,
            "backend = 'native'\nallow_check = true\nallow_compile = true",
            &format!("app.main.{reflected}"),
        );
        let (code, stdout, stderr) =
            run_process(["run", path(workspace.path()), "--profile", "local"], "");
        assert_eq!(code, 0, "{reflected}: {stderr}\n{stdout}");
        let expected = if reflected == "guarded" { 9 } else { 7 };
        assert!(
            stdout.contains(&format!(r#""value":"{expected}""#)),
            "{reflected}: {stdout}"
        );
    }
}

#[test]
fn native_compiler_profile_rejects_unproved_flow_arithmetic_outcomes() {
    let workspace = project_retaining(
        r#"
flow normalize(value: i32) -> i32 ![] { return value + 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("return value * 2;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^normalize).with_body(body));
    return revised(21);
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(
        stderr.contains("code::MissingOutcomeSensitiveSummary"),
        "{stderr}"
    );
}

#[test]
fn native_compiler_profile_revises_a_bound_generic_flow() {
    let workspace = project_retaining(
        r#"
flow identity<T>(value: T) -> T ![] { return value; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let selected: (bool) -> bool ![] = identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^selected).with_body(body));
    if revised(false) && !selected(false) { return 1; }
    return 0;
}

"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.identity",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_preserves_distinct_generic_flow_bindings() {
    let workspace = project_retaining(
        r#"
flow first<T, U>(left: T, right: U) -> T ![] { return left; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, i32) -> bool ![] = first;
    let body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^original).with_body(body));
    if original(true, 7) && !revised(true, 7) { return 1; }
    return 0;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.first",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_rejects_wrong_bound_generic_flow_result() {
    let workspace = project_retaining(
        r#"
flow first<T, U>(left: T, right: U) -> T ![] { return left; }
flow main(args: Array<string>) -> i32 {
    let original: (bool, i32) -> bool ![] = first;
    let body = match parse_body("return right;") { Ok(body) => body, Err(_) => { return -1; }, };
    let draft = (^original).with_body(body);
    return match compile(draft)? {
        Err(error) => { if error.kind == "rejected" && original(true, 7) { return 1; } return -2; },
        Ok(_) => 0,
    };
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.first",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_revises_a_flow_with_permuted_outer_generics() {
    let workspace = project_retaining(
        r#"
flow first<A, B>(left: A, right: B) -> A ![] { return left; }
flow invoke<T, U>(left: T, right: U) -> U ![Error<CompileError>] {
    let selected: (U, T) -> U ![] = first;
    let body = match parse_body("return left;") { Ok(body) => body, Err(_) => { return right; }, };
    let revised = compile((^selected).with_body(body));
    return revised(right, left);
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if invoke<bool, i32>(true, 7) == 7 { return 1; }
    return 0;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.first",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_revises_a_flow_using_the_outer_generic_application() {
    let workspace = project_with_imports(
        r#"
flow normalize<T>(value: T) -> T ![] { return value; }
flow invoke<U>(value: U) -> U ![Error<CompileError>] {
    let definition: CallableDef<(U) -> U ![]> = ^normalize;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return value; }, };
    let revised = compile(definition.with_body(body));
    return revised(value);
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if invoke<bool>(true) { return 1; }
    return 0;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
        "import std.compiler.CallableDef;",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_revises_a_flow_using_the_outer_effect_row() {
    let workspace = project_with_imports(
        r#"
flow normalize<effect E>() -> unit ![E] { return; }
flow invoke<effect F>() -> unit ![F, Error<CompileError>] {
    let definition: CallableDef<() -> unit ![F]> = ^normalize;
    let body = match parse_body("return;") { Ok(body) => body, Err(_) => { return; }, };
    let revised = compile(definition.with_body(body));
    revised();
    return;
}

flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    invoke<![Console.stdout_write]>();
    return 1;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
        "import std.compiler.CallableDef;",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_reflects_a_generic_flow_inside_a_revised_flow() {
    let workspace = project_with_imports(
        r#"
flow normalize<T>(value: T) -> T ![] { return value; }
flow wrapper<U>(value: U) -> U ![] {
    return value;
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let selected: (bool) -> bool ![] = wrapper;
    let body = match parse_body("let definition: CallableDef<(U) -> U ![]> = ^normalize; return value;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let revised = compile((^selected).with_body(body));
    if revised(true) { return 1; }
    return 0;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.wrapper', 'app.main.normalize",
        "import std.compiler.CallableDef;",
    );
    let trace = workspace.path().join("trace.json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    let trace = fs::read_to_string(trace).unwrap_or_default();
    assert_eq!(code, 0, "{stderr}\n{stdout}\n{trace}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_reflects_an_effect_row_flow_inside_a_revision() {
    let workspace = project_with_imports(
        r#"
flow normalize<effect E>() -> unit ![E] { return; }
flow wrapper<effect F>() -> unit ![F] {
    return;
}
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let selected: () -> unit ![Console.stdout_write] = wrapper;
    let body = match parse_body("let definition: CallableDef<() -> unit ![F]> = ^normalize; return;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let revised = compile((^selected).with_body(body));
    revised();
    return 1;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.wrapper', 'app.main.normalize",
        "import std.compiler.CallableDef;",
    );
    let trace = workspace.path().join("trace.json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    let trace = fs::read_to_string(trace).unwrap_or_default();
    assert_eq!(code, 0, "{stderr}\n{stdout}\n{trace}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_revises_a_direct_effect_row_generic_flow() {
    let workspace = project_with_imports(
        r#"
flow normalize<effect E>() -> unit ![E] { return; }
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let definition: CallableDef<() -> unit ![Console.stdout_write]> = ^normalize;
    let body = match parse_body("return;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    revised();
    return 1;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
        "import std.compiler.CallableDef;",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_executes_revised_flow_with_scoped_action_handler() {
    let workspace = project_with_imports(
        r#"
effect Choice { action decide() -> unit; }
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("old-first")?; println("old-second")?; resume; } };
    return;
}
flow callback() -> unit ![Choice.decide] { perform Choice.decide(); return; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("handle { f(); } with { Choice.decide() => { println(\"first\")?; println(\"second\")?; resume; } }; return;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let revised = compile((^normalize).with_body(body));
    revised(callback);
    return 1;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
        "import std.io.println;",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "first\nsecond\n");
}

#[test]
fn native_compiler_profile_rejects_revised_flow_with_a_wider_handler_trace() {
    let workspace = project_with_imports(
        r#"
effect Choice { action decide() -> unit; }
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("old-first")?; println("old-second")?; resume; } };
    return;
}
flow callback() -> unit ![Choice.decide] { perform Choice.decide(); return; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("handle { f(); } with { Choice.decide() => { println(\"first\")?; println(\"second\")?; println(\"third\")?; resume; } }; return;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let revised = compile((^normalize).with_body(body));
    revised(callback);
    return 1;
}
"#,
        "backend = 'native'\nallow_compile = true",
        "app.main.normalize",
        "import std.io.println;",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("code::ActionTraceNotContained"), "{stderr}");
}

#[test]
fn native_compiler_profile_revises_a_public_exported_dependency_flow() {
    let workspace = project(
        "flow main() -> i32 { return 0; }",
        "backend = 'native'\nallow_compile = true",
    );
    let library = workspace.path().join("library");
    fs::create_dir_all(library.join("src/library")).unwrap();
    fs::write(
        library.join("etas.toml"),
        "[package]\nname = 'code-library'\nversion = '0.1.0'\nedition = '2026'\n[reflection]\nexport_bodies = ['library.api.lookup']\n",
    )
    .unwrap();
    fs::write(
        library.join("src/library/api.es"),
        "module library.api; import std.io.println; public flow lookup() -> i32 ![Console.stdout_write] { println(\"original\")?; return 7; }",
    )
    .unwrap();
    let options = etas_driver::DriverOptions {
        cache_mode: etas_driver::DriverCacheMode::Off,
        ..Default::default()
    };
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    let manifest_path = workspace.path().join("etas.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(manifest_path, format!("{manifest}\n[dependencies]\nfirst = {{ package = 'code-library', version = '0.1.0', import = 'first', path = 'library' }}\n")).unwrap();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup as original;
import first.library.api.lookup as reflected;
import std.compiler.{compile, parse_body, CompileError};
import std.io.println;
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let body = match parse_body("println(\"revised\")?; return 9;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^reflected).with_body(body));
    let value = revised() + original();
    if value == 16 { println("value-ok")?; }
    return value;
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert_eq!(stdout, "revised\noriginal\nvalue-ok\n");

    let main = workspace.path().join("src/app/main.es");
    let source = fs::read_to_string(&main).unwrap();
    let original = r#"println(\"revised\")?; return 9;"#;
    let widened = r#"println(\"revised\")?; println(\"extra\")?; return 9;"#;
    assert!(source.contains(original));
    fs::write(main, source.replace(original, widened)).unwrap();
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("code::ActionTraceNotContained"), "{stderr}");
}

fn generic_dependency_project() -> (etas_host::TestWorkspace, etas_driver::DriverOptions) {
    generic_dependency_project_with_source(
        "module library.api; public flow identity<T>(value: T) -> T ![] { return value; }",
    )
}

fn generic_dependency_project_with_source(
    source: &str,
) -> (etas_host::TestWorkspace, etas_driver::DriverOptions) {
    let workspace = project(
        "flow main() -> i32 { return 0; }",
        "backend = 'native'\nallow_compile = true",
    );
    let library = workspace.path().join("library");
    fs::create_dir_all(library.join("src/library")).unwrap();
    fs::write(
        library.join("etas.toml"),
        "[package]\nname = 'code-library'\nversion = '0.1.0'\nedition = '2026'\n[reflection]\nexport_bodies = ['library.api.identity']\n",
    )
    .unwrap();
    fs::write(library.join("src/library/api.es"), source).unwrap();
    let options = etas_driver::DriverOptions {
        cache_mode: etas_driver::DriverCacheMode::Off,
        ..Default::default()
    };
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    let manifest_path = workspace.path().join("etas.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(manifest_path, format!("{manifest}\n[dependencies]\nfirst = {{ package = 'code-library', version = '0.1.0', import = 'first', path = 'library' }}\n")).unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    (workspace, options)
}

#[test]
fn native_compiler_profile_revises_a_bound_generic_dependency_flow() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let selected: (bool) -> bool ![] = identity;
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    if revised(false) && !selected(false) { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_reflection_has_sealed_checked_application() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    return 1;
}
"#,
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let store =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.checked_reflections().keys().next().unwrap();
    assert!(
        matches!(
            checked.reflection_resolution(expression),
            Some(etas_frontend::HirPathResolution::ExplicitImport {
                target: etas_frontend::ImportTarget::ExternalItem {
                    package: Some(_),
                    ..
                }
            })
        ),
        "{:?}",
        checked.reflection_resolution(expression)
    );
    let application = store
        .checked_external_callable_application(expression, None, Default::default())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(application.program(), &checked));
    assert_eq!(application.expression(), expression);
    assert_eq!(
        application
            .callable()
            .as_declaration()
            .unwrap()
            .declaration(),
        ["identity"]
    );
    assert_eq!(application.arguments().len(), 1);
    assert!(
        matches!(application.arguments()[0], etas_types::CallableGenericArg::Type(ty)
        if checked.type_store.get(ty) == Some(&etas_types::Type::Primitive(etas_types::PrimitiveType::Bool)))
    );
    assert_eq!(
        application.signature().params,
        [application.signature().output]
    );
    let error = store
        .checked_external_callable_application(expression, None, {
            let mut limits = etas_driver::compiler::NativeCompilerConfig::default().descriptors;
            limits.max_nodes = 0;
            limits
        })
        .unwrap_err();
    assert!(
        matches!(
            error,
            etas_frontend::code::definition::NativeDefinitionError::Projection(
                etas_frontend::code::descriptor::ProjectionError::Limit { .. }
            )
        ),
        "{error:?}"
    );
}

#[test]
fn locked_producer_environment_replays_transitive_dependency_metadata() {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    let leaf = workspace.path().join("leaf");
    fs::create_dir_all(leaf.join("src/leaf")).unwrap();
    fs::write(
        leaf.join("etas.toml"),
        "[package]\nname = 'code-leaf'\nversion = '0.1.0'\nedition = '2026'\n",
    )
    .unwrap();
    fs::write(
        leaf.join("src/leaf/api.es"),
        "module leaf.api; public flow value() -> bool ![] { return true; }",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: leaf,
            bin: None,
        },
    )
    .unwrap();
    let manifest_path = library.join("etas.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(manifest_path, format!("{manifest}\n[dependencies]\nleaf = {{ package = 'code-leaf', version = '0.1.0', import = 'leaf', path = '../leaf' }}\n")).unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: library.clone(),
        },
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library.clone(),
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    let consumer = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let owner = consumer
        .package_environment
        .as_ref()
        .unwrap()
        .dependencies
        .iter()
        .find(|dependency| dependency.identity.name == "code-library")
        .unwrap();
    let snapshot = etas_package::read_locked_package_sources(
        &library,
        owner.source.payload_checksum().unwrap(),
        Default::default(),
        || Ok::<_, std::convert::Infallible>(()),
    )
    .unwrap();
    let expected = etas_package::prepare_package(etas_package::PreparePackageOptions {
        package_root: library,
        selected_bin: None,
    })
    .unwrap()
    .environment;
    let replayed = etas_package::environment_from_locked_snapshot(&snapshot, owner).unwrap();
    assert_eq!(replayed, expected);
    let mut forged = owner.clone();
    forged.dependencies[0].identity.version = "9.9.9".into();
    assert!(etas_package::environment_from_locked_snapshot(&snapshot, &forged).is_err());
}

#[test]
fn metadata_generic_application_matches_the_producer_frozen_environment() {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    fs::write(
        library.join("src/library/api.es"),
        r#"module library.api;
public flow identity<T>(value: T) -> T ![] { return value; }
flow prepare() -> bool ![] { return identity<bool>(true); }
"#,
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library.clone(),
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow main() -> unit { let definition: CallableDef<(bool) -> bool ![]> = ^identity; return; }
"#,
    )
    .unwrap();
    let load = |root: &std::path::Path| {
        etas_driver::load_project(
            &options,
            etas_driver::LoadProjectRequest {
                paths: vec![root.to_owned()],
                all: false,
                entry_policy: etas_frontend::EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap()
    };
    let producer = load(&library);
    let producer_output = etas_frontend::Frontend.check_project(producer.loaded.input.clone());
    assert!(
        producer_output.diagnostics.is_empty(),
        "{:?}",
        producer_output.diagnostics
    );
    let producer_checked = producer_output.checked.unwrap();
    let producer_store = etas_driver::compiler::prepare_native_definitions(
        &producer,
        &producer_checked,
        Default::default(),
    )
    .unwrap();
    let owner = producer_checked
        .items
        .iter()
        .find_map(|(id, item)| match item {
            etas_hir::HirItem::Flow(flow)
                if producer_checked.symbols.get(flow.symbol).unwrap().name == "identity" =>
            {
                Some(id)
            }
            _ => None,
        })
        .unwrap();
    let boolean = producer_checked
        .type_store
        .iter()
        .find_map(|(id, ty)| {
            (*ty == etas_types::Type::Primitive(etas_types::PrimitiveType::Bool)).then_some(id)
        })
        .unwrap();
    let native = producer_checked
        .checked_callable_application(owner, &[("T".into(), boolean)], &[])
        .unwrap();
    let frozen = producer_store
        .catalog()
        .freeze_callable_application(&native, Default::default())
        .unwrap();

    let consumer = load(workspace.path());
    let consumer_output = etas_frontend::Frontend.check_project(consumer.loaded.input.clone());
    assert!(
        consumer_output.diagnostics.is_empty(),
        "{:?}",
        consumer_output.diagnostics
    );
    let consumer_checked = consumer_output.checked.unwrap();
    let consumer_store = etas_driver::compiler::prepare_native_definitions(
        &consumer,
        &consumer_checked,
        Default::default(),
    )
    .unwrap();
    let expression = *consumer_checked
        .checked_reflections()
        .keys()
        .next()
        .unwrap();
    let definition = consumer_store
        .reflect_external_callable_application(expression, None, Default::default())
        .unwrap();
    let view = consumer_store.resolve(&definition).unwrap();
    assert_eq!(
        view.revision().inputs().contract,
        frozen.contract().fingerprint()
    );
    assert_eq!(view.revision().inputs().environment, frozen.fingerprint());
    struct Continue;
    impl etas_utils::PipelineExecutionControl for Continue {
        fn checkpoint(&self) -> Result<(), etas_utils::PipelineInterruption> {
            Ok(())
        }
    }
    let body = consumer_store
        .syntax_operations()
        .parse_body_bounded("return true;", Default::default())
        .unwrap();
    let draft = consumer_store
        .with_body_controlled(&definition, body, &Continue)
        .unwrap();
    let restored_draft = consumer_store
        .restore_external_application_controlled(
            &draft,
            &producer_store,
            Default::default(),
            &Continue,
        )
        .unwrap();
    let restored = consumer_store.resolve(&restored_draft).unwrap();
    assert_eq!(
        restored.revision(),
        consumer_store.resolve(&draft).unwrap().revision()
    );
    assert_eq!(
        restored.environment().unwrap().fingerprint(),
        frozen.fingerprint()
    );
    assert!(restored.external_application().is_some());
    let overlay = consumer_store.body_overlay(&restored_draft).unwrap();
    let analysis = etas_frontend::FrontendSession::new()
        .analyze_body_overlay_controlled(overlay, std::sync::Arc::new(Continue))
        .unwrap();
    let report = analysis.check_report(Default::default()).unwrap();
    assert_eq!(format!("{:?}", report.status), "Accepted", "{report:?}");
    let candidate = analysis.into_checked_candidate(Default::default()).unwrap();
    assert!(candidate.matches_external_definition(&consumer_store, &draft));
    assert!(candidate.matches_external_definition(&consumer_store, &definition));
    assert!(!candidate.matches_external_definition(&producer_store, &draft));
    assert!(
        consumer_store
            .reflect_checked(&draft, &candidate, Default::default())
            .is_ok()
    );

    // The public signature and behavior remain the same, but this is no
    // longer the producer snapshot whose metadata authorized the draft.
    fs::write(
        library.join("src/library/api.es"),
        r#"module library.api;
public flow identity<T>(value: T) -> T ![] { let copy = value; return copy; }
flow prepare() -> bool ![] { return identity<bool>(true); }
"#,
    )
    .unwrap();
    let changed = load(&library);
    let changed_output = etas_frontend::Frontend.check_project(changed.loaded.input.clone());
    assert!(
        changed_output.diagnostics.is_empty(),
        "{:?}",
        changed_output.diagnostics
    );
    let changed_store = etas_driver::compiler::prepare_native_definitions(
        &changed,
        &changed_output.checked.unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        consumer_store.restore_external_application_controlled(
            &draft,
            &changed_store,
            Default::default(),
            &Continue,
        ),
        Err(etas_frontend::code::definition::NativeDefinitionError::CandidateInputMismatch)
    ));
}

#[test]
fn metadata_generic_application_preserves_repeated_callback_trace() {
    assert_metadata_generic_callback_application(
        "public flow identity<effect E>(f: () -> unit ![E]) -> unit ![E] { f(); f(); return; }",
    );
}

#[test]
fn metadata_generic_application_deduplicates_explicit_and_bound_actions() {
    assert_metadata_generic_callback_application(
        "public flow identity<effect E>(f: () -> unit ![E]) -> unit ![Console.stdout_write, E] { perform Console.stdout_write(\"fixed\"); f(); f(); return; }",
    );
}

fn assert_metadata_generic_callback_application(declaration: &str) {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    fs::write(
        library.join("src/library/api.es"),
        format!("module library.api; {declaration}"),
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 {
    let definition: CallableDef<(() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write]> = ^identity;
    return 1;
}
"#).unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let store =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.checked_reflections().keys().next().unwrap();
    let application = store
        .checked_external_callable_application(expression, None, Default::default())
        .unwrap();
    let (external_graph, external_header) = store
        .catalog()
        .external_callable_application_description(&application, Default::default())
        .unwrap();

    let equivalent = workspace.path().join("equivalent-source");
    fs::create_dir_all(equivalent.join("src/library")).unwrap();
    fs::write(
        equivalent.join("etas.toml"),
        "[package]\nname = 'code-library'\nversion = '0.1.0'\nedition = '2026'\n",
    )
    .unwrap();
    fs::write(equivalent.join("src/library/api.es"), format!(r#"module library.api;
{declaration}
flow callback() -> unit ![Console.stdout_write] {{ perform Console.stdout_write("x"); return; }}
flow prepare() -> unit ![Console.stdout_write] {{ identity<![Console.stdout_write]>(callback); return; }}
"#)).unwrap();
    let local = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![equivalent],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = etas_frontend::Frontend.check_project(local.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let local_checked = output.checked.unwrap();
    let owner = local_checked
        .items
        .iter()
        .find_map(|(id, item)| match item {
            etas_hir::HirItem::Flow(flow)
                if local_checked.symbols.get(flow.symbol).unwrap().name == "identity" =>
            {
                Some(id)
            }
            _ => None,
        })
        .unwrap();
    let row = match &application.arguments()[0] {
        etas_types::CallableGenericArg::EffectRow(row) => row.clone(),
        _ => panic!("effect argument"),
    };
    let native = local_checked
        .checked_callable_application(owner, &[], &[("E".into(), row)])
        .unwrap();
    let local_store = etas_driver::compiler::prepare_native_definitions(
        &local,
        &local_checked,
        Default::default(),
    )
    .unwrap();
    let frozen = local_store
        .catalog()
        .freeze_callable_application(&native, Default::default())
        .unwrap();
    let contract = frozen.contract();
    let native_header = contract.callable(contract.entry().unwrap()).unwrap();
    assert_eq!(
        external_graph
            .type_fingerprint(external_header.signature)
            .unwrap(),
        contract
            .descriptors()
            .type_fingerprint(native_header.signature)
            .unwrap()
    );
    assert_eq!(
        external_header
            .analysis
            .trace
            .map(|trace| external_graph.trace_fingerprint(trace).unwrap()),
        native_header
            .analysis
            .trace
            .map(|trace| contract.descriptors().trace_fingerprint(trace).unwrap())
    );
    for (left, right) in [
        (
            external_header.analysis.effects.declared_bound,
            native_header.analysis.effects.declared_bound,
        ),
        (
            external_header.analysis.effects.escaping,
            native_header.analysis.effects.escaping,
        ),
        (
            external_header.analysis.effects.requested,
            native_header.analysis.effects.requested,
        ),
        (
            external_header.analysis.effects.default_handled,
            native_header.analysis.effects.default_handled,
        ),
    ] {
        assert_eq!(
            left.map(|row| external_graph.row_fingerprint(row).unwrap()),
            right.map(|row| contract.descriptors().row_fingerprint(row).unwrap())
        );
    }
    assert_eq!(
        external_header
            .analysis
            .outcomes
            .map(|trace| external_graph.trace_fingerprint(trace).unwrap()),
        native_header
            .analysis
            .outcomes
            .map(|trace| contract.descriptors().trace_fingerprint(trace).unwrap())
    );
    assert_eq!(
        external_header.analysis.determinism,
        native_header.analysis.determinism
    );
    assert_eq!(
        external_header.analysis.independent_determinism,
        native_header.analysis.independent_determinism
    );
    let trace = serde_json::to_string(&external_graph.data().traces).unwrap();
    assert!(trace.matches("ParameterCall").count() >= 2, "{trace}");
}

#[test]
fn metadata_generic_application_preserves_effect_row_argument() {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    fs::write(
        library.join("src/library/api.es"),
        "module library.api; public flow identity<effect E>() -> unit ![E] { return; }",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 {
    let definition: CallableDef<() -> unit ![Console.stdout_write]> = ^identity;
    return 1;
}
"#,
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let store =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.checked_reflections().keys().next().unwrap();
    let application = store
        .checked_external_callable_application(expression, None, Default::default())
        .unwrap();
    assert!(
        matches!(&application.arguments()[0], etas_types::CallableGenericArg::EffectRow(row)
        if row.tail.is_none() && row.effects.len() == 1 && row.effects[0].name == "Console.stdout_write")
    );
    assert!(application.signature().effects.is_some());
    let (graph, signature) = store
        .catalog()
        .external_callable_application_signature(&application, Default::default())
        .unwrap();
    let signature = serde_json::to_value(graph.type_info(signature).unwrap()).unwrap();
    assert_eq!(signature["Function"]["generics"], serde_json::json!([]));
    let row = signature["Function"]["effects"].as_u64().unwrap() as usize;
    assert!(graph.data().rows[row].tail.is_none());
    let effects = serde_json::to_string(&graph.data().rows[row].effects).unwrap();
    assert!(effects.contains("stdout_write"), "{effects}");
}

#[test]
fn metadata_generic_application_requires_its_checked_lexical_owner() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow outer<T>() -> CallableDef<(T) -> T ![]> { return ^identity; }
flow main(args: Array<string>) -> i32 { outer<bool>(); outer<string>(); return 1; }
"#,
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let checked = etas_frontend::Frontend
        .check_project(loaded.loaded.input.clone())
        .checked
        .unwrap();
    let foreign = etas_frontend::Frontend
        .check_project(loaded.loaded.input.clone())
        .checked
        .unwrap();
    let store =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.checked_reflections().keys().next().unwrap();
    let owner = *checked.hir.index().expr_item.get(&expression).unwrap();
    let primitive = |program: &etas_frontend::CheckedProgram, kind| {
        program
            .type_store
            .iter()
            .find_map(|(id, ty)| (ty == &etas_types::Type::Primitive(kind)).then_some(id))
            .unwrap()
    };
    let boolean = primitive(&checked, etas_types::PrimitiveType::Bool);
    let string = primitive(&checked, etas_types::PrimitiveType::String);
    let lexical = checked
        .checked_callable_application(owner, &[("T".into(), boolean)], &[])
        .unwrap();
    let other = checked
        .checked_callable_application(owner, &[("T".into(), string)], &[])
        .unwrap();
    let foreign_bool = primitive(&foreign, etas_types::PrimitiveType::Bool);
    let foreign_expression = *foreign.checked_reflections().keys().next().unwrap();
    let foreign_owner = *foreign
        .hir
        .index()
        .expr_item
        .get(&foreign_expression)
        .unwrap();
    let foreign_lexical = foreign
        .checked_callable_application(foreign_owner, &[("T".into(), foreign_bool)], &[])
        .unwrap();
    let first = store
        .checked_external_callable_application(expression, Some(&lexical), Default::default())
        .unwrap();
    let second = store
        .checked_external_callable_application(expression, Some(&other), Default::default())
        .unwrap();
    let first_fingerprint = store
        .catalog()
        .external_callable_application_fingerprint(&first, Default::default())
        .unwrap();
    let second_fingerprint = store
        .catalog()
        .external_callable_application_fingerprint(&second, Default::default())
        .unwrap();
    assert_ne!(first_fingerprint, second_fingerprint);
    let foreign_store =
        etas_driver::compiler::prepare_native_definitions(&loaded, &foreign, Default::default())
            .unwrap();
    let foreign_application = foreign_store
        .checked_external_callable_application(
            foreign_expression,
            Some(&foreign_lexical),
            Default::default(),
        )
        .unwrap();
    assert_eq!(
        first_fingerprint,
        foreign_store
            .catalog()
            .external_callable_application_fingerprint(&foreign_application, Default::default())
            .unwrap()
    );
    assert!(
        store
            .catalog()
            .external_callable_application_fingerprint(&foreign_application, Default::default())
            .is_err()
    );
    let equivalent = workspace.path().join("equivalent-source");
    fs::create_dir_all(equivalent.join("src/library")).unwrap();
    fs::write(
        equivalent.join("etas.toml"),
        "[package]\nname = 'code-library'\nversion = '0.1.0'\nedition = '2026'\n",
    )
    .unwrap();
    fs::write(
        equivalent.join("src/library/api.es"),
        r#"module library.api;
public flow identity<T>(value: T) -> T ![] { return value; }
flow prepare() -> unit { identity<bool>(true); return; }
"#,
    )
    .unwrap();
    let local = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![equivalent],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let local_output = etas_frontend::Frontend.check_project(local.loaded.input.clone());
    assert!(
        local_output.diagnostics.is_empty(),
        "{:?}",
        local_output.diagnostics
    );
    let local_checked = local_output.checked.unwrap();
    let local_owner = local_checked
        .items
        .iter()
        .find_map(|(id, item)| match item {
            etas_hir::HirItem::Flow(flow)
                if local_checked.symbols.get(flow.symbol).unwrap().name == "identity" =>
            {
                Some(id)
            }
            _ => None,
        })
        .unwrap();
    let local_bool = primitive(&local_checked, etas_types::PrimitiveType::Bool);
    let local_application = local_checked
        .checked_callable_application(local_owner, &[("T".into(), local_bool)], &[])
        .unwrap();
    let local_store = etas_driver::compiler::prepare_native_definitions(
        &local,
        &local_checked,
        Default::default(),
    )
    .unwrap();
    let (metadata_graph, metadata_signature) = store
        .catalog()
        .external_callable_application_signature(&first, Default::default())
        .unwrap();
    assert!(metadata_graph.data().traces.is_empty());
    let local_frozen = local_store
        .catalog()
        .freeze_callable_application(&local_application, Default::default())
        .unwrap();
    let local_contract = local_frozen.contract();
    let metadata_definition = store
        .reflect_external_callable_application(expression, Some(&lexical), Default::default())
        .unwrap();
    let metadata_view = store.resolve(&metadata_definition).unwrap();
    assert_eq!(
        metadata_view.revision().inputs().contract,
        local_contract.fingerprint()
    );
    assert!(metadata_view.external_application().is_some());
    assert!(matches!(
        metadata_view.environment(),
        Err(etas_frontend::code::definition::NativeDefinitionError::MissingSourceEnvironment)
    ));
    let local_header = local_contract
        .callable(local_contract.entry().unwrap())
        .unwrap();
    assert_eq!(
        metadata_graph.type_fingerprint(metadata_signature).unwrap(),
        local_contract
            .descriptors()
            .type_fingerprint(local_header.signature)
            .unwrap()
    );
    let (behavior_graph, behavior_header) = store
        .catalog()
        .external_callable_application_description(&first, Default::default())
        .unwrap();
    assert_eq!(behavior_header.id, local_header.id);
    assert_eq!(
        behavior_header.analysis.determinism,
        local_header.analysis.determinism
    );
    assert_eq!(
        behavior_header.analysis.independent_determinism,
        local_header.analysis.independent_determinism
    );
    let external_effects = &behavior_header.analysis.effects;
    let source_effects = &local_header.analysis.effects;
    for (left, right) in [
        (
            external_effects.declared_bound,
            source_effects.declared_bound,
        ),
        (external_effects.escaping, source_effects.escaping),
        (external_effects.requested, source_effects.requested),
        (
            external_effects.default_handled,
            source_effects.default_handled,
        ),
    ] {
        assert_eq!(
            left.map(|row| behavior_graph.row_fingerprint(row).unwrap()),
            right.map(|row| local_contract.descriptors().row_fingerprint(row).unwrap())
        );
    }
    for (left, right) in [
        (behavior_header.analysis.trace, local_header.analysis.trace),
        (
            behavior_header.analysis.outcomes,
            local_header.analysis.outcomes,
        ),
    ] {
        assert_eq!(
            left.map(|trace| behavior_graph.trace_fingerprint(trace).unwrap()),
            right.map(|trace| local_contract
                .descriptors()
                .trace_fingerprint(trace)
                .unwrap())
        );
    }
    assert_eq!(
        first_fingerprint,
        local_store
            .catalog()
            .callable_application_fingerprint(&local_application, Default::default())
            .unwrap()
    );
    assert!(
        matches!(first.arguments()[0], etas_types::CallableGenericArg::Type(ty) if ty == boolean)
    );
    assert!(
        matches!(second.arguments()[0], etas_types::CallableGenericArg::Type(ty) if ty == string)
    );
    assert!(
        store
            .checked_external_callable_application(expression, None, Default::default())
            .is_err()
    );
    assert!(
        store
            .checked_external_callable_application(
                expression,
                Some(&foreign_lexical),
                Default::default()
            )
            .is_err()
    );
}

#[test]
fn metadata_only_generic_definition_supports_a_pure_body_draft() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{parse_body, CallableDef};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return -1; }, };
    let draft = definition.with_body(body);
    if definition.signature.generics.len() == 0
        && definition.body != None && draft.body != None
        && definition.effects.requested != None && draft.effects.requested == None
        && definition.effects.escaping != None && draft.effects.escaping == None
        && definition.determinism != None && draft.determinism == None {
        return 1;
    }
    return 0;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_definition_compiles_its_edited_body() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    if revised(false) { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compiled_flow_survives_checkpoint() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
import std.runtime.checkpoint;
agent Worker() -> i32 { return 1; }
flow replacement() -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement();
    checkpoint("after-compile");
    if revised(false) { return 1; }
    return 0;
}

"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let checkpoints = workspace.path().join("checkpoints");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
    assert!(checkpoints.join("checkpoint-0.json").is_file());
    let (code, stdout, stderr) =
        run_process(["resume", "0", "--checkpoint-dir", path(&checkpoints)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");

    let checkpoint_path = checkpoints.join("checkpoint-0.json");
    let artifact = read_checkpoint_file(&checkpoint_path);
    assert_eq!(artifact["schema"], "etas.cli.interpreter-checkpoint.v72");
}

#[test]
fn metadata_only_generic_compiled_flow_propagates_through_return() {
    let (workspace, options) = generic_dependency_project();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow replacement() -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement();
    if revised(false) { return 1; }
    return 0;
}
"#).unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn generic_factory_returns_metadata_only_generic_compiled_flow() {
    let (workspace, options) = generic_dependency_project();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow replacement<T>() -> ((T) -> T ![]) ![Error<CompileError>] {
    let definition: CallableDef<(T) -> T ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement<bool>();
    if revised(true) { return 1; }
    return 0;
}
"#).unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_accepts_a_caller_nominal_type() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
type Token = string;
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Token) -> Token ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let result: Token = revised(Token("hello"));
    return 1;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_nominal_record() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
type Token = { value: string };
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Token) -> Token ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let result: Token = revised(Token { value = "hello" });
    if result.value == "hello" { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_recursive_enum() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
enum Tree { Leaf(i32), Branch(Tree, Tree), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tree) -> Tree ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let result = revised(Tree.Branch(Tree.Leaf(1), Tree.Leaf(2)));
    return match result {
        Tree.Leaf(_) => 0,
        Tree.Branch(left, _) => match left {
            Tree.Leaf(value) => value,
            Tree.Branch(_, _) => 0,
        },
    };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_generic_recursive_enum() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
enum Tree<T> { Leaf(T), Branch(Tree<T>, Tree<T>), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tree<i32>) -> Tree<i32> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tree<i32> = Tree.Branch(Tree.Leaf(1), Tree.Leaf(2));
    let result = revised(value);
    return match result {
        Tree.Leaf(_) => 0,
        Tree.Branch(left, _) => match left {
            Tree.Leaf(value) => value,
            Tree.Branch(_, _) => 0,
        },
    };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_caller_nominal_revision_survives_checkpoint() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
import std.runtime.checkpoint;
agent Worker() -> i32 { return 1; }
type Token = { value: string };
flow replacement() -> ((Token) -> Token ![]) ![Error<CompileError>] {
    let definition: CallableDef<(Token) -> Token ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement();
    checkpoint("after-foreign-compile");
    let result = revised(Token { value = "hello" });
    if result.value == "hello" { return 1; }
    return 0;
}
"#,
    ).unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let checkpoints = workspace.path().join("checkpoints");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
    assert!(checkpoints.join("checkpoint-0.json").is_file());
    let (code, stdout, stderr) =
        run_process(["resume", "0", "--checkpoint-dir", path(&checkpoints)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_keeps_same_named_caller_types_distinct() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/a.es"),
        "module app.a; public type Token = { value: string };\n",
    )
    .unwrap();
    fs::write(
        workspace.path().join("src/app/b.es"),
        "module app.b; public type Token = { value: string };\n",
    )
    .unwrap();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import app.a.Token as FirstToken;
import app.b.Token as SecondToken;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let first: CallableDef<(FirstToken) -> FirstToken ![]> = ^identity;
    let second: CallableDef<(SecondToken) -> SecondToken ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let first_flow = compile(first.with_body(body));
    let second_flow = compile(second.with_body(body));
    let a = first_flow(FirstToken { value = "first" });
    let b = second_flow(SecondToken { value = "second" });
    if a.value == "first" && b.value == "second" { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked_with_sources(&workspace, &options, 3);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_std_spec_bound() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
enum Indexed<T ~ Index> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Indexed<i32>) -> Indexed<i32> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Indexed<i32> = Indexed.Value(7);
    return match revised(value) { Indexed.Value(result) => result, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"7""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_spec_bound() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
spec Allowed;
type Marker = i32;
impl Marker ~ Allowed;
enum Tagged<T ~ Allowed> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tagged<Marker>) -> Tagged<Marker> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tagged<Marker> = Tagged.Value(Marker(7));
    return match revised(value) { Tagged.Value(_) => 1, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_caller_super_spec_evidence() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
spec Base;
spec Derived ~ Base;
type Marker = i32;
impl Marker ~ Derived;
enum Tagged<T ~ Base> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tagged<Marker>) -> Tagged<Marker> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tagged<Marker> = Tagged.Value(Marker(7));
    return match revised(value) { Tagged.Value(_) => 1, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_parameterized_super_spec() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
spec Region;
type Root;
impl Root ~ Region;
spec Within<R ~ Region>;
spec Reports<R ~ Region> ~ Within<R>;
type Marker = i32;
impl Marker ~ Reports<Root>;
enum Tagged<T ~ Within<Root>> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tagged<Marker>) -> Tagged<Marker> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tagged<Marker> = Tagged.Value(Marker(7));
    return match revised(value) { Tagged.Value(_) => 1, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_preserves_external_parameterized_super_spec() {
    let (workspace, options) = generic_dependency_project_with_source(
        r#"module library.api;
public spec Region;
public type Root;
impl Root ~ Region;
public spec Within<R ~ Region>;
public spec Reports<R ~ Region> ~ Within<R>;
public flow identity<T>(value: T) -> T ![] { return value; }
"#,
    );
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.{identity, Root, Within, Reports};
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
type Marker = i32;
impl Marker ~ Reports<Root>;
enum Tagged<T ~ Within<Root>> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tagged<Marker>) -> Tagged<Marker> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tagged<Marker> = Tagged.Value(Marker(7));
    return match revised(value) { Tagged.Value(_) => 1, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_generic_compilation_keeps_external_spec_binders_scoped() {
    let (workspace, options) = generic_dependency_project_with_source(
        r#"module library.api;
public spec Region;
public type Root;
impl Root ~ Region;
public spec Within<T ~ Region>;
public flow identity<T>(value: T) -> T ![] { return value; }
"#,
    );
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.{identity, Root, Within};
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
type Marker = i32;
impl Marker ~ Within<Root>;
enum Tagged<T ~ Within<Root>> { Value(T), }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition: CallableDef<(Tagged<Marker>) -> Tagged<Marker> ![]> = ^identity;
    let body = match parse_body("return value;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile(definition.with_body(body));
    let value: Tagged<Marker> = Tagged.Value(Marker(7));
    return match revised(value) { Tagged.Value(_) => 1, };
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn metadata_only_external_spec_rejects_unsatisfied_generic_bound() {
    let (workspace, options) = generic_dependency_project_with_source(
        r#"module library.api;
public spec Region;
public spec Within<R ~ Region>;
public flow identity<T>(value: T) -> T ![] { return value; }
"#,
    );
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.Within;
agent Worker() -> i32 { return 1; }
type NotRegion;
type Marker = i32;
impl Marker ~ Within<NotRegion>;
flow main(args: Array<string>) -> i32 { return 0; }
"#,
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(
        output.checked.is_none(),
        "unsatisfied external generic bound was accepted"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("does not satisfy spec bound")),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
}

#[test]
fn metadata_only_external_super_spec_rejects_unsatisfied_generic_bound() {
    let (workspace, options) = generic_dependency_project_with_source(
        r#"module library.api;
public spec Region;
public spec Within<R ~ Region>;
public flow identity<T>(value: T) -> T ![] { return value; }
"#,
    );
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.Within;
agent Worker() -> i32 { return 1; }
type NotRegion;
spec Bad ~ Within<NotRegion>;
flow main(args: Array<string>) -> i32 { return 0; }
"#,
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![workspace.path().to_owned()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let output = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(
        output.checked.is_none(),
        "unsatisfied external super-spec bound was accepted"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("does not satisfy spec bound")),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
}

#[test]
fn exported_flow_preserves_returned_external_frozen_contract_in_metadata() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
public flow replacement() -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    let body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement();
    if revised(false) { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    let built = etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: workspace.path().to_owned(),
            bin: None,
        },
    )
    .unwrap();
    let bytes = fs::read(&built.artifact_path).unwrap();
    let (_, metadata) =
        etas_package_metadata::package_metadata_from_artifact(&built.artifact_path, &bytes)
            .unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| {
            summary
                .item
                .last()
                .is_some_and(|name| name == "replacement")
        })
        .expect("public flow must export its solved effect summary");
    let graph = summary
        .returned_callables
        .as_ref()
        .expect("returned callable graph must be preserved");
    let frozen = graph
        .sources
        .iter()
        .find_map(|source| match source {
            etas_package_metadata::CallableValueSource::FrozenContract(index) => Some(*index),
            _ => None,
        })
        .expect("returned callable must keep its frozen-contract origin");
    let node = &graph.values[frozen as usize];
    assert_eq!(
        node.public_effects,
        etas_package_metadata::EffectRow::default()
    );
    assert_eq!(
        node.requested_actions,
        etas_package_metadata::EffectRow::default()
    );
    assert_eq!(node.action_trace, etas_package_metadata::ActionTrace::Empty);

    let consumer_workspace = etas_host::TestWorkspace::create(&format!(
        "cli-metadata-consumer-{}",
        super::NEXT_FILE.fetch_add(1, super::Ordering::Relaxed)
    ))
    .unwrap();
    let consumer = consumer_workspace.path().to_owned();
    fs::create_dir_all(consumer.join("src/consumer")).unwrap();
    fs::write(consumer.join("etas.toml"), format!("[package]\nname = 'metadata-consumer'\nversion = '0.1.0'\nedition = '2026'\n[dependencies]\nproducer = {{ package = 'native-compiler-app', version = '0.1.0', import = 'producer', path = '{}' }}\n[[bin]]\nname = 'app'\nmodule = 'consumer.main'\nflow = 'main'\n", workspace.path().display())).unwrap();
    fs::write(
        consumer.join("src/consumer/main.es"),
        r#"module consumer.main;
import producer.app.main.replacement;
import std.compiler.CompileError;
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let revised = replacement();
    if revised(false) { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: consumer.clone(),
        },
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![consumer],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let result = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(result.checked.is_some(), "{:?}", result.diagnostics);
    let checked = result.checked.unwrap();
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let requested = summary
        .requested_actions
        .effects
        .iter()
        .map(|effect| {
            let action = match effect {
                etas_effects::Effect::Action(action) => action,
                etas_effects::Effect::AppliedAction(instance) => &instance.action,
                other => panic!("unexpected requested effect: {other:?}"),
            };
            format!(
                "{}.{}",
                checked.effect_registry.tag_name(action.tag).unwrap(),
                checked
                    .effect_registry
                    .action_name(action.tag, action.action)
                    .unwrap()
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(requested, ["Code.compile".to_owned()].into());
    assert!(!summary.action_trace.action_set().is_empty());
}

#[test]
fn exported_external_frozen_contract_keeps_effectful_requested_trace() {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    fs::write(
        library.join("src/library/api.es"),
        r#"module library.api;
public flow identity<T>(value: T) -> T ![Console.stdout_write] {
    perform Console.stdout_write("source");
    return value;
}
"#,
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
public flow replacement() -> ((bool) -> bool ![Console.stdout_write]) ![Error<CompileError>] {
    let definition: CallableDef<(bool) -> bool ![Console.stdout_write]> = ^identity;
    let body = match parse_body("perform Console.stdout_write(\"new\"); return value;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let revised = replacement();
    if revised(true) { return 1; }
    return 0;
}
"#).unwrap();
    let built = etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: workspace.path().to_owned(),
            bin: None,
        },
    )
    .unwrap();
    let bytes = fs::read(&built.artifact_path).unwrap();
    let (_, metadata) =
        etas_package_metadata::package_metadata_from_artifact(&built.artifact_path, &bytes)
            .unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| {
            summary
                .item
                .last()
                .is_some_and(|name| name == "replacement")
        })
        .unwrap();
    let graph = summary
        .returned_callables
        .as_ref()
        .expect("returned callable graph must be preserved");
    let index = graph
        .sources
        .iter()
        .find_map(|source| match source {
            etas_package_metadata::CallableValueSource::FrozenContract(index) => Some(*index),
            _ => None,
        })
        .expect("frozen contract origin must be preserved");
    let node = &graph.values[index as usize];
    assert!(
        node.requested_actions.effects.iter().any(|effect| effect
            .path
            .last()
            .is_some_and(|name| name == "stdout_write")),
        "{node:?}"
    );
    assert_ne!(node.action_trace, etas_package_metadata::ActionTrace::Empty);

    let consumer_workspace = etas_host::TestWorkspace::create(&format!(
        "cli-effectful-consumer-{}",
        super::NEXT_FILE.fetch_add(1, super::Ordering::Relaxed)
    ))
    .unwrap();
    let consumer = consumer_workspace.path().to_owned();
    fs::create_dir_all(consumer.join("src/consumer")).unwrap();
    fs::write(consumer.join("etas.toml"), format!("[package]\nname = 'effectful-consumer'\nversion = '0.1.0'\nedition = '2026'\n[dependencies]\nproducer = {{ package = 'native-compiler-app', version = '0.1.0', import = 'producer', path = '{}' }}\n[[bin]]\nname = 'app'\nmodule = 'consumer.main'\nflow = 'main'\n", workspace.path().display())).unwrap();
    fs::write(
        consumer.join("src/consumer/main.es"),
        r#"module consumer.main;
import producer.app.main.replacement;
import std.compiler.CompileError;
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let revised = replacement();
    if revised(true) { return 1; }
    return 0;
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: consumer.clone(),
        },
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![consumer],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let result = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(result.checked.is_some(), "{:?}", result.diagnostics);
    let checked = result.checked.unwrap();
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let requested = summary
        .requested_actions
        .effects
        .iter()
        .map(|effect| {
            let action = match effect {
                etas_effects::Effect::Action(action) => action,
                etas_effects::Effect::AppliedAction(instance) => &instance.action,
                other => panic!("unexpected requested effect: {other:?}"),
            };
            format!(
                "{}.{}",
                checked.effect_registry.tag_name(action.tag).unwrap(),
                checked
                    .effect_registry
                    .action_name(action.tag, action.action)
                    .unwrap()
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        requested,
        ["Code.compile".to_owned(), "Console.stdout_write".to_owned()].into()
    );
    assert!(
        summary.action_trace.action_set().iter().count() >= 2,
        "{:?}",
        summary.action_trace
    );
}

#[test]
fn exported_external_frozen_contract_preserves_callback_invocations() {
    assert_exported_external_callback_contract(false);
}

#[test]
fn exported_external_frozen_contract_keeps_independent_action_alongside_callback() {
    assert_exported_external_callback_contract(true);
}

fn assert_exported_external_callback_contract(independent_action: bool) {
    let (workspace, options) = generic_dependency_project();
    let library = workspace.path().join("library");
    fs::write(
        library.join("src/library/api.es"),
        if independent_action {
            r#"module library.api;
public flow identity<effect E>(callback: () -> unit ![E]) -> unit ![Console.stdout_write, E] {
    perform Console.stdout_write("independent"); callback(); callback(); return;
}
"#
        } else {
            r#"module library.api;
public flow identity<effect E>(callback: () -> unit ![E]) -> unit ![E] {
    callback(); callback(); return;
}
"#
        },
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
public flow replacement() -> (((() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write])) ![Error<CompileError>] {
    let definition: CallableDef<(() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write]> = ^identity;
    let body = match parse_body("callback(); callback(); return;") { Ok(body) => body, Err(_) => { return abort("invalid body"); }, };
    return compile(definition.with_body(body));
}
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let invoke = replacement();
    invoke(() => { perform Console.stdout_write("callback"); return; });
    return 1;
}
"#).unwrap();
    if !independent_action {
        let loaded = etas_driver::load_project(
            &options,
            etas_driver::LoadProjectRequest {
                paths: vec![workspace.path().to_owned()],
                all: true,
                entry_policy: etas_frontend::EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap();
        let mut malformed = loaded.loaded.input;
        let source = malformed
            .environment
            .external_public_metadata
            .iter_mut()
            .flat_map(|package| &mut package.effect_summaries)
            .find(|summary| summary.item.last().is_some_and(|name| name == "identity"))
            .expect("external identity must have a checked effect summary");
        source.action_trace = etas_frontend::ProjectExternalActionTraceInput::Empty;
        let rejected = etas_frontend::Frontend.check_project(malformed);
        assert!(
            rejected.checked.is_none(),
            "an unproven symbolic request row was accepted: {:?}",
            rejected.diagnostics
        );
        assert!(
            rejected.diagnostics.iter().any(
                |diagnostic| format!("{:?}", diagnostic.code).contains("IncompleteEffectFacts")
            ),
            "{:?}",
            rejected.diagnostics
        );
    }
    let built = etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: workspace.path().to_owned(),
            bin: None,
        },
    )
    .unwrap();
    let bytes = fs::read(&built.artifact_path).unwrap();
    let (_, metadata) =
        etas_package_metadata::package_metadata_from_artifact(&built.artifact_path, &bytes)
            .unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| {
            summary
                .item
                .last()
                .is_some_and(|name| name == "replacement")
        })
        .unwrap();
    let graph = summary
        .returned_callables
        .as_ref()
        .expect("returned callable graph must be preserved");
    let index = graph
        .sources
        .iter()
        .find_map(|source| match source {
            etas_package_metadata::CallableValueSource::FrozenContract(index) => Some(*index),
            _ => None,
        })
        .expect("frozen contract origin must be preserved");
    let node = &graph.values[index as usize];
    fn count_parameter_calls(trace: &etas_package_metadata::ActionTrace) -> usize {
        use etas_package_metadata::ActionTrace;
        match trace {
            ActionTrace::ParameterCall { .. } => 1,
            ActionTrace::Seq(parts) | ActionTrace::Choice(parts) => {
                parts.iter().map(count_parameter_calls).sum()
            }
            ActionTrace::Repeat(part) => count_parameter_calls(part),
            _ => 0,
        }
    }
    assert_eq!(count_parameter_calls(&node.action_trace), 2, "{node:?}");
    assert_eq!(node.parameter_bindings.len(), 1, "{node:?}");
    assert!(
        node.public_effects.effects.iter().any(|effect| effect
            .path
            .last()
            .is_some_and(|name| name == "stdout_write")),
        "{node:?}"
    );
    if independent_action {
        assert!(
            node.requested_actions.effects.iter().any(|effect| effect
                .path
                .last()
                .is_some_and(|name| name == "stdout_write")),
            "{node:?}"
        );
    } else {
        assert!(
            node.requested_actions.effects.is_empty(),
            "callback bound is not an independent action request: {node:?}"
        );
    }

    let consumer_workspace = etas_host::TestWorkspace::create(&format!(
        "cli-callback-consumer-{}",
        super::NEXT_FILE.fetch_add(1, super::Ordering::Relaxed)
    ))
    .unwrap();
    let consumer = consumer_workspace.path().to_owned();
    fs::create_dir_all(consumer.join("src/consumer")).unwrap();
    fs::write(consumer.join("etas.toml"), format!("[package]\nname = 'callback-consumer'\nversion = '0.1.0'\nedition = '2026'\n[dependencies]\nproducer = {{ package = 'native-compiler-app', version = '0.1.0', import = 'producer', path = '{}' }}\n[[bin]]\nname = 'app'\nmodule = 'consumer.main'\nflow = 'main'\n", workspace.path().display())).unwrap();
    fs::write(
        consumer.join("src/consumer/main.es"),
        r#"module consumer.main;
import producer.app.main.replacement;
import std.compiler.CompileError;
flow main(args: Array<string>) -> i32 ![Console.stdout_write, Error<CompileError>] {
    let revised = replacement();
    revised(() => { perform Console.stdout_write("callback"); return; });
    return 1;
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: consumer.clone(),
        },
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &options,
        etas_driver::LoadProjectRequest {
            paths: vec![consumer],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let result = etas_frontend::Frontend.check_project(loaded.loaded.input);
    assert!(result.checked.is_some(), "{:?}", result.diagnostics);
    let checked = result.checked.unwrap();
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let requested = summary
        .requested_actions
        .effects
        .iter()
        .map(|effect| {
            let action = match effect {
                etas_effects::Effect::Action(action) => action,
                etas_effects::Effect::AppliedAction(instance) => &instance.action,
                other => panic!("unexpected requested effect: {other:?}"),
            };
            format!(
                "{}.{}",
                checked.effect_registry.tag_name(action.tag).unwrap(),
                checked
                    .effect_registry
                    .action_name(action.tag, action.action)
                    .unwrap()
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        requested,
        ["Code.compile".to_owned(), "Console.stdout_write".to_owned()].into()
    );
    assert!(
        summary
            .escaping_effects
            .effects
            .iter()
            .any(|effect| matches!(effect, etas_effects::Effect::Error(_)))
    );
    assert!(summary.escaping_effects.effects.iter().any(|effect| {
        match effect {
            etas_effects::Effect::Action(action) => {
                checked
                    .effect_registry
                    .action_name(action.tag, action.action)
                    == Some("stdout_write")
            }
            etas_effects::Effect::AppliedAction(instance) => {
                checked
                    .effect_registry
                    .action_name(instance.action.tag, instance.action.action)
                    == Some("stdout_write")
            }
            _ => false,
        }
    }));
    fn stdout_events(
        trace: &etas_effects::ActionTraceDomain,
        registry: &etas_effects::EffectRegistry,
    ) -> usize {
        use etas_effects::ActionTraceDomain;
        match trace {
            ActionTraceDomain::Event(event) => {
                let action = match &event.action {
                    etas_effects::Effect::Action(action) => Some(action),
                    etas_effects::Effect::AppliedAction(instance) => Some(&instance.action),
                    _ => None,
                };
                usize::from(action.is_some_and(|action| {
                    registry.tag_name(action.tag) == Some("Console")
                        && registry.action_name(action.tag, action.action) == Some("stdout_write")
                }))
            }
            ActionTraceDomain::Seq(parts) | ActionTraceDomain::Choice(parts) => {
                parts.iter().map(|part| stdout_events(part, registry)).sum()
            }
            ActionTraceDomain::Repeat(part) => stdout_events(part, registry),
            _ => 0,
        }
    }
    assert_eq!(
        stdout_events(&summary.action_trace, &checked.effect_registry),
        if independent_action { 3 } else { 2 },
        "{:?}",
        summary.action_trace
    );
    if independent_action {
        let mut input = etas_driver::load_project(
            &options,
            etas_driver::LoadProjectRequest {
                paths: vec![consumer_workspace.path().to_owned()],
                all: true,
                entry_policy: etas_frontend::EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap()
        .loaded
        .input;
        let source = input
            .environment
            .external_public_metadata
            .iter_mut()
            .flat_map(|package| &mut package.effect_summaries)
            .find(|summary| {
                summary
                    .item
                    .last()
                    .is_some_and(|name| name == "replacement")
            })
            .unwrap();
        let graph = source.returned_callables.as_mut().unwrap();
        let index = graph
            .sources
            .iter()
            .find_map(|source| match source {
                etas_package_metadata::CallableValueSource::FrozenContract(index) => Some(*index),
                _ => None,
            })
            .unwrap();
        let node = &mut graph.values[index as usize];
        node.requested_actions.effects.clear();
        node.behavior.determinism = etas_package_metadata::CallableValueDeterminism::Deterministic;
        node.behavior.independent_determinism =
            Some(etas_package_metadata::CallableValueDeterminism::Deterministic);
        node.behavior.host_requirements.clear();
        let rejected = etas_frontend::Frontend.check_project(input);
        assert!(
            rejected.checked.is_none(),
            "inconsistent direct action metadata was accepted: {:?}",
            rejected.diagnostics
        );
        assert!(
            rejected.diagnostics.iter().any(
                |diagnostic| format!("{:?}", diagnostic.code).contains("IncompleteEffectFacts")
            ),
            "{:?}",
            rejected.diagnostics
        );
    }
}

#[test]
fn metadata_only_generic_compilation_keeps_distinct_type_applications() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let boolean: CallableDef<(bool) -> bool ![]> = ^identity;
    let integer: CallableDef<(i32) -> i32 ![]> = ^identity;
    let bool_body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return -1; }, };
    let int_body = match parse_body("return 42;") { Ok(body) => body, Err(_) => { return -1; }, };
    let b = compile(boolean.with_body(bool_body));
    let n = compile(integer.with_body(int_body));
    if b(false) { return n(0); }
    return 0;
}
"#,
    ).unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"42""#), "{stdout}");
}

#[test]
fn native_compiler_profile_reflects_metadata_only_generic_dependency() {
    let (workspace, options) = generic_dependency_project();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.identity;
import std.compiler.CallableDef;
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 {
    let definition: CallableDef<(bool) -> bool ![]> = ^identity;
    return 1;
}
"#,
    )
    .unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let trace = workspace.path().join("metadata-reflection-trace.json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    let trace = fs::read_to_string(trace).unwrap_or_default();
    assert_eq!(code, 0, "{stderr}\n{stdout}\n{trace}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_reflects_metadata_only_generic_dependency_inside_revision() {
    let (workspace, options) = generic_dependency_project();
    let manifest_path = workspace.path().join("etas.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(
        &manifest_path,
        manifest.replace(
            "retain_bodies = ['app.main.Worker']",
            "retain_bodies = ['app.main.Worker', 'app.main.wrapper']",
        ),
    )
    .unwrap();
    fs::write(workspace.path().join("src/app/main.es"), r#"module app.main;
import first.library.api.identity;
import std.compiler.{compile, parse_body, CallableDef, CompileError};
agent Worker() -> i32 { return 1; }
flow wrapper<U>(value: U) -> U ![] { return value; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let selected: (bool) -> bool ![] = wrapper;
    let body = match parse_body("let definition: CallableDef<(U) -> U ![]> = ^identity; return value;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let revised = compile((^selected).with_body(body));
    if revised(true) { return 1; }
    return 0;
}
"#).unwrap();
    assert_metadata_only_project_checked(&workspace, &options);
    let trace = workspace.path().join("nested-dependency-trace.json");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--trace-out",
            path(&trace),
        ],
        "",
    );
    let trace = fs::read_to_string(trace).unwrap_or_default();
    assert_eq!(code, 0, "{stderr}\n{stdout}\n{trace}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_is_rebuilt_for_resume_before_a_compiler_request() {
    let workspace = project(
        r#"
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    checkpoint("before-compile");
    let body = match parse_body("return 42;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^Worker).with_body(body));
    return revised();
}
"#,
        "backend = 'native'\nallow_compile = true",
    );
    let checkpoints = workspace.path().join("checkpoints");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"42""#), "{stdout}");
    assert!(checkpoints.join("checkpoint-0.json").is_file());
    let (code, stdout, stderr) =
        run_process(["resume", "0", "--checkpoint-dir", path(&checkpoints)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"42""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn native_compiler_profile_keeps_check_and_compile_authority_separate() {
    let workspace = project(
        "flow main(args: Array<string>) -> i32 ![Error<CompileError>] { let report = check(^Worker); if report.status == \"accepted\" { return 1; } return 0; }",
        "backend = 'native'\nallow_check = true",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"
module app.main;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 1; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] { let candidate = compile(^Worker); return candidate(); }
"#,
    )
    .unwrap();
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(
        !stdout.contains(r#"run value: {"kind":"number""#),
        "{stdout}"
    );
    assert!(stderr.contains("AuthorityDenied"), "{stderr}");
}

#[test]
fn native_compiler_profile_does_not_grant_authority_without_explicit_permissions() {
    let workspace = project(
        "flow main(args: Array<string>) -> i32 ![Error<CompileError>] { let report = check(^Worker); if report.status == \"accepted\" { return 1; } return 0; }",
        "backend = 'native'",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(stderr.contains("AuthorityDenied"), "{stderr}");
    assert!(
        !stdout.contains(r#"run value: {"kind":"number""#),
        "{stdout}"
    );
}

#[test]
fn native_compiler_profile_reports_rejection_without_running_or_replacing_the_agent() {
    let workspace = project(
        r#"
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return -1; }, };
    let edited = (^Worker).with_body(body);
    let report = check(edited);
    if report.status != "rejected" { return -2; }
    return match compile(edited)? {
        Err(error) => { if error.kind == "rejected" { return Worker(); } return -3; },
        Ok(_) => -4,
    };
}
"#,
        "backend = 'native'\nallow_check = true\nallow_compile = true",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"1""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn native_compiler_profile_enforces_worker_limits_without_capturing_compile_error() {
    let workspace = project(
        r#"
flow main(args: Array<string>) -> i32 {
    return match check(^Worker)? {
        Err(error) => { if error.kind == "service" { return 1; } return 0; },
        Ok(_) => 0,
    };
}
"#,
        "backend = 'native'\nallow_check = true\nmax_process_peak_resident_bytes = 1",
    );
    let (code, stdout, stderr) =
        run_process(["run", path(workspace.path()), "--profile", "local"], "");
    assert_eq!(code, 3, "{stderr}\n{stdout}");
    assert!(stderr.contains("BudgetExceeded"), "{stderr}");
    assert!(!stdout.contains(r#""value":"1""#), "{stdout}");
}

#[test]
fn native_compiler_profile_merges_local_limits_without_adding_permissions() {
    let workspace = project(
        "flow main() -> unit { return; }",
        "backend = 'native'\nallow_check = true\nmax_worker_cpu_ms = 1000",
    );
    fs::write(
        workspace.path().join("etas.local.toml"),
        "[runtime.profiles.local.compiler]\nmax_worker_cpu_ms = 250\nmax_active_workers = 2\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run([
        "etas",
        "run",
        path(workspace.path()),
        "--profile",
        "local",
        "--print-runtime-profile",
    ]);
    assert_eq!(code, 0, "{stderr}");
    let profile: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(profile["compiler"]["max_worker_cpu_ms"], 250);
    assert_eq!(profile["compiler"]["max_active_workers"], 2);
    assert_eq!(profile["compiler"]["allow_check"], true);
    assert_eq!(profile["compiler"]["allow_compile"], false);
}

#[test]
fn native_compiler_profile_rejects_zero_limits_and_unknown_backends() {
    let workspace = project("flow main() -> unit { return; }", "backend = 'native'");
    for invalid in [
        "max_active_workers = 0",
        "max_retained_candidates = 0",
        "max_retained_candidate_bytes = 0",
        "max_worker_cpu_ms = 0",
        "max_process_peak_resident_bytes = 0",
        "backend = 'shell'",
    ] {
        fs::write(
            workspace.path().join("etas.local.toml"),
            format!("[runtime.profiles.local.compiler]\n{invalid}\n"),
        )
        .unwrap();
        let (code, stdout, stderr) = run([
            "etas",
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--print-runtime-profile",
        ]);
        assert_eq!(code, 2, "{invalid}: {stderr}\n{stdout}");
        assert!(stderr.contains("runtime.compiler"), "{invalid}: {stderr}");
    }
}

#[test]
fn native_compiler_completes_locked_dependency_sources_during_run_and_resume() {
    let workspace = project(
        "flow main() -> i32 { return 0; }",
        "backend = 'native'\nallow_compile = true",
    );
    let library = workspace.path().join("library");
    fs::create_dir_all(library.join("src/library")).unwrap();
    fs::write(
        library.join("etas.toml"),
        "[package]\nname = 'code-library'\nversion = '0.1.0'\nedition = '2026'\n",
    )
    .unwrap();
    fs::write(
        library.join("src/library/api.es"),
        "module library.api; public flow lookup() -> i32 ![] { return 7; }",
    )
    .unwrap();
    let options = etas_driver::DriverOptions {
        cache_mode: etas_driver::DriverCacheMode::Off,
        ..Default::default()
    };
    etas_driver::build_package_metadata(
        &options,
        etas_driver::PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    let manifest_path = workspace.path().join("etas.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(manifest_path, format!("{manifest}\n[dependencies]\nfirst = {{ package = 'code-library', version = '0.1.0', import = 'first', path = 'library' }}\n")).unwrap();
    fs::write(
        workspace.path().join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup;
import std.compiler.{compile, parse_body, CompileError};
import std.runtime.checkpoint;
agent Worker() -> i32 ![] { return 0; }
flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    checkpoint("before-compile");
    let body = match parse_body("return lookup();") { Ok(body) => body, Err(_) => { return -1; }, };
    return compile((^Worker).with_body(body))();
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &options,
        etas_driver::PackageUpdateRequest {
            package_root: workspace.path().to_owned(),
        },
    )
    .unwrap();
    let checkpoints = workspace.path().join("checkpoints");
    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(workspace.path()),
            "--profile",
            "local",
            "--checkpoint-dir",
            path(&checkpoints),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"7""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let (code, stdout, stderr) =
        run_process(["resume", "0", "--checkpoint-dir", path(&checkpoints)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(stdout.contains(r#""value":"7""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}
