mod support;

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use support::{fixtures_under, path_str, read_source, reported_value, run_cli};

macro_rules! positive_fixtures {
    ($($name:ident, $requests:literal, $retained:literal;)*) => {
        const POSITIVE: &[(&str, usize, bool)] = &[
            $((stringify!($name), $requests, $retained),)*
        ];
        $(
            #[test]
            fn $name() {
                assert_positive_fixture(stringify!($name), $requests, $retained);
            }
        )*
    };
}

positive_fixtures! {
    pure_snapshot, 0, true;
    callable_descriptors, 0, true;
    bound_flow_snapshot, 0, true;
    bound_flow_compile, 1, true;
    bound_effect_flow_compile, 1, true;
    bound_effect_row_compile, 1, true;
    bound_nonempty_effect_row_compile, 3, true;
    bound_callback_nonreturn_rejected, 1, true;
    bound_callback_argument_change_rejected, 1, true;
    bound_callback_literal_argument_compile, 1, true;
    bound_callback_input_argument_compile, 2, true;
    bound_callback_input_identity_rejected, 1, true;
    bound_callback_integer_literal_compile, 2, true;
    bound_callback_signed_boundary_compile, 1, true;
    conditional_callback_rejected, 1, true;
    bound_flow_revision_checkpoint, 1, true;
    pure_syntax_operations, 0, true;
    compile_and_adopt, 1, true;
    rejection_keeps_binding, 2, true;
    custom_compile_handler, 0, true;
    custom_compile_handler_two_requests, 0, true;
    custom_compile_handler_deferred_request, 0, true;
    custom_compile_handler_transitive, 0, true;
    custom_compile_handler_frozen_helpers, 1, true;
    frozen_helper_unproved_outcomes, 1, true;
    generic_handler_payload_inference, 0, false;
    bare_agent_rejected, 1, false;
    returned_nominal_callable, 1, true;
    frozen_lexical_scope, 1, true;
    successive_revisions, 2, true;
    diff_surface, 0, true;
    diff_unavailable_bodies, 0, false;
    diff_checkpoint, 0, true;
    flow_diff_surface, 0, true;
    flow_diff_unavailable_bodies, 0, false;
    flow_diff_checkpoint, 0, true;
    method_revision_snapshots, 1, true;
    returned_flow_reflection, 1, true;
    flow_binding_reflection_in_revision, 1, true;
    flow_parameter_reflection_in_revision, 1, true;
    returned_generic_flow, 1, true;
    flow_revision_checkpoint, 1, true;
    flow_value_revision_checkpoint, 1, true;
}

const NEGATIVE: &[(&str, &str)] = &[
    ("reflect_call", "syntax::InvalidExpression"),
    ("string_body", "type::TypeMismatch"),
    ("wrong_agent_identity", "type::Mismatch"),
    ("definition_not_callable", "type::TypeMismatch"),
    ("compile_erased_callable", "type::TypeMismatch"),
    ("compile_method_descriptor", "type::TypeMismatch"),
    ("diff_wrong_identity", "type::TypeMismatch"),
    ("diff_report_readonly", "type::TypeMismatch"),
    ("diff_wrong_flow_signature", "type::TypeMismatch"),
    ("handler_wrong_identity", "type::TypeMismatch"),
    ("handler_heterogeneous_requests", "type::TypeMismatch"),
    ("handler_deferred_wrong_identity", "type::TypeMismatch"),
];

#[test]
fn every_reflection_fixture_is_registered_for_real_compiler_execution() {
    for (polarity, expected) in [
        (
            "positive",
            POSITIVE
                .iter()
                .map(|(name, _, _)| *name)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "negative",
            NEGATIVE
                .iter()
                .map(|(name, _)| *name)
                .collect::<BTreeSet<_>>(),
        ),
    ] {
        let actual = fixtures_under(&format!("compiler/reflection/{polarity}"))
            .into_iter()
            .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected.into_iter().map(str::to_owned).collect());
    }
}

fn assert_positive_fixture(name: &str, compiler_requests: usize, retained: bool) {
    let fixture = support::fixture(&format!("compiler/reflection/positive/{name}.es"));
    let parsed = etas_syntax::parse_program(read_source(&fixture));
    assert!(
        parsed.diagnostics.is_empty(),
        "{name}: {:?}",
        parsed.diagnostics
    );
    let project = FixtureProject::new(&fixture, retained);
    let (code, stdout, stderr) = run_cli(["check", path_str(&project.0)]);
    assert_eq!(code, 0, "{name}: {stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{name}: {stderr}");
    let trace = project.0.join("trace.json");
    let (code, stdout, stderr) = run_cli([
        "run",
        path_str(&project.0),
        "--profile",
        "fixture",
        "--trace-out",
        path_str(&trace),
    ]);
    let trace_source = std::fs::read_to_string(&trace);
    assert_eq!(
        code, 0,
        "{name}: {stdout}\n{stderr}\ntrace: {trace_source:?}"
    );
    assert!(stderr.is_empty(), "{name}: {stderr}");
    let result = reported_value(&stdout, "run value: ");
    assert_eq!(
        result,
        serde_json::json!({"kind": "number", "type": "i32", "value": "0"}),
        "{name}"
    );
    let trace: serde_json::Value = serde_json::from_str(&trace_source.unwrap()).unwrap();
    let requests = trace["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "host_request_started")
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), compiler_requests, "{name}: {trace}");
    assert!(
        requests
            .iter()
            .all(|event| event["request_kind"] == "compiler"),
        "{name}: {trace}"
    );
}

#[test]
fn reflection_negative_fixtures_are_rejected_with_stable_diagnostic_codes() {
    for (name, diagnostic) in NEGATIVE {
        let fixture = support::fixture(&format!("compiler/reflection/negative/{name}.es"));
        let project = FixtureProject::new(&fixture, false);
        let (code, stdout, stderr) = run_cli(["check", path_str(&project.0)]);
        assert_eq!(code, 1, "{name}: {stdout}\n{stderr}");
        assert!(stderr.contains(diagnostic), "{name}: {stdout}\n{stderr}");
        assert!(!stdout.contains("run value:"), "{name}: {stdout}");
    }
}

#[test]
fn default_input_uses_the_selected_workspace_and_its_compiler_profile() {
    let source = support::fixture("compiler/reflection/positive/compile_and_adopt.es");
    let project = FixtureProject::new(&source, true);
    let (code, stdout, stderr) = run_cli([
        "--workspace",
        path_str(&project.0),
        "run",
        "--profile",
        "fixture",
    ]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        reported_value(&stdout, "run value: "),
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"})
    );
}

#[test]
fn diff_report_survives_checkpoint_and_resume_as_descriptive_data() {
    for name in ["diff_checkpoint", "flow_diff_checkpoint"] {
        reflection_checkpoint_round_trip(name);
    }
}

#[test]
fn flow_revision_checkpoint_restores_a_scoped_invocation() {
    for name in [
        "flow_revision_checkpoint",
        "flow_value_revision_checkpoint",
        "bound_flow_revision_checkpoint",
    ] {
        reflection_checkpoint_round_trip(name);
    }
}

#[test]
fn bound_effectful_callback_revision_executes_its_checked_host_action() {
    let source = support::fixture("compiler/reflection/effectful/bound_callback.es");
    let project = FixtureProject::new(&source, true);
    let trace = project.0.join("trace.json");
    let (code, stdout, stderr) = run_cli([
        "--format",
        "json",
        "run",
        path_str(&project.0),
        "--profile",
        "fixture",
        "--trace-out",
        path_str(&trace),
    ]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        result["value"],
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"})
    );
    let trace: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(trace).unwrap()).unwrap();
    let requests = trace["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "host_request_started")
        .collect::<Vec<_>>();
    assert_eq!(
        requests
            .iter()
            .filter(|event| event["request_kind"] == "compiler")
            .count(),
        3
    );
    assert_eq!(
        requests
            .iter()
            .filter(|event| event["request_kind"] == "console")
            .count(),
        1
    );
}

#[test]
fn callback_revision_accepts_an_enclosing_disjoint_handler() {
    let source = support::fixture("compiler/reflection/effectful/disjoint_handler.es");
    let project = FixtureProject::new(&source, true);
    let trace = project.0.join("trace.json");
    let (code, stdout, stderr) = run_cli([
        "--format",
        "json",
        "run",
        path_str(&project.0),
        "--profile",
        "fixture",
        "--trace-out",
        path_str(&trace),
    ]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        result["value"],
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"}),
        "{stderr}"
    );
    let trace: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(trace).unwrap()).unwrap();
    let requests = trace["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "host_request_started")
        .collect::<Vec<_>>();
    assert_eq!(
        requests
            .iter()
            .filter(|event| event["request_kind"] == "compiler")
            .count(),
        10
    );
    assert_eq!(
        requests
            .iter()
            .filter(|event| event["request_kind"] == "console")
            .count(),
        4
    );
}

fn reflection_checkpoint_round_trip(name: &str) {
    let source = support::fixture(&format!("compiler/reflection/positive/{name}.es"));
    let project = FixtureProject::new(&source, true);
    let checkpoints = project.0.join("checkpoints");
    let (code, stdout, stderr) = run_cli([
        "run",
        path_str(&project.0),
        "--profile",
        "fixture",
        "--checkpoint-dir",
        path_str(&checkpoints),
    ]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        reported_value(&stdout, "run value: "),
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"})
    );
    let (code, stdout, stderr) =
        run_cli(["resume", "0", "--checkpoint-dir", path_str(&checkpoints)]);
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        reported_value(&stdout, "resumed checkpoint 0: "),
        serde_json::json!({"kind":"number", "type":"i32", "value":"0"})
    );
}

struct FixtureProject(PathBuf);

impl FixtureProject {
    fn new(fixture: &Path, retained: bool) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-reflection-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let project = Self(root);
        std::fs::create_dir_all(project.0.join("src/tests")).unwrap();
        std::fs::copy(fixture, project.0.join("src/tests/reflection.es")).unwrap();
        let reflection = if retained {
            "[reflection]\nretain_bodies = ['tests.reflection.Worker']\n"
        } else {
            ""
        };
        std::fs::write(
            project.0.join("etas.toml"),
            format!(
                "[package]\nname = 'reflection-fixture'\nversion = '0.1.0'\n{reflection}[[bin]]\nname = 'app'\nmodule = 'tests.reflection'\nflow = 'main'\n[runtime.execution]\nmax_steps = 10000\n[runtime.profiles.fixture.compiler]\nbackend = 'native'\nallow_check = true\nallow_compile = true\n"
            ),
        )
        .unwrap();
        project
    }
}

impl Drop for FixtureProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
