mod support;

use std::path::{Path, PathBuf};

use support::{fixture, fixtures_under, path_str, run_cli};

const EFFECT_POSITIVE: &str = "compiler/effects/positive";
const EFFECT_NEGATIVE: &str = "compiler/effects/negative";
const POLICY_POSITIVE: &str = "compiler/policies/positive";
const POLICY_NEGATIVE: &str = "compiler/policies/negative";

#[test]
fn effect_fixtures_are_real_compiler_regressions() {
    assert_source_file_count(EFFECT_POSITIVE, 115);
    assert_source_file_count(EFFECT_NEGATIVE, 115);
    assert_regression_case_count(EFFECT_POSITIVE, 107);
    assert_regression_case_count(EFFECT_NEGATIVE, 107);
    assert_no_obsolete_effect_or_policy_syntax(EFFECT_POSITIVE);
    assert_no_obsolete_effect_or_policy_syntax(EFFECT_NEGATIVE);

    let mut failures = Vec::new();

    for fixture in effect_regression_entries(EFFECT_POSITIVE) {
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        let (effects_code, effects_stdout, effects_stderr) =
            run_cli(["--format", "json", "effects", path_str(&fixture)]);
        if code != 0 {
            failures.push(format!(
                "{} should compile under the current Effect/Handler SPEC\nstdout:\n{}\nstderr:\n{}",
                fixture.display(),
                stdout,
                stderr
            ));
        }
        if effects_code != 0 {
            failures.push(format!(
                "{} should expose an effect summary through public `etas effects --format json`\nstdout:\n{}\nstderr:\n{}",
                fixture.display(),
                effects_stdout,
                effects_stderr
            ));
        } else if let Err(error) = assert_positive_effect_summary(&fixture, &effects_stdout) {
            failures.push(error);
        }
    }

    for fixture in effect_regression_entries(EFFECT_NEGATIVE) {
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        let expected = match read_expected_effect_diagnostics(&fixture) {
            Ok(expected) => expected,
            Err(error) => {
                failures.push(error);
                continue;
            }
        };
        let output = format!("{stdout}\n{stderr}");
        let actual = parse_human_diagnostics(&output);

        if code == 0 {
            failures.push(format!(
                "{} should be rejected by Effect/Handler checking",
                fixture.display()
            ));
        }

        for needle in expected {
            if !actual.iter().any(|diagnostic| diagnostic.matches(&needle)) {
                failures.push(format!(
                    "{} missing expected diagnostic `{}`\nstdout:\n{}\nstderr:\n{}",
                    fixture.display(),
                    needle.original,
                    stdout,
                    stderr
                ));
            }
        }
        assert_negative_failed_for_expected_effect_reason(&fixture, &actual, &mut failures);
    }

    assert!(
        failures.is_empty(),
        "effect fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn effect_project_import_fixtures_run_in_directory_mode() {
    let mut failures = Vec::new();

    let positive_projects = project_import_dirs(EFFECT_POSITIVE, "project_import_");
    assert_eq!(
        positive_projects.len(),
        8,
        "positive effects should contain exactly 8 project import fixture directories"
    );
    for project in positive_projects {
        let (code, stdout, stderr) = run_cli(["check", path_str(&project)]);
        if code != 0 || stderr.contains("error[") {
            failures.push(format!(
                "{} should compile as a project directory\nstdout:\n{}\nstderr:\n{}",
                project.display(),
                stdout,
                stderr
            ));
        }

        let (effects_code, effects_stdout, effects_stderr) =
            run_cli(["--format", "json", "effects", path_str(&project)]);
        if effects_code != 0 || effects_stderr.contains("error[") {
            failures.push(format!(
                "{} should expose a project-level effect summary\nstdout:\n{}\nstderr:\n{}",
                project.display(),
                effects_stdout,
                effects_stderr
            ));
        }
    }

    let negative_projects = project_import_dirs(EFFECT_NEGATIVE, "project_import_errors_");
    assert_eq!(
        negative_projects.len(),
        8,
        "negative effects should contain exactly 8 project import fixture directories"
    );
    for project in negative_projects {
        let main = project.join("main.es");
        let (code, stdout, stderr) = run_cli(["check", path_str(&project)]);
        let output = format!("{stdout}\n{stderr}");
        let actual = parse_human_diagnostics(&output);

        if code == 0 {
            failures.push(format!(
                "{} should be rejected by its project-level effect contract",
                project.display()
            ));
        }
        for forbidden in [
            "syntax::InvalidItem",
            "name::DuplicateSymbol",
            "duplicate symbol `main`",
            "module declaration does not match source path",
            "missing imported module",
        ] {
            if output.contains(forbidden) {
                failures.push(format!(
                    "{} failed for project/module plumbing `{forbidden}` instead of the pinned effect rule\nstdout:\n{}\nstderr:\n{}",
                    project.display(),
                    stdout,
                    stderr
                ));
            }
        }

        let expected = match read_expected_effect_diagnostics(&main) {
            Ok(expected) => expected,
            Err(error) => {
                failures.push(error);
                continue;
            }
        };
        for needle in expected {
            if !actual.iter().any(|diagnostic| diagnostic.matches(&needle)) {
                failures.push(format!(
                    "{} missing expected project diagnostic `{}`\nstdout:\n{}\nstderr:\n{}",
                    project.display(),
                    needle.original,
                    stdout,
                    stderr
                ));
            }
        }
        assert_negative_failed_for_expected_effect_reason(&main, &actual, &mut failures);
    }

    assert!(
        failures.is_empty(),
        "project import effect fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn policy_fixtures_are_real_compiler_regressions() {
    assert_source_file_count(POLICY_POSITIVE, 15);
    assert_source_file_count(POLICY_NEGATIVE, 15);
    assert_no_obsolete_effect_or_policy_syntax(POLICY_POSITIVE);
    assert_no_obsolete_effect_or_policy_syntax(POLICY_NEGATIVE);

    let mut failures = Vec::new();

    for fixture in fixtures_under(POLICY_POSITIVE) {
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        if code != 0 {
            failures.push(format!(
                "{} should satisfy its policy under the current SPEC\nstdout:\n{}\nstderr:\n{}",
                fixture.display(),
                stdout,
                stderr
            ));
        }
    }

    for fixture in fixtures_under(POLICY_NEGATIVE) {
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        let expected = read_expected_fragments(&fixture);
        let output = format!("{stdout}\n{stderr}");

        if code == 0 {
            failures.push(format!(
                "{} should be rejected by policy checking",
                fixture.display()
            ));
        }

        for needle in expected {
            if !output.contains(&needle) {
                failures.push(format!(
                    "{} missing expected diagnostic fragment `{}`\nstdout:\n{}\nstderr:\n{}",
                    fixture.display(),
                    needle,
                    stdout,
                    stderr
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "policy fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

fn assert_source_file_count(relative: &str, expected: usize) {
    let actual = fixtures_under(relative).len();
    assert_eq!(
        actual, expected,
        "{relative} should contain exactly {expected} .es source files"
    );
}

fn assert_regression_case_count(relative: &str, expected: usize) {
    let actual = effect_regression_entries(relative).len();
    assert_eq!(
        actual, expected,
        "{relative} should contain exactly {expected} end-to-end regression cases"
    );
}

fn effect_regression_entries(relative: &str) -> Vec<PathBuf> {
    fixtures_under(relative)
        .into_iter()
        .filter(|fixture| {
            let file_name = fixture.file_name().and_then(|name| name.to_str());
            let parent = fixture
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .unwrap_or_default();

            !(file_name == Some("support.es")
                && (parent.starts_with("project_import_")
                    || parent.starts_with("project_import_errors_")))
        })
        .collect()
}

fn project_import_dirs(relative: &str, prefix: &str) -> Vec<PathBuf> {
    let mut dirs = std::fs::read_dir(fixture(relative))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn assert_no_obsolete_effect_or_policy_syntax(relative: &str) {
    let banned = [
        "Capability(",
        "Sandbox(",
        "Web.*",
        " effect [",
        "[effect =",
        "allow [",
        "deny [",
        "require [",
        "prompt ",
    ];

    for fixture in fixtures_under(relative) {
        let source = std::fs::read_to_string(&fixture).unwrap();
        for pattern in banned {
            assert!(
                !source.contains(pattern),
                "{} should use the current SPEC and avoid obsolete `{pattern}` syntax",
                fixture.display()
            );
        }
    }
}

#[test]
fn effect_and_policy_fixtures_satisfy_complexity_contract() {
    for relative in [
        EFFECT_POSITIVE,
        EFFECT_NEGATIVE,
        POLICY_POSITIVE,
        POLICY_NEGATIVE,
    ] {
        for fixture in fixtures_under(relative) {
            let source = std::fs::read_to_string(&fixture).unwrap();
            let flows = extract_flow_bodies(&source);

            assert!(
                source.lines().count() >= 100,
                "{} should have at least 100 lines",
                fixture.display()
            );
            assert!(
                source.contains("=>"),
                "{} should contain at least one lambda expression",
                fixture.display()
            );
            assert!(
                flows.len() >= 4,
                "{} should contain at least four flows so at least three can call each other",
                fixture.display()
            );
            assert_unique_flow_names(&fixture, &flows);
            assert_minimum_flow_body_size(&fixture, &flows);
            assert_flow_call_graph_has_three_links(&fixture, &flows);
            assert_flow_similarity_below_half(&fixture, &flows);
        }
    }
}

#[test]
fn effect_fixtures_match_category_and_do_not_fake_std_registry() {
    let mut failures = Vec::new();

    for relative in [EFFECT_POSITIVE, EFFECT_NEGATIVE] {
        for fixture in fixtures_under(relative) {
            let source = std::fs::read_to_string(&fixture).unwrap();
            let file_name = fixture
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let category = effect_fixture_category(&fixture);

            if source.contains("\neffect Web extends Network")
                || source.contains("\neffect Error<E>")
                || source.contains("type ProjectMemorySchema = MemoryRegion")
            {
                failures.push(format!(
                    "{} copies a local fake effect registry instead of importing std/compiler prelude",
                    fixture.display()
                ));
            }

            if category.starts_with("project_import_") {
                if source.contains("tests.compiler.effects") {
                    failures.push(format!(
                        "{} encodes the test fixture directory in its module namespace",
                        fixture.display()
                    ));
                }
                if file_name == "main.es" && !source.contains("import ") {
                    failures.push(format!(
                        "{} is named project_import but contains no import",
                        fixture.display()
                    ));
                }
                if file_name == "main.es" {
                    if !source
                        .lines()
                        .next()
                        .is_some_and(|line| line.trim() == "module main;")
                    {
                        failures.push(format!(
                            "{} should declare the project-local `module main;`",
                            fixture.display()
                        ));
                    }
                    if !source.contains("import support.{") {
                        failures.push(format!(
                            "{} should import from project-local `support` module",
                            fixture.display()
                        ));
                    }
                }
                if file_name == "support.es" && !source.contains("public flow ") {
                    failures.push(format!(
                        "{} is project_import support but exports no public flow",
                        fixture.display()
                    ));
                }
                if file_name == "support.es"
                    && !source
                        .lines()
                        .next()
                        .is_some_and(|line| line.trim() == "module support;")
                {
                    failures.push(format!(
                        "{} should declare the project-local `module support;`",
                        fixture.display()
                    ));
                }
            }

            for (prefix, required) in effect_category_markers() {
                if category.starts_with(prefix)
                    && !required.iter().any(|marker| source.contains(*marker))
                {
                    failures.push(format!(
                        "{} is named `{prefix}` but does not contain any required semantic marker {:?}",
                        fixture.display(),
                        required
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "effect fixture category/registry regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn effect_fixtures_do_not_share_large_generated_bodies() {
    let fixtures = [EFFECT_POSITIVE, EFFECT_NEGATIVE]
        .into_iter()
        .flat_map(fixtures_under)
        .collect::<Vec<_>>();
    let mut fingerprints = std::collections::BTreeMap::<String, Vec<PathBuf>>::new();

    for fixture in fixtures {
        let source = std::fs::read_to_string(&fixture).unwrap();
        let body = source
            .lines()
            .skip(3)
            .take(126)
            .map(normalize_fixture_line)
            .collect::<Vec<_>>()
            .join("\n");
        fingerprints.entry(body).or_default().push(fixture);
    }

    let duplicates = fingerprints
        .into_values()
        .filter(|paths| paths.len() >= 3)
        .map(|paths| {
            paths
                .into_iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>();

    assert!(
        duplicates.is_empty(),
        "effect fixtures share large generated bodies and do not provide independent semantic coverage:\n{}",
        duplicates.join("\n\n")
    );
}

fn effect_category_markers() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "checked_index_capture_",
            vec!["[", ".at(", "Error<IndexError>"],
        ),
        (
            "command_sandbox_",
            vec!["Command.run", "CommandSandbox", "DefaultCommandSandbox"],
        ),
        (
            "stdio_console_",
            vec!["std.io", "Console", "println", "read_line", "read_all"],
        ),
        ("approval_", vec!["Approval.request", "Approval"]),
        (
            "memory_effects_",
            vec!["Memory.read", "Memory.write", "Store.get", "Store.put"],
        ),
        ("model_inference_", vec!["Agentic", "agent ", "model "]),
        ("handler_elimination_", vec!["handle ", " with"]),
        ("higher_order_latent_", vec!["=>", "flow value", "pipeline"]),
        (
            "interprocedural_fixpoint_",
            vec!["recursive", "mutual", "while "],
        ),
    ]
}

fn effect_fixture_category(fixture: &Path) -> String {
    let file_name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name == "main.es" || file_name == "support.es" {
        return fixture
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or(file_name)
            .to_owned();
    }
    file_name.to_owned()
}

fn normalize_fixture_line(line: &str) -> String {
    line.trim()
        .chars()
        .map(|ch| if ch.is_ascii_digit() { '0' } else { ch })
        .collect()
}

fn read_expected_effect_diagnostics(fixture: &Path) -> Result<Vec<ExpectedDiagnostic>, String> {
    let path = diagnostics_path(fixture);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    let mut errors = Vec::new();
    let mut expected = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match ExpectedDiagnostic::parse(line) {
            Ok(diagnostic) => expected.push(diagnostic),
            Err(error) => errors.push(format!("{}:{}: {error}", path.display(), index + 1)),
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    if expected.is_empty() {
        return Err(format!(
            "{} must contain at least one expected diagnostic",
            path.display()
        ));
    }
    if expected.iter().any(|line| {
        line.message.contains("UnsupportedHandler") || line.label.contains("UnsupportedHandler")
    }) {
        return Err(format!(
            "{} must not accept UnsupportedHandler fallback diagnostics as a contract",
            path.display()
        ));
    }
    if !expected
        .iter()
        .any(|line| line.code.starts_with("effect::"))
    {
        return Err(format!(
            "{} must contain at least one stable effect diagnostic code",
            path.display()
        ));
    }

    Ok(expected)
}

fn read_expected_fragments(fixture: &Path) -> Vec<String> {
    let path = diagnostics_path(fixture);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    let expected = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    assert!(
        !expected.is_empty(),
        "{} must contain at least one expected diagnostic fragment",
        path.display()
    );

    expected
}

#[derive(Debug)]
struct ExpectedDiagnostic {
    original: String,
    code: String,
    message: String,
    label: String,
    line: usize,
    column: usize,
}

impl ExpectedDiagnostic {
    fn parse(line: &str) -> Result<Self, String> {
        let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 5 {
            return Err(format!(
                "effect diagnostics must use `code | message | label | line | column`, got `{line}`"
            ));
        }
        if !parts[0].starts_with("effect::") {
            return Err(format!(
                "effect diagnostics must pin an effect diagnostic code, got `{}`",
                parts[0]
            ));
        }
        if parts[1].is_empty() || parts[2].is_empty() {
            return Err(format!(
                "effect diagnostics must pin core message and primary label, got `{line}`"
            ));
        }

        Ok(Self {
            original: line.to_owned(),
            code: parts[0].to_owned(),
            message: parts[1].to_owned(),
            label: parts[2].to_owned(),
            line: parts[3]
                .parse()
                .map_err(|_| format!("invalid diagnostic line in `{line}`"))?,
            column: parts[4]
                .parse()
                .map_err(|_| format!("invalid diagnostic column in `{line}`"))?,
        })
    }
}

#[derive(Debug)]
struct ActualDiagnostic {
    code: String,
    message: String,
    label: String,
    line: usize,
    column: usize,
}

impl ActualDiagnostic {
    fn matches(&self, expected: &ExpectedDiagnostic) -> bool {
        self.code == expected.code
            && diagnostic_text_matches(&self.message, &expected.message)
            && diagnostic_text_matches(&self.label, &expected.label)
            && self.line == expected.line
            && self.column == expected.column
    }
}

fn diagnostic_text_matches(actual: &str, expected: &str) -> bool {
    if actual.contains(expected) {
        return true;
    }
    if expected == "inferred effect escapes the declared effect row" {
        return actual.starts_with("inferred effect ")
            && actual.ends_with(" escapes the declared effect row");
    }
    if let Some(expected_actions) = expected
        .strip_prefix("requested action ")
        .and_then(|text| text.strip_suffix(" escapes the declared effect row"))
    {
        let Some(actual_actions) = actual
            .strip_prefix("requested action ")
            .and_then(|text| text.strip_suffix(" escapes the declared effect row"))
        else {
            return false;
        };
        let expected = expected_actions
            .split(',')
            .map(str::trim)
            .collect::<std::collections::BTreeSet<_>>();
        return actual_actions
            .split(',')
            .map(str::trim)
            .all(|action| expected.contains(action));
    }
    false
}

fn parse_human_diagnostics(output: &str) -> Vec<ActualDiagnostic> {
    let lines = output.lines().collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let Some(rest) = line.strip_prefix("error[") else {
            index += 1;
            continue;
        };
        let Some((code, message)) = rest.split_once("]: ") else {
            index += 1;
            continue;
        };

        let mut diagnostic = ActualDiagnostic {
            code: code.to_owned(),
            message: message.to_owned(),
            label: message.to_owned(),
            line: 0,
            column: 0,
        };

        let mut cursor = index + 1;
        while cursor < lines.len() && !lines[cursor].starts_with("error[") {
            if let Some((_, location)) = lines[cursor].split_once("-->") {
                let segments = location.trim().rsplit(':').take(2).collect::<Vec<_>>();
                if segments.len() == 2 {
                    diagnostic.column = segments[0].parse().unwrap_or(0);
                    diagnostic.line = segments[1].parse().unwrap_or(0);
                }
            }
            if lines[cursor].contains(" | ") && lines[cursor].contains('^') {
                if let Some((_, label)) = lines[cursor].split_once('^') {
                    let label = label.trim();
                    if !label.is_empty() {
                        diagnostic.label = label.to_owned();
                    }
                }
            }
            cursor += 1;
        }

        diagnostics.push(diagnostic);
        index = cursor;
    }

    diagnostics
}

fn assert_negative_failed_for_expected_effect_reason(
    fixture: &Path,
    actual: &[ActualDiagnostic],
    failures: &mut Vec<String>,
) {
    let expected_path = diagnostics_path(fixture);
    let expected_text = std::fs::read_to_string(&expected_path).unwrap();
    let allows_incomplete_facts = expected_text.contains("effect::IncompleteEffectFacts");

    for diagnostic in actual {
        if diagnostic.code == "effect::IncompleteEffectFacts" && !allows_incomplete_facts {
            failures.push(format!(
                "{} failed with effect::IncompleteEffectFacts, but this fixture is not an incomplete-facts regression",
                fixture.display()
            ));
        }
        if diagnostic.message.contains("UnsupportedHandler")
            || diagnostic.label.contains("UnsupportedHandler")
        {
            failures.push(format!(
                "{} failed with UnsupportedHandler fallback diagnostic instead of the pinned handler rule",
                fixture.display()
            ));
        }
    }

    let expected_codes = match read_expected_effect_diagnostics(fixture) {
        Ok(expected) => expected
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<std::collections::BTreeSet<_>>(),
        Err(error) => {
            failures.push(error);
            return;
        }
    };
    for diagnostic in actual
        .iter()
        .filter(|diagnostic| diagnostic.code.starts_with("effect::"))
    {
        if !expected_codes.contains(&diagnostic.code) {
            failures.push(format!(
                "{} produced unexpected effect diagnostic `{}`; negative fixtures must fail for the pinned rule only",
                fixture.display(),
                diagnostic.code
            ));
        }
    }
}

fn diagnostics_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("diagnostics.txt")
}

fn assert_positive_effect_summary(fixture: &Path, stdout: &str) -> Result<(), String> {
    let json: serde_json::Value = serde_json::from_str(stdout).map_err(|error| {
        format!(
            "{} emitted invalid JSON from `etas effects --format json`: {error}\n{}",
            fixture.display(),
            stdout
        )
    })?;
    let items = json
        .get("items")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{} effect summary is missing `items`", fixture.display()))?;
    let main = items
        .iter()
        .find(|item| item.get("name").and_then(serde_json::Value::as_str) == Some("main"))
        .ok_or_else(|| format!("{} effect summary has no `main` flow", fixture.display()))?;

    for key in [
        "requested_actions",
        "default_actions",
        "escaping_effects",
        "residual_checks",
        "summary",
    ] {
        if main.get(key).is_none() {
            return Err(format!(
                "{} main effect summary is missing `{key}`",
                fixture.display()
            ));
        }
    }

    assert_category_matches_positive_summary(fixture, main)
}

fn assert_category_matches_positive_summary(
    fixture: &Path,
    main: &serde_json::Value,
) -> Result<(), String> {
    let file_name = effect_fixture_category(fixture);
    let source = std::fs::read_to_string(fixture).unwrap();
    let requested = main
        .get("requested_actions")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    let required_needles: &[&str] = if file_name.starts_with("checked_index_capture_") {
        &["[", "Error<IndexError>"]
    } else if file_name.starts_with("command_sandbox_") {
        &["Command.run"]
    } else if file_name.starts_with("project_import_") {
        &["import "]
    } else if file_name.starts_with("stdio_console_") {
        &["std.io", "Console"]
    } else if file_name.starts_with("approval_") {
        &["Approval"]
    } else if file_name.starts_with("memory_effects_") {
        &["Memory."]
    } else if file_name.starts_with("model_inference_") {
        &["Agentic"]
    } else {
        &[]
    };

    for needle in required_needles {
        if !source.contains(needle) && !requested.contains(needle) {
            return Err(format!(
                "{} is named as `{}` coverage but does not contain required semantic marker `{needle}`",
                fixture.display(),
                file_name
            ));
        }
    }

    Ok(())
}

fn extract_flow_bodies(source: &str) -> Vec<(String, Vec<String>)> {
    let lines = source.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut flows = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim_start();
        let Some(_flow_rest) = line
            .strip_prefix("flow ")
            .or_else(|| line.strip_prefix("public flow "))
        else {
            index += 1;
            continue;
        };

        let name = line
            .strip_prefix("flow ")
            .or_else(|| line.strip_prefix("public flow "))
            .and_then(|rest| rest.split_once('('))
            .map(|(name, _)| name.trim().to_owned())
            .unwrap_or_else(|| format!("<unknown-{index}>"));
        let start = index;
        let mut depth = brace_delta(&lines[index]);
        let mut saw_opening_brace = lines[index].contains('{');
        index += 1;

        while index < lines.len() {
            if lines[index].contains('{') {
                saw_opening_brace = true;
            }
            depth += brace_delta(&lines[index]);
            index += 1;
            if saw_opening_brace && depth == 0 {
                break;
            }
        }

        flows.push((name, lines[start..index].to_vec()));
    }

    flows
}

fn brace_delta(line: &str) -> i32 {
    line.chars().fold(0, |depth, ch| match ch {
        '{' => depth + 1,
        '}' => depth - 1,
        _ => depth,
    })
}

fn assert_unique_flow_names(fixture: &Path, flows: &[(String, Vec<String>)]) {
    let mut names = std::collections::BTreeSet::new();
    for (name, _) in flows {
        assert!(
            names.insert(name),
            "{} should not repeat flow `{name}`",
            fixture.display()
        );
    }
}

fn assert_minimum_flow_body_size(fixture: &Path, flows: &[(String, Vec<String>)]) {
    for (name, body) in flows {
        assert!(
            body.len() >= 10,
            "{} flow `{name}` should have at least 10 lines, got {}",
            fixture.display(),
            body.len()
        );
    }
}

fn assert_flow_call_graph_has_three_links(fixture: &Path, flows: &[(String, Vec<String>)]) {
    let source = flows
        .iter()
        .flat_map(|(_, body)| body)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let mut edges = std::collections::BTreeSet::new();

    for (caller, body) in flows {
        let body = body.join("\n");
        for (callee, _) in flows {
            if caller == callee {
                continue;
            }

            let needle = format!("{callee}(");
            if body.contains(&needle) {
                edges.insert((caller.clone(), callee.clone()));
            }
        }
    }

    assert!(
        edges.len() >= 3,
        "{} should have at least three inter-flow call links, got {:?}\n{}",
        fixture.display(),
        edges,
        source
    );
}

fn assert_flow_similarity_below_half(fixture: &Path, flows: &[(String, Vec<String>)]) {
    for (index, (left_name, left_body)) in flows.iter().enumerate() {
        if left_name == "main" {
            continue;
        }

        for (right_name, right_body) in flows.iter().skip(index + 1) {
            if right_name == "main" {
                continue;
            }

            let left = normalized_body_lines(left_body);
            let right = normalized_body_lines(right_body);
            let intersection = left.intersection(&right).count();
            let union = left.union(&right).count();
            let similarity = intersection as f32 / union.max(1) as f32;

            assert!(
                similarity <= 0.5,
                "{} flows `{left_name}` and `{right_name}` are too similar: {:.1}%",
                fixture.display(),
                similarity * 100.0
            );
        }
    }
}

fn normalized_body_lines(lines: &[String]) -> std::collections::BTreeSet<String> {
    lines
        .iter()
        .map(|line| {
            let mut normalized = String::with_capacity(line.len());
            let mut previous_was_digit = false;

            for ch in line.trim().chars() {
                if ch.is_ascii_digit() {
                    if !previous_was_digit {
                        normalized.push('N');
                    }
                    previous_was_digit = true;
                } else {
                    normalized.push(ch);
                    previous_was_digit = false;
                }
            }

            normalized
        })
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn effect_and_policy_fixture_corpora_have_expected_total_size() {
    let effect_positive = fixtures_under(EFFECT_POSITIVE).len();
    let effect_negative = fixtures_under(EFFECT_NEGATIVE).len();
    let policy_positive = fixtures_under(POLICY_POSITIVE).len();
    let policy_negative = fixtures_under(POLICY_NEGATIVE).len();

    assert_eq!(
        effect_positive, 115,
        "Effect/Handler positive corpus must stay at 115 fixtures"
    );
    assert_eq!(
        effect_negative, 115,
        "Effect/Handler negative corpus must stay at 115 fixtures"
    );
    assert_eq!(
        policy_positive, 15,
        "Policy positive corpus must stay at 15 fixtures"
    );
    assert_eq!(
        policy_negative, 15,
        "Policy negative corpus must stay at 15 fixtures"
    );
    assert!(fixture(EFFECT_POSITIVE).is_dir());
    assert!(fixture(POLICY_POSITIVE).is_dir());
}
