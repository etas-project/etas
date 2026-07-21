mod support;

use support::{fixture, fixtures_under};

#[test]
fn compiler_fixtures_use_feature_before_polarity_layout() {
    for obsolete in ["compiler/positive", "compiler/negative"] {
        let path = fixture(obsolete);
        assert!(
            !path.exists(),
            "{} should not exist; use fixtures/compiler/<feature>/<polarity>/...",
            path.display()
        );
    }

    for expected in [
        "compiler/algorithms/positive",
        "compiler/algorithms/negative",
        "compiler/functional/positive",
        "compiler/type_system/positive",
        "compiler/type_system/negative",
        "compiler/projects/positive",
        "compiler/projects/negative",
        "compiler/effects/positive",
        "compiler/effects/negative",
        "compiler/handlers/positive",
        "compiler/handlers/negative",
        "compiler/policies/positive",
        "compiler/policies/negative",
    ] {
        let path = fixture(expected);
        assert!(path.is_dir(), "{} should exist", path.display());
    }
}

#[test]
fn root_legacy_fixture_polarity_dirs_are_not_reintroduced() {
    for obsolete in ["positive", "negative", "golden"] {
        let path = fixture(obsolete);
        assert!(
            !path.exists(),
            "{} is a legacy root fixture directory; migrate cases into syntax/, hir/, cli/, or compiler/",
            path.display()
        );
    }
}

#[test]
fn memory_schema_abbreviations_use_alias_not_nominal_type() {
    let mut failures = Vec::new();
    for path in fixtures_under("") {
        let source = std::fs::read_to_string(&path).unwrap();
        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let declaration = trimmed.strip_prefix("public ").unwrap_or(trimmed);
            if declaration.starts_with("type ")
                && declaration.contains("MemorySchema")
                && declaration.contains("= MemoryRegion")
            {
                failures.push(format!(
                    "{}:{} uses nominal `type` for a transparent memory schema abbreviation; use `alias`",
                    path.display(),
                    line_index + 1
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "memory schema alias regressions:\n{}",
        failures.join("\n")
    );
}

#[test]
fn type_record_declarations_use_current_terminated_form() {
    let mut failures = Vec::new();
    for path in fixtures_under("") {
        let source = std::fs::read_to_string(&path).unwrap();
        let mut in_type_record = false;
        let mut depth = 0isize;
        let mut start_line = 0usize;

        for (line_index, line) in source.lines().enumerate() {
            if !in_type_record && starts_type_record_declaration(line) {
                depth = brace_delta(line);
                start_line = line_index + 1;
                if depth == 0 && !line_has_terminating_semicolon(line) {
                    failures.push(format!("{}:{}", path.display(), line_index + 1));
                } else if depth != 0 {
                    in_type_record = true;
                }
                continue;
            }

            if in_type_record {
                depth += brace_delta(line);
                if depth == 0 {
                    if !line_has_terminating_semicolon(line) {
                        failures.push(format!(
                            "{}:{} closes type record started at line {} without `;`",
                            path.display(),
                            line_index + 1,
                            start_line
                        ));
                    }
                    in_type_record = false;
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "type record declarations must terminate with `;` under the current SPEC:\n{}",
        failures.join("\n")
    );
}

fn starts_type_record_declaration(line: &str) -> bool {
    let trimmed = line.trim_start();
    let declaration = trimmed.strip_prefix("public ").unwrap_or(trimmed);
    declaration.starts_with("type ") && declaration.contains("= {")
}

fn line_has_terminating_semicolon(line: &str) -> bool {
    let before_comment = line.split_once("//").map_or(line, |(before, _)| before);
    before_comment.trim_end().ends_with(';')
}

fn brace_delta(line: &str) -> isize {
    let mut delta = 0;
    let mut in_string = false;
    let mut escaped = false;
    for ch in line.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            delta += 1;
        } else if ch == '}' {
            delta -= 1;
        }
    }
    delta
}
