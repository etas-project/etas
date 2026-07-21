mod support;

use std::path::{Path, PathBuf};

use support::{fixtures_under, path_str, run_cli};

const HANDLER_POSITIVE: &str = "compiler/handlers/positive";
const HANDLER_NEGATIVE: &str = "compiler/handlers/negative";
const ALLOWED_WITH_HANDLER_REGRESSION: &str =
    "compiler/handlers/negative/reject_with_handler_keyword.es";
const ALLOWED_WITH_HANDLER_DIAGNOSTIC: &str =
    "compiler/handlers/negative/reject_with_handler_keyword.diagnostics.txt";

#[test]
fn handler_positive_fixtures_cover_current_spec_surface() {
    let fixtures = fixtures_under(HANDLER_POSITIVE);
    assert_eq!(
        fixtures.len(),
        12,
        "expected 12 positive handler compiler fixtures"
    );

    let mut failures = Vec::new();
    for fixture in fixtures {
        assert_handler_fixture_shape(&fixture, true, &mut failures);
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        if code != 0 {
            failures.push(format!(
                "{} should compile under the current handler SPEC\nstdout:\n{stdout}\nstderr:\n{stderr}",
                fixture.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "handler positive fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn handler_negative_fixtures_report_targeted_diagnostics() {
    let fixtures = fixtures_under(HANDLER_NEGATIVE);
    assert_eq!(
        fixtures.len(),
        17,
        "expected 17 negative handler compiler fixtures"
    );

    let mut failures = Vec::new();
    for fixture in fixtures {
        assert_handler_fixture_shape(&fixture, false, &mut failures);
        let expected = read_expected_handler_diagnostics(&fixture, &mut failures);
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        let output = format!("{stdout}\n{stderr}");

        if code == 0 {
            failures.push(format!(
                "{} should be rejected by handler checking",
                fixture.display()
            ));
            continue;
        }

        for expectation in &expected {
            if !output.contains(&expectation.code) || !output.contains(&expectation.message) {
                failures.push(format!(
                    "{} missing expected diagnostic `{} | {}`\nstdout:\n{stdout}\nstderr:\n{stderr}",
                    fixture.display(),
                    expectation.code,
                    expectation.message
                ));
            }
        }

        let expects_syntax_diagnostic = expected
            .iter()
            .any(|expectation| expectation.code.starts_with("syntax::"));
        if !expects_syntax_diagnostic {
            for actual_code in diagnostic_codes(&output) {
                if is_fallback_diagnostic(&actual_code) {
                    failures.push(format!(
                        "{} reported fallback diagnostic `{actual_code}`; handler fixtures need targeted compiler diagnostics instead of implementation-gap placeholders\nstdout:\n{stdout}\nstderr:\n{stderr}",
                        fixture.display()
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "handler negative fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn only_rejection_fixture_uses_obsolete_with_handler_keyword() {
    let fixture_root = support::fixture("");
    let mut failures = Vec::new();
    collect_with_handler_violations(&fixture_root, &fixture_root, &mut failures);

    assert!(
        failures.is_empty(),
        "`with handler {{ ... }}` should appear only in the dedicated rejection fixture:\n{}",
        failures.join("\n")
    );
}

fn assert_handler_fixture_shape(fixture: &Path, positive: bool, failures: &mut Vec<String>) {
    let source = std::fs::read_to_string(fixture).unwrap();
    let Some(file_name) = fixture.file_name().and_then(|name| name.to_str()) else {
        failures.push(format!("{} has no file name", fixture.display()));
        return;
    };

    if source.contains("\neffect Web")
        || source.contains("\neffect Error[")
        || source.contains("ProjectMemorySchema")
    {
        failures.push(format!(
            "{} should not inline a fake effect registry",
            fixture.display()
        ));
    }
    if source.contains("try ") || source.contains("catch ") || source.contains("throw ") {
        failures.push(format!(
            "{} should use Error<E>, ?, and handle; try/catch/throw are not current SPEC",
            fixture.display()
        ));
    }
    if source.contains("with handler {") && file_name != "reject_with_handler_keyword.es" {
        failures.push(format!(
            "{} should apply anonymous handler blocks as `with {{ ... }}`; `with handler {{ ... }}` is rejected",
            fixture.display()
        ));
    }
    let marker = handler_marker(file_name, positive);
    if !source.contains(marker) {
        failures.push(format!(
            "{} should contain handler SPEC marker `{marker}`",
            fixture.display()
        ));
    }
}

fn collect_with_handler_violations(root: &Path, dir: &Path, failures: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_with_handler_violations(root, &path, failures);
            continue;
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if extension != "es" && extension != "txt" {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        if !source.contains("with handler") {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        if relative == ALLOWED_WITH_HANDLER_REGRESSION
            || relative == ALLOWED_WITH_HANDLER_DIAGNOSTIC
        {
            continue;
        }
        failures.push(relative.into_owned());
    }
}

fn handler_marker(file_name: &str, positive: bool) -> &'static str {
    match (positive, file_name) {
        (true, "handler_type_forms.es") => "alias HandleOnly = ![Gate]",
        (true, "inline_handler_block.es") => "} with {",
        (true, "reusable_handler_value.es") => "handler {",
        (true, "handler_produced_console.es") => "let HumanGate: ![Gate => Error<IOError>]",
        (true, "error_fallback_result_type.es") => "![Error<IndexError> => [] for string]",
        (true, "flow_trailing_handler.es") => "} with {",
        (true, "handler_expr_body_arm.es") => "Gate.request(req) => resume true;",
        (true, "handler_parameter_value.es") => "gate_handler: ![Gate => []]",
        (true, "handler_return_value.es") => "flow choose_handler(auto: bool) -> ![Gate => []]",
        (true, "nested_handlers.es") => "guarded_label() with AutoGate",
        (true, "index_error_inline_handler.es") => "Error<IndexError>.raise(err)",
        (true, "handler_value_no_top_level_effect.es") => "let AutoGate: ![Gate => []] = handler",
        (false, "resume_outside_handler.es") => "resume true;",
        (false, "resume_twice.es") => "resume false;",
        (false, "resume_never_action.es") => "resume \"fallback\";",
        (false, "resume_captured_by_lambda.es") => "=> {",
        (false, "bare_handler_arm_block.es") => "let invalid = {",
        (false, "handler_bare_effect_tag.es") => "Gate(req) =>",
        (false, "handler_empty_block.es") => "handler {\n    }",
        (false, "handler_produced_effect_outside_row.es") => "let BadGate: ![Gate => []]",
        (false, "handler_handled_row_mismatch.es") => "![Error<IOError> => [] for string]",
        (false, "handler_arm_arity_mismatch.es") => "Gate.request() =>",
        (false, "top_level_handle_expression.es") => "handle perform Gate.request",
        (false, "handle_with_non_handler.es") => "} with 2",
        (false, "finish_outside_handler.es") => "finish 1;",
        (false, "return_inside_handler_arm.es") => "return true;",
        (false, "finish_type_mismatch.es") => "finish 1;",
        (false, "implicit_handler_fallback.es") => "\"fallback\"",
        (false, "reject_with_handler_keyword.es") => "with handler {",
        _ => panic!("unregistered handler fixture `{file_name}`"),
    }
}

fn read_expected_handler_diagnostics(
    fixture: &Path,
    failures: &mut Vec<String>,
) -> Vec<ExpectedHandlerDiagnostic> {
    let path = diagnostics_path(fixture);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("failed to read {}: {error}", path.display()));
            return Vec::new();
        }
    };

    let mut expected = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((code, message)) = line.split_once('|') else {
            failures.push(format!(
                "{}:{} must use `diagnostic-code | stable message fragment`",
                path.display(),
                line_index + 1
            ));
            continue;
        };
        expected.push(ExpectedHandlerDiagnostic {
            code: code.trim().to_owned(),
            message: message.trim().to_owned(),
        });
    }

    if expected.is_empty() {
        failures.push(format!(
            "{} must contain at least one expected handler diagnostic",
            path.display()
        ));
    }

    expected
}

fn diagnostics_path(fixture: &Path) -> PathBuf {
    let mut path = fixture.to_path_buf();
    path.set_extension("diagnostics.txt");
    path
}

fn diagnostic_codes(output: &str) -> Vec<String> {
    let mut codes = Vec::new();
    for line in output.lines() {
        let Some(rest) = line.strip_prefix("error[") else {
            continue;
        };
        let Some((code, _)) = rest.split_once("]:") else {
            continue;
        };
        codes.push(code.to_owned());
    }
    codes
}

fn is_fallback_diagnostic(code: &str) -> bool {
    matches!(
        code,
        "type::IncompleteTypeFacts" | "effect::IncompleteEffectFacts"
    )
}

struct ExpectedHandlerDiagnostic {
    code: String,
    message: String,
}
