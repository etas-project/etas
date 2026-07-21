mod support;

use std::path::{Path, PathBuf};

use etas_syntax::parse_program;
use support::{fixture, fixtures_under, read_source};

#[test]
fn positive_functional_fixtures_use_current_cli_entry_contract() {
    let fixtures = functional_sources("compiler/functional/positive");
    assert_eq!(
        fixtures.len(),
        10,
        "expected 10 positive functional fixtures"
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
        assert_no_prompt_keyword_contract(&fixture, &source);
        assert!(
            source.contains("import std.io.{read_line, println};"),
            "{} should import command-line I/O with the current grouped import syntax",
            fixture.display()
        );
        assert!(
            source.contains("let input = read_line();")
                && source.contains("println(solve(input));"),
            "{} should call imported stdin/stdout flows directly",
            fixture.display()
        );
        assert!(
            !source.contains("std.io.read_line(") && !source.contains("std.io.println("),
            "{} should not bypass the import contract with fully qualified std.io calls",
            fixture.display()
        );
        assert!(
            source.contains("=>") || source.contains(" | "),
            "{} should exercise functional flow values, lambdas, or composition",
            fixture.display()
        );

        let io = fixture.with_extension("io.txt");
        assert!(
            io.exists(),
            "{} is missing its command-line I/O contract sidecar",
            fixture.display()
        );
        let io_text = std::fs::read_to_string(&io).unwrap();
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

#[test]
fn positive_functional_fixtures_cover_core_functional_surface_area() {
    let corpus = functional_sources("compiler/functional/positive")
        .into_iter()
        .map(|fixture| std::fs::read_to_string(fixture).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let required_forms = [
        "=>",
        "i32 -> i32",
        "(i32, i32) -> i32",
        "unit -> i32",
        "return value =>",
        "f(f(value))",
        "map_i32(",
        "filter_i32(",
        "fold_i32(",
        "let pipeline = increment | double;",
        "() => base * base",
    ];

    for required in required_forms {
        assert!(
            corpus.contains(required),
            "functional fixture corpus should cover `{required}`"
        );
    }
}

#[test]
fn positive_functional_fixtures_are_syntax_valid() {
    for fixture in functional_sources("compiler/functional/positive") {
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
fn functional_support_module_exports_imported_helpers() {
    let support = std::fs::read_to_string(fixture("compiler/support/algorithms.es")).unwrap();
    let exports = public_flow_names(&support);
    assert!(
        !exports.is_empty(),
        "compiler support module should expose public helper flows"
    );

    for fixture in functional_sources("compiler/functional/positive") {
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
fn functional_support_module_is_not_placeholder_stubbed() {
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

fn functional_sources(relative: &str) -> Vec<PathBuf> {
    fixtures_under(relative)
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "es"))
        .collect()
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

fn assert_no_prompt_keyword_contract(fixture: &Path, source: &str) {
    assert!(
        !uses_source_prompt_keyword(source),
        "{} should not use the removed prompt keyword",
        fixture.display()
    );
}

fn uses_source_prompt_keyword(source: &str) -> bool {
    let bytes = source.as_bytes();
    let needle = b"prompt";
    let mut index = 0;

    while let Some(offset) = source[index..].find("prompt") {
        let start = index + offset;
        let end = start + needle.len();
        let before = start.checked_sub(1).map(|i| bytes[i]);
        let after = bytes.get(end).copied();

        let starts_identifier =
            before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
        let ends_identifier =
            after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
        let followed_by_block = source[end..].trim_start().starts_with('{');
        if !starts_identifier && !ends_identifier && followed_by_block {
            return true;
        }

        index = end;
    }

    false
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
