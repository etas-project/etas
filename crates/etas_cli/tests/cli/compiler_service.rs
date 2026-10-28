use super::{fs, path, run, run_process};

fn project(body: &str, profile: &str) -> etas_host::TestWorkspace {
    let workspace = etas_host::TestWorkspace::create(&format!(
        "cli-native-compiler-{}",
        super::NEXT_FILE.fetch_add(1, super::Ordering::Relaxed)
    ))
    .unwrap();
    fs::create_dir_all(workspace.path().join("src/app")).unwrap();
    fs::write(workspace.path().join("etas.toml"), format!(
        "[package]\nname = 'native-compiler-app'\nversion = '0.1.0'\n[reflection]\nretain_bodies = ['app.main.Worker']\n[[bin]]\nname = 'app'\nmodule = 'app.main'\nflow = 'main'\n[runtime.profiles.local.compiler]\n{profile}\n"
    )).unwrap();
    fs::write(workspace.path().join("src/app/main.es"), format!(
        "module app.main; import std.compiler.{{check, compile, parse_body, CompileError}}; import std.runtime.checkpoint; agent Worker() -> i32 {{ return 1; }} {body}"
    )).unwrap();
    workspace
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
