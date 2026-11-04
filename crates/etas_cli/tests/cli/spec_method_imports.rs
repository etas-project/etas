use super::*;

fn spec_method_cli<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_etas"));
    remove_legacy_runtime_host_env(&mut command);
    run_process_with_command(command, args, "")
}

#[test]
fn generic_spec_method_with_grouped_unused_imports_checks_and_runs() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spec_method_imports");
    let root = copy_project_fixture("spec-method-imports", &source);
    let (code, stdout, stderr) = spec_method_cli(["pkg", "lock", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    let (code, stdout, stderr) = spec_method_cli(["check", "--all", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("checked 3 files"), "{stdout}\n{stderr}");
    let (code, stdout, stderr) = spec_method_cli(["run", path(&root)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(
        stdout.contains(r#""type":"i32","value":"21""#),
        "{stdout}\n{stderr}"
    );
}
