mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use etas_syntax::{DiagnosticCode, parse_program};
use support::{fixtures_under, path_str, read_source, run_cli, run_cli_with_env};

const MATRIX: &str = include_str!("../fixtures/compiler/std_conformance/matrix.csv");
const ROOT: &str = "compiler/std_conformance";

const EXPECTED_SUPPORT_TYPES: &[&str] = &[
    "Array",
    "Slice",
    "Map",
    "Range",
    "Deque",
    "Queue",
    "Stack",
    "PriorityQueue",
    "OrderedMap",
    "OrderedSet",
    "Prompt",
    "PromptEncode",
    "Schema",
    "Trusted",
    "Untrusted",
    "Public",
    "Sanitized",
    "Secret",
    "Provenance",
    "ModelResponse",
    "ResponseDecode",
    "Message",
    "SessionConfig",
    "MemoryRegion",
    "Store",
    "MemorySelection",
    "MemoryVersion",
    "MemoryConflict",
    "ValidationError",
    "ToolTimeout",
    "ToolDenied",
    "PolicyDenied",
    "EffectBoundaryViolation",
    "SandboxViolation",
    "PromptInjectionRisk",
    "MissingCitation",
    "ProtocolViolation",
    "HumanRejected",
    "IndexError",
    "Command",
    "CommandResult",
    "SandboxProfile",
    "JsonValue",
    "JsonError",
    "Command.run",
    "Error.raise",
    "Memory.read",
    "Memory.write",
    "TraceSpec",
    "Fs.read",
    "Fs.write",
    "SpecMethodDispatch",
    "ActionSelectorDescriptor",
    "Workspace.read",
    "Browser.navigate",
    "Email.send",
];

const ALLOWED_STATUSES: &[&str] = &[
    "missing",
    "partial",
    "implemented",
    "covered-positive",
    "covered-negative",
    "blocked-by-impl",
];

const REQUIRED_LAYERS: &[&str] = &["syntax", "hir", "type", "effect", "interpreter"];

#[derive(Debug)]
struct MatrixRow<'a> {
    support_type: &'a str,
    statuses: [&'a str; 6],
}

#[derive(Debug)]
struct FixtureMeta {
    path: PathBuf,
    support: Vec<String>,
    layer: String,
    polarity: String,
    status: String,
    expects: Vec<String>,
}

#[test]
fn std_conformance_matrix_has_required_support_type_coverage() {
    let rows = parse_matrix();
    let actual = rows
        .iter()
        .map(|row| row.support_type)
        .collect::<BTreeSet<_>>();

    for expected in EXPECTED_SUPPORT_TYPES {
        assert!(
            actual.contains(expected),
            "std conformance matrix is missing support type `{expected}`"
        );
    }

    assert_eq!(
        actual.len(),
        EXPECTED_SUPPORT_TYPES.len(),
        "matrix should not silently add support types without updating the expected coverage list"
    );
}

#[test]
fn std_conformance_matrix_uses_only_fixed_status_vocabulary() {
    for row in parse_matrix() {
        for status in row.statuses {
            assert!(
                ALLOWED_STATUSES.contains(&status),
                "{} uses invalid conformance status `{status}`",
                row.support_type
            );
        }
    }
}

#[test]
fn std_conformance_fixtures_are_layered_and_cover_matrix_rows() {
    let fixtures = parse_fixture_metadata();
    assert!(
        fixtures.len() >= 18,
        "std conformance should contain enough focused fixtures, got {}",
        fixtures.len()
    );

    let mut layers = BTreeMap::<String, BTreeSet<String>>::new();
    let mut support = BTreeSet::new();

    for fixture in &fixtures {
        layers
            .entry(fixture.layer.clone())
            .or_default()
            .insert(fixture.polarity.clone());
        for item in &fixture.support {
            support.insert(item.as_str());
        }
    }

    for layer in REQUIRED_LAYERS {
        assert!(
            layers.contains_key(*layer),
            "missing std conformance fixture layer `{layer}`"
        );
    }

    for layer in ["type", "effect", "interpreter"] {
        let polarities = layers.get(layer).unwrap();
        assert!(
            polarities.contains("positive") && polarities.contains("negative"),
            "layer `{layer}` must include both positive and negative fixtures"
        );
    }

    for expected in EXPECTED_SUPPORT_TYPES {
        assert!(
            support.contains(expected),
            "no std conformance fixture references support type `{expected}`"
        );
    }
}

#[test]
fn std_conformance_fixtures_do_not_hide_missing_features_with_fake_support() {
    let banned_fragments = [
        "module tests.compiler.support.algorithms",
        "return false;",
        "return \"\";",
        "return [];",
        "return input;",
        "TODO fake",
        "stub",
        "placeholder",
        "Capability(",
        "Sandbox(",
        "allow [",
        "deny [",
        "require [",
        "type T where",
        "Refinement",
    ];

    for fixture in fixtures_under(ROOT) {
        let source = std::fs::read_to_string(&fixture).unwrap();
        assert!(
            source.contains("flow main(args: Array<string>) -> i32"),
            "{} must use the canonical CLI entry shape",
            fixture.display()
        );
        assert!(
            !source.contains("\nprompt "),
            "{} must not use obsolete prompt keyword syntax",
            fixture.display()
        );
        for banned in banned_fragments {
            assert!(
                !source.contains(banned),
                "{} contains banned fake or obsolete fragment `{banned}`",
                fixture.display()
            );
        }
    }
}

#[test]
fn std_conformance_syntax_positive_fixtures_are_parser_regressions() {
    for fixture in fixtures_under("compiler/std_conformance/syntax/positive") {
        let parsed = parse_program(read_source(&fixture));
        assert!(
            !parsed
                .diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::Syntax(_))),
            "{} should parse at the syntax layer without syntax diagnostics: {:#?}",
            fixture.display(),
            parsed.diagnostics
        );
    }
}

#[test]
fn std_conformance_positive_fixtures_are_checked_or_run() {
    for meta in parse_fixture_metadata() {
        if meta.polarity != "positive" || meta.layer == "syntax" {
            continue;
        }

        let command = if meta.layer == "interpreter" {
            "run"
        } else {
            "check"
        };
        let (code, stdout, stderr) = run_positive_fixture(&meta, command);
        assert_eq!(
            code,
            0,
            "{} positive std conformance fixture failed `{command}`\nstdout:\n{}\nstderr:\n{}",
            meta.path.display(),
            stdout,
            stderr
        );
    }
}

fn run_positive_fixture(meta: &FixtureMeta, command: &str) -> (i32, String, String) {
    if meta.layer == "interpreter"
        && meta
            .support
            .iter()
            .any(|support| matches!(support.as_str(), "Memory.read" | "Memory.write"))
    {
        return run_cli_with_env(
            ["run", "--allow-effects", path_str(&meta.path)],
            [("ETAS_HOST_MEMORY", "memory")],
        );
    }

    run_cli([command, path_str(&meta.path)])
}

#[test]
fn std_conformance_negative_fixtures_record_expected_failures() {
    for meta in parse_fixture_metadata() {
        if meta.polarity == "negative" {
            assert!(
                !meta.expects.is_empty(),
                "{} is a negative fixture and must document expected diagnostics with `// expect:`",
                meta.path.display()
            );
        }

        if meta.status == "blocked-by-impl" {
            let source = std::fs::read_to_string(&meta.path).unwrap();
            assert!(
                source.contains("// status: blocked-by-impl"),
                "{} must explicitly mark implementation-blocked coverage",
                meta.path.display()
            );
        }
    }
}

#[test]
fn std_conformance_negative_fixtures_are_rejected_or_explicitly_blocked() {
    let mut accepted_blocked = Vec::new();

    for meta in parse_fixture_metadata() {
        if meta.polarity != "negative" {
            continue;
        }

        let (code, stdout, stderr) = run_cli(["check", path_str(&meta.path)]);
        if code == 0 {
            assert_eq!(
                meta.status,
                "blocked-by-impl",
                "{} was accepted by `etas check` but is not marked blocked-by-impl\nstdout:\n{}\nstderr:\n{}",
                meta.path.display(),
                stdout,
                stderr
            );
            accepted_blocked.push(meta.path);
        }
    }

    assert!(
        !accepted_blocked.is_empty(),
        "when all negative fixtures are rejected, update their matrix/test statuses away from blocked-by-impl"
    );
}

fn parse_matrix() -> Vec<MatrixRow<'static>> {
    let mut lines = MATRIX.lines();
    let header = lines.next().unwrap();
    assert_eq!(
        header,
        "support_type,std_registry,type_lowering,method_checking,effect_checking,interpreter,tests"
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts = line.split(',').collect::<Vec<_>>();
            assert_eq!(parts.len(), 7, "bad matrix row: {line}");
            MatrixRow {
                support_type: parts[0],
                statuses: [parts[1], parts[2], parts[3], parts[4], parts[5], parts[6]],
            }
        })
        .collect()
}

fn parse_fixture_metadata() -> Vec<FixtureMeta> {
    fixtures_under(ROOT)
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).unwrap();
            parse_meta(&path, &source)
        })
        .collect()
}

fn parse_meta(path: &Path, source: &str) -> FixtureMeta {
    let mut support = None;
    let mut layer = None;
    let mut polarity = None;
    let mut status = None;
    let mut expects = Vec::new();

    for line in source.lines().take(12) {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("// support:") {
            support = Some(
                rest.split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
            );
        } else if let Some(rest) = line.strip_prefix("// layer:") {
            layer = Some(rest.trim().to_owned());
        } else if let Some(rest) = line.strip_prefix("// polarity:") {
            polarity = Some(rest.trim().to_owned());
        } else if let Some(rest) = line.strip_prefix("// status:") {
            status = Some(rest.trim().to_owned());
        } else if let Some(rest) = line.strip_prefix("// expect:") {
            expects.push(rest.trim().to_owned());
        }
    }

    let meta = FixtureMeta {
        path: path.to_path_buf(),
        support: support.unwrap_or_else(|| panic!("{} missing // support:", path.display())),
        layer: layer.unwrap_or_else(|| panic!("{} missing // layer:", path.display())),
        polarity: polarity.unwrap_or_else(|| panic!("{} missing // polarity:", path.display())),
        status: status.unwrap_or_else(|| panic!("{} missing // status:", path.display())),
        expects,
    };

    assert!(
        REQUIRED_LAYERS.contains(&meta.layer.as_str()),
        "{} uses unknown layer `{}`",
        path.display(),
        meta.layer
    );
    assert!(
        matches!(meta.polarity.as_str(), "positive" | "negative"),
        "{} uses unknown polarity `{}`",
        path.display(),
        meta.polarity
    );
    assert!(
        ALLOWED_STATUSES.contains(&meta.status.as_str()),
        "{} uses unknown status `{}`",
        path.display(),
        meta.status
    );

    meta
}
