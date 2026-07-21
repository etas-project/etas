use std::{
    cell::Cell,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use etas_frontend::{
    CheckRequest, FrontendSession, ModulePath, ProjectEntry, ProjectSourceLoadOptions,
    ProjectSourceLoadScope, ProjectSourceLoader,
};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);
static CLI_PROCESS_LOCK: Mutex<()> = Mutex::new(());
static MODEL_CLI_E2E_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static CLI_PROCESS_LOCK_HELD: Cell<usize> = const { Cell::new(0) };
}

struct ModelCliE2eGuard {
    _model: MutexGuard<'static, ()>,
    _cli: MutexGuard<'static, ()>,
}

impl Drop for ModelCliE2eGuard {
    fn drop(&mut self) {
        CLI_PROCESS_LOCK_HELD.with(|held| {
            let count = held.get();
            debug_assert!(count > 0);
            held.set(count.saturating_sub(1));
        });
    }
}

fn lock_model_cli_e2e() -> ModelCliE2eGuard {
    let model = MODEL_CLI_E2E_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let cli = lock_cli_process();
    CLI_PROCESS_LOCK_HELD.with(|held| held.set(held.get() + 1));
    ModelCliE2eGuard {
        _model: model,
        _cli: cli,
    }
}

fn lock_cli_process() -> MutexGuard<'static, ()> {
    CLI_PROCESS_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn cli_process_lock_is_held_by_current_thread() -> bool {
    CLI_PROCESS_LOCK_HELD.with(|held| held.get() > 0)
}

#[test]
fn check_success_exits_zero() {
    let file = fixture("ok", "flow main() -> unit { return; }");
    let (code, stdout, stderr) = run(["etas", "check", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.is_empty());
}

#[test]
fn check_with_syntax_error_exits_with_diagnostics() {
    let file = fixture("broken", "flow broken() -> unit {\n  let value = ;\n");
    let (code, stdout, stderr) = run(["etas", "check", path(&file)]);

    assert_eq!(code, 1);
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.contains("error[syntax::"));
}

#[test]
fn run_max_call_depth_returns_runtime_diagnostic() {
    let file = fixture(
        "max-call-depth",
        r#"
module tests.cli.max_call_depth;

flow recurse(value: i32) -> i32 {
  return 1 + recurse(value + 1);
}

flow main(args: Array<string>) -> i32 {
  return recurse(0);
}
"#,
    );
    let (code, _stdout, stderr) = run(["etas", "run", path(&file), "--max-call-depth", "32"]);

    assert_eq!(code, 3, "{stderr}");
    assert!(
        stderr.contains("maximum interpreter call depth (32)"),
        "{stderr}"
    );
    assert!(!stderr.contains("stack overflow"), "{stderr}");
}

#[test]
fn run_default_non_tail_recursion_returns_diagnostic_without_stack_overflow() {
    let file = fixture(
        "default-non-tail-call-depth",
        r#"
module tests.cli.default_non_tail_call_depth;

flow recurse(value: i32) -> i32 {
  return 1 + recurse(value + 1);
}

flow main(args: Array<string>) -> i32 {
  return recurse(0);
}
"#,
    );
    let (code, _stdout, stderr) = run_process(["run", path(&file)], "");

    assert_eq!(code, 3, "{stderr}");
    assert!(
        stderr.contains("maximum interpreter call depth (4096)"),
        "{stderr}"
    );
    assert!(!stderr.contains("stack overflow"), "{stderr}");
}

#[test]
fn run_default_call_depth_allows_one_hundred_non_tail_calls() {
    let file = fixture(
        "default-one-hundred-non-tail-calls",
        r#"
module tests.cli.default_one_hundred_non_tail_calls;

flow recurse(value: i32) -> i32 {
  if value >= 100 {
    return 0;
  }
  return 1 + recurse(value + 1);
}

flow main(args: Array<string>) -> i32 {
  return recurse(0);
}
"#,
    );
    let (code, stdout, stderr) = run_process(["run", path(&file)], "");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"value\":\"100\""), "{stdout}");
    assert!(!stderr.contains("stack overflow"), "{stderr}");
}

#[test]
fn run_max_call_depth_rejects_value_above_hard_cap() {
    let file = fixture("max-call-depth-hard-cap", "flow main() -> unit { return; }");
    let (code, _stdout, stderr) = run(["etas", "run", path(&file), "--max-call-depth", "65537"]);

    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("hard cap 65536"), "{stderr}");
}

#[test]
fn run_decodes_http_chunked_response_body() {
    let file = fixture(
        "http-chunked-decode",
        r#"
module tests.cli.http_chunked_decode;
import std.bytes.len;
import std.codec.text.utf8_encode;
import std.http.codec.decode_response;

flow main(args: Array<string>) -> i32 ![] {
  let decoded_len = match decode_response(utf8_encode("HTTP/1.1 200 OK\nTransfer-Encoding: chunked\n\n5\nhello\n6;ext=value\n world\n0\n\n")) {
    Ok(response) => len(response.body),
    Err(_) => len(utf8_encode(""))
  };
  if decoded_len == 11 { return 0; }
  return 1;
}
"#,
    );
    let (code, stdout, stderr) = run(["etas", "run", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(r#""value":"0""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_tls_failure_reports_host_error_without_panic() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("local addr");
    listener
        .set_nonblocking(true)
        .expect("set local listener nonblocking");
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > deadline {
                        return;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => return,
            }
        }
    });
    let file = fixture(
        "tls-failure",
        &format!(
            r#"
module tests.cli.tls_failure;
import std.net.tcp.{{Host as TcpHost, NetworkError, Port, TcpOptions, connect as tcp_connect}};
import std.tls.{{Host as TlsHost, TlsConfig, TlsError, connect as tls_connect}};

flow main(args: Array<string>) -> i32 ![Error<NetworkError>, Error<TlsError>] {{
  let stream = tcp_connect(TcpHost {{ host = "127.0.0.1" }}, Port {{ port = {} }}, TcpOptions {{}});
  let tls = tls_connect(stream, TlsHost {{ host = "127.0.0.1" }}, TlsConfig {{}});
  return 0;
}}
"#,
            address.port()
        ),
    );
    let allow_net = format!("127.0.0.1:{}", address.port());
    let (code, _stdout, stderr) = run([
        "etas",
        "run",
        path(&file),
        "--allow-effects",
        "--allow-net",
        &allow_net,
    ]);

    server.join().expect("server should finish");
    assert_ne!(code, 0, "{stderr}");
    assert!(stderr.contains("TLS"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn profile_check_writes_structured_report() {
    let file = fixture("profile-check", "flow main() -> unit { return; }");
    let profile = temp_path("profile-check", "json");
    let (code, stdout, stderr) = run([
        "etas",
        "--profile-out",
        path(&profile),
        "check",
        path(&file),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&profile);
    assert_eq!(report["schema"], "etas.profile.v1");
    assert_eq!(report["command"], "check");
    assert_eq!(report["status"], "ok");
    assert_profile_has_span(&report, "cli.check.total");
    assert_profile_has_span(&report, "frontend.check");
    assert_profile_has_span(&report, "effects.solve_summaries");
    assert_profile_has_counter(&report, "frontend.pipeline.records");
    assert_profile_has_counter(&report, "effects.pass.effects.solve_summaries.duration_ns");
    assert!(
        !profile_contains(&profile, "flow main"),
        "profile leaked source"
    );
}

#[test]
fn profile_tree_dumps_stage_timing_to_stderr() {
    let file = fixture("profile-tree", "flow main() -> unit { return; }");
    let (code, stdout, stderr) = run(["etas", "--profile-tree", "check", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.contains("profile check status=ok"), "{stderr}");
    assert!(stderr.contains("cli.check.total"), "{stderr}");
    assert!(stderr.contains("frontend.check"), "{stderr}");
    assert!(stderr.contains("pipeline passes:"), "{stderr}");
    assert!(stderr.contains("use --profile-detail"), "{stderr}");
    assert!(
        stderr.contains("use --profile-pass-timing for details"),
        "{stderr}"
    );
    assert!(!stderr.contains("BuildSourceSetPass "), "{stderr}");
}

#[test]
fn profile_tree_detail_flag_dumps_nested_spans() {
    let file = fixture("profile-tree-detail", "flow main() -> unit { return; }");
    let (code, stdout, stderr) = run([
        "etas",
        "--profile-tree",
        "--profile-detail",
        "check",
        path(&file),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.contains("profile check status=ok"), "{stderr}");
    assert!(stderr.contains("driver.resolve_project_inputs"), "{stderr}");
    assert!(!stderr.contains("use --profile-detail"), "{stderr}");
}

#[test]
fn profile_tree_pass_timing_flag_dumps_pass_details() {
    let file = fixture(
        "profile-tree-pass-timing",
        "flow main() -> unit { return; }",
    );
    let (code, stdout, stderr) = run([
        "etas",
        "--profile-tree",
        "--profile-pass-timing",
        "check",
        path(&file),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.contains("pipeline passes"), "{stderr}");
    assert!(!stderr.contains("use --profile-pass-timing"), "{stderr}");
    assert!(stderr.contains("BuildSourceSetPass "), "{stderr}");
}

#[test]
fn profile_run_failure_still_writes_report_without_payload() {
    let file = fixture(
        "profile-run-failure",
        "flow broken() -> unit {\n  let secret_payload = ;\n",
    );
    let profile = temp_path("profile-run-failure", "json");
    let (code, _stdout, stderr) =
        run(["etas", "--profile-out", path(&profile), "run", path(&file)]);

    assert_ne!(code, 0);
    assert!(stderr.contains("error[syntax::"), "{stderr}");
    let report = read_profile_report(&profile);
    assert_eq!(report["schema"], "etas.profile.v1");
    assert_eq!(report["command"], "run");
    assert_eq!(report["status"], "error");
    assert_profile_has_span(&report, "cli.run.total");
    assert!(!profile_contains(&profile, "secret_payload"));
}

#[test]
fn profile_run_success_records_interpreter_eval() {
    let file = fixture(
        "profile-run-success",
        "flow main(args: Array<string>) -> i32 { return 0; }",
    );
    let profile = temp_path("profile-run-success", "json");
    let (code, stdout, stderr) = run(["etas", "--profile-out", path(&profile), "run", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&profile);
    assert_eq!(report["schema"], "etas.profile.v1");
    assert_eq!(report["command"], "run");
    assert_eq!(report["status"], "ok");
    assert_profile_has_span(&report, "cli.run.total");
    assert_profile_has_span(&report, "interpreter.eval");
}

#[test]
fn implicit_run_accepts_path_like_file_input() {
    let file = fixture(
        "implicit-run-file",
        "flow main(args: Array<string>) -> i32 { return 0; }",
    );
    let (code, stdout, stderr) = run(["etas", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn implicit_run_accepts_current_directory_input() {
    let root = package_fixture(
        "implicit-run-dir",
        &[(
            "src/app/main.es",
            r#"module app.main;

flow main(args: Array<string>) -> i32 {
    return 0;
}
"#,
        )],
    );
    let (code, stdout, stderr) = run_process_in(&root, ["."], "");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn unknown_bare_word_does_not_fall_back_to_run() {
    let (code, stdout, stderr) = run(["etas", "effetcs", "."]);

    assert_ne!(code, 0);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("unrecognized subcommand"), "{stderr}");
    assert!(stderr.contains("effects"), "{stderr}");
    assert!(
        !stderr.contains("module declaration does not match source path"),
        "{stderr}"
    );
}

#[test]
fn singular_effect_is_unknown_command_not_implicit_run() {
    let (code, stdout, stderr) = run(["etas", "effect", "."]);

    assert_ne!(code, 0);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("unrecognized subcommand"), "{stderr}");
    assert!(stderr.contains("effects"), "{stderr}");
    assert!(
        !stderr.contains("module declaration does not match source path"),
        "{stderr}"
    );
}

#[test]
fn check_runtime_required_is_non_blocking_without_phase1_flag() {
    let file = fixture(
        "runtime-required",
        r#"module tests.cli.runtime_required;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    println("status");
    return 0;
}"#,
    );
    let (code, stdout, stderr) = run(["etas", "check", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "-v", "check", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_phase1_runtime_required_exits_with_diagnostics() {
    let file = fixture(
        "runtime-required-phase1",
        r#"module tests.cli.runtime_required_phase1;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    println("status");
    return 0;
}"#,
    );
    let (code, stdout, stderr) = run(["etas", "check", "--phase1", path(&file)]);

    assert_eq!(code, 1);
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.contains("RuntimeRequiredInPhase1"));
}

#[test]
fn run_with_boundary_policy_denies_console_by_default() {
    let file = fixture(
        "boundary-policy-deny-console",
        r#"module tests.cli.boundary_policy_deny_console;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-boundary-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 3, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(
        stderr.contains("console policy denied request"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
    assert!(
        stderr.contains("CLI policy provider denies unknown policy references"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
}

#[test]
fn run_with_http_boundary_policy_denies_console() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "boundary-policy-http-deny-console",
        r#"module tests.cli.boundary_policy_http_deny_console;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let (policy_url, policy_server) = spawn_policy_server(
        serde_json::json!({
            "decision": "deny",
            "reason": "http policy denied console",
        })
        .to_string(),
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "http");
    command.env("ETAS_HOST_POLICY_URL", policy_url.clone());
    command.env("ETAS_HOST_POLICY_ALLOW_PRIVATE", "true");
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-http-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");
    assert_eq!(code, 3, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(
        stderr.contains("console policy denied request: http policy denied console"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
    shutdown_mock_server(&policy_url);
    let request = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert!(
        request.contains("/policy/evaluate"),
        "policy request should use HTTP policy provider:\n{request}"
    );
    assert!(
        request.contains("\"policy_ref\":\"cli-http-policy\""),
        "policy request should carry the configured boundary policy ref:\n{request}"
    );
    assert!(
        request.contains("\"active_trace_specs\":[\"Gate\"]"),
        "policy request should carry source active trace specs:\n{request}"
    );
    assert!(
        request.contains("\"trace_spec_facts\"")
            && request.contains("\"kind\":\"trace_spec_reference\"")
            && request.contains("\"name\":\"Gate\"")
            && request.contains("\"kind\":\"allow\"")
            && request.contains("\"target_label\":\"Console\""),
        "policy request should carry materialized source trace spec facts, not only local policy names:\n{request}"
    );
    assert!(
        request.contains("\"kind\":\"console\""),
        "policy request should describe the console boundary subject:\n{request}"
    );
}

#[test]
fn run_with_http_boundary_policy_does_not_invent_unreferenced_active_trace_specs() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "boundary-policy-http-no-invented-policy",
        r#"module tests.cli.boundary_policy_http_no_invented_policy;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    println("status");
    return 0;
}
"#,
    );
    let (policy_url, policy_server) = spawn_policy_server(
        serde_json::json!({
            "decision": "deny",
            "reason": "http policy denied console",
        })
        .to_string(),
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "http");
    command.env("ETAS_HOST_POLICY_URL", policy_url.clone());
    command.env("ETAS_HOST_POLICY_ALLOW_PRIVATE", "true");
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-http-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");
    assert_eq!(code, 3, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(
        stderr.contains("console policy denied request: http policy denied console"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
    shutdown_mock_server(&policy_url);
    let request = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert!(
        request.contains("\"active_trace_specs\":[]"),
        "unreferenced trace spec declarations must not become active runtime policy:\n{request}"
    );
    assert!(
        request.contains("\"trace_spec_facts\":[]"),
        "unreferenced trace spec declarations must not export runtime policy facts:\n{request}"
    );
}

#[test]
fn run_with_trace_spec_runtime_allows_matching_console_boundary() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "trace-spec-runtime-allows-console",
        r#"module tests.cli.trace_spec_runtime_allows_console;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "unsafe-trace-spec-runtime");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 0, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.contains("status"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_trace_spec_runtime_uses_configured_http_provider() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "trace-spec-runtime-http-provider",
        r#"module tests.cli.trace_spec_runtime_http_provider;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let (policy_url, policy_server) = spawn_policy_server(
        serde_json::json!({
            "decision": "allow",
        })
        .to_string(),
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "http");
    command.env("ETAS_HOST_POLICY_URL", policy_url.clone());
    command.env("ETAS_HOST_POLICY_ALLOW_PRIVATE", "true");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");
    shutdown_mock_server(&policy_url);
    let request = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert_eq!(code, 0, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.contains("status"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        request.contains("\"policy_ref\":\"etas.trace-spec-runtime\""),
        "trace spec runtime must be evaluated by the configured HTTP provider:\n{request}"
    );
    assert!(
        request.contains("\"active_trace_specs\":[\"Gate\"]")
            && request.contains("\"trace_spec_facts\""),
        "HTTP trace spec runtime request should carry active policy facts:\n{request}"
    );
}

#[test]
fn run_with_local_static_policy_does_not_interpret_trace_spec_runtime() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "trace-spec-runtime-not-local-static",
        r#"module tests.cli.trace_spec_runtime_not_local_static;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "local-static");
    command.env("ETAS_HOST_POLICY_RULES", "model=allow");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 3, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("console policy denied request: local static policy has no allow rule for console boundary"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
}

#[test]
fn run_with_local_static_policy_denies_unmatched_boundary() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "boundary-policy-local-static-deny-console",
        r#"module tests.cli.boundary_policy_local_static_deny_console;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "local-static");
    command.env("ETAS_HOST_POLICY_RULES", "model=allow");
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-local-static-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 3, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("console policy denied request: local static policy has no allow rule for console boundary"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
}

#[test]
fn run_with_local_static_policy_can_require_approval() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "boundary-policy-local-static-approval-console",
        r#"module tests.cli.boundary_policy_local_static_approval_console;

import std.io.println;

spec Gate: trace = +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
    ~ Gate
{
    println("status");
    return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "local-static");
    command.env("ETAS_HOST_POLICY_RULES", "console=approval");
    command.env("ETAS_HOST_APPROVAL", "auto");
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-local-static-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 0, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.contains("status"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_rejects_non_unsafe_local_policy_alias() {
    let _guard = lock_model_cli_e2e();
    let file = fixture(
        "boundary-policy-old-alias",
        "flow main(args: Array<string>) -> i32 { return 0; }",
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_POLICY", "require-approval-local-static");
    command.env("ETAS_HOST_BOUNDARY_POLICY", "cli-local-static-policy");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_ne!(code, 0, "stdout={stdout:?}\nstderr={stderr:?}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("unsupported ETAS_HOST_POLICY mode `require-approval-local-static`"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
}

#[test]
fn dump_hir_exits_zero_and_writes_artifact() {
    let file = fixture("ok-hir", "flow main() -> unit { return; }");
    let (code, stdout, stderr) = run(["etas", "dump", "hir", path(&file), "--format", "text"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("HirProgram"));
    assert!(stderr.is_empty());
}

#[test]
fn std_io_checks_and_runs_without_explicit_imports_for_core_prelude() {
    let file = fixture(
        "std-io-core-prelude",
        r#"
module tests.cli.std_io_core_prelude;

import std.io.{read_line, println};

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(input);
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&file)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run_process(["run", path(&file)], "from-prelude");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "from-prelude\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_captures_std_io_error_effect_with_postfix_try() {
    let file = fixture(
        "std-io-try",
        r#"
module tests.cli.std_io_try;

import std.io.{read_line, println};

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let _input = read_line()?;
    println("captured");
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run_process(["run", path(&file)], "hello");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "captured\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_discovers_imported_package_sources_from_main_file() {
    let root = import_discovery_package("check-import-discovery");
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run(["etas", "--workspace", path(&root), "check", path(&main)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_does_not_discover_imports_outside_workspace_boundary() {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!("etas-cli-import-boundary-{id}"));
    let workspace = base.join("workspace");
    let app = workspace.join("src").join("app");
    let outside = base.join("leak");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let main = app.join("main.es");
    fs::write(
        &main,
        r#"module app.main;

import leak.support.{helper};

flow main() -> unit {
    helper();
    return;
}
"#,
    )
    .unwrap();
    fs::write(
        outside.join("support.es"),
        r#"module leak.support;

public flow helper() -> unit {
    return;
}
"#,
    )
    .unwrap();

    let (code, stdout, stderr) = run([
        "etas",
        "--workspace",
        path(&workspace),
        "check",
        path(&main),
    ]);

    assert_eq!(code, 1);
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(
        stderr.contains("missing imported module `leak.support`"),
        "{stderr}"
    );
}

#[test]
fn check_source_directory_discovers_sources_without_all() {
    let root = package_fixture(
        "source-dir",
        &[
            (
                "app/main.es",
                r#"module app.main;

flow main() -> unit {
    return;
}
"#,
            ),
            (
                "app/support.es",
                r#"module app.support;

public flow helper() -> unit {
    return;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_without_input_uses_current_directory_as_source_root() {
    let root = package_fixture(
        "cwd-source-root",
        &[(
            "app/main.es",
            r#"module app.main;

flow main() -> unit {
    return;
}
"#,
        )],
    );

    let (code, stdout, stderr) = run_process_in(&root, ["check"], "");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_explicit_source_directory_uses_that_directory_as_source_root() {
    let root = package_fixture(
        "explicit-source-root",
        &[
            (
                "etas.toml",
                "this file is package-manager input and is ignored by source loading",
            ),
            (
                "sources/app/main.es",
                r#"module app.main;

flow main() -> unit {
    return;
}
"#,
            ),
            (
                "sources/app/support.es",
                r#"module app.support;

public flow helper() -> unit {
    return;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root.join("sources"))]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_project_uses_driver_manifest_tool_binding() {
    let root = package_fixture(
        "driver-manifest-tool-binding",
        &[
            (
                "etas.toml",
                r#"[package]
name = "manifest-tool-binding"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "manifest-tool-binding"
module = "app.main"
flow = "main"

[bindings.tools]
"app.main.search" = { kind = "mcp", server = "browser", tool = "search", effects = ["Network"] }
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

tool search(q: string) -> string ![Network];

flow main(q: string) -> string ![Network] {
    return search(q);
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn package_flow_override_keeps_selected_bin_module() {
    let root = package_fixture(
        "package-flow-override-module",
        &[
            (
                "etas.toml",
                r#"[package]
name = "flow-override-module"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "flow-override-module"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    println("default");
    return 0;
}

flow alt(args: Array<string>) -> i32 ![Error<IOError>]
{
    println("selected-bin-module");
    return 0;
}
"#,
            ),
            (
                "src/app/other.es",
                r#"module app.other;

import std.io.println;

flow alt(args: Array<string>) -> i32 ![Error<IOError>]
{
    println("wrong-module");
    return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run_process(["run", path(&root), "--flow", "alt"], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "selected-bin-module\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn package_flow_override_without_selected_bin_is_rejected() {
    let root = package_fixture(
        "package-flow-override-no-bin",
        &[
            (
                "etas.toml",
                r#"[package]
name = "flow-override-no-bin"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

flow main() -> i32
{
    return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "run", path(&root), "--flow", "main"]);

    assert_eq!(code, 2);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("has no selected [[bin]] module for --flow `main`"),
        "{stderr}"
    );
}

#[test]
fn pkg_lock_writes_lockfile_for_builtin_dependencies() {
    let root = package_fixture(
        "pkg-lock-std",
        &[(
            "etas.toml",
            r#"[package]
name = "pkg-lock-std"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[dependencies]
std = { version = "0.1" }
"#,
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");
    assert!(stdout.contains("1 locked package"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
    assert!(lockfile.contains("name = \"std\""), "{lockfile}");
    assert!(lockfile.contains("source = \"builtin\""), "{lockfile}");
}

#[test]
fn profile_pkg_update_writes_package_spans() {
    let root = package_fixture(
        "profile-pkg-update",
        &[(
            "etas.toml",
            r#"[package]
name = "profile-pkg-update"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[dependencies]
std = { version = "0.1" }
"#,
        )],
    );
    let profile = temp_path("profile-pkg-update", "json");

    let (code, stdout, stderr) = run([
        "etas",
        "--profile-out",
        path(&profile),
        "pkg",
        "update",
        path(&root),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&profile);
    assert_eq!(report["schema"], "etas.profile.v1");
    assert_eq!(report["command"], "pkg");
    assert_eq!(report["status"], "ok");
    assert_profile_has_span(&report, "cli.pkg.total");
    assert_profile_has_span(&report, "package.update");
    assert_profile_has_span(&report, "package.materialize");
}

#[test]
fn pkg_lock_fails_closed_for_unmaterialized_dependencies() {
    let root = package_fixture(
        "pkg-lock-unresolved",
        &[(
            "etas.toml",
            r#"[package]
name = "pkg-lock-unresolved"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);

    assert_eq!(code, 2);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("dependency `tools` cannot be materialized")
            && stderr.contains("supported sources are"),
        "{stderr}"
    );
    assert!(!root.join("etas.lock").exists());
}

#[test]
fn pkg_update_materializes_supported_sources_and_writes_lockfile() {
    let dependency = package_fixture(
        "pkg-update-path-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public flow answer() -> i32 {
    return 42;
}
"#,
            ),
            (
                ".etas/package-index.json",
                r#"{
  "version": 1,
  "package": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" },
  "public_metadata": {
    "modules": [
      { "id": 1, "path": ["tools"], "exports": [] }
    ]
  }
}
"#,
            ),
        ],
    );
    seal_package_metadata(&dependency);
    let root = package_fixture(
        "pkg-update-path-root",
        &[(
            "etas.toml",
            &format!(
                r#"[package]
name = "pkg-update-path-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let package_index = fs::read_to_string(root.join(".etas/package-index.json")).unwrap();
    assert!(
        package_index.contains("\"name\": \"agent-tools\""),
        "{package_index}"
    );
    assert!(
        package_index.contains("\"kind\": \"vendor\""),
        "{package_index}"
    );
    let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
    assert!(lockfile.contains("name = \"agent-tools\""), "{lockfile}");
    assert!(lockfile.contains("source = \"vendor+"), "{lockfile}");
}

#[test]
fn pkg_update_then_check_consumes_dependency_import() {
    let dependency = package_fixture(
        "pkg-update-check-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "answer"
module = "tools"
flow = "answer"
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public flow answer() -> i32 {
    return 42;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-update-check-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-update-check-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import tools.{answer};

public flow main(args: Array<string>) -> i32 {
    return answer();
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "run", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(r#""value":"42""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_loads_only_entry_reachable_dependency_runtime_source() {
    let dependency = package_fixture(
        "runtime-reachable-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-http"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/edk/http/api.es",
                r#"module edk.http.api;

import edk.http.transport.{status};

public flow get() -> i32 {
    return status();
}
"#,
            ),
            (
                "src/edk/http/transport.es",
                r#"module edk.http.transport;

import edk.http.url.parse.{code};

public flow status() -> i32 {
    return code();
}
"#,
            ),
            (
                "src/edk/http/url/parse.es",
                r#"module edk.http.url.parse;

public flow code() -> i32 {
    return 204;
}
"#,
            ),
            (
                "src/edk/http/mocks/server.es",
                "module edk.http.mocks.server;\npublic flow unused_mock() -> i32 { return 501; }\n",
            ),
            (
                "src/edk/http/package_smoke.es",
                "module edk.http.package_smoke;\npublic flow unused_smoke() -> i32 { return 502; }\n",
            ),
            (
                "src/edk/http/package_api_contract.es",
                "module edk.http.package_api_contract;\npublic flow unused_contract() -> i32 { return 503; }\n",
            ),
            (
                ".etas/package-index.json",
                r#"{
  "version": 1,
  "package": { "name": "edk-http", "version": "0.1.0", "edition": "2026" },
  "public_metadata": {
    "modules": [
      { "id": 1, "path": ["edk", "http", "api"], "exports": [] }
    ],
    "flows": [
      {
        "path": ["edk", "http", "api", "get"],
        "param_names": [],
        "params": [],
        "output": { "kind": "primitive", "name": "i32" },
        "visibility": "public"
      }
    ],
    "effect_summaries": [
      {
        "item": ["edk", "http", "api", "get"],
        "public_effects": [],
        "requested_actions": [],
        "handled_requested_actions": [],
        "latent_flows": []
      }
    ]
  }
}
"#,
            ),
        ],
    );
    seal_package_metadata(&dependency);
    let root = package_fixture(
        "runtime-reachable-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "runtime-reachable-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
edk_http = {{ package = "edk-http", version = "0.1", import = "edk.http", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import edk.http.api.{get};

flow main(args: Array<string>) -> i32 {
    return get();
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let check_profile = temp_path("runtime-reachable-check", "json");
    let (code, stdout, stderr) = run([
        "etas",
        "--profile-out",
        path(&check_profile),
        "check",
        path(&root),
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&check_profile);
    assert_eq!(
        max_profile_counter(&report, "frontend.total_source_files"),
        Some(1),
        "check should stay metadata-only for dependencies: {report:#}"
    );

    let effects_profile = temp_path("runtime-reachable-effects", "json");
    let (code, stdout, stderr) = run([
        "etas",
        "--profile-out",
        path(&effects_profile),
        "effects",
        path(&root),
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("flow main"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&effects_profile);
    assert_eq!(
        max_profile_counter(&report, "frontend.total_source_files"),
        Some(1),
        "effects should stay metadata-only for dependencies: {report:#}"
    );

    let profile = temp_path("runtime-reachable-run", "json");
    let (code, stdout, stderr) = run(["etas", "--profile-out", path(&profile), "run", path(&root)]);
    assert_eq!(code, 0, "{stderr}\nstdout={stdout}");
    assert!(stdout.contains(r#""value":"204""#), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let report = read_profile_report(&profile);
    assert_eq!(
        max_profile_counter(&report, "frontend.total_source_files"),
        Some(4),
        "run should load root main plus api/transport/url.parse only: {report:#}"
    );
    assert_profile_category_nested_under(&report, "effects", "frontend.check");
}

#[test]
fn interpreter_stress_large_linear_flow_check_and_run_without_stack_overflow() {
    assert_interpreter_stress_project("large_linear_flow");
}

#[test]
fn interpreter_stress_wide_literals_check_and_run_without_stack_overflow() {
    assert_interpreter_stress_project("wide_literals");
}

#[test]
fn interpreter_stress_deep_expression_load_check_and_run_without_stack_overflow() {
    assert_interpreter_stress_project("deep_expression_load");
}

fn assert_interpreter_stress_project(name: &str) {
    let source = interpreter_stress_project(name);
    let project = copy_project_fixture(&format!("interpreter-stress-{name}"), &source);

    let (code, stdout, stderr) = run_process(["check", path(&project)], "");
    assert_no_process_stack_overflow(&stdout, &stderr);
    assert_eq!(
        code, 0,
        "`etas check` failed for interpreter stress fixture `{name}`\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let (code, stdout, stderr) = run_process(["run", path(&project)], "");
    assert_no_process_stack_overflow(&stdout, &stderr);
    assert_eq!(
        code, 0,
        "`etas run` failed for interpreter stress fixture `{name}`\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"0"}"#),
        "`etas run` returned an unexpected value for interpreter stress fixture `{name}`\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn assert_no_process_stack_overflow(stdout: &str, stderr: &str) {
    let combined = format!("{stdout}\n{stderr}");
    for needle in ["has overflowed its stack", "fatal runtime error", "SIGABRT"] {
        assert!(
            !combined.contains(needle),
            "process output contained crash marker `{needle}`\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn pkg_update_closes_local_path_dependency_metadata_before_check_all() {
    let dependency = package_fixture(
        "pkg-update-edk-like-http-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-http"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/edk/http/client.es",
                r#"module edk.http.client;

public flow answer() -> i32 {
    return 42;
}
"#,
            ),
        ],
    );
    assert!(!dependency.join(".etas/package.etasmeta").exists());

    let root = package_fixture(
        "pkg-update-edk-like-vector-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "edk-vector"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "edk.vector.main"
flow = "main"

[dependencies]
edk_http = {{ package = "edk-http", version = "0.1", import = "edk.http", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/edk/vector/main.es",
                r#"module edk.vector.main;

import edk.http.client.{answer};

public flow main(args: Array<string>) -> i32 {
    return answer();
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_ne!(code, 0);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("etas.lock is required"), "{stderr}");
    assert!(stderr.contains("etas pkg lock"), "{stderr}");
    assert!(!stderr.contains("<metadata package>"), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(dependency.join(".etas/package.etasmeta").exists());
    assert!(root.join(".etas/package-index.json").exists());
    assert!(root.join("etas.lock").exists());

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, _stdout, stderr) = run(["etas", "check", "--all", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn pkg_lock_materializes_local_path_dependency_metadata_before_check_all() {
    let dependency = package_fixture(
        "pkg-lock-edk-like-http-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-http"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/edk/http/client.es",
                r#"module edk.http.client;

public flow answer() -> i32 {
    return 42;
}
"#,
            ),
        ],
    );
    assert!(!dependency.join(".etas/package.etasmeta").exists());

    let root = package_fixture(
        "pkg-lock-edk-like-vector-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "edk-vector"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "edk.vector.main"
flow = "main"

[dependencies]
edk_http = {{ package = "edk-http", version = "0.1", import = "edk.http", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/edk/vector/main.es",
                r#"module edk.vector.main;

import edk.http.client.{answer};

public flow main(args: Array<string>) -> i32 {
    return answer();
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(dependency.join(".etas/package.etasmeta").exists());
    assert!(root.join(".etas/package-index.json").exists());
    assert!(root.join("etas.lock").exists());

    let (code, stdout, stderr) = run(["etas", "check", "--all", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn pkg_update_then_check_consumes_dependency_typed_error_metadata() {
    let dependency = package_fixture(
        "pkg-update-typed-error-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "typed-errors"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/errors.es",
                r#"module errors;

public enum DepError {
  Failed;
}

public effect Net extends Network {
  action request(method: string, host: string, payload: string) -> unit;
}

public flow fail(err: DepError) -> never ![Error<DepError>] {
  perform Error<DepError>.raise(err);
}

public flow fail_string(err: DepError) -> string ![Error<DepError>] {
  perform Error<DepError>.raise(err);
  return "unreachable";
}

public flow ping() -> unit ![Net.request] {
  perform Net.request("GET", "api.example.com", "");
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-update-typed-error-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-update-typed-error-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
errors = {{ package = "typed-errors", version = "0.1", import = "errors", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import errors.{DepError, Net, fail, fail_string, ping};

public flow propagate(err: DepError) -> never ![Error<DepError>] {
  fail(err);
}

public flow recover(err: DepError) -> string ![] {
  return fail_string(err) with {
    Error<DepError>.raise(_caught) => {
      finish "recovered";
    }
  };
}

public flow call_remote() -> unit ![Net.request] {
  ping();
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("Error<errors.DepError>"), "{stdout}");
    assert!(stdout.contains("Net.request"), "{stdout}");
    assert!(!stdout.contains("TypeId("), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run([
        "etas",
        "--format",
        "json",
        "effects",
        path(&root),
        "--flow",
        "recover",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(!stderr.contains("error["), "{stderr}");
    let json = serde_json::from_str::<serde_json::Value>(&stdout).expect("effects json");
    let items = json["items"].as_array().expect("effects items");
    assert_eq!(items.len(), 1, "{json}");
    assert_eq!(items[0]["name"], "recover", "{json}");
    assert_eq!(
        items[0]["escaping_effects"], "[]",
        "package-defined typed error should be eliminated by handler: {json}"
    );
}

#[test]
fn pkg_update_then_effects_preserves_dependency_scoped_action_wildcards() {
    let dependency = package_fixture(
        "pkg-update-scoped-action-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "http-actions"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/http.es",
                r#"module http;

public type Request = {
  id: i32,
};

public type Response = {
  status: i32,
};

public effect EdkHttp extends Network {
  action request<Scope>(request: Request) -> Response;
}

public flow proxy(method: string, host: string) -> Response ![EdkHttp.request<_>] {
  return perform EdkHttp.request(Request { id = 1 });
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-update-scoped-action-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-update-scoped-action-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
http = {{ package = "http-actions", version = "0.1", import = "http", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import http.{EdkHttp, proxy};

public flow call_remote(method: string, host: string) -> unit ![EdkHttp.request<_>] {
  let response = proxy(method, host);
  if response.status != response.status {
    return;
  }
  return;
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("EdkHttp.request[_]"),
        "dependency effect metadata must preserve scoped wildcard action args: {stdout}"
    );
    assert!(
        !stdout.contains("EdkHttp.request]\n")
            && !stdout.contains("EdkHttp.request,")
            && !stdout.contains("EdkHttp.request]"),
        "dependency effect metadata must not collapse scoped action args into a bare action: {stdout}"
    );
}

#[test]
fn pkg_update_then_check_preserves_dependency_precise_scoped_action_args() {
    let dependency = package_fixture(
        "pkg-update-precise-scoped-action-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "precise-http-actions"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/http.es",
                r#"module http;

public type Request = {
  id: i32,
};

public type Response = {
  status: i32,
};

public type ApiScope;

public effect EdkHttp extends Network {
  action request<Scope>(request: Request) -> Response;
}

public flow get_status() -> Response ![EdkHttp.request<ApiScope>] {
  return perform EdkHttp.request<ApiScope>(Request { id = 1 });
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-update-precise-scoped-action-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-update-precise-scoped-action-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
http = {{ package = "precise-http-actions", version = "0.1", import = "http", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import http.{ApiScope, EdkHttp, get_status};

public flow call_remote() -> unit ![EdkHttp.request<ApiScope>] {
  let response = get_status();
  if response.status != response.status {
    return;
  }
  return;
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("EdkHttp.request[http.ApiScope]"),
        "dependency effect metadata must preserve precise scoped action args: {stdout}"
    );
    assert!(
        !stdout.contains("EdkHttp.request[_]"),
        "precise dependency action args must not be widened to wildcard args: {stdout}"
    );
    assert!(
        !stdout.contains("EdkHttp.request]\n")
            && !stdout.contains("EdkHttp.request,")
            && !stdout.contains("EdkHttp.request]"),
        "dependency effect metadata must not collapse precise scoped action args into a bare action: {stdout}"
    );
}

#[test]
fn pkg_update_materializes_transitive_dependency_effect_action_metadata() {
    let workspace = package_fixture(
        "pkg-update-transitive-effect-workspace",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-workspace"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/edk/workspace/types.es",
                r#"module edk.workspace.types;

public type WorkspacePath = {
  raw: string,
};
"#,
            ),
            (
                "src/edk/workspace/effects.es",
                r#"module edk.workspace.effects;

import edk.workspace.types.WorkspacePath;

public effect EdkWorkspace extends FileIO {
  action read(path: WorkspacePath) -> string;
}
"#,
            ),
            (
                "src/edk/workspace/main.es",
                r#"module edk.workspace.main;

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&workspace)]);
    assert_eq!(code, 0, "{stderr}");

    let pdf = package_fixture(
        "pkg-update-transitive-effect-pdf",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "edk-pdf"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[dependencies]
edk_workspace = {{ path = "{}", import = "edk.workspace" }}
"#,
                    workspace.display()
                ),
            ),
            (
                "src/edk/pdf/reader.es",
                r#"module edk.pdf.reader;

import edk.workspace.effects.EdkWorkspace;
import edk.workspace.types.WorkspacePath;

public effect EdkPdf extends FileIO {
  action read(path: WorkspacePath) -> unit;
}

public flow read_default() -> unit ![EdkPdf.read, EdkWorkspace.read] {
  let path = WorkspacePath { raw = "docs/sample.pdf" };
  let _bytes = perform EdkWorkspace.read(path);
  perform EdkPdf.read(path);
  return;
}
"#,
            ),
            (
                "src/edk/pdf/main.es",
                r#"module edk.pdf.main;

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );
    assert!(!pdf.join(".etas/package.etasmeta").exists());

    let root = package_fixture(
        "pkg-update-transitive-effect-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-update-transitive-effect-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
pdf = {{ package = "edk-pdf", version = "0.1", import = "edk.pdf", path = "{}" }}
"#,
                    pdf.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import edk.pdf.reader.{EdkPdf, read_default};
import edk.workspace.effects.EdkWorkspace;

public flow read_document() -> unit ![EdkPdf.read, EdkWorkspace.read] {
  read_default();
  return;
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    assert!(
        pdf.join(".etas/package.etasmeta").is_file(),
        "path dependency metadata should be materialized"
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("edk.pdf.reader.EdkPdf.read") || stdout.contains("EdkPdf.read"),
        "consumer should see the direct package effect action: {stdout}"
    );
    assert!(
        stdout.contains("edk.workspace.effects.EdkWorkspace.read")
            || stdout.contains("EdkWorkspace.read"),
        "consumer should see the transitive dependency effect action: {stdout}"
    );
    assert!(
        !stdout.contains("effect tag ") && !stdout.contains("cannot be emitted"),
        "dependency effect canonicalization must not leak unresolved tag diagnostics: {stdout}"
    );
}

#[test]
fn pkg_prepare_materializes_path_dependency_metadata() {
    let dependency = package_fixture(
        "pkg-prepare-path-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
            ),
            ("src/tools.es", "module tools;\n"),
            (
                ".etas/package-index.json",
                r#"{
  "version": 1,
  "package": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" }
}
"#,
            ),
        ],
    );
    seal_package_metadata(&dependency);
    let root = package_fixture(
        "pkg-prepare-path-root",
        &[(
            "etas.toml",
            &format!(
                r#"[package]
name = "pkg-prepare-path-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "prepare", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("materialized"), "{stdout}");
    let package_index = fs::read_to_string(root.join(".etas/package-index.json")).unwrap();
    assert!(
        package_index.contains("\"name\": \"agent-tools\""),
        "{package_index}"
    );
    assert!(
        package_index.contains("\"kind\": \"vendor\""),
        "{package_index}"
    );
    assert!(
        package_index.contains("\"checksum\": \"blake3:"),
        "{package_index}"
    );
    assert!(
        package_index.contains("\"store\": \".etas/store/packages/blake3/"),
        "{package_index}"
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");
    let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
    assert!(lockfile.contains("name = \"agent-tools\""), "{lockfile}");
    assert!(lockfile.contains("source = \"vendor+"), "{lockfile}");
    assert!(lockfile.contains("checksum = \"blake3:"), "{lockfile}");
}

#[test]
fn pkg_prepare_generates_path_dependency_metadata_without_existing_artifact() {
    let dependency = package_fixture(
        "pkg-prepare-generated-metadata-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public flow answer() -> i32 ![Network] {
    return 42;
}
"#,
            ),
        ],
    );
    let root = package_fixture(
        "pkg-prepare-generated-metadata-root",
        &[(
            "etas.toml",
            &format!(
                r#"[package]
name = "pkg-prepare-generated-metadata-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "prepare", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("materialized"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(dependency.join(".etas/package.etasmeta").exists());
    let package_index = fs::read_to_string(root.join(".etas/package-index.json")).unwrap();
    assert!(
        package_index.contains("\"name\": \"agent-tools\""),
        "{package_index}"
    );
    assert!(package_index.contains("Network"), "{package_index}");
}

#[test]
fn pkg_prepare_rebuilds_stale_path_dependency_metadata() {
    let dependency = package_fixture(
        "pkg-prepare-stale-metadata-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public flow answer() -> i32 {
    return 42;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    fs::write(
        dependency.join("src/tools.es"),
        r#"module tools;

public flow answer() -> i32 {
    return 42;
}

public flow bonus() -> i32 {
    return 7;
}
"#,
    )
    .unwrap();

    let root = package_fixture(
        "pkg-prepare-stale-metadata-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "pkg-prepare-stale-metadata-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import tools.{bonus};

flow main() -> i32 {
    return bonus();
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "prepare", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("materialized"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_executes_public_handler_value_from_path_dependency_source() {
    let dependency = package_fixture(
        "runtime-path-handler-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "gate-tools"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/gate/effects.es",
                r#"module gate.effects;

public effect Gate {
    action request(message: string) -> bool;
}

public let AutoGate: ![Gate => []] = handler {
    Gate.request(req) => {
        resume true;
    }
};

public flow ask_once() -> bool ![Gate.request] {
    return perform Gate.request("ship");
}
"#,
            ),
        ],
    );
    let root = package_fixture(
        "runtime-path-handler-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "runtime-path-handler-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
gate = {{ package = "gate-tools", version = "0.1", import = "gate", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import gate.effects.{AutoGate, ask_once};

flow main(args: Array<string>) -> i32 ![] {
    let accepted = ask_once() with AutoGate;
    if accepted {
        return 0;
    }
    return 1;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "run", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"), "{stdout}");
    assert!(stdout.contains("0"), "{stdout}");
    assert!(
        !stderr.contains("metadata-only package mode"),
        "path dependency handler should execute from source overlay, not metadata-only fallback: {stderr}"
    );
    assert!(
        !stderr.contains("UnsupportedPhase2RuntimeFeature"),
        "path dependency handler should not hit unsupported runtime fallback: {stderr}"
    );
}

#[test]
fn pkg_lock_reports_path_dependency_metadata_context_on_frontend_failure() {
    let dependency = package_fixture(
        "pkg-lock-bad-metadata-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "bad-tools"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public flow broken() -> i32 {
    return "not an integer";
}
"#,
            ),
        ],
    );
    let root = package_fixture(
        "pkg-lock-bad-metadata-root",
        &[(
            "etas.toml",
            &format!(
                r#"[package]
name = "pkg-lock-bad-metadata-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "bad-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);

    assert_ne!(code, 0);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("dependency `tools`"), "{stderr}");
    assert!(stderr.contains(path(&dependency)), "{stderr}");
    assert!(
        stderr.contains("could not produce usable package metadata"),
        "{stderr}"
    );
    assert!(stderr.contains("Type(TypeMismatch)"), "{stderr}");
}

#[test]
fn pkg_metadata_emits_frontend_package_metadata_artifact() {
    let root = package_fixture(
        "pkg-metadata",
        &[
            (
                "etas.toml",
                r#"[package]
name = "pkg-metadata"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "pkg-metadata"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

public flow main() -> i32 {
    return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "metadata", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("package.etasmeta"), "{stdout}");
    assert!(stdout.contains("blake3:"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(root.join(".etas/package.etasmeta").is_file());
    assert!(
        etas_package::read_package_metadata_artifact(&root)
            .unwrap()
            .is_some()
    );
}

#[test]
fn pkg_metadata_preserves_higher_order_function_type_effects() {
    let root = package_fixture(
        "pkg-metadata-higher-order",
        &[
            (
                "etas.toml",
                r#"[package]
name = "pkg-metadata-higher-order"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "pkg-metadata-higher-order"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

public effect Payment extends Network {
  action ping() -> unit;
}

public flow apply(callback: unit -> unit ![Network]) -> unit ![Network] {
  callback();
  return;
}

public flow main() -> i32 {
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "metadata", path(&root)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("package.etasmeta"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let index = etas_package::read_package_metadata_artifact(&root)
        .unwrap()
        .expect("metadata artifact should decode");
    let apply = index
        .public_metadata
        .flows
        .iter()
        .find(|flow| flow.path == ["app", "main", "apply"])
        .expect("public apply flow should be exported");
    let callback = apply.params.first().expect("callback input should exist");
    let etas_package::PackageTypeMetadata::Function {
        input,
        output,
        effects,
    } = callback
    else {
        panic!("callback type must be a published function type: {callback:?}");
    };
    assert!(input.is_empty(), "{callback:?}");
    assert!(matches!(
        output.as_ref(),
        etas_package::PackageTypeMetadata::Primitive { name } if name == "unit"
    ));
    let effects = effects
        .as_ref()
        .expect("callback function effect row must be preserved");
    assert!(
        effects
            .effects
            .iter()
            .any(|effect| effect.path == ["Network"]),
        "{effects:?}"
    );
}

#[test]
fn pkg_lock_then_check_consumes_dependency_higher_order_latent_effects() {
    let dependency = package_fixture(
        "pkg-lock-higher-order-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-algorithm"
version = "0.1.0"
edition = "2026"

[source]
root = "src"
"#,
            ),
            (
                "src/edk/algorithm/callbacks.es",
                r#"module edk.algorithm.callbacks;

public flow with_callback(callback: unit -> unit ![Error<IOError>]) -> unit ![Error<IOError>] {
    callback();
    return;
}
"#,
            ),
        ],
    );
    assert!(!dependency.join(".etas/package.etasmeta").exists());

    let root = package_fixture(
        "pkg-lock-higher-order-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "higher-order-consumer"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
algorithm = {{ package = "edk-algorithm", version = "0.1", import = "edk.algorithm", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import edk.algorithm.callbacks.{with_callback};
import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    with_callback(() => {
        println("callback");
        return;
    });
    return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("wrote"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(dependency.join(".etas/package.etasmeta").exists());

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("Error<std.io.IOError>"),
        "external higher-order call must realize callback effects through package metadata: {stdout}"
    );
    assert!(!stdout.contains("TypeId("), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn pkg_pack_writes_etaspkg_from_existing_metadata() {
    let root = package_fixture(
        "pkg-pack",
        &[
            (
                "etas.toml",
                r#"[package]
name = "pkg-pack"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "pkg-pack"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

public flow main() -> i32 {
    return 0;
}
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&root)]);
    assert_eq!(code, 0, "{stderr}");

    let output = root
        .join("target")
        .join("package")
        .join("pkg-pack-0.1.0.etaspkg");
    let (code, stdout, stderr) = run(["etas", "pkg", "pack", path(&root), "--out", path(&output)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("packed"), "{stdout}");
    assert!(stdout.contains("blake3:"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(output.is_file());
}

#[test]
fn cli_check_matches_direct_frontend_session_pipeline() {
    let root = package_fixture(
        "check-session-pipeline",
        &[
            (
                "src/app/main.es",
                r#"module app.main;

import app.support.{helper};

flow main() -> unit {
    helper();
    return;
}
"#,
            ),
            (
                "src/app/support.es",
                r#"module app.support;

public flow helper() -> unit {
    return;
}
"#,
            ),
        ],
    );
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run(["etas", "--workspace", path(&root), "check", path(&main)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let loaded = ProjectSourceLoader::default()
        .load(ProjectSourceLoadOptions {
            project_root: root.clone(),
            roots: vec![main],
            entry: ProjectEntry {
                module: None,
                flow: "main".to_owned(),
            },
            scope: ProjectSourceLoadScope::FullSourceTree,
            source_kind: None,
            import_search_roots: vec![root.clone(), root.join("src"), root.join("src/app")],
            external_module_roots: vec![std_module_root()],
        })
        .expect("direct frontend loader should discover the imported support source");
    let mut session = FrontendSession::new();
    let project = session.open_project(loaded.input);
    let response = session
        .check(project, CheckRequest::full_project())
        .expect("direct frontend session check should succeed");

    let direct_sources = response
        .output
        .sources
        .as_ref()
        .expect("full project check should include source detail");
    assert_eq!(direct_sources.files.len(), 2);
    assert!(response.diagnostics.diagnostics.is_empty());
}

fn std_module_root() -> ModulePath {
    ModulePath {
        segments: vec!["std".to_owned()],
    }
}

#[test]
fn dump_hir_discovers_imported_package_sources_from_main_file() {
    let root = import_discovery_package("dump-hir-import-discovery");
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run([
        "etas",
        "--workspace",
        path(&root),
        "dump",
        "hir",
        path(&main),
        "--format",
        "text",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("name=app.support"), "{stdout}");
    assert!(stdout.contains("Flow"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn effects_discovers_imported_package_sources_from_main_file() {
    let root = import_discovery_package("effects-import-discovery");
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run([
        "etas",
        "--workspace",
        path(&root),
        "effects",
        path(&main),
        "--flow",
        "main",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("effects 1 item"), "{stdout}");
    assert!(stdout.contains("flow main"), "{stdout}");
    assert!(stdout.contains("Console"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn effects_reports_source_import_flow_requirements_from_project_facts() {
    let root = package_fixture(
        "effects-source-import-requirements",
        &[
            (
                "src/app/main.es",
                r#"module app.main;

import app.support.{helper};

flow main(q: string) -> i32 ![Error<IOError>] {
    helper(q);
    return 0;
}
"#,
            ),
            (
                "src/app/support.es",
                r#"module app.support;

import std.io.println;

public flow helper(q: string) -> unit ![Error<IOError>] {
    println(q);
    return;
}
"#,
            ),
        ],
    );
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run([
        "etas",
        "--workspace",
        path(&root),
        "effects",
        path(&main),
        "--flow",
        "main",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("effects 1 item"), "{stdout}");
    assert!(stdout.contains("flow main"), "{stdout}");
    assert!(stdout.contains("Console.stdout_write"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn effects_reports_trace_spec_obligations_from_split_summary() {
    let file = fixture(
        "effects-trace-spec-obligations",
        r#"module tests.cli.effects_trace_spec_obligations;

spec Gate: trace = -Command;

flow main() -> unit
    ~ Gate
{
    return;
}
"#,
    );

    let (code, stdout, stderr) = run(["etas", "effects", path(&file), "--flow", "main"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("trace spec obligations:"), "{stdout}");
    assert!(stdout.contains("trace spec Gate"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_discovers_module_part_siblings() {
    let root = package_fixture(
        "module-parts",
        &[
            (
                "src/app/main.es",
                r#"module app.main;

import std.io.println;
import app.feature.{message};

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    println(message());
    return 0;
}
"#,
            ),
            (
                "src/app/feature/mod.es",
                r#"module app.feature;

public flow message() -> string {
    return extra();
}
"#,
            ),
            (
                "src/app/feature/extra.es",
                r#"module app.feature;

public flow extra() -> string {
    return "part-loaded";
}
"#,
            ),
        ],
    );
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run_process(["--workspace", path(&root), "run", path(&main)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "part-loaded\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_source_directory_uses_default_main_entry() {
    let root = package_fixture(
        "source-dir-run",
        &[
            (
                "app/main.es",
                r#"module app.main;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    println("source-entry");
    return 0;
}
"#,
            ),
            (
                "app/other.es",
                r#"module app.other;

flow not_the_entry() -> i32 {
    return 1;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run_process(["run", path(&root)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "source-entry\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn unsupported_air_dump_has_stable_internal_error_exit() {
    let file = fixture("empty-air", "");
    let (code, stdout, stderr) = run(["etas", "dump", "air", path(&file)]);

    assert_eq!(code, 4);
    assert!(stdout.is_empty());
    assert!(stderr.contains("`etas dump air` is not available yet"));
}

#[test]
fn watch_has_stable_unsupported_session_boundary() {
    let file = fixture("watch-boundary", "flow main() -> unit { return; }");
    let (code, stdout, stderr) = run(["etas", "watch", path(&file)]);

    assert_eq!(code, 4);
    assert!(stdout.is_empty());
    assert!(stderr.contains("`etas watch` is not available yet"));
    assert!(stderr.contains("one live FrontendSession"));
    assert!(stderr.contains("apply_changes"));
}

#[test]
fn run_executes_algorithm_fixtures_with_command_io() {
    run_executes_compiler_fixtures_with_command_io("algorithms/positive", 30);
}

#[test]
fn run_executes_functional_fixtures_with_command_io() {
    run_executes_compiler_fixtures_with_command_io("functional/positive", 10);
}

fn run_executes_compiler_fixtures_with_command_io(relative: &str, expected_count: usize) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler");
    let fixture_dir = fixtures.join(relative);
    let mut sources = fs::read_dir(&fixture_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "es"))
        .collect::<Vec<_>>();
    sources.sort();

    assert_eq!(sources.len(), expected_count);
    let mut failures = Vec::new();

    for source in sources {
        let io_contract = source.with_extension("io.txt");
        let io_text = fs::read_to_string(&io_contract).unwrap();
        let stdin = io_section(&io_text, "stdin:", "stdout:");
        let expected_stdout = io_section(&io_text, "stdout:", "exit:");
        let expected_exit = io_section(&io_text, "exit:", "__end__")
            .trim()
            .parse::<i32>()
            .expect("fixture exit section should be an integer");

        let _guard = (!cli_process_lock_is_held_by_current_thread()).then(lock_cli_process);
        let mut child = Command::new(env!("CARGO_BIN_EXE_etas"))
            .arg("--workspace")
            .arg(&fixtures)
            .arg("run")
            .arg(&source)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        let mut fixture_failures = Vec::new();
        if output.status.code() != Some(expected_exit) {
            fixture_failures.push(format!(
                "exit status {:?} did not match expected {expected_exit}",
                output.status.code()
            ));
        }
        if !output.status.success() {
            fixture_failures.push(format!(
                "process did not exit successfully\nstderr:\n{stderr}\nstdout:\n{stdout}"
            ));
        }
        if stderr.contains("error[") {
            fixture_failures.push(format!("emitted runtime/compiler errors:\n{stderr}"));
        }
        if !stderr.is_empty() {
            fixture_failures.push(format!(
                "emitted diagnostics during command execution:\n{stderr}"
            ));
        }
        if stdout != expected_stdout {
            fixture_failures.push(format!(
                "stdout did not match .io.txt contract\nexpected:\n{expected_stdout}\nactual:\n{stdout}"
            ));
        }
        if !fixture_failures.is_empty() {
            failures.push(format!(
                "{}\n{}",
                source.display(),
                fixture_failures.join("\n")
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{relative} command I/O fixtures did not match their contracts:\n\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn run_verbose_shows_runtime_readiness_warnings() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler");
    let source = fixtures
        .join("algorithms")
        .join("positive")
        .join("binary_search.es");

    let (_, stdout, stderr) = run_process(
        ["--workspace", path(&fixtures), "run", path(&source)],
        "7 1 3 5 7 9 11 13\n",
    );
    assert_eq!(stdout, "3\n");
    assert!(stderr.is_empty(), "{stderr}");

    let (_, stdout, stderr) = run_process(
        ["-v", "--workspace", path(&fixtures), "run", path(&source)],
        "7 1 3 5 7 9 11 13\n",
    );
    assert_eq!(stdout, "3\n");
    assert!(stderr.contains("RuntimeRequiredInPhase1"));
    assert!(stderr.contains("Console"));
}

#[test]
fn run_entry_reachable_readiness_ignores_unreachable_agent_host_support() {
    let root = package_fixture(
        "entry-reachable-readiness",
        &[
            (
                "etas.toml",
                r#"[package]
name = "entry-reachable-readiness"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"
"#,
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import std.collections.Array;
import std.io.println;

agent UnreachableReviewer(input: string) -> string {
    return Prompt.new()
        .system(Trusted("Return a short review."))
        .data(input);
}

public flow unused_agent_path(topic: string) -> string {
    return UnreachableReviewer.run(topic);
}

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    println("reachable");
    return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run_process(["-v", "run", path(&root), "--flow", "main"], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "reachable\n");
    assert!(
        stderr.contains("RuntimeRequiredInPhase1") && stderr.contains("Console"),
        "reachable console effect should still produce verbose readiness warning:\n{stderr}"
    );
    assert!(
        !stderr.contains("Agentic") && !stderr.contains("Model"),
        "unreachable agent flow must not contribute runtime readiness warnings:\n{stderr}"
    );
}

#[test]
fn run_discovers_imports_for_relative_file_input() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler");
    let algorithms = fixtures.join("algorithms").join("positive");

    let (code, stdout, stderr) = run_process_in(
        &algorithms,
        ["--workspace", path(&fixtures), "run", "binary_search.es"],
        "7 1 3 5 7 9 11 13\n",
    );

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "3\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_passes_program_args_to_main() {
    let file = fixture(
        "program-args",
        r#"module tests.cli.program_args;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    println(args[0]);
    println(args[1]);
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run_process(["run", path(&file), "--args", "alpha", "--beta"], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "alpha\n--beta\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_suppresses_value_report_when_console_is_inferred_from_called_flow() {
    let file = fixture(
        "run-inferred-console",
        r#"module tests.cli.run_inferred_console;

import std.io.println;

flow write_message() -> unit ![Error<IOError>]
{
    println("from helper");
    return;
}

flow main(args: Array<string>) -> i32
{
    write_message();
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run_process(["run", path(&file)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "from helper\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn default_command_runs_file_inputs() {
    let file = fixture(
        "default-run",
        r#"module tests.cli.default_run;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    println(args[0]);
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run_process([path(&file), "--args", "direct"], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "direct\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_accepts_subcommand_args_without_program_name() {
    let file = fixture(
        "run-with-no-argv0-check",
        r#"module tests.cli.run_with_no_argv0_check;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    println("should not run during check");
    return 0;
}
"#,
    );

    let (code, stdout, stderr) = run(["check", path(&file)]);

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "checked 1 file\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_discovers_imported_package_sources_from_main_file() {
    let root = import_discovery_package("run-import-discovery");
    let main = root.join("src").join("app").join("main.es");

    let (code, stdout, stderr) = run_process(["--workspace", path(&root), "run", path(&main)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "loaded\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_rejects_allow_effects_without_host_adapters() {
    let file = fixture(
        "allow-effects",
        "flow main(args: Array<string>) -> i32 { return 0; }",
    );
    let (code, stdout, stderr) = run(["etas", "run", "--allow-effects", path(&file)]);

    assert_eq!(code, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("requires configured host adapters"));
}

#[test]
fn run_with_approval_prompt_non_tty_denies_without_hanging() {
    let file = fixture(
        "approval-prompt-non-tty",
        r#"module tests.cli.approval_prompt_non_tty;

effect Approval {
  action request(reason: string) -> bool;
}

flow main(args: Array<string>) -> i32 ![Approval] {
  let approved = perform Approval.request("ship?");
  if approved {
    return 1;
  }
  return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_APPROVAL", "prompt");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"value\":\"0\""), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_memory_host_adapter_executes_memory_get_put() {
    let file = fixture(
        "memory-host-adapter",
        r#"module tests.cli.memory_host_adapter;

alias ProjectMemorySchema = MemoryRegion<{
  Papers: Store<string, string>
}>;

let ProjectMemory =
  std.memory.region<ProjectMemorySchema>(
    stable_id = "project_memory",
    store = "project-main"
  );

flow main(args: Array<string>) -> i32 ![Memory.read<ProjectMemory>, Memory.write<ProjectMemory>] {
  ProjectMemory.Papers.put("paper-1", "draft");
  let _value = ProjectMemory.Papers.get("paper-1");
  return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_MEMORY", "memory");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"value\":\"0\""), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_sqlite_memory_host_persists_across_processes() {
    let writer = fixture(
        "sqlite-memory-writer",
        r#"module tests.cli.sqlite_memory_writer;

import std.io.println;

alias ProjectMemorySchema = MemoryRegion<{
  Papers: Store<string, string>
}>;

let ProjectMemory =
  std.memory.region<ProjectMemorySchema>(
    stable_id = "project_memory",
    store = "project-main"
  );

flow main(args: Array<string>) -> i32 ![Memory.write<ProjectMemory>, Console, Error<IOError>, Error<IndexError>] {
  ProjectMemory.Papers.put("paper-1", args[0]);
  println("stored");
  return 0;
}
"#,
    );
    let reader = fixture(
        "sqlite-memory-reader",
        r#"module tests.cli.sqlite_memory_reader;

import std.io.println;
import std.option.unwrap;

alias ProjectMemorySchema = MemoryRegion<{
  Papers: Store<string, string>
}>;

let ProjectMemory =
  std.memory.region<ProjectMemorySchema>(
    stable_id = "project_memory",
    store = "project-main"
  );

flow main(args: Array<string>) -> i32 ![Memory.read<ProjectMemory>, Console, Error<IOError>] {
  let value = unwrap(ProjectMemory.Papers.get("paper-1"));
  println(value);
  return 0;
}
"#,
    );
    let db_path = std::env::temp_dir().join(format!(
        "etas-cli-sqlite-memory-{}-{}.sqlite",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let memory_mode = format!("sqlite:{}", path(&db_path));

    let mut write = Command::new(env!("CARGO_BIN_EXE_etas"));
    write.env("ETAS_HOST_MEMORY", &memory_mode);
    let (code, stdout, stderr) = run_process_with_command(
        write,
        [
            "run",
            "--allow-effects",
            path(&writer),
            "--args",
            "persisted",
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "stored\n");
    assert!(stderr.is_empty(), "{stderr}");

    let mut read = Command::new(env!("CARGO_BIN_EXE_etas"));
    read.env("ETAS_HOST_MEMORY", &memory_mode);
    let (code, stdout, stderr) =
        run_process_with_command(read, ["run", "--allow-effects", path(&reader)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "persisted\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_runtime_session_env_exposes_current_session() {
    let file = fixture(
        "runtime-session-current",
        r#"module tests.cli.runtime_session_current;

import std.agent.session.current_session;
import std.agent.session.SessionId;
import std.agent.message.Message;
import std.io.println;

flow main(args: Array<string>) -> i32 ![Memory, Console, Error<IOError>] {
  let message = Message.new("hello");
  let session = SessionConfig.continue_or_new(current_session());
  let _scoped = Message.with_session(message, session);
  println("session-ok");
  return 0;
}
"#,
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_MEMORY", "memory");
    command.env("ETAS_HOST_SESSION_ID", "session-cli-42");
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&file)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "session-ok\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_with_sqlite_runtime_session_uses_persistent_session_backend() {
    let writer = fixture(
        "runtime-session-sqlite-writer",
        r#"module tests.cli.runtime_session_sqlite_writer;

import std.agent.session.current_session;
import std.agent.session.SessionId;
import std.agent.message.Message;
import std.io.println;

flow main(args: Array<string>) -> i32 ![Memory, Console, Error<IOError>] {
  let message = Message.new("persist");
  let session = SessionConfig.continue_or_new(current_session());
  let _scoped = Message.with_session(message, session);
  println("session-sqlite-ok");
  return 0;
}
"#,
    );
    let reader = fixture(
        "runtime-session-sqlite-reader",
        r#"module tests.cli.runtime_session_sqlite_reader;

import std.agent.session.current_session;
import std.agent.session.SessionId;
import std.agent.session.Conversation;
import std.io.println;

flow main(args: Array<string>) -> i32 ![Memory, Console, Error<IOError>] {
  let session = SessionConfig.continue_or_new(current_session());
  let conversation = Conversation.load(session);
  if conversation.messages.len() == 1 {
    println("session-loaded-one");
  } else {
    println("session-loaded-other");
  }
  return 0;
}
"#,
    );
    let db_path = std::env::temp_dir().join(format!(
        "etas-cli-sqlite-session-{}-{}.sqlite",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let session_mode = format!("sqlite:{}", path(&db_path));

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_MEMORY", "memory");
    command.env("ETAS_HOST_SESSION_ID", "session-cli-sqlite-42");
    command.env("ETAS_HOST_SESSION", &session_mode);
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&writer)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "session-sqlite-ok\n");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(db_path.exists(), "SQLite session database was not created");

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_MEMORY", "memory");
    command.env("ETAS_HOST_SESSION_ID", "session-cli-sqlite-42");
    command.env("ETAS_HOST_SESSION", &session_mode);
    let (code, stdout, stderr) =
        run_process_with_command(command, ["run", "--allow-effects", path(&reader)], "");

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "session-loaded-one\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn run_checkpoint_records_memory_resource_versions() {
    let file = fixture(
        "checkpoint-memory-version",
        r#"module tests.cli.checkpoint_memory_version;

import std.runtime.{checkpoint};

alias ProjectMemorySchema = MemoryRegion<{
  Papers: Store<string, string>
}>;

let ProjectMemory =
  std.memory.region<ProjectMemorySchema>(
    stable_id = "project_memory",
    store = "project-main"
  );

flow main(args: Array<string>) -> i32 ![Memory.write<ProjectMemory>] {
  ProjectMemory.Papers.put("paper-1", "draft");
  checkpoint("after-write");
  return 0;
}
"#,
    );
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-cli-memory-version-checkpoints-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.env("ETAS_HOST_MEMORY", "memory");
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            "--allow-effects",
            path(&file),
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checkpoints: 1"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-0.json")).unwrap())
            .unwrap();
    let versions = checkpoint_json["checkpoint"]["resource_versions"]
        .as_array()
        .expect("checkpoint should include resource_versions");
    assert!(
        versions.iter().any(|record| {
            record["resource"] == "memory:project_memory:Papers:\"paper-1\""
                && record["version"] == "1"
        }),
        "checkpoint should record the memory entry version: {checkpoint_json}"
    );
}

#[test]
fn project_agent_system_smoke_checks_effects_and_runs_with_openai_mock() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture("agent-system-smoke", &agent_system_smoke_project());

    let (code, stdout, stderr) = run(["etas", "check", path(&project)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked "), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "--format", "json", "effects", path(&project)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(stdout.contains("Agentic.infer"), "{stdout}");
    assert!(stdout.contains("Console.stdout_write"), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (base_url, server) = spawn_openai_completion_server("agent smoke reply");
    write_local_static_mock_runtime_profile(
        &project,
        "agent-smoke-mock",
        &base_url,
        "agent-smoke-mock",
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "agent-smoke-mock",
            "--allow-effects",
            "--budget-tokens",
            "128",
        ],
        "",
    );
    shutdown_mock_server(&base_url);
    let request = server.join().expect("mock server thread should finish");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("agent smoke reply"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        request.contains("chat/completions"),
        "CLI should call the OpenAI-compatible chat completions endpoint:\n{request}"
    );
    assert!(
        request.contains("\"max_tokens\":128"),
        "CLI budget override should reach the model request:\n{request}"
    );
}

#[test]
fn run_print_runtime_profile_uses_manifest_and_local_override() {
    let project = package_fixture(
        "runtime-profile-print",
        &[
            (
                "etas.toml",
                r#"
[package]
name = "runtime-profile-print"
version = "0.1.0"

[runtime]
default_profile = "local-omlx"

[runtime.execution]
max_call_depth = 120
max_steps = 1000000

[runtime.profiles.local-omlx.model]
adapter = "omlx-openai"
model = "manifest-model"
base_url = "http://127.0.0.1:8848/v1"
api_key_env = "ETAS_TEST_OMLX_API_KEY"

[runtime.profiles.local-omlx.memory]
backend = "memory"
"#,
            ),
            (
                "etas.local.toml",
                r#"
[runtime.execution]
max_call_depth = 96

[runtime.profiles.local-omlx.model]
base_url = "http://127.0.0.1:9999/v1"
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run([
        "etas",
        "run",
        path(&project),
        "--profile",
        "local-omlx",
        "--print-runtime-profile",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    let profile: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(profile["profile"], "local-omlx");
    assert_eq!(profile["model"]["model"], "manifest-model");
    assert_eq!(profile["execution"]["max_call_depth"], 96);
    assert_eq!(profile["execution"]["max_steps"], 1_000_000);
    assert!(
        profile["adapter_transport_endpoints"]
            .as_array()
            .expect("adapter transport endpoints")
            .iter()
            .any(|endpoint| endpoint["host"] == "127.0.0.1" && endpoint["port"] == 9999),
        "{profile:#}"
    );
    assert_eq!(
        profile["program_network_endpoints"],
        serde_json::json!([]),
        "model transport must not grant source-visible network authority"
    );
    assert!(!stdout.contains("ETAS_TEST_OMLX_API_KEY"), "{stdout}");
}

#[test]
fn project_agent_system_smoke_runs_with_runtime_profile() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture("agent-system-smoke-profile", &agent_system_smoke_project());
    let (base_url, server) = spawn_openai_completion_server("profile agent reply");
    std::fs::write(
        project.join("etas.local.toml"),
        format!(
            r#"
[runtime.profiles.local-openai.model]
adapter = "openai"
model = "mock-agent-model"
base_url = "{base_url}"
allow_private = true

[runtime.profiles.local-openai.memory]
backend = "memory"
"#
        ),
    )
    .unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    for name in [
        "ETAS_HOST_MODEL_ADAPTER",
        "ETAS_HOST_MODEL_BASE_URL",
        "ETAS_HOST_MODEL_NAME",
        "ETAS_HOST_MODEL_API_KEY",
        "ETAS_HOST_OMLX_API_KEY",
        "ETAS_HOST_MEMORY",
    ] {
        command.env_remove(name);
    }
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "local-openai",
            "--budget-tokens",
            "128",
        ],
        "",
    );
    shutdown_mock_server(&base_url);
    let request = server.join().expect("mock server thread should finish");

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("profile agent reply"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        request.contains("chat/completions"),
        "CLI should call model using runtime profile:\n{request}"
    );
}

#[test]
fn full_phase1_agent_runtime_check_effects_and_runs_with_mock_host() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "full-phase1-agent-runtime-profile",
        &full_phase1_agent_runtime_project(),
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&project)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked "), "{stdout}");
    assert!(!stderr.contains("error["), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "--format", "json", "effects", path(&project)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(
        !stdout.contains("Error<IOError>, Error<IOError>"),
        "{stdout}"
    );
    assert!(stdout.contains("Agentic.infer"), "{stdout}");
    assert!(stdout.contains("Memory.write"), "{stdout}");
    assert!(stdout.contains("Memory.read"), "{stdout}");
    assert!(stdout.contains("Console.stdout_write"), "{stdout}");
    assert!(stdout.contains("trace spec RuntimeGate"), "{stdout}");
    assert!(
        !stdout.contains("RuntimeConflict"),
        "full runtime fixture must use real memory conflict handling, not synthetic effects:\n{stdout}"
    );
    assert!(!stderr.contains("error["), "{stderr}");

    let (model_base_url, model_server) = spawn_openai_tool_loop_server("review via tool", 9);
    write_local_static_mock_runtime_profile(
        &project,
        "full-phase1-mock",
        &model_base_url,
        "full-phase1-runtime",
    );
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-full-phase1-checkpoints-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "full-phase1-mock",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
            "--args",
            "runtime",
            "draft",
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    let model_requests = model_server
        .join()
        .expect("mock OpenAI tool-loop server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}"
    );
    assert!(
        stdout.contains("review via tool"),
        "stdout={stdout:?}\nstderr={stderr:?}"
    );
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        3,
        "full runtime fixture should perform one Planner model round plus the Reviewer two-round tool loop:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"name\":\"EvidenceLookup\"")),
        "Reviewer model request should expose the tool schema:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"tool_choice\"")
                && request.contains("\"EvidenceLookup\"")
                && !request.contains("\"role\":\"tool\"")),
        "Reviewer first model request should require the EvidenceLookup tool:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"role\":\"tool\"")),
        "final Reviewer model request should include the tool result message:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("local evidence for runtime")),
        "source-bodied tool result should be returned to the model without a host tool adapter:\n{model_requests:#?}"
    );
    assert!(
        checkpoint_dir.join("checkpoint-1.json").is_file(),
        "full runtime fixture should record a post-memory checkpoint"
    );
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-1.json")).unwrap())
            .unwrap();
    let completed_boundaries = checkpoint_json["checkpoint"]["completed_host_boundaries"]
        .as_array()
        .expect("checkpoint should contain completed host boundary ledger");
    assert!(
        completed_boundaries
            .iter()
            .any(|boundary| boundary["kind"] == "tool"
                && boundary["key"]
                    .as_str()
                    .is_some_and(|key| key.contains("tool:app.runtime.tools.EvidenceLookup"))),
        "full runtime checkpoint must record completed source/tool boundary ledger: {checkpoint_json}"
    );

    let mut resume = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut resume);
    let (code, stdout, stderr) = run_process_with_command(
        resume,
        ["resume", "1", "--checkpoint-dir", path(&checkpoint_dir)],
        "",
    );

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}"
    );
    assert!(stdout.contains("resumed checkpoint 1"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn full_phase1_agent_runtime_repairs_invalid_model_tool_args() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "full-phase1-tool-repair",
        &full_phase1_agent_runtime_project(),
    );
    let (model_base_url, model_server) =
        spawn_openai_tool_arg_repair_server("review after repair", 10);
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-full-phase1-tool-repair-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));

    write_local_static_mock_runtime_profile(
        &project,
        "tool-repair",
        &model_base_url,
        "full-phase1-runtime-tool-repair",
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "tool-repair",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
            "--args",
            "runtime",
            "draft",
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    let model_requests = model_server
        .join()
        .expect("mock OpenAI tool-repair server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}"
    );
    assert!(stdout.contains("review after repair"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        4,
        "tool arg repair should add one model round before executing the source tool:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("InvalidToolArguments")
                && request.contains("model supplied invalid arguments")),
        "invalid tool args should be returned to the model as a tool result:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("local evidence for runtime")),
        "valid repaired tool args should still execute the source-bodied tool:\n{model_requests:#?}"
    );
}

#[test]
fn full_phase1_agent_runtime_uses_trace_spec_with_http_provider() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "full-phase1-http-policy",
        &full_phase1_agent_runtime_project(),
    );
    let (model_base_url, model_server) = spawn_openai_tool_loop_server("review via http policy", 7);
    let (policy_url, policy_server) = spawn_memory_approval_policy_server();
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-full-phase1-http-policy-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));

    write_http_policy_mock_runtime_profile(
        &project,
        "http-policy",
        &model_base_url,
        &policy_url,
        "full-phase1-runtime-http",
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "http-policy",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
            "--args",
            "runtime",
            "draft",
        ],
        "",
    );
    assert_eq!(code, 0, "stderr={stderr:?}\nstdout={stdout:?}");
    assert!(stdout.contains("review via http policy"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    shutdown_mock_server(&model_base_url);
    shutdown_mock_server(&policy_url);
    let model_requests = model_server
        .join()
        .expect("mock OpenAI tool-loop server should finish");
    let policy_requests = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        3,
        "full runtime HTTP policy run should keep Planner plus Reviewer tool-loop model calls:\n{model_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"active_trace_specs\":[\"RuntimeGate\"]")),
        "HTTP policy requests should carry the trace spec active on the entry flow:\n{policy_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"kind\":\"memory\"")),
        "HTTP policy provider should mediate memory boundaries in the full runtime fixture:\n{policy_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"effect\":\"Memory\"")
                && request.contains("\"action\":\"write\"")
                && request.contains("\"args\":[{\"kind\":\"any\"}]")),
        "approval grant precision must be preserved after HTTP policy approval:\n{policy_requests:#?}"
    );
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-1.json")).unwrap())
            .unwrap();
    assert!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["policy"]["active_trace_specs"]
            .as_array()
            .is_some_and(|policies| policies.iter().any(|policy| policy == "RuntimeGate")),
        "checkpoint should persist source active trace specs: {checkpoint_json}"
    );
    assert_eq!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["policy"]["boundary_policy_ref"]
            ["value"],
        "full-phase1-runtime-http",
        "checkpoint should persist boundary policy ref: {checkpoint_json}"
    );
    assert!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["grants"]
            .as_array()
            .is_some_and(|grants| grants.iter().any(|grant| {
                grant["pattern"]["effect"] == "Memory"
                    && grant["pattern"]["action"] == "write"
                    && grant["pattern"]["args"]
                        .as_array()
                        .is_some_and(|args| args.iter().any(|arg| arg["kind"] == "any"))
            })),
        "checkpoint should persist approval grants with argument precision: {checkpoint_json}"
    );
}

#[test]
fn full_phase1_agent_runtime_resume_rejects_incompatible_host_profile() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "full-phase1-profile-mismatch",
        &full_phase1_agent_runtime_project(),
    );
    let (model_base_url, model_server) = spawn_openai_tool_loop_server("review via tool", 9);
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-full-phase1-profile-mismatch-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));

    write_local_static_mock_runtime_profile(
        &project,
        "profile-compatible",
        &model_base_url,
        "full-phase1-runtime",
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "profile-compatible",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
            "--args",
            "runtime",
            "draft",
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    let _model_requests = model_server
        .join()
        .expect("mock OpenAI tool-loop server should finish");

    assert_eq!(code, 0, "stderr={stderr:?}\nstdout={stdout:?}");
    assert!(
        checkpoint_dir.join("checkpoint-1.json").is_file(),
        "run should write a resumable checkpoint"
    );

    write_local_static_mock_runtime_profile(
        &project,
        "profile-incompatible",
        &model_base_url,
        "full-phase1-runtime",
    );
    let local_manifest = project.join("etas.local.toml");
    let mut local_profile = std::fs::read_to_string(&local_manifest).unwrap();
    local_profile = local_profile.replace(
        "model = \"mock-agent-model\"",
        "model = \"different-model\"",
    );
    std::fs::write(&local_manifest, local_profile).unwrap();
    let mut resume = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut resume);
    let (code, stdout, stderr) = run_process_with_command(
        resume,
        [
            "resume",
            "1",
            "--profile",
            "profile-incompatible",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );

    assert_ne!(code, 0, "resume should reject incompatible runtime profile");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("current host runtime profile is not compatible with the checkpoint"),
        "{stderr}"
    );
}

#[test]
fn package_external_tool_runs_through_cli_http_tool_binding() {
    let _guard = lock_model_cli_e2e();
    let dependency = package_fixture(
        "pkg-external-tool-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "external-tools"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[bindings.tools]
"tools.EvidenceLookup" = { kind = "http", provider = "test.evidence", effects = ["Network"] }
"#,
            ),
            (
                "src/tools.es",
                r#"module tools;

public tool EvidenceLookup(query: string) -> string ![Network];
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-external-tool-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "external-tool-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
dep = {{ package = "external-tools", version = "0.1", import = "dep", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import dep.tools.{EvidenceLookup};
import std.agent.prompt.Prompt;
import std.io.println;

@model(model = "mock-agent-model")
@tools([EvidenceLookup])
agent Reviewer(input: string) -> string ![Network] {
  return Prompt.new().user(Public(input));
}

flow main(args: Array<string>) -> i32 ![Network, Error<IOError>] {
  let answer = Reviewer.run("runtime");
  println(answer);
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (model_base_url, model_server) =
        spawn_openai_external_tool_loop_server("final answer from external tool");
    let (tool_base_url, tool_server) = spawn_http_tool_value_server(
        serde_json::json!("external evidence from package tool").to_string(),
    );
    let tool_network_authority = mock_server_addr_and_token(&tool_base_url)
        .expect("mock tool URL")
        .0;
    std::fs::write(
        root.join("etas.local.toml"),
        format!(
            r#"
[runtime]
default_profile = "external-tool-mock"

[runtime.profiles.external-tool-mock.model]
adapter = "openai"
model = "mock-agent-model"
base_url = "{model_base_url}"
allow_private = true

[runtime.profiles.external-tool-mock.tools]
allow_private = true

[runtime.profiles.external-tool-mock.tools.http]
"dep.tools.EvidenceLookup" = "{tool_base_url}"

[runtime.profiles.external-tool-mock.network]
allow = ["{tool_network_authority}"]
"#
        ),
    )
    .expect("write external tool mock runtime profile");

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&root),
            "--profile",
            "external-tool-mock",
            "--allow-effects",
            "--budget-tokens",
            "128",
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    shutdown_mock_server(&tool_base_url);
    let model_requests = model_server
        .join()
        .expect("mock OpenAI external tool-loop server should finish");
    let tool_requests = tool_server
        .join()
        .expect("mock HTTP tool server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}\ntool_requests={tool_requests:#?}"
    );
    assert!(
        stdout.contains("final answer from external tool"),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        2,
        "external package tool should produce one model round with a tool call and one final model round:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"name\":\"EvidenceLookup\"")),
        "external package tool schema must be exposed to the model:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"role\":\"tool\"")
                && request.contains("external evidence from package tool")),
        "host tool response must be fed back into the final model round:\n{model_requests:#?}"
    );
    assert_eq!(tool_requests.len(), 1, "{tool_requests:#?}");
    assert!(
        tool_requests[0].contains("\"query\":\"runtime\""),
        "CLI HTTP tool binding must receive the model-provided typed arguments:\n{tool_requests:#?}"
    );
}

#[test]
fn edk_backed_external_tool_multi_agent_runtime_uses_package_metadata() {
    let _guard = lock_model_cli_e2e();
    let dependency = package_fixture(
        "pkg-edk-like-external-tool-dep",
        &[
            (
                "etas.toml",
                r#"[package]
name = "edk-evidence"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[bindings.tools]
"edk.evidence.tools.EvidenceLookup" = { kind = "http", provider = "test.evidence", effects = ["Network"] }
"#,
            ),
            (
                "src/edk/evidence/tools.es",
                r#"module edk.evidence.tools;

public spec EvidenceRuntimeGate: trace = +Network;

public tool EvidenceLookup(query: string) -> string ![Network];
"#,
            ),
        ],
    );
    let (code, _stdout, stderr) = run(["etas", "pkg", "metadata", path(&dependency)]);
    assert_eq!(code, 0, "{stderr}");

    let root = package_fixture(
        "pkg-edk-like-multi-agent-root",
        &[
            (
                "etas.toml",
                &format!(
                    r#"[package]
name = "edk-like-multi-agent-root"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[[bin]]
name = "main"
module = "app.main"
flow = "main"

[dependencies]
evidence = {{ package = "edk-evidence", version = "0.1", import = "edk.evidence", path = "{}" }}
"#,
                    dependency.display()
                ),
            ),
            (
                "src/app/main.es",
                r#"module app.main;

import edk.evidence.tools.{EvidenceLookup, EvidenceRuntimeGate};
import std.agent.prompt.Prompt;
import std.io.println;

spec RuntimeGate: trace = +Agentic & +Network & +Console;

@model(model = "mock-agent-model")
agent Planner(input: string) -> string {
  return Prompt.new().user(Public(input));
}

@model(model = "mock-agent-model")
@tools([EvidenceLookup])
agent Reviewer(input: string) -> string ![Network] {
  return Prompt.new().user(Public(input));
}

flow main(args: Array<string>) -> i32 ![Network, Error<IOError>] ~ RuntimeGate + EvidenceRuntimeGate
{
  let plan = Planner.run("runtime");
  let answer = Reviewer.run(plan);
  println(answer);
  return 0;
}
"#,
            ),
        ],
    );

    let (code, stdout, stderr) = run(["etas", "pkg", "update", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("updated"), "{stdout}");
    let dependency_tool_source = dependency.join("src/edk/evidence/tools.es");
    std::fs::write(
        &dependency_tool_source,
        r#"module edk.evidence.tools;

policy LegacyEvidencePolicy {
  allow Network;
}

public tool EvidenceLookup(query: string) -> string ![Network] follows LegacyEvidencePolicy;

policy LegacyTracePolicy {
  allow Network;
}
"#,
    )
    .expect("rewrite original dependency source after root package metadata is materialized");
    let dependency_tool_source_text = std::fs::read_to_string(&dependency_tool_source)
        .unwrap_or_else(|error| panic!("read dependency source after rewrite: {error}"));
    assert!(
        dependency_tool_source_text.contains("follows LegacyEvidencePolicy"),
        "EDK-like fixture should prove check/run consume materialized package metadata instead of re-reading the original path dependency source"
    );

    let (code, stdout, stderr) = run(["etas", "check", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run(["etas", "--format", "json", "effects", path(&root)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(stdout.contains("Agentic.infer"), "{stdout}");
    assert!(stdout.contains("Network"), "{stdout}");
    assert!(stdout.contains("trace spec RuntimeGate"), "{stdout}");
    assert!(
        stdout.contains("trace spec edk.evidence.tools.EvidenceRuntimeGate"),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");

    let (model_base_url, model_server) =
        spawn_openai_edk_like_multi_agent_tool_server("final answer from edk-like evidence");
    let (tool_base_url, tool_server) = spawn_http_tool_value_server(
        serde_json::json!("edk-like evidence from package tool").to_string(),
    );
    let tool_network_authority = mock_server_addr_and_token(&tool_base_url)
        .expect("mock tool URL")
        .0;
    let (policy_url, policy_server) = spawn_memory_approval_policy_server();
    std::fs::write(
        root.join("etas.local.toml"),
        format!(
            r#"
[runtime]
default_profile = "edk-like-mock"

[runtime.profiles.edk-like-mock]
boundary_policy = "edk-backed-multi-agent"

[runtime.profiles.edk-like-mock.model]
adapter = "openai"
model = "mock-agent-model"
base_url = "{model_base_url}"
allow_private = true

[runtime.profiles.edk-like-mock.policy]
mode = "http"
url = "{policy_url}"
allow_private = true

[runtime.profiles.edk-like-mock.approval]
mode = "auto"

[runtime.profiles.edk-like-mock.tools]
allow_private = true

[runtime.profiles.edk-like-mock.tools.http]
"edk.evidence.tools.EvidenceLookup" = "{tool_base_url}"

[runtime.profiles.edk-like-mock.network]
allow = ["{tool_network_authority}"]
"#
        ),
    )
    .expect("write EDK-like mock runtime profile");

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&root),
            "--profile",
            "edk-like-mock",
            "--allow-effects",
            "--budget-tokens",
            "128",
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    shutdown_mock_server(&tool_base_url);
    shutdown_mock_server(&policy_url);
    let model_requests = model_server
        .join()
        .expect("mock OpenAI EDK-like multi-agent tool-loop server should finish");
    let tool_requests = tool_server
        .join()
        .expect("mock HTTP EDK-like tool server should finish");
    let policy_requests = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}\ntool_requests={tool_requests:#?}"
    );
    assert!(
        stdout.contains("final answer from edk-like evidence"),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        3,
        "EDK-like multi-agent fixture should perform Planner plus Reviewer tool-loop model calls:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"name\":\"EvidenceLookup\"")),
        "external EDK-like package tool schema must be exposed to the model:\n{model_requests:#?}"
    );
    assert!(
        model_requests
            .iter()
            .any(|request| request.contains("\"role\":\"tool\"")
                && request.contains("edk-like evidence from package tool")),
        "external EDK-like tool response must be fed back into the final model round:\n{model_requests:#?}"
    );
    assert_eq!(tool_requests.len(), 1, "{tool_requests:#?}");
    assert!(
        tool_requests[0].contains("\"query\":\"runtime\""),
        "CLI HTTP tool binding must receive model-provided typed arguments through the qualified EDK-like tool identity:\n{tool_requests:#?}"
    );
    assert!(
        policy_requests.iter().any(|request| {
            request.contains(
                "\"active_trace_specs\":[\"RuntimeGate\",\"edk.evidence.tools.EvidenceRuntimeGate\"]",
            ) || request.contains(
                "\"active_trace_specs\":[\"edk.evidence.tools.EvidenceRuntimeGate\",\"RuntimeGate\"]",
            )
        }),
        "runtime policy requests must carry root and external dependency trace specs from package metadata:\n{policy_requests:#?}"
    );
}

#[test]
fn multi_agent_system_runtime_variants_check_effects() {
    let _guard = lock_model_cli_e2e();
    let project = multi_agent_system_project();
    let (code, stdout, stderr) = run(["etas", "--format", "json", "effects", path(&project)]);

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    for flow in [
        "run_three_agent_version",
        "run_retry_checkpoint_version",
        "run_memory_conflict_version",
        "run_cached_review_version",
        "run_abort_guard_version",
    ] {
        assert!(
            stdout.contains(flow),
            "multi-agent runtime variant `{flow}` should be present in effects output:\n{stdout}"
        );
    }
    assert!(stdout.contains("Agentic.infer"), "{stdout}");
    assert!(stdout.contains("Memory.write"), "{stdout}");
    assert!(stdout.contains("Memory.read"), "{stdout}");
    assert!(stdout.contains("LimitRequirement"), "{stdout}");
    assert!(stdout.contains("EffectHandler"), "{stdout}");
    assert!(
        stdout.contains("trace spec MultiAgentRuntimeGate"),
        "{stdout}"
    );
    assert!(!stderr.contains("error["), "{stderr}");
}

#[test]
fn multi_agent_system_default_main_runs_full_runtime_with_mock_host() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture("multi-agent-system-runtime", &multi_agent_system_project());
    let (base_url, server) = spawn_multi_agent_runtime_variants_server();
    write_local_static_mock_runtime_profile(
        &project,
        "multi-agent-runtime-mock",
        &base_url,
        "multi-agent-runtime-variants",
    );
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-multi-agent-runtime-variants-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&checkpoint_dir);

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "multi-agent-runtime-mock",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );
    shutdown_mock_server(&base_url);
    let requests = server
        .join()
        .expect("mock multi-agent runtime server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nrequests={requests:#?}"
    );
    assert!(stdout.contains("checkpointed review"), "{stdout}");
    assert!(
        stdout.contains("fallback review for runtime-conflict"),
        "{stdout}"
    );
    assert!(stdout.contains("cached review"), "{stdout}");
    assert!(
        stdout.contains("fallback review for runtime-guard"),
        "{stdout}"
    );
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        16,
        "runtime variants should execute four 3-agent sequences, with each Researcher using a two-round tool loop:\n{requests:#?}"
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("\"name\":\"EvidenceLookup\"")
                && !request.contains("\"role\":\"tool\""))
            .count(),
        4,
        "all Researcher calls should expose the source-bodied tool:\n{requests:#?}"
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("\"tool_choice\"")
                && request.contains("\"EvidenceLookup\"")
                && !request.contains("\"role\":\"tool\""))
            .count(),
        4,
        "all Researcher first model requests should require EvidenceLookup:\n{requests:#?}"
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("\"role\":\"tool\""))
            .count(),
        4,
        "all Researcher calls should feed source tool results back to the model:\n{requests:#?}"
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("\"response_format\"")
                && request.contains("\"json_schema\"")
                && request.contains("\"summary\"")
                && request.contains("\"score\""))
            .count(),
        4,
        "all Reviewer typed-record calls should carry a JSON schema response contract:\n{requests:#?}"
    );
    assert!(
        checkpoint_dir.join("checkpoint-0.json").is_file(),
        "retry/checkpoint variant should write the start checkpoint"
    );
    assert!(
        checkpoint_dir.join("checkpoint-1.json").is_file(),
        "retry/checkpoint variant should write the end checkpoint"
    );
    assert!(
        checkpoint_dir.join("checkpoint-4.json").is_file(),
        "cached review variant should write the cache-read checkpoint after a nested retry"
    );
}

#[test]
fn multi_agent_system_runtime_variants_resume_does_not_replay_completed_boundaries() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "multi-agent-system-runtime-resume",
        &multi_agent_system_project(),
    );
    let (base_url, server) = spawn_multi_agent_runtime_variants_server();
    write_local_static_mock_runtime_profile(
        &project,
        "multi-agent-runtime-resume",
        &base_url,
        "multi-agent-runtime-resume",
    );
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-multi-agent-runtime-resume-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&checkpoint_dir);

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "multi-agent-runtime-resume",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );
    assert_eq!(code, 0, "stderr={stderr:?}\nstdout={stdout:?}");
    assert!(checkpoint_dir.join("checkpoint-1.json").is_file());

    let mut resume = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut resume);
    let (code, stdout, stderr) = run_process_with_command(
        resume,
        ["resume", "1", "--checkpoint-dir", path(&checkpoint_dir)],
        "",
    );
    shutdown_mock_server(&base_url);
    let requests = server
        .join()
        .expect("mock multi-agent runtime/resume server should finish");

    assert_eq!(
        code, 0,
        "resume should continue the outer machine stack without replaying completed boundaries\nstderr={stderr:?}\nstdout={stdout:?}\nrequests={requests:#?}"
    );
    assert!(stdout.contains("resumed checkpoint 1"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    let runtime_plan_requests = requests
        .iter()
        .filter(|request| {
            request.contains("Return a concise plan for the task.")
                && request.contains("\\\"topic\\\":\\\"runtime\\\"")
        })
        .count();
    assert_eq!(
        runtime_plan_requests, 1,
        "checkpoint resume replayed the completed runtime planner boundary:\n{requests:#?}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.contains("\"role\":\"tool\"")),
        "runtime should retain source tool-call loop behavior:\n{requests:#?}"
    );
}

#[test]
fn multi_agent_system_runtime_variants_http_policy_approval_preserves_grants() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture(
        "multi-agent-system-runtime-http-policy",
        &multi_agent_system_project(),
    );
    let (model_base_url, model_server) = spawn_multi_agent_runtime_variants_server();
    let (policy_url, policy_server) = spawn_memory_approval_policy_server();
    write_http_policy_mock_runtime_profile(
        &project,
        "multi-agent-runtime-http-policy",
        &model_base_url,
        &policy_url,
        "multi-agent-runtime-http-policy",
    );
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-multi-agent-runtime-http-policy-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&checkpoint_dir);

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "multi-agent-runtime-http-policy",
            "--flow",
            "runtime_main",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );
    shutdown_mock_server(&model_base_url);
    shutdown_mock_server(&policy_url);
    let model_requests = model_server
        .join()
        .expect("mock multi-agent runtime server should finish");
    let policy_requests = policy_server
        .join()
        .expect("mock HTTP policy server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nmodel_requests={model_requests:#?}\npolicy_requests={policy_requests:#?}"
    );
    assert!(stdout.contains("checkpointed review"), "{stdout}");
    assert!(
        stdout.contains("fallback review for runtime-conflict"),
        "{stdout}"
    );
    assert!(stdout.contains("cached review"), "{stdout}");
    assert!(
        stdout.contains("fallback review for runtime-guard"),
        "{stdout}"
    );
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        16,
        "HTTP policy run should keep the full multi-agent model/tool loop:\n{model_requests:#?}"
    );
    assert_eq!(
        model_requests
            .iter()
            .filter(|request| request.contains("\"response_format\"")
                && request.contains("\"json_schema\"")
                && request.contains("\"summary\"")
                && request.contains("\"score\""))
            .count(),
        4,
        "HTTP policy run should keep typed Reviewer response schemas on every typed agent call:\n{model_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"active_trace_specs\":[\"MultiAgentRuntimeGate\"]")),
        "HTTP policy requests should carry the runtime trace spec active on runtime_main:\n{policy_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"kind\":\"memory\"")),
        "HTTP policy server should see memory boundaries:\n{policy_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"name\":\"expected_version\",\"value\":\"999\"")),
        "HTTP policy server should see the expected version on the versioned memory write that triggers conflict handling:\n{policy_requests:#?}"
    );
    assert!(
        policy_requests
            .iter()
            .any(|request| request.contains("\"effect\":\"Memory\"")
                && request.contains("\"action\":\"write\"")
                && request.contains("\"args\":[{\"kind\":\"any\"}]")),
        "approved memory grant should be preserved on later HTTP policy requests:\n{policy_requests:#?}"
    );
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-4.json")).unwrap())
            .unwrap();
    assert!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["policy"]["active_trace_specs"]
            .as_array()
            .is_some_and(|policies| policies
                .iter()
                .any(|policy| policy == "MultiAgentRuntimeGate")),
        "multi-agent checkpoint should persist source active trace specs: {checkpoint_json}"
    );
    assert_eq!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["policy"]["boundary_policy_ref"]
            ["value"],
        "multi-agent-runtime-http-policy",
        "multi-agent checkpoint should persist boundary policy ref: {checkpoint_json}"
    );
    assert!(
        checkpoint_json["checkpoint"]["host_context"]["authority"]["grants"]
            .as_array()
            .is_some_and(|grants| grants.iter().any(|grant| {
                grant["pattern"]["effect"] == "Memory"
                    && grant["pattern"]["action"] == "write"
                    && grant["pattern"]["args"]
                        .as_array()
                        .is_some_and(|args| args.iter().any(|arg| arg["kind"] == "any"))
            })),
        "multi-agent checkpoint should persist approval grants with argument precision: {checkpoint_json}"
    );
}

#[test]
fn multi_agent_system_runs_simple_multi_agent_with_mock_model() {
    let _guard = lock_model_cli_e2e();
    let project = copy_project_fixture("multi-agent-system-simple", &multi_agent_system_project());
    let (base_url, server) =
        spawn_openai_completion_sequence_server(&["planner mock output", "reviewer mock output"]);
    write_local_static_mock_runtime_profile(
        &project,
        "simple-multi-agent-mock",
        &base_url,
        "simple-multi-agent-mock",
    );

    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "simple-multi-agent-mock",
            "--flow",
            "smoke_main",
            "--allow-effects",
            "--budget-tokens",
            "128",
        ],
        "",
    );
    shutdown_mock_server(&base_url);
    let requests = server
        .join()
        .expect("mock OpenAI sequence server should finish");

    assert_eq!(
        code, 0,
        "stderr={stderr:?}\nstdout={stdout:?}\nrequests={requests:#?}"
    );
    assert!(stdout.contains("planner mock output"), "{stdout}");
    assert!(stdout.contains("reviewer mock output"), "{stdout}");
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("chat/completions"))
            .count(),
        2,
        "multi-agent fixture should perform one model request per agent call:\n{requests:#?}"
    );
}

#[test]
#[ignore = "requires local omlx server and ETAS_RUN_NETWORK_TESTS=1"]
fn full_phase1_agent_runtime_runs_with_live_omlx_openai() {
    let _guard = lock_model_cli_e2e();
    if std::env::var("ETAS_RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        return;
    }
    let project = full_phase1_agent_runtime_project();
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-full-phase1-live-omlx-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let command = live_omlx_openai_command();
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "local-omlx",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
            "--args",
            "runtime",
            "draft",
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.trim().is_empty(), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(!stdout.contains("UnhandledRuntimeError"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        checkpoint_dir.join("checkpoint-1.json").is_file(),
        "live full runtime run should write a checkpoint"
    );
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-1.json")).unwrap())
            .unwrap();
    let completed_boundaries = checkpoint_json["checkpoint"]["completed_host_boundaries"]
        .as_array()
        .expect("live checkpoint should contain completed host boundary ledger");
    assert!(
        completed_boundaries
            .iter()
            .any(|boundary| boundary["kind"] == "model"),
        "live full runtime checkpoint must record completed model boundaries: {checkpoint_json}"
    );
    assert!(
        completed_boundaries
            .iter()
            .any(|boundary| boundary["kind"] == "tool"
                && boundary["key"]
                    .as_str()
                    .is_some_and(|key| key.contains("tool:app.runtime.tools.EvidenceLookup"))),
        "live full runtime checkpoint must record the source tool-call boundary: {checkpoint_json}"
    );
    assert!(
        checkpoint_json["checkpoint"]["resource_versions"]
            .as_array()
            .is_some_and(|versions| !versions.is_empty()),
        "live full runtime checkpoint must record memory resource versions: {checkpoint_json}"
    );

    let resume = live_omlx_openai_command();
    let (code, stdout, stderr) = run_process_with_command(
        resume,
        [
            "resume",
            "1",
            "--profile",
            "local-omlx",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("resumed checkpoint 1"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(!stdout.contains("UnhandledRuntimeError"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
#[ignore = "requires local omlx server and ETAS_RUN_NETWORK_TESTS=1"]
fn multi_agent_system_runs_with_live_omlx_openai() {
    let _guard = lock_model_cli_e2e();
    if std::env::var("ETAS_RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        return;
    }
    let project = multi_agent_system_project();
    let checkpoint_dir = std::env::temp_dir().join(format!(
        "etas-multi-agent-live-omlx-{}",
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let command = live_omlx_openai_command();
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "local-omlx",
            "--allow-effects",
            "--budget-tokens",
            "256",
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.trim().is_empty(), "{stdout}");
    assert!(
        stdout.contains("fallback review for runtime-conflict"),
        "{stdout}"
    );
    assert!(!stdout.contains("Unknown.action"), "{stdout}");
    assert!(!stdout.contains("ExecutionNotImplemented"), "{stdout}");
    assert!(!stdout.contains("UnhandledRuntimeError"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        checkpoint_dir.join("checkpoint-0.json").is_file(),
        "live multi-agent run should write the retry-start checkpoint"
    );
    assert!(
        checkpoint_dir.join("checkpoint-1.json").is_file(),
        "live multi-agent run should write the retry-end checkpoint"
    );
    assert!(
        checkpoint_dir.join("checkpoint-4.json").is_file(),
        "live multi-agent run should write the cache-read checkpoint"
    );
    let checkpoint_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(checkpoint_dir.join("checkpoint-4.json")).unwrap())
            .unwrap();
    assert!(
        checkpoint_json["checkpoint"]["completed_host_boundaries"]
            .as_array()
            .is_some_and(|boundaries| boundaries.iter().any(|boundary| {
                boundary["kind"] == "tool"
                    && boundary["key"].as_str().is_some_and(|key| {
                        key.contains("tool:app.agent.runtime_variants.EvidenceLookup")
                    })
            })),
        "live multi-agent checkpoint must record the source tool-call boundary: {checkpoint_json}"
    );
}

#[test]
#[ignore = "requires local omlx server and ETAS_RUN_NETWORK_TESTS=1"]
fn project_agent_system_smoke_runs_with_live_omlx_openai() {
    let _guard = lock_model_cli_e2e();
    if std::env::var("ETAS_RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        return;
    }
    let project = agent_system_smoke_project();
    let command = live_omlx_openai_command();
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "local-omlx",
            "--allow-effects",
            "--budget-tokens",
            "128",
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.trim().is_empty(), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
#[ignore = "requires local omlx server and ETAS_RUN_NETWORK_TESTS=1"]
fn project_agent_typed_record_runs_with_live_omlx_openai() {
    let _guard = lock_model_cli_e2e();
    if std::env::var("ETAS_RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        return;
    }
    let source_project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler")
        .join("projects")
        .join("positive")
        .join("effect_agent_console_bridge");
    let project = copy_project_fixture("effect-agent-console-bridge-live-omlx", &source_project);
    write_live_omlx_runtime_profile(&project, "effect-agent-console-live-omlx");
    let command = live_omlx_openai_command();
    let (code, stdout, stderr) = run_process_with_command(
        command,
        [
            "run",
            path(&project),
            "--profile",
            "local-omlx",
            "--allow-effects",
            "--budget-tokens",
            "256",
        ],
        "",
    );

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.trim().is_empty(), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn top_level_examples_check_and_run_from_cli() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples");
    let mut sources = fs::read_dir(&examples)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "es"))
        .collect::<Vec<_>>();
    sources.sort();

    assert_eq!(sources.len(), 10);

    for source in &sources {
        let (code, stdout, stderr) = run(["etas", "check", path(source)]);
        assert_eq!(code, 0, "{} failed check:\n{stderr}", source.display());
        assert!(stdout.contains("checked 1 file"), "{stdout}");
        assert!(
            !stderr.contains("error["),
            "{} emitted check errors:\n{stderr}",
            source.display()
        );
    }

    let run_cases = [
        ("hello.es", "", "", "hello from Etas\n"),
        ("args.es", "", "alpha beta", "alpha\nbeta\n"),
        ("stdin_stdout.es", "echoed input\n", "", "echoed input\n\n"),
        ("gcd.es", "", "", "6\n"),
        ("binary_search.es", "", "", "4\n"),
        ("linear_search.es", "", "", "3\n"),
        ("max_subarray.es", "", "", "5\n"),
        ("two_sum_indices.es", "", "", "1\n"),
    ];

    for (file_name, stdin, args, expected_stdout) in run_cases {
        let source = examples.join(file_name);
        let run_args = if args.is_empty() {
            vec!["run".to_owned(), path(&source).to_owned()]
        } else {
            let mut values = vec![
                "run".to_owned(),
                path(&source).to_owned(),
                "--args".to_owned(),
            ];
            values.extend(args.split(' ').map(str::to_owned));
            values
        };
        let _guard = (!cli_process_lock_is_held_by_current_thread()).then(lock_cli_process);
        let mut child = Command::new(env!("CARGO_BIN_EXE_etas"))
            .args(&run_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        assert!(
            output.status.success(),
            "{} failed with status {:?}\nstderr:\n{}\nstdout:\n{}",
            source.display(),
            output.status.code(),
            stderr,
            stdout
        );
        assert_eq!(stdout, expected_stdout, "{}", source.display());
        assert!(
            stderr.is_empty(),
            "{} emitted stderr:\n{stderr}",
            source.display()
        );
    }

    let checkpoint_source = examples.join("checkpoint_resume.es");
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let trace = std::env::temp_dir().join(format!("etas-example-trace-{id}.json"));
    let checkpoint_dir = std::env::temp_dir().join(format!("etas-example-checkpoints-{id}"));

    let (code, stdout, stderr) = run_process(
        ["run", path(&checkpoint_source), "--trace-out", path(&trace)],
        "",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"value\":\"0\""), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        fs::read_to_string(&trace)
            .unwrap()
            .contains("etas.cli.interpreter-report.v1")
    );

    let (code, stdout, stderr) = run_process(["replay", path(&trace)], "");
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("replayed interpreter report"), "{stdout}");
    assert!(stdout.contains("\"value\":\"0\""), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");

    let (code, stdout, stderr) = run_process(
        [
            "run",
            path(&checkpoint_source),
            "--checkpoint-dir",
            path(&checkpoint_dir),
        ],
        "",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checkpoints: 1"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(checkpoint_dir.join("checkpoint-0.json").is_file());

    let (code, stdout, stderr) = run_process(
        ["resume", "0", "--checkpoint-dir", path(&checkpoint_dir)],
        "",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("resumed checkpoint 0"), "{stdout}");
    assert!(stdout.contains("\"value\":\"0\""), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

fn run<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let _guard = (!cli_process_lock_is_held_by_current_thread()).then(lock_cli_process);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = etas_cli::run_with(args, &mut stdout, &mut stderr);

    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn run_process<const N: usize>(args: [&str; N], stdin: &str) -> (i32, String, String) {
    run_process_with_command(Command::new(env!("CARGO_BIN_EXE_etas")), args, stdin)
}

fn run_process_in<const N: usize>(
    current_dir: &Path,
    args: [&str; N],
    stdin: &str,
) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    command.current_dir(current_dir);
    run_process_with_command(command, args, stdin)
}

fn run_process_with_command<const N: usize>(
    mut command: Command,
    args: [&str; N],
    stdin: &str,
) -> (i32, String, String) {
    let _guard = (!cli_process_lock_is_held_by_current_thread()).then(lock_cli_process);
    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap_or(1),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn agent_system_smoke_project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler")
        .join("projects")
        .join("positive")
        .join("agent_system_smoke")
}

fn full_phase1_agent_runtime_project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler")
        .join("projects")
        .join("positive")
        .join("full_phase1_agent_runtime")
}

fn multi_agent_system_project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler")
        .join("projects")
        .join("positive")
        .join("multi_agent_system")
}

fn interpreter_stress_project(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("etas_tests")
        .join("fixtures")
        .join("compiler")
        .join("interpreter_stress")
        .join("positive")
        .join(name)
}

fn spawn_policy_server(response_body: String) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock policy server");
    let addr = listener.local_addr().expect("mock policy server address");
    let base_url = mock_server_base_url(addr, "");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock policy server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_http_request(&mut stream);
                    if let Some(should_shutdown) =
                        mock_server_request_was_shutdown(&request, &shutdown_token)
                    {
                        if should_shutdown {
                            return String::new();
                        }
                        continue;
                    }
                    if request.is_empty() {
                        continue;
                    }
                    if !request.contains("/policy/evaluate") {
                        continue;
                    }
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response_body.len(),
                        response_body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write policy response");
                    return request;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "timed out waiting for policy request"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("mock policy server failed: {error}"),
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock policy server should be ready");
    (base_url, handle)
}

fn spawn_memory_approval_policy_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock policy server");
    let addr = listener.local_addr().expect("mock policy server address");
    let base_url = mock_server_base_url(addr, "");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock policy server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let mut memory_approval_requested = false;
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_http_request(&mut stream);
                    if let Some(should_shutdown) =
                        mock_server_request_was_shutdown(&request, &shutdown_token)
                    {
                        if should_shutdown {
                            return requests;
                        }
                        continue;
                    }
                    if request.is_empty() {
                        continue;
                    }
                    if !request.contains("/policy/evaluate") {
                        continue;
                    }
                    let body =
                        if request.contains("\"kind\":\"memory\"") && !memory_approval_requested {
                            memory_approval_requested = true;
                            serde_json::json!({
                                "decision": "require_approval",
                                "approval": {
                                    "id": 7001,
                                    "reason": "memory write requires approval",
                                    "requested_grants": [{
                                        "kind": "allow_pattern",
                                        "effect": "Memory",
                                        "action": "write",
                                        "args": [{ "kind": "any" }]
                                    }]
                                }
                            })
                        } else {
                            serde_json::json!({
                                "decision": "allow",
                            })
                        }
                        .to_string();
                    write_http_json_response(&mut stream, &body);
                    requests.push(request);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "timed out waiting for policy requests"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("mock policy server failed: {error}"),
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock policy server should be ready");
    (base_url, handle)
}

fn live_omlx_openai_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    if let Ok(key) = std::env::var("ETAS_HOST_OMLX_API_KEY") {
        command.env("ETAS_HOST_OMLX_API_KEY", key);
    }
    command
}

fn remove_legacy_runtime_host_env(command: &mut Command) {
    for name in [
        "ETAS_HOST_MODEL_ADAPTER",
        "ETAS_HOST_MODEL_BASE_URL",
        "ETAS_HOST_MODEL_NAME",
        "ETAS_HOST_MODEL_API_KEY",
        "ETAS_HOST_MODEL_ALLOW_PRIVATE",
        "ETAS_HOST_OMLX_API_KEY",
        "ETAS_HOST_MEMORY",
        "ETAS_HOST_APPROVAL",
        "ETAS_HOST_BOUNDARY_POLICY",
        "ETAS_HOST_POLICY",
        "ETAS_HOST_POLICY_RULES",
        "ETAS_HOST_POLICY_URL",
        "ETAS_HOST_POLICY_TOKEN",
        "ETAS_HOST_POLICY_PATH",
        "ETAS_HOST_POLICY_ALLOW_PRIVATE",
        "ETAS_HOST_TOOL_HTTP",
        "ETAS_HOST_TOOL_MCP",
        "ETAS_HOST_TOOL_PROCESS",
        "ETAS_HOST_TOOL_ALLOW_PRIVATE",
    ] {
        command.env_remove(name);
    }
}

fn write_local_static_mock_runtime_profile(
    project: &Path,
    profile: &str,
    model_base_url: &str,
    boundary_policy: &str,
) {
    std::fs::write(
        project.join("etas.local.toml"),
        format!(
            r#"
[runtime]
default_profile = "{profile}"

[runtime.profiles.{profile}]
boundary_policy = "{boundary_policy}"

[runtime.profiles.{profile}.model]
adapter = "openai"
model = "mock-agent-model"
base_url = "{model_base_url}"
allow_private = true

[runtime.profiles.{profile}.memory]
backend = "memory"

[runtime.profiles.{profile}.policy]
mode = "local-static"
rules = ["model=allow", "tool=allow", "memory=approval", "console=allow"]

[runtime.profiles.{profile}.approval]
mode = "auto"
"#
        ),
    )
    .expect("write full Phase1 mock runtime profile");
}

fn write_http_policy_mock_runtime_profile(
    project: &Path,
    profile: &str,
    model_base_url: &str,
    policy_url: &str,
    boundary_policy: &str,
) {
    std::fs::write(
        project.join("etas.local.toml"),
        format!(
            r#"
[runtime]
default_profile = "{profile}"

[runtime.profiles.{profile}]
boundary_policy = "{boundary_policy}"

[runtime.profiles.{profile}.model]
adapter = "openai"
model = "mock-agent-model"
base_url = "{model_base_url}"
allow_private = true

[runtime.profiles.{profile}.memory]
backend = "memory"

[runtime.profiles.{profile}.policy]
mode = "http"
url = "{policy_url}"
allow_private = true

[runtime.profiles.{profile}.approval]
mode = "auto"
"#
        ),
    )
    .expect("write HTTP policy mock runtime profile");
}

fn write_live_omlx_runtime_profile(project: &Path, boundary_policy: &str) {
    std::fs::write(
        project.join("etas.local.toml"),
        format!(
            r#"
[runtime]
default_profile = "local-omlx"

[runtime.profiles.local-omlx]
boundary_policy = "{boundary_policy}"

[runtime.profiles.local-omlx.model]
adapter = "omlx-openai"
model = "Qwen3.5-0.8B-MLX-4bit"
base_url = "http://127.0.0.1:8848/v1"
api_key_env = "ETAS_HOST_OMLX_API_KEY"

[runtime.profiles.local-omlx.memory]
backend = "memory"

[runtime.profiles.local-omlx.policy]
mode = "local-static"
rules = ["model=allow", "tool=allow", "memory=approval", "console=allow"]

[runtime.profiles.local-omlx.approval]
mode = "auto"

[runtime.profiles.local-omlx.network]
allow = ["127.0.0.1:8848"]
"#
        ),
    )
    .expect("write live OMLX runtime profile");
}

fn spawn_openai_external_tool_loop_server(
    final_answer: &'static str,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        for round in 0..2 {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if let Some(should_shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if should_shutdown {
                                return requests;
                            }
                            continue;
                        }
                        if request.is_empty() {
                            continue;
                        }
                        let body = match round {
                            0 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": null,
                                        "tool_calls": [{
                                            "id": "tool-call-1",
                                            "type": "function",
                                            "function": {
                                                "name": "EvidenceLookup",
                                                "arguments": r#"{"query":"runtime"}"#,
                                            },
                                        }],
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            _ => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": final_answer,
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 2,
                                }
                            }),
                        }
                        .to_string();
                        write_http_json_response(&mut stream, &body);
                        requests.push(request);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
                }
            }
        }
        requests
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn spawn_openai_edk_like_multi_agent_tool_server(
    final_answer: &'static str,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        for round in 0..3 {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if let Some(should_shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if should_shutdown {
                                return requests;
                            }
                            continue;
                        }
                        if request.is_empty() {
                            continue;
                        }
                        let body = match round {
                            0 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": "planner output for runtime",
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            1 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": null,
                                        "tool_calls": [{
                                            "id": "tool-call-1",
                                            "type": "function",
                                            "function": {
                                                "name": "EvidenceLookup",
                                                "arguments": r#"{"query":"runtime"}"#,
                                            },
                                        }],
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            _ => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": final_answer,
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 2,
                                }
                            }),
                        }
                        .to_string();
                        write_http_json_response(&mut stream, &body);
                        requests.push(request);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
                }
            }
        }
        requests
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn spawn_http_tool_value_server(
    response_json: String,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock HTTP tool server");
    let addr = listener
        .local_addr()
        .expect("mock HTTP tool server address");
    let base_url = mock_server_base_url(addr, "");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock HTTP tool server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_http_request(&mut stream);
                    if let Some(should_shutdown) =
                        mock_server_request_was_shutdown(&request, &shutdown_token)
                    {
                        if should_shutdown {
                            return requests;
                        }
                        continue;
                    }
                    if request.is_empty() {
                        continue;
                    }
                    write_http_json_response(&mut stream, &response_json);
                    requests.push(request);
                    return requests;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > deadline {
                        return requests;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("mock HTTP tool server failed to accept request: {error}"),
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock HTTP tool server should be ready");
    (base_url, handle)
}

fn spawn_openai_tool_loop_server(
    summary: &'static str,
    score: i32,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        for round in 0..3 {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if let Some(should_shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if should_shutdown {
                                return requests;
                            }
                            continue;
                        }
                        if request.is_empty() {
                            continue;
                        }
                        let body = match round {
                            0 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": "runtime plan from planner",
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            1 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": null,
                                        "tool_calls": [{
                                            "id": "tool-call-1",
                                            "type": "function",
                                            "function": {
                                                "name": "EvidenceLookup",
                                                "arguments": r#"{"input":{"topic":"runtime","draft":"draft"}}"#,
                                            },
                                        }],
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            _ => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": format!(r#"{{"summary":"{}","score":{}}}"#, summary, score),
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 2,
                                }
                            }),
                        }
                        .to_string();
                        write_http_json_response(&mut stream, &body);
                        requests.push(request);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
                }
            }
        }
        requests
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn spawn_openai_tool_arg_repair_server(
    summary: &'static str,
    score: i32,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        for round in 0..4 {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if let Some(should_shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if should_shutdown {
                                return requests;
                            }
                            continue;
                        }
                        if request.is_empty() {
                            continue;
                        }
                        let body = match round {
                            0 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": "runtime plan from planner",
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            1 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": null,
                                        "tool_calls": [{
                                            "id": "tool-call-invalid",
                                            "type": "function",
                                            "function": {
                                                "name": "EvidenceLookup",
                                                "arguments": r#"{"topic":"runtime"}"#,
                                            },
                                        }],
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            2 => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": null,
                                        "tool_calls": [{
                                            "id": "tool-call-repaired",
                                            "type": "function",
                                            "function": {
                                                "name": "EvidenceLookup",
                                                "arguments": r#"{"input":{"topic":"runtime","draft":"draft"}}"#,
                                            },
                                        }],
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 1,
                                }
                            }),
                            _ => serde_json::json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": format!(r#"{{"summary":"{}","score":{}}}"#, summary, score),
                                    }
                                }],
                                "usage": {
                                    "prompt_tokens": 1,
                                    "completion_tokens": 2,
                                }
                            }),
                        }
                        .to_string();
                        write_http_json_response(&mut stream, &body);
                        requests.push(request);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
                }
            }
        }
        requests
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn spawn_multi_agent_runtime_variants_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_http_request(&mut stream);
                    if let Some(should_shutdown) =
                        mock_server_request_was_shutdown(&request, &shutdown_token)
                    {
                        if should_shutdown {
                            return requests;
                        }
                        continue;
                    }
                    if request.is_empty() {
                        continue;
                    }
                    let body = multi_agent_runtime_response_for_request(&request, requests.len())
                        .to_string();
                    write_http_json_response(&mut stream, &body);
                    requests.push(request);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > deadline {
                        return requests;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn multi_agent_runtime_response_for_request(
    request: &str,
    request_index: usize,
) -> serde_json::Value {
    let topic = multi_agent_runtime_topic(request);
    if request.contains("\"role\":\"tool\"") {
        return serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": format!(r#"{{"evidence":"{}"}}"#, multi_agent_runtime_evidence(topic)),
                }
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 2,
            }
        });
    }
    if request.contains("\"response_format\"") && request.contains("\"json_schema\"") {
        let (summary, score) = multi_agent_runtime_review(topic);
        return serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": format!(r#"{{"summary":"{}","score":{}}}"#, summary, score),
                }
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 2,
            }
        });
    }
    if request.contains("\"name\":\"EvidenceLookup\"") {
        return serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": format!("tool-call-{request_index}"),
                        "type": "function",
                        "function": {
                            "name": "EvidenceLookup",
                            "arguments": format!(r#"{{"input":{{"topic":"{topic}","draft":"draft"}}}}"#),
                        },
                    }],
                }
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 1,
            }
        });
    }
    serde_json::json!({
        "choices": [{
            "message": {
                "role": "assistant",
                "content": multi_agent_runtime_plan(topic),
            }
        }],
        "usage": {
            "prompt_tokens": 1,
            "completion_tokens": 2,
        }
    })
}

fn multi_agent_runtime_topic(request: &str) -> &'static str {
    if request.contains("runtime-conflict") {
        "runtime-conflict"
    } else if request.contains("runtime-cache") {
        "runtime-cache"
    } else if request.contains("runtime-guard") {
        "runtime-guard"
    } else {
        "runtime"
    }
}

fn multi_agent_runtime_plan(topic: &str) -> &'static str {
    match topic {
        "runtime-conflict" => "conflict plan",
        "runtime-cache" => "cached plan",
        "runtime-guard" => "guard plan",
        _ => "checkpointed plan",
    }
}

fn multi_agent_runtime_evidence(topic: &str) -> &'static str {
    match topic {
        "runtime-conflict" => "conflict evidence",
        "runtime-cache" => "cached evidence",
        "runtime-guard" => "guard evidence",
        _ => "checkpointed evidence",
    }
}

fn multi_agent_runtime_review(topic: &str) -> (&'static str, i32) {
    match topic {
        "runtime-conflict" => ("conflict review", 7),
        "runtime-cache" => ("cached review", 9),
        "runtime-guard" => ("guard review", -1),
        _ => ("checkpointed review", 8),
    }
}

fn spawn_openai_completion_server(
    response_text: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_http_request(&mut stream);
                    if let Some(should_shutdown) =
                        mock_server_request_was_shutdown(&request, &shutdown_token)
                    {
                        if should_shutdown {
                            return String::new();
                        }
                        continue;
                    }
                    if request.is_empty() {
                        continue;
                    }
                    let body = serde_json::json!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": response_text,
                            }
                        }],
                        "usage": {
                            "prompt_tokens": 1,
                            "completion_tokens": 2,
                        }
                    })
                    .to_string();
                    write_http_json_response(&mut stream, &body);
                    return request;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > deadline {
                        return String::new();
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn spawn_openai_completion_sequence_server(
    responses: &[&str],
) -> (String, thread::JoinHandle<Vec<String>>) {
    let responses = responses
        .iter()
        .map(|response| (*response).to_owned())
        .collect::<Vec<_>>();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock OpenAI server");
    let addr = listener.local_addr().expect("mock server address");
    let base_url = mock_server_base_url(addr, "/v1");
    let shutdown_token = mock_server_shutdown_token(&base_url);
    listener
        .set_nonblocking(true)
        .expect("mock server nonblocking mode");
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let mut requests = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(300);
        for response_text in responses {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        if let Some(should_shutdown) =
                            mock_server_request_was_shutdown(&request, &shutdown_token)
                        {
                            if should_shutdown {
                                return requests;
                            }
                            continue;
                        }
                        if request.is_empty() {
                            continue;
                        }
                        let body = serde_json::json!({
                            "choices": [{
                                "message": {
                                    "role": "assistant",
                                    "content": response_text,
                                }
                            }],
                            "usage": {
                                "prompt_tokens": 1,
                                "completion_tokens": 2,
                            }
                        })
                        .to_string();
                        write_http_json_response(&mut stream, &body);
                        requests.push(request);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock OpenAI server failed to accept request: {error}"),
                }
            }
        }
        requests
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock OpenAI server should be ready");
    (base_url, handle)
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    while let Ok(read) = stream.read(&mut chunk) {
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if request_body_complete(&buffer) {
            break;
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}

fn is_mock_server_shutdown_request(request: &str) -> bool {
    shutdown_token_from_request(request).is_some()
}

fn mock_server_request_was_shutdown(request: &str, server_token: &str) -> Option<bool> {
    is_mock_server_shutdown_request(request)
        .then(|| shutdown_token_from_request(request).is_some_and(|token| token == server_token))
}

fn shutdown_mock_server(base_url: &str) {
    let Some((addr, token)) = mock_server_addr_and_token(base_url) else {
        return;
    };
    if let Ok(mut stream) = std::net::TcpStream::connect(addr) {
        let request = format!(
            "GET /__shutdown/{token} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
        );
        let _ = stream.write_all(request.as_bytes());
    }
}

fn mock_server_base_url(addr: std::net::SocketAddr, suffix: &str) -> String {
    let token = format!("mock-{}", NEXT_FILE.fetch_add(1, Ordering::Relaxed));
    let addr = addr.to_string();
    format!("http://{addr}/__etas_mock/{token}{suffix}")
}

fn mock_server_shutdown_token(base_url: &str) -> String {
    mock_server_addr_and_token(base_url)
        .expect("mock server base URL should contain a shutdown token")
        .1
        .to_owned()
}

fn mock_server_addr_and_token(base_url: &str) -> Option<(&str, &str)> {
    let rest = base_url.strip_prefix("http://")?;
    let (addr, path) = rest.split_once('/')?;
    let token = path.strip_prefix("__etas_mock/")?.split('/').next()?;
    Some((addr, token))
}

fn shutdown_token_from_request(request: &str) -> Option<String> {
    let request_line = request.lines().next()?;
    let path = request_line
        .strip_prefix("GET ")?
        .split_whitespace()
        .next()?;
    path.strip_prefix("/__shutdown/")
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

fn write_http_json_response(stream: &mut std::net::TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .expect("write mock response");
    stream.flush().expect("flush mock response");
}

fn request_body_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    buffer.len() >= header_end + 4 + content_length
}

fn fixture(name: &str, contents: &str) -> PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("etas-cli-{name}-{id}.es"));
    std::fs::write(&path, contents).unwrap();
    path
}

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("etas-cli-{name}-{id}.{extension}"))
}

fn read_profile_report(path: &Path) -> serde_json::Value {
    let bytes = fs::read(path).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn assert_profile_has_span(report: &serde_json::Value, name: &str) {
    let spans = report["spans"].as_array().expect("profile spans");
    assert!(
        spans.iter().any(|span| span["name"] == name),
        "missing profile span `{name}` in {report:#}"
    );
}

fn assert_profile_has_counter(report: &serde_json::Value, name: &str) {
    let counters = report["counters"].as_array().expect("profile counters");
    assert!(
        counters.iter().any(|counter| counter["name"] == name),
        "missing profile counter `{name}` in {report:#}"
    );
}

fn assert_profile_category_nested_under(
    report: &serde_json::Value,
    child_category: &str,
    parent_name: &str,
) {
    let spans = report["spans"].as_array().expect("profile spans");
    let parents = spans
        .iter()
        .filter(|span| span["name"] == parent_name)
        .collect::<Vec<_>>();
    assert!(
        !parents.is_empty(),
        "missing profile parent span `{parent_name}` in {report:#}"
    );

    let children = spans
        .iter()
        .filter(|span| span["category"] == child_category)
        .collect::<Vec<_>>();
    assert!(
        !children.is_empty(),
        "missing profile child category `{child_category}` in {report:#}"
    );

    for child in children {
        let child_start = profile_span_start_ns(child);
        let child_end = profile_span_end_ns(child);
        assert!(
            parents.iter().any(|parent| {
                let parent_start = profile_span_start_ns(parent);
                let parent_end = profile_span_end_ns(parent);
                parent_start <= child_start && child_end <= parent_end
            }),
            "profile span `{}` was not nested under `{parent_name}` by timing: {report:#}",
            child["name"].as_str().unwrap_or("<unnamed>")
        );
    }
}

fn profile_span_start_ns(span: &serde_json::Value) -> u64 {
    span["start_ns"].as_u64().expect("profile span start_ns")
}

fn profile_span_end_ns(span: &serde_json::Value) -> u64 {
    profile_span_start_ns(span).saturating_add(
        span["duration_ns"]
            .as_u64()
            .expect("profile span duration_ns"),
    )
}

fn max_profile_counter(report: &serde_json::Value, name: &str) -> Option<u64> {
    report["counters"]
        .as_array()
        .expect("profile counters")
        .iter()
        .filter(|counter| counter["name"] == name)
        .filter_map(|counter| counter["value"].as_u64())
        .max()
}

fn profile_contains(path: &Path, needle: &str) -> bool {
    fs::read_to_string(path).unwrap().contains(needle)
}

fn package_fixture(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("etas-cli-{name}-{id}"));
    let _ = std::fs::remove_dir_all(&root);
    for (relative, contents) in files {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    root
}

fn copy_project_fixture(name: &str, source: &Path) -> PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("etas-cli-{name}-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    copy_dir_contents(source, &root);
    root
}

fn copy_dir_contents(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            if entry.file_name() == ".etas" || entry.file_name() == "target" {
                continue;
            }
            copy_dir_contents(&source_path, &destination_path);
        } else if file_type.is_file() {
            std::fs::copy(&source_path, &destination_path).unwrap();
        }
    }
}

fn seal_package_metadata(package_root: &Path) {
    let text = fs::read_to_string(package_root.join(".etas/package-index.json")).unwrap();
    let index = serde_json::from_str::<etas_package::PackageIndex>(&text).unwrap();
    etas_package::write_fixture_package_metadata_artifact(package_root, &index).unwrap();
}

fn import_discovery_package(name: &str) -> PathBuf {
    package_fixture(
        name,
        &[
            (
                "src/app/main.es",
                r#"module app.main;

import std.io.println;
import app.support.{message};

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    println(message());
    return 0;
}
"#,
            ),
            (
                "src/app/support.es",
                r#"module app.support;

public flow message() -> string {
    return "loaded";
}
"#,
            ),
        ],
    )
}

fn path(path: &std::path::Path) -> &str {
    path.to_str().unwrap()
}

fn io_section(text: &str, start: &str, end: &str) -> String {
    let mut in_section = false;
    let mut output = String::new();
    for line in text.lines() {
        if line == start {
            in_section = true;
            continue;
        }
        if line == end {
            break;
        }
        if in_section {
            output.push_str(line.strip_prefix("    ").unwrap_or(line));
            output.push('\n');
        }
    }
    output
}
