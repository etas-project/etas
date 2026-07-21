mod support;

use std::path::{Path, PathBuf};

use etas_core::DiagnosticCode;
use etas_syntax::parse_program;
use support::{fixture, fixtures_under, path_str, read_source, run_cli};

#[test]
fn positive_algorithm_fixtures_use_current_cli_entry_contract() {
    let fixtures = algorithm_sources("compiler/algorithms/positive");
    assert_eq!(
        fixtures.len(),
        30,
        "expected 30 positive algorithm fixtures"
    );

    for fixture in fixtures {
        let source = std::fs::read_to_string(&fixture).unwrap();
        assert_module_import_contract(&fixture, &source);
        assert!(
            source.contains("flow main(args: Array<string>) -> i32"),
            "{} does not use the canonical command-line entry shape",
            fixture.display()
        );
        assert!(
            !source.contains("unit -> unit") && !source.contains("return Run;"),
            "{} still uses the old returned-entry-flow shape",
            fixture.display()
        );
        assert!(
            !source.contains("\nagent "),
            "{} must not use agent declarations or agent calls",
            fixture.display()
        );
        assert_no_refinement_type_contract(&fixture, &source);
        let io = fixture.with_extension("io.txt");
        assert!(
            io.exists(),
            "{} is missing its command-line I/O contract sidecar",
            fixture.display()
        );
        let io_text = std::fs::read_to_string(&io).unwrap();
        let input_flow = if stdin_line_count(&io_text) == 1 {
            "read_line"
        } else {
            "read_all"
        };
        assert!(
            source.contains(&format!("import std.io.{{{input_flow}, println}};")),
            "{} should import command-line I/O with the current grouped import syntax",
            fixture.display()
        );
        let expected_input_call = if input_flow == "read_line" {
            "let input = read_line();"
        } else {
            "let input = read_all();"
        };
        assert!(
            source.contains(expected_input_call) && source.contains("println(solve(input));"),
            "{} should call imported stdin/stdout flows directly",
            fixture.display()
        );
        assert!(
            !source.contains("std.io.read_all(")
                && !source.contains("std.io.read_line(")
                && !source.contains("std.io.println("),
            "{} should not bypass the import contract with fully qualified std.io calls",
            fixture.display()
        );
        assert!(
            io_text.contains("command:\n"),
            "{} missing command section",
            io.display()
        );
        let expected_command = format!(
            "    etas {} ../../support/algorithms.es\n",
            fixture.file_name().unwrap().to_string_lossy()
        );
        assert!(
            io_text.contains(&expected_command),
            "{} should document the default interpreter command with its support module",
            io.display()
        );
        assert!(
            !io_text.contains("--allow-effects"),
            "{} should not document the host-adapter-gated --allow-effects path",
            io.display()
        );
        assert!(
            io_text.contains("stdin:\n"),
            "{} missing stdin section",
            io.display()
        );
        assert!(
            io_text.contains("stdout:\n"),
            "{} missing stdout section",
            io.display()
        );
        assert!(
            io_text.contains("exit:\n"),
            "{} missing exit section",
            io.display()
        );
    }
}

fn stdin_line_count(io_text: &str) -> usize {
    let mut in_section = false;
    let mut count = 0;
    for line in io_text.lines() {
        if line == "stdin:" {
            in_section = true;
            continue;
        }
        if line == "stdout:" {
            break;
        }
        if in_section && line.starts_with("    ") {
            count += 1;
        }
    }
    count
}

#[test]
fn positive_algorithm_fixtures_are_syntax_valid() {
    for fixture in algorithm_sources("compiler/algorithms/positive") {
        let parsed = parse_program(read_source(&fixture));
        assert!(
            parsed.diagnostics.is_empty(),
            "{} produced syntax diagnostics: {:#?}",
            fixture.display(),
            parsed.diagnostics
        );
    }
}

#[test]
fn compiler_algorithm_support_module_exports_imported_helpers() {
    let support = std::fs::read_to_string(fixture("compiler/support/algorithms.es")).unwrap();
    let exports = public_flow_names(&support);
    assert!(
        !exports.is_empty(),
        "compiler support module should expose public helper flows"
    );

    for fixture in algorithm_sources("compiler/algorithms/positive")
        .into_iter()
        .chain(algorithm_sources("compiler/algorithms/negative"))
    {
        let source = std::fs::read_to_string(&fixture).unwrap();
        for imported in support_algorithm_imports(&source) {
            assert!(
                exports.iter().any(|export| *export == imported),
                "{} imports tests.compiler.support.algorithms.{imported}, but the support module does not export it",
                fixture.display()
            );
        }
    }
}

#[test]
fn compiler_algorithm_support_module_is_not_placeholder_stubbed() {
    let support = std::fs::read_to_string(fixture("compiler/support/algorithms.es")).unwrap();
    let placeholder_returns = [
        "return false;",
        "return \"\";",
        "return 0;",
        "return [];",
        "return values;",
        "return [input];",
        "return input;",
    ];

    for placeholder in placeholder_returns {
        assert!(
            !support.contains(placeholder),
            "compiler support module still contains placeholder implementation: {placeholder}"
        );
    }
}

#[test]
fn negative_algorithm_fixtures_are_targeted_compiler_regressions() {
    let fixtures = algorithm_sources("compiler/algorithms/negative");
    assert_eq!(
        fixtures.len(),
        10,
        "expected 10 negative algorithm fixtures"
    );
    let mut compiler_failures = Vec::new();

    for fixture in fixtures {
        let source = std::fs::read_to_string(&fixture).unwrap();
        assert_module_import_contract(&fixture, &source);
        assert!(
            source.contains("flow main(args: Array<string>) -> i32"),
            "{} does not use the canonical command-line entry shape",
            fixture.display()
        );
        assert!(
            !source.contains("unit -> unit") && !source.contains("return Run;"),
            "{} still uses the old returned-entry-flow shape",
            fixture.display()
        );
        assert!(
            !source.contains("\nagent "),
            "{} must not use agent declarations or agent calls",
            fixture.display()
        );
        assert_no_refinement_type_contract(&fixture, &source);
        assert!(
            imports_std_collections_list(&source),
            "{} should import the standard List type explicitly",
            fixture.display()
        );

        let expected = fixture.with_extension("diagnostics.txt");
        assert!(
            expected.exists(),
            "{} is missing expected diagnostic metadata",
            fixture.display()
        );
        let expected_text = std::fs::read_to_string(&expected).unwrap();
        assert!(
            !expected_text.trim().is_empty(),
            "{} has empty expected diagnostic metadata",
            expected.display()
        );
        assert!(
            !expected_text.contains("Refinement"),
            "{} should not expect refinement-type diagnostics after the SPEC removed refinement types",
            expected.display()
        );

        let parsed = parse_program(read_source(&fixture));
        assert!(
            !parsed
                .diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::Syntax(_))),
            "{} should be a semantic/compiler negative fixture, not a syntax negative fixture: {:#?}",
            fixture.display(),
            parsed.diagnostics
        );

        let (code, stdout, stderr) = run_cli(["etas", "check", path_str(&fixture)]);
        let mut failures = Vec::new();
        if code != 1 {
            failures.push(format!(
                "expected exit code 1, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
            ));
        }
        if !stdout.contains("checked 1 file") {
            failures.push(format!("expected stdout check summary, got:\n{stdout}"));
        }
        for expected_diagnostic in expected_diagnostic_lines(&expected_text) {
            if !stderr.contains(expected_diagnostic) {
                failures.push(format!(
                    "missing expected diagnostic `{expected_diagnostic}`\nstderr:\n{stderr}"
                ));
            }
        }
        if !failures.is_empty() {
            compiler_failures.push(format!("{}\n{}", fixture.display(), failures.join("\n")));
        }
    }

    assert!(
        compiler_failures.is_empty(),
        "negative algorithm fixtures did not match their expected diagnostics:\n\n{}",
        compiler_failures.join("\n\n")
    );
}

fn algorithm_sources(relative: &str) -> Vec<PathBuf> {
    fixtures_under(relative)
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "es"))
        .collect()
}

fn expected_diagnostic_lines(expected_text: &str) -> Vec<&str> {
    let lines = expected_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert!(
        lines.len() >= 2,
        "expected diagnostic metadata should include a diagnostic code and message"
    );
    lines
}

fn assert_module_import_contract(fixture: &Path, source: &str) {
    let lines = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert!(
        lines
            .first()
            .is_some_and(|line| line.starts_with("module ") && line.ends_with(';')),
        "{} should start with a module declaration",
        fixture.display()
    );
    assert!(
        lines.get(1).is_some_and(|line| is_import_decl(line)),
        "{} should place import declarations immediately after the module declaration",
        fixture.display()
    );

    let mut saw_import = false;
    let mut saw_item = false;
    for line in lines.iter().skip(1) {
        if is_import_decl(line) {
            assert!(
                !saw_item,
                "{} has an import after an item declaration: {line}",
                fixture.display()
            );
            saw_import = true;
            assert!(
                line.ends_with(';'),
                "{} import declaration should end with ';': {line}",
                fixture.display()
            );
        } else if is_item_decl_start(line) {
            saw_item = true;
        }
    }

    assert!(
        saw_import,
        "{} should contain at least one import declaration",
        fixture.display()
    );
}

fn assert_no_refinement_type_contract(fixture: &Path, source: &str) {
    assert!(
        !source
            .lines()
            .map(str::trim)
            .any(|line| line.starts_with("type ") && line.contains(" where ")),
        "{} should not use refinement type syntax after the SPEC removed it",
        fixture.display()
    );
}

fn imports_std_collections_list(source: &str) -> bool {
    source.lines().map(str::trim).any(|line| {
        line == "import std.collections.List;"
            || line
                .strip_prefix("import std.collections.{")
                .and_then(|line| line.strip_suffix("};"))
                .is_some_and(|members| members.split(',').any(|member| member.trim() == "List"))
    })
}

fn is_import_decl(line: &str) -> bool {
    line.starts_with("import ") || line.starts_with("public import ")
}

fn is_item_decl_start(line: &str) -> bool {
    let line = line
        .strip_prefix("public ")
        .or_else(|| line.strip_prefix("private "))
        .unwrap_or(line);

    [
        "type ",
        "enum ",
        "impl ",
        "effect ",
        "memory ",
        "tool ",
        "agent ",
        "policy ",
        "protocol ",
        "flow ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn support_algorithm_imports(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("import tests.compiler.support.algorithms.{")
                .and_then(|tail| tail.strip_suffix("};"))
        })
        .flat_map(|names| names.split(','))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

fn public_flow_names(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("public flow ")
                .and_then(|tail| tail.split(['(', '[']).next())
        })
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

#[allow(dead_code)]
fn display_name(path: &Path) -> String {
    path.strip_prefix(fixture(""))
        .unwrap_or(path)
        .display()
        .to_string()
}
