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
fn packaged_cli_keeps_secret_wrapper_and_effect_separate() {
    let file = fixture(
        "secret-prelude",
        r#"
module app.main;
spec NoSecrets: trace = -Secret;
flow wrap(value: i32) -> Secret<i32> ![] { return Secret(value); }
flow identity(value: Secret<i32>) -> Secret<i32> ![] { return value; }
flow main(args: Array<string>) -> i32 ![] ~ NoSecrets {
    let Secret = 21;
    let hidden = identity(wrap(Secret));
    return Secret;
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
        stdout.contains(r#"run value: {"kind":"number","type":"i32","value":"21"}"#),
        "{stdout}"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn packaged_cli_broad_secret_constraint_rejects_a_real_action() {
    let file = fixture(
        "secret-prelude-denied",
        r#"
module app.main;
type ApiKey;
spec NoSecrets: trace = -Secret;
flow protected(key: SecretKey<ApiKey>) -> unit ![Error<SecretError>] ~ NoSecrets {
    std.secret.read(key);
    return;
}
flow main(args: Array<string>) -> i32 ![] { return 0; }
"#,
    );
    let (code, stdout, stderr) =
        run_process_with_command(isolated_command(), ["check", "--all", path(&file)], "");
    assert_eq!(code, 1, "{stderr}\n{stdout}");
    assert!(stderr.contains("TraceSpecDenied"), "{stderr}");
    assert!(!stderr.contains("IncompleteEffectFacts"), "{stderr}");
}
