mod support;

use std::path::{Path, PathBuf};

use support::{fixtures_under, path_str, run_cli};

const TYPE_SYSTEM_POSITIVE: &str = "compiler/type_system/positive";
const TYPE_SYSTEM_NEGATIVE: &str = "compiler/type_system/negative";

#[test]
fn type_system_positive_fixtures_cover_alias_and_nominal_spec() {
    let fixtures = fixtures_under(TYPE_SYSTEM_POSITIVE);
    assert_eq!(
        fixtures.len(),
        6,
        "expected six positive type-system fixtures for alias transparency, generic alias expansion, nominal construction, nominal record accessors, host-returned bodyless types, and generic trait bounds"
    );

    let mut failures = Vec::new();
    for fixture in fixtures {
        assert_type_system_fixture_shape(&fixture, true, &mut failures);
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        if code != 0 {
            failures.push(format!(
                "{} should satisfy the latest nominal-type SPEC\nstdout:\n{stdout}\nstderr:\n{stderr}",
                fixture.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "type-system positive fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn type_system_negative_fixtures_reject_nominal_and_alias_misuse() {
    let fixtures = fixtures_under(TYPE_SYSTEM_NEGATIVE);
    assert_eq!(
        fixtures.len(),
        8,
        "expected eight negative type-system fixtures for raw nominal assignment, nominal identity mismatch, record opacity, nominal representation opacity, alias cycles, bodyless construction, unsatisfied generic trait bounds, and unresolved bound rejection"
    );

    let mut failures = Vec::new();
    for fixture in fixtures {
        assert_type_system_fixture_shape(&fixture, false, &mut failures);
        let expected = read_expected_type_diagnostics(&fixture, &mut failures);
        let (code, stdout, stderr) = run_cli(["check", path_str(&fixture)]);
        let output = format!("{stdout}\n{stderr}");

        if code == 0 {
            failures.push(format!(
                "{} should be rejected by type checking under the nominal-type SPEC",
                fixture.display()
            ));
            continue;
        }

        for fragment in expected {
            if !output.contains(&fragment) {
                failures.push(format!(
                    "{} missing expected diagnostic fragment `{fragment}`\nstdout:\n{stdout}\nstderr:\n{stderr}",
                    fixture.display()
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "type-system negative fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

fn assert_type_system_fixture_shape(path: &Path, positive: bool, failures: &mut Vec<String>) {
    let source = std::fs::read_to_string(path).unwrap();
    if !source.contains("flow main(args: Array<string>) -> i32") {
        failures.push(format!(
            "{} should use the current command-line entry contract",
            path.display()
        ));
    }
    if source.contains("type ProjectMemorySchema = MemoryRegion")
        || source.contains("type MemorySchema = MemoryRegion")
    {
        failures.push(format!(
            "{} should use `alias` for transparent memory schema abbreviations",
            path.display()
        ));
    }

    let name = path.file_name().unwrap().to_string_lossy();
    match name.as_ref() {
        "alias_path_transparent.es" => assert_contains_all(
            path,
            &source,
            &[
                "alias Path = string",
                "let path: Path = raw",
                "let third: string = second",
            ],
            failures,
        ),
        "generic_alias_identity.es" => assert_contains_all(
            path,
            &source,
            &[
                "alias Id<T> = T",
                "alias Boxed<T>",
                "let same: Id<string> = raw",
            ],
            failures,
        ),
        "nominal_user_id_explicit_constructor.es" => assert_contains_all(
            path,
            &source,
            &["type UserId = string", "UserId(raw)", "accept_user(id)"],
            failures,
        ),
        "nominal_record_accessor.es" => assert_contains_all(
            path,
            &source,
            &[
                "type Review = {",
                "Review { title = \"spec\", score = 5 }",
                "return value.title",
                "return value.score",
            ],
            failures,
        ),
        "bodyless_tcp_stream_from_std_host.es" => assert_contains_all(
            path,
            &source,
            &["TcpStream", "tcp_connect(", "accept_stream(stream)"],
            failures,
        ),
        "generic_type_constraints.es" => assert_contains_all(
            path,
            &source,
            &[
                "R ~ RegionWithin<R, WorkspaceRoot>",
                "R ~ Region + Within<ReportsRoot>",
                "read_under<ReportsRoot>",
                "read_report<ReportsRoot>",
            ],
            failures,
        ),
        "raw_string_to_user_id.es" => assert_contains_all(
            path,
            &source,
            &["type UserId = string", "needs_user(raw)"],
            failures,
        ),
        "nominal_ids_not_interchangeable.es" => assert_contains_all(
            path,
            &source,
            &[
                "type UserId = string",
                "type ProjectId = string",
                "needs_project(user)",
            ],
            failures,
        ),
        "nominal_record_not_structural.es" => assert_contains_all(
            path,
            &source,
            &["type Review = {", "let raw = {", "publish(raw)"],
            failures,
        ),
        "nominal_cannot_return_representation.es" => assert_contains_all(
            path,
            &source,
            &[
                "type HeaderName = string",
                "flow raw(header: HeaderName) -> string",
                "return header",
            ],
            failures,
        ),
        "alias_cycle.es" => assert_contains_all(
            path,
            &source,
            &["alias A = B", "alias B = A", "use_alias(value: A)"],
            failures,
        ),
        "bodyless_tcp_stream_cannot_construct.es" => {
            assert_contains_all(path, &source, &["type TcpStream;", "TcpStream()"], failures)
        }
        "generic_type_constraint_missing_impl.es" => assert_contains_all(
            path,
            &source,
            &[
                "R ~ Region + Within<ReportsRoot>",
                "impl ArchiveRoot ~ Region",
                "read_report<ArchiveRoot>",
            ],
            failures,
        ),
        "unresolved_index_bound_no_std_fallback.es" => {
            assert_contains_all(path, &source, &["T ~ Index", "id<i32>(1)"], failures)
        }
        _ => failures.push(format!(
            "unrecognized type-system fixture {}",
            path.display()
        )),
    }

    if positive && name.contains("raw_string") {
        failures.push(format!(
            "{} encodes a negative nominal assignment but is stored under positive fixtures",
            path.display()
        ));
    }
}

fn assert_contains_all(path: &Path, source: &str, fragments: &[&str], failures: &mut Vec<String>) {
    for fragment in fragments {
        if !source.contains(fragment) {
            failures.push(format!("{} should contain `{fragment}`", path.display()));
        }
    }
}

fn read_expected_type_diagnostics(path: &Path, failures: &mut Vec<String>) -> Vec<String> {
    let expected_path = diagnostics_path(path);
    if !expected_path.exists() {
        failures.push(format!(
            "{} is missing its .diagnostics.txt sidecar",
            path.display()
        ));
        return Vec::new();
    }

    std::fs::read_to_string(&expected_path)
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn diagnostics_path(path: &Path) -> PathBuf {
    path.with_extension("diagnostics.txt")
}
