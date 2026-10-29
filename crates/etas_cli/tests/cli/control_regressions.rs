use super::*;

fn isolated_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(|name| name.starts_with("ETAS_")) {
            command.env_remove(name);
        }
    }
    command.arg("--no-config");
    command
}

#[test]
fn packaged_cli_preserves_nested_records_and_tail_control_values() {
    let file = fixture(
        "control-tail-record",
        r#"
module app.main;
type Session = { value: i32 };
flow exercise(session: Session) -> bool { return session.value == 7; }
flow choose(flag: bool) -> i32 {
    if flag { 2 } else { match flag { true => 3, false => 4 } }
}
flow main(args: Array<string>) -> i32 {
    if !exercise(Session { value = 7 }) { return 99; }
    choose(false) + choose(true)
}
"#,
    );
    let (code, _, stderr) =
        run_process_with_command(isolated_command(), ["check", path(&file)], "");
    assert_eq!(code, 0, "{stderr}");
    let (code, stdout, stderr) =
        run_process_with_command(isolated_command(), ["run", path(&file)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"6"}"#),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn packaged_cli_accepts_mutually_exclusive_resume_paths() {
    let file = fixture(
        "control-resume-branches",
        r#"
module app.main;
effect Solver { action ask() -> i32; }
flow main(args: Array<string>) -> i32 {
    let cached = args.is_empty();
    return perform Solver.ask() with {
        Solver.ask() => { if cached { resume 11; } resume 12; }
    };
}
"#,
    );
    for (args, expected) in [(Vec::new(), "11"), (vec!["uncached"], "12")] {
        let mut command = isolated_command();
        command.args(["run", path(&file), "--args"]).args(args);
        let (code, stdout, stderr) = run_process_with_command(command, [], "");
        assert_eq!(code, 0, "{stderr}\n{stdout}");
        assert!(
            stdout.contains(&format!(
                r#"run value: {{"kind":"number","type":"i32","value":"{expected}"}}"#
            )),
            "{stdout}"
        );
        assert!(stderr.is_empty(), "{stderr}");
    }
}

#[test]
fn packaged_cli_executes_unicode_string_ordering() {
    let file = fixture(
        "control-string-order",
        r#"
module app.main;
flow ordered() -> bool {
    let prefix = "中";
    let key = prefix + "😀";
    return "a" < key && key > "a" && key <= "中😀" && key >= "中😀";
}
flow main(args: Array<string>) -> i32 { if ordered() { 1 } else { 0 } }
"#,
    );
    let (code, stdout, stderr) =
        run_process_with_command(isolated_command(), ["run", path(&file)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert!(
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"1"}"#),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn packaged_cli_retains_progress_handler_after_outer_resume() {
    let file = fixture(
        "control-scoped-resume",
        r#"
module app.main;
import std.io.println;
effect Stages { action main() -> i32; }
effect Progress { action write(text: string) -> unit; }
flow body() -> i32 ![Stages, Progress] {
    perform Progress.write("before");
    let value = perform Stages.main();
    perform Progress.write("after");
    return value;
}
flow facade() -> i32 ![Stages, Console, Error<IOError>] {
    return body() with { Progress.write(text) => { println(text); resume; } };
}
flow main(args: Array<string>) -> i32 ![Console, Error<IOError>] {
    return facade() with { Stages.main() => { resume 0; } };
}
"#,
    );
    let (code, stdout, stderr) =
        run_process_with_command(isolated_command(), ["run", path(&file)], "");
    assert_eq!(code, 0, "{stderr}\n{stdout}");
    assert_eq!(stdout, "before\nafter\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn packaged_cli_reports_the_embedded_compiler_patch_version() {
    let (code, stdout, stderr) = run_process_with_command(isolated_command(), ["--version"], "");
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
    assert_eq!(
        etas_frontend::FRONTEND_COMPILER_VERSION,
        env!("CARGO_PKG_VERSION")
    );
}
