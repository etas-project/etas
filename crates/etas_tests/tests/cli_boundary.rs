mod support;

use std::{fs, path::PathBuf};

use support::{fixture, path_str, run_cli};

#[test]
fn cli_check_maps_valid_and_invalid_programs_to_stable_exit_codes() {
    let positive = fixture("cli/positive/check_ok.es");
    let (code, stdout, stderr) = run_cli(["etas", "check", path_str(&positive)]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.is_empty());

    let negative = fixture("cli/negative/check_syntax_error.es");
    let (code, stdout, stderr) = run_cli(["etas", "check", path_str(&negative)]);
    assert_eq!(code, 1);
    assert!(stdout.contains("checked 1 file"));
    assert!(stderr.contains("error[syntax::"));
}

#[test]
fn cli_json_check_uses_stdout_for_artifacts_and_stderr_for_diagnostics() {
    let fixture = fixture("cli/negative/check_syntax_error.es");
    let (code, stdout, stderr) = run_cli(["etas", "--format", "json", "check", path_str(&fixture)]);

    assert_eq!(code, 1);
    assert!(stdout.contains("\"command\":\"check\""));
    assert!(stdout.contains("\"diagnostics\""));
    assert!(stderr.is_empty());
}

#[test]
fn cli_check_compiles_explicit_inputs_as_one_project() {
    let root = temp_project_root("explicit-project-inputs");
    let src = root.join("src").join("app");
    fs::create_dir_all(&src).unwrap();
    let main = src.join("main.es");
    let util = src.join("util.es");
    fs::write(
        &main,
        "module app.main;\nimport app.util.{helper};\nflow main(args: Array<string>) -> i32 { return 0; }\n",
    )
    .unwrap();
    fs::write(
        &util,
        "module app.util;\npublic flow helper() -> unit { return; }\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "check",
        path_str(&main),
        path_str(&util),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_check_all_discovers_workspace_sources_as_one_project() {
    let root = temp_project_root("workspace-all");
    let src = root.join("src").join("app");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        root.join("etas.toml"),
        "[package]\nname = \"workspace_all\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        src.join("main.es"),
        "module app.main;\nimport app.util.{helper};\nflow main(args: Array<string>) -> i32 { return 0; }\n",
    )
    .unwrap();
    fs::write(
        src.join("util.es"),
        "module app.util;\npublic flow helper() -> unit { return; }\n",
    )
    .unwrap();

    let (code, stdout, stderr) =
        run_cli(["etas", "--workspace", path_str(&root), "check", "--all"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_check_uses_explicit_frontend_cache_root() {
    let root = temp_project_root("check-cache-root");
    let (main, _) = write_two_file_project(&root);
    let cache_root = root.join("custom-cache");

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "--cache-root",
        path_str(&cache_root),
        "check",
        path_str(&main),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"));
    assert!(stderr.is_empty());
    assert!(
        cache_root.join("v1").join("cache.sqlite").exists(),
        "explicit cache root should receive the frontend disk cache index"
    );

    let off_root = root.join("off-cache");
    let (code, _stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "--cache",
        "off",
        "--cache-root",
        path_str(&off_root),
        "check",
        path_str(&main),
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        !off_root.exists(),
        "`--cache off` should keep the frontend session memory-only"
    );
}

#[test]
fn cli_disk_cache_rebuilds_from_current_source_input() {
    let root = temp_project_root("check-cache-current-source");
    let (main, _) = write_two_file_project(&root);
    let cache_root = root.join("custom-cache");

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "--cache-root",
        path_str(&cache_root),
        "check",
        path_str(&main),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("checked 2 files"));
    assert!(stderr.is_empty());

    fs::write(
        &main,
        "module app.main;\nimport app.util.{helper};\nflow main(args: Array<string>) -> i32 { let broken = ; }\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "--cache-root",
        path_str(&cache_root),
        "check",
        path_str(&main),
    ]);

    assert_eq!(code, 1);
    assert!(stdout.contains("checked 2 files"));
    assert!(stderr.contains("error[syntax::"), "{stderr}");
    assert!(stderr.contains("let broken = ;"), "{stderr}");
}

#[test]
fn cli_dump_hir_compiles_explicit_inputs_as_project_hir() {
    let root = temp_project_root("dump-hir-explicit-project");
    let (main, util) = write_two_file_project(&root);

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "dump",
        "hir",
        path_str(&main),
        path_str(&util),
        "--symbols",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("HirProgram"));
    assert!(stdout.contains("main"));
    assert!(stdout.contains("helper"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_dump_hir_all_discovers_workspace_sources_as_project_hir() {
    let root = temp_project_root("dump-hir-all-project");
    write_two_file_project(&root);

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "dump",
        "hir",
        "--all",
        "--symbols",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("HirProgram"));
    assert!(stdout.contains("main"));
    assert!(stdout.contains("helper"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_dump_ast_and_hir_stay_on_their_own_layers() {
    let fixture = fixture("cli/positive/dump_pipeline.es");

    let (ast_code, ast_stdout, ast_stderr) = run_cli([
        "etas",
        "dump",
        "ast",
        path_str(&fixture),
        "--format",
        "text",
    ]);
    assert_eq!(ast_code, 0, "{ast_stderr}");
    assert!(ast_stdout.contains("Program"));
    assert!(ast_stdout.contains("PipelineExpr"));
    assert!(!ast_stdout.contains("Symbols"));

    let (hir_code, hir_stdout, hir_stderr) = run_cli([
        "etas",
        "dump",
        "hir",
        path_str(&fixture),
        "--format",
        "text",
    ]);
    assert_eq!(hir_code, 0, "{hir_stderr}");
    assert!(hir_stdout.contains("HirProgram"));
    assert!(hir_stdout.contains("Path Writer(s"));
    assert!(!hir_stdout.contains("PipelineExpr"));
}

#[test]
fn cli_run_executes_checked_hir_with_interpreter() {
    let root = temp_project_root("run-interpreter");
    fs::create_dir_all(&root).unwrap();
    let main = root.join("main.es");
    fs::write(
        &main,
        "module app.main;\nflow main(args: Array<string>) -> i32 { return 42; }\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        path_str(&main),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"));
    assert!(stdout.contains("\"kind\":\"int\""));
    assert!(stdout.contains("\"value\":42"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_run_executes_explicit_multi_source_project_with_interpreter() {
    let root = temp_project_root("run-explicit-multi-source");
    let (main, util) = write_two_file_project(&root);

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        path_str(&main),
        path_str(&util),
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"));
    assert!(stdout.contains("\"kind\":\"int\""));
    assert!(stdout.contains("\"value\":0"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_run_all_discovers_workspace_sources_as_one_project() {
    let root = temp_project_root("run-all-project");
    write_two_file_project(&root);

    let (code, stdout, stderr) = run_cli(["etas", "--workspace", path_str(&root), "run", "--all"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("run value"));
    assert!(stdout.contains("\"kind\":\"int\""));
    assert!(stdout.contains("\"value\":0"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_run_writes_interpreter_trace_and_replay_reads_it() {
    let root = temp_project_root("run-replay-interpreter");
    fs::create_dir_all(&root).unwrap();
    let main = root.join("main.es");
    let trace = root.join("trace.json");
    fs::write(
        &main,
        "module app.main;\nflow main(args: Array<string>) -> i32 { return 0; }\n",
    )
    .unwrap();

    let (run_code, run_stdout, run_stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        path_str(&main),
        "--trace-out",
        path_str(&trace),
    ]);

    assert_eq!(run_code, 0, "{run_stderr}");
    assert!(run_stdout.contains("\"value\":0"));
    assert!(run_stderr.is_empty());
    let trace_text = fs::read_to_string(&trace).unwrap();
    assert!(trace_text.contains("etas.cli.interpreter-report.v1"));

    let (replay_code, replay_stdout, replay_stderr) = run_cli(["etas", "replay", path_str(&trace)]);

    assert_eq!(replay_code, 0, "{replay_stderr}");
    assert!(replay_stdout.contains("replayed interpreter report"));
    assert!(replay_stdout.contains("\"value\":0"));
    assert!(replay_stderr.is_empty());
}

#[test]
fn cli_resume_continues_multi_source_project_checkpoint() {
    let root = temp_project_root("resume-multi-source-project");
    let src = root.join("src").join("app");
    fs::create_dir_all(src.join("support")).unwrap();
    let main = src.join("main.es");
    let support = src.join("support").join("checkpoint.es");
    let checkpoints = root.join("checkpoints");
    fs::write(
        &main,
        "module app.main;\nimport app.support.checkpoint.{finish_value};\nimport std.runtime.checkpoint.{checkpoint};\nflow main(args: Array<string>) -> i32 { checkpoint(\"pause\"); let value = finish_value(); if value == \"done\" { return 0; } return 1; }\n",
    )
    .unwrap();
    fs::write(
        &support,
        "module app.support.checkpoint;\npublic flow finish_value() -> string ![] { return \"done\"; }\n",
    )
    .unwrap();

    let (run_code, _run_stdout, run_stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        "--all",
        "--checkpoint-dir",
        path_str(&checkpoints),
    ]);
    assert_eq!(run_code, 0, "{run_stderr}");

    let checkpoint = checkpoints.join("checkpoint-0.json");
    let checkpoint_text = fs::read_to_string(&checkpoint).unwrap();
    assert!(checkpoint_text.contains("\"sources\""));
    assert!(checkpoint_text.contains(path_str(&main)));
    assert!(checkpoint_text.contains(path_str(&support)));

    let (resume_code, resume_stdout, resume_stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "resume",
        "0",
        "--checkpoint-dir",
        path_str(&checkpoints),
    ]);
    assert_eq!(resume_code, 0, "{resume_stderr}");
    assert!(resume_stdout.contains("\"value\":0"));
}

#[test]
fn cli_run_accepts_budget_options_as_runtime_overrides() {
    let root = temp_project_root("run-budget-accepted");
    fs::create_dir_all(&root).unwrap();
    let main = root.join("main.es");
    fs::write(
        &main,
        "module app.main;\nflow main(args: Array<string>) -> i32 { return 1; }\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        path_str(&main),
        "--budget-tokens",
        "100",
    ]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"value\":1"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn cli_replay_rejects_until_without_cursor_addressable_trace() {
    let root = temp_project_root("replay-until-rejected");
    fs::create_dir_all(&root).unwrap();
    let trace = root.join("trace.json");
    fs::write(
        &trace,
        r#"{"schema":"etas.cli.interpreter-report.v1","events":[],"checkpoints":[]}"#,
    )
    .unwrap();

    let (code, stdout, stderr) = run_cli([
        "etas",
        "replay",
        path_str(&trace),
        "--until",
        "checkpoint:0",
    ]);

    assert_eq!(code, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("requires cursor-addressable trace events"));
}

#[test]
fn cli_resume_continues_from_interpreter_checkpoint() {
    let root = temp_project_root("resume-interpreter");
    fs::create_dir_all(&root).unwrap();
    let main = root.join("main.es");
    let checkpoint_dir = root.join("checkpoints");
    fs::write(
        &main,
        r#"module app.main;
import std.runtime.{checkpoint};

flow main(args: Array<string>) -> i32 {
    checkpoint("pause");
    return 0;
}
"#,
    )
    .unwrap();

    let (run_code, run_stdout, run_stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "run",
        path_str(&main),
        "--checkpoint-dir",
        path_str(&checkpoint_dir),
    ]);

    assert_eq!(run_code, 0, "{run_stderr}");
    assert!(run_stdout.contains("checkpoints: 1"));
    assert!(run_stderr.is_empty(), "{run_stderr}");
    assert!(checkpoint_dir.join("checkpoint-0.json").is_file());

    let (resume_code, resume_stdout, resume_stderr) = run_cli([
        "etas",
        "--workspace",
        path_str(&root),
        "resume",
        "0",
        "--checkpoint-dir",
        path_str(&checkpoint_dir),
    ]);

    assert_eq!(resume_code, 0, "{resume_stderr}");
    assert!(resume_stdout.contains("resumed checkpoint 0"));
    assert!(resume_stdout.contains("\"value\":0"));
    assert!(resume_stderr.is_empty(), "{resume_stderr}");
}

fn temp_project_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("etas-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    root
}

fn write_two_file_project(root: &PathBuf) -> (PathBuf, PathBuf) {
    let src = root.join("src").join("app");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        root.join("etas.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"demo\"\nmodule = \"app.main\"\nflow = \"main\"\n",
    )
    .unwrap();
    let main = src.join("main.es");
    let util = src.join("util.es");
    fs::write(
        &main,
        "module app.main;\nimport app.util.{helper};\nflow main(args: Array<string>) -> i32 { return 0; }\n",
    )
    .unwrap();
    fs::write(
        &util,
        "module app.util;\npublic flow helper() -> unit { return; }\n",
    )
    .unwrap();
    (main, util)
}

#[test]
fn cli_deferred_air_dump_fails_explicitly() {
    let fixture = fixture("cli/positive/check_ok.es");
    let (code, stdout, stderr) = run_cli(["etas", "dump", "air", path_str(&fixture)]);

    assert_eq!(code, 4);
    assert!(stdout.is_empty());
    assert!(stderr.contains("`etas dump air` is not available yet"));
}
