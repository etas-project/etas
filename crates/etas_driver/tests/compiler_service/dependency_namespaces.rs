use super::*;

const SOURCES: &[(&str, &str)] = &[
    (
        "library/api.es",
        r#"module library.api;
import library.internal.increment;
import library.types.UserId;
public flow apply(value: UserId<i32>) -> UserId<i32> ![] {
    return increment(value);
}
public flow lookup() -> i32 { return 7; }
"#,
    ),
    (
        "library/internal.es",
        r#"module library.internal;
import library.types.UserId;
public flow increment(value: UserId<i32>) -> UserId<i32> ![] {
    return UserId<i32> { value = value.value + 1 };
}
"#,
    ),
    (
        "library/types.es",
        "module library.types; public type UserId<T> = { value: T };",
    ),
];

pub(super) fn compile(root: &std::path::Path) -> etas_driver::CompileRunnableProjectResponse {
    etas_driver::compile_runnable_project_with_loaded(
        &origin_metadata::options(root),
        etas_driver::LoadProjectRequest {
            paths: vec![root.to_owned()],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        etas_frontend::SnapshotDetailLevel::Full,
    )
    .unwrap()
}

pub(super) fn execute(
    compiled: etas_driver::CompileRunnableProjectResponse,
    sources: usize,
    expected: i32,
) {
    let result = execute_result(compiled, sources);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(expected))
    );
}

pub(super) fn execute_result(
    compiled: etas_driver::CompileRunnableProjectResponse,
    sources: usize,
) -> etas_interpreter::api::RunResult {
    assert!(
        compiled.response.output.diagnostics.is_empty(),
        "{:?}",
        compiled.response.output.diagnostics
    );
    let checked = compiled.response.output.checked.unwrap();
    assert_eq!(
        checked.sources.sources.len(),
        sources,
        "requirements: {:?}; source paths: {:?}",
        compiled.response.output.runtime_source_requirements,
        compiled
            .loaded
            .loaded
            .sources
            .iter()
            .map(|(path, _)| path)
            .collect::<Vec<_>>(),
    );
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &compiled.loaded,
        &checked,
        Default::default(),
    )
    .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&compiled.loaded)
        .unwrap();
    revisions::execute_using_service(
        checked,
        definitions,
        etas_interpreter::api::ExecutionLimits {
            max_steps: std::num::NonZeroU64::new(4_000),
            ..Default::default()
        },
        service,
        true,
    )
}

#[test]
fn published_trace_alternatives_survive_binary_metadata_and_driver_replay() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Safe: trace = (+Console.stdout_write & -Console.stdin_read_line) |
    (+Console.stdin_read_line & -Console.stdout_write);
public flow lookup() -> i32 { return 7; }
"#,
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    let check = |body: &str| {
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
import first.library.api.Safe;
flow main() -> unit ![Console.stdin_read_line, Console.stdout_write] ~ Safe {{ {body} }}
"#
            ),
        )
        .unwrap();
        let loaded = etas_driver::load_project(
            &origin_metadata::options(&root),
            etas_driver::LoadProjectRequest {
                paths: vec![root.clone()],
                all: true,
                entry_policy: EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap();
        assert_eq!(
            loaded.loaded.input.sources.len(),
            1,
            "dependency must stay metadata-only"
        );
        Frontend.check_project(loaded.loaded.input).diagnostics
    };
    for body in [
        "perform Console.stdout_write(\"x\");",
        "perform Console.stdin_read_line();",
    ] {
        let diagnostics = check(body);
        assert!(
            diagnostics.is_empty(),
            "one complete imported alternative must accept: {diagnostics:#?}"
        );
    }
    let diagnostics =
        check("perform Console.stdin_read_line(); perform Console.stdout_write(\"x\");");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == etas_core::Severity::Error),
        "both actions must not borrow permissions across branches: {diagnostics:#?}"
    );
}

#[test]
fn published_action_trace_order_survives_binary_driver_and_consumer_analysis() {
    fn event_names(trace: &etas_package_metadata::ActionTrace, names: &mut Vec<String>) {
        match trace {
            etas_package_metadata::ActionTrace::Empty => {}
            etas_package_metadata::ActionTrace::Event(event) => {
                names.push(event.action.path.join("."))
            }
            etas_package_metadata::ActionTrace::Seq(children) => {
                for child in children {
                    event_names(child, names);
                }
            }
            other => panic!("unexpected source trace shape: {other:?}"),
        }
    }
    for (first, second, expected) in [
        (
            "stdin_read_line",
            "stdout_write",
            etas_core::EffectDiagnosticCode::TraceSpecResidualUnsupported,
        ),
        (
            "stdout_write",
            "stdin_read_line",
            etas_core::EffectDiagnosticCode::TraceSpecRequirementNotDominating,
        ),
    ] {
        let call = |name| {
            if name == "stdout_write" {
                "perform Console.stdout_write(\"message\");"
            } else {
                "perform Console.stdin_read_line();"
            }
        };
        let source = format!(
            "module library.api; public flow lookup() -> i32 ![Console.stdin_read_line, Console.stdout_write] {{ {} {} return 7; }}",
            call(first),
            call(second),
        );
        let workspace = origin_metadata::workspace_with_source(&source);
        let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
        let bytes = std::fs::read(&artifact).unwrap();
        let (_, metadata) =
            etas_package_metadata::package_metadata_from_artifact(&artifact, &bytes).unwrap();
        let summary = metadata
            .public_metadata
            .effect_summaries
            .iter()
            .find(|summary| summary.item.last().is_some_and(|name| name == "lookup"))
            .expect("published effect summary");
        let mut names = Vec::new();
        event_names(&summary.action_trace, &mut names);
        assert_eq!(
            names,
            [format!("Console.{first}"), format!("Console.{second}")]
        );

        let mut loaded = origin_metadata::consumer(&workspace, "first");
        assert_eq!(
            loaded.loaded.input.sources.len(),
            1,
            "dependency must remain metadata-only"
        );
        loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.lookup;
spec Safe: trace = Console.stdin_read_line >> Console.stdout_write;
flow main() -> i32 ![Console.stdin_read_line, Console.stdout_write] ~ Safe { return lookup(); }
"#
        .into();
        let mut supported = loaded.loaded.input.clone();
        supported.options.residual_target = etas_interpreter::interpreter_residual_capabilities();
        let output = Frontend.check_project(loaded.loaded.input);
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(expected.clone())),
            "published order {first} then {second} was not replayed: {:#?}",
            output.diagnostics
        );
        let output = Frontend.check_project(supported);
        if first == "stdin_read_line" {
            assert!(
                output
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.severity != etas_core::Severity::Error),
                "{:#?}",
                output.diagnostics
            );
            let checked = output
                .checked
                .expect("supported consumer has a guarded plan");
            assert!(!checked.residual_plans().is_empty());
            assert!(checked.residual_plans().binding().is_some());
        } else {
            assert!(output.checked.is_none());
            assert!(
                output.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(
                        etas_core::EffectDiagnosticCode::TraceSpecRequirementNotDominating
                    )),
                "{:#?}",
                output.diagnostics
            );
        }
    }
}

#[test]
fn same_package_trace_spec_replays_through_two_import_aliases() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public type Root;
public type Other;
public effect Gate { action read<R>() -> unit; }
public spec Safe: trace = +Gate.read<Array<Root>>;
public flow lookup() -> i32 ![Gate.read<Array<Root>>] ~ Safe {
    perform Gate.read<Array<Root>>();
    return 7;
}
"#,
    );
    let root = workspace.0.join("double-alias");
    std::fs::create_dir_all(root.join("src/app")).unwrap();
    std::fs::write(
        root.join("etas.toml"),
        r#"[package]
name = "double-alias"
version = "0.1.0"
edition = "2026"
[dependencies]
one = { package = "origin-library", version = "0.1.0", import = "first", path = "../library" }
two = { package = "origin-library", version = "0.1.0", import = "second", path = "../library" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup as one;
import second.library.api.lookup as two;
import first.library.api.Root as FirstRoot;
import second.library.api.Root as SecondRoot;
flow main() -> i32 ![first.library.api.Gate.read<Array<FirstRoot>>, second.library.api.Gate.read<Array<SecondRoot>>] {
    return one() + two();
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&root),
        etas_driver::PackageUpdateRequest {
            package_root: root.clone(),
        },
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&root),
        etas_driver::LoadProjectRequest {
            paths: vec![root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let mut wrong_type = loaded.loaded.input.clone();
    let metadata = wrong_type
        .environment
        .external_public_metadata
        .iter_mut()
        .find(|metadata| {
            metadata.trace_spec_summaries.iter().any(|summary| {
                summary
                    .trace_spec
                    .first()
                    .is_some_and(|part| part == "second")
            })
        })
        .unwrap();
    let effect = &mut metadata.trace_spec_summaries[0].alternatives[0][0]
        .pattern
        .as_mut()
        .unwrap()
        .effects[0];
    effect.args[0] = etas_frontend::ProjectExternalEffectArgInput::Type(
        etas_frontend::ProjectExternalTypeInput::Array(Box::new(
            etas_frontend::ProjectExternalTypeInput::Named(vec![
                "second".into(),
                "library".into(),
                "api".into(),
                "Other".into(),
            ]),
        )),
    );
    let rejected = Frontend.check_project(wrong_type);
    assert!(rejected.checked.is_none());
    assert!(
        rejected.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("conflicting metadata for trace spec definition identity")),
        "{:#?}",
        rejected.diagnostics
    );
    let mut conflicting = loaded.loaded.input.clone();
    let second = conflicting
        .environment
        .external_public_metadata
        .iter_mut()
        .find(|metadata| {
            metadata.trace_spec_summaries.iter().any(|summary| {
                summary
                    .trace_spec
                    .first()
                    .is_some_and(|part| part == "second")
            })
        })
        .expect("second alias metadata");
    second.trace_spec_summaries[0].alternatives[0][0].kind =
        etas_frontend::ProjectExternalTraceSpecClauseKindInput::Deny;
    let rejected = Frontend.check_project(conflicting);
    assert!(rejected.checked.is_none());
    assert!(
        rejected.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("conflicting metadata for trace spec definition identity")),
        "{:#?}",
        rejected.diagnostics
    );
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_some(), "{:#?}", output.diagnostics);
}

#[test]
fn published_external_trace_spec_identity_survives_transitive_metadata_replay() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Safe: trace = +Console.stdout_write;
public flow lookup() -> i32 { return 7; }
"#,
    );
    let middle = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        middle.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.Safe;
public flow exported() -> i32 ~ Safe { return 5; }
flow main() -> i32 { return exported(); }
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&middle),
        etas_driver::PackageUpdateRequest {
            package_root: middle.clone(),
        },
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &origin_metadata::options(&middle),
        etas_driver::PackageMetadataRequest {
            package_root: middle.clone(),
            bin: None,
        },
    )
    .unwrap();
    let library = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let middle_metadata = etas_package::read_package_metadata_artifact(&middle)
        .unwrap()
        .unwrap();
    let expected = &library.public_metadata.trace_spec_summaries[0].definition;
    let etas_package::PackageTraceSpecConformanceTargetMetadata::Named { definition, .. } =
        &middle_metadata.public_metadata.trace_spec_conformances[0].target
    else {
        panic!("expected named trace conformance");
    };
    assert_eq!(definition, expected);

    let downstream = workspace.0.join("downstream");
    std::fs::create_dir_all(downstream.join("src/client")).unwrap();
    std::fs::write(downstream.join("etas.toml"), r#"[package]
name = "downstream"
version = "0.1.0"
edition = "2026"
[dependencies]
middle = { package = "consumer-first", version = "0.1.0", import = "second", path = "../consumer-first" }
[[bin]]
name = "app"
module = "client.main"
flow = "main"
"#).unwrap();
    std::fs::write(
        downstream.join("src/client/main.es"),
        r#"module client.main;
import second.app.main.exported;
flow main() -> i32 { return exported(); }
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&downstream),
        etas_driver::PackageUpdateRequest {
            package_root: downstream.clone(),
        },
    )
    .unwrap();
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&downstream),
        etas_driver::LoadProjectRequest {
            paths: vec![downstream],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_some(), "{:#?}", output.diagnostics);
}

#[test]
fn external_trace_spec_replay_rejects_forged_definition_identity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Safe: trace = +Console.stdout_write;
public flow lookup() -> i32 ~ Safe { return 7; }
"#,
    );
    let loaded = origin_metadata::consumer(&workspace, "first");
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let bytes = std::fs::read(&artifact).unwrap();
    let (header, metadata) =
        etas_package_metadata::package_metadata_from_artifact(&artifact, &bytes).unwrap();

    for corrupt_summary in [false, true] {
        let mut modified = metadata.clone();
        let target = &mut modified.public_metadata.trace_spec_conformances[0].target;
        let etas_package_metadata::TraceSpecConformanceTarget::Named { definition, .. } = target
        else {
            panic!("expected named trace conformance");
        };
        let forged = etas_code::DefinitionKey::new(
            etas_code::PackageKey::from_name_and_version("different-package", "1.0.0").unwrap(),
            definition.module().to_vec(),
            definition.declaration().to_vec(),
            etas_code::DefinitionKind::Spec,
        )
        .unwrap();
        if corrupt_summary {
            modified.public_metadata.trace_spec_summaries[0].definition = forged;
        } else {
            *definition = forged;
        }
        let sections = etas_package_metadata::package_metadata_to_sections(&modified).unwrap();
        std::fs::write(
            &artifact,
            etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap(),
        )
        .unwrap();
        let published = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
            .unwrap()
            .unwrap();
        let mut environment = loaded.package_environment.clone().unwrap();
        environment
            .dependencies
            .iter_mut()
            .find(|dependency| dependency.import_root == "first")
            .unwrap()
            .public_metadata = published.public_metadata;
        let mut input = loaded.loaded.input.clone();
        input.environment = etas_driver::package_env::frontend_environment(environment).unwrap();
        input.sources[0].text = "module app.main; import first.library.api.lookup; flow main() -> i32 { return lookup(); }".into();
        let output = Frontend.check_project(input);
        assert!(output.checked.is_none());
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts
                )),
            "{:#?}",
            output.diagnostics
        );
    }
}

#[test]
fn external_conformance_validation_uses_binary_trace_not_requested_set_order() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Safe: trace = +Console.stdin_read_line & +Console.stdout_write;
public flow lookup() -> i32 ![Console.stdin_read_line, Console.stdout_write] ~ Safe {
    perform Console.stdout_write("message");
    perform Console.stdin_read_line();
    return 7;
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let bytes = std::fs::read(&artifact).unwrap();
    let (header, mut metadata) =
        etas_package_metadata::package_metadata_from_artifact(&artifact, &bytes).unwrap();
    let spec = metadata
        .public_metadata
        .trace_spec_summaries
        .iter_mut()
        .find(|summary| summary.trace_spec.last().is_some_and(|name| name == "Safe"))
        .unwrap();
    let row = |name: &str| {
        spec.alternatives[0]
            .iter()
            .filter_map(|clause| clause.pattern.as_ref())
            .find(|row| {
                row.effects
                    .iter()
                    .any(|effect| effect.path.last().is_some_and(|part| part == name))
            })
            .cloned()
            .unwrap()
    };
    let guard = row("stdin_read_line");
    let target = row("stdout_write");
    spec.alternatives = vec![vec![etas_package_metadata::TraceSpecClause {
        kind: etas_package_metadata::TraceSpecClauseKind::RequireBefore,
        guard: Some(guard),
        target: Some(target),
        ..Default::default()
    }]];
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    std::fs::write(
        &artifact,
        etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap(),
    )
    .unwrap();
    let published = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let mut environment = loaded.package_environment.clone().unwrap();
    environment
        .dependencies
        .iter_mut()
        .find(|dependency| dependency.import_root == "first")
        .unwrap()
        .public_metadata = published.public_metadata;
    loaded.loaded.input.environment =
        etas_driver::package_env::frontend_environment(environment).unwrap();
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.lookup;
flow main() -> i32 ![Console.stdin_read_line, Console.stdout_write] { return lookup(); }
"#
    .into();
    assert_eq!(
        loaded.loaded.input.sources.len(),
        1,
        "dependency must stay metadata-only"
    );
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::TraceSpecRequirementNotDominating
            )),
        "reversed trace must not be reconstructed from the requested set: {:#?}",
        output.diagnostics
    );
}

#[test]
fn trace_selector_bounds_survive_binary_to_frontend_and_fail_closed() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Within<T>;
public type ReportsRoot;
public effect Gate { action read<R>() -> unit; }
public spec Safe: trace = +Gate.read<_>;
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let bytes = std::fs::read(&artifact).unwrap();
    let (header, mut metadata) =
        etas_package_metadata::package_metadata_from_artifact(&artifact, &bytes).unwrap();
    let summary = metadata
        .public_metadata
        .trace_spec_summaries
        .iter_mut()
        .find(|summary| summary.trace_spec.last().is_some_and(|name| name == "Safe"))
        .expect("published trace spec");
    let clause = &mut summary.alternatives[0][0];
    let effect_path = clause.pattern.as_ref().unwrap().effects[0].path.clone();
    clause
        .selector_bounds
        .push(etas_package_metadata::TraceSelectorBound {
            operand: etas_package_metadata::TraceSelectorOperand::Pattern,
            effect_path,
            argument: 0,
            binding: "R".into(),
            spec_path: vec!["library".into(), "api".into(), "Within".into()],
            spec_args: vec![etas_package_metadata::Type {
                kind: etas_package_metadata::TypeKind::Named,
                path: vec!["library".into(), "api".into(), "ReportsRoot".into()],
                ..Default::default()
            }],
        });
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    std::fs::write(
        &artifact,
        etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap(),
    )
    .unwrap();
    let published = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    assert!(
        !published
            .public_metadata
            .trace_spec_summaries
            .iter()
            .find(|summary| summary.trace_spec.last().is_some_and(|name| name == "Safe"))
            .unwrap()
            .alternatives[0][0]
            .selector_bounds
            .is_empty()
    );
    let mut package_environment = loaded
        .package_environment
        .clone()
        .expect("prepared environment");
    let dependency = package_environment
        .dependencies
        .iter_mut()
        .find(|dependency| dependency.import_root == "first")
        .expect("dependency");
    dependency.public_metadata = published.public_metadata;
    loaded.loaded.input.environment =
        etas_driver::package_env::frontend_environment(package_environment).unwrap();
    let metadata = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter()
        .find(|metadata| {
            metadata
                .trace_spec_summaries
                .iter()
                .any(|summary| summary.trace_spec.last().is_some_and(|name| name == "Safe"))
        })
        .expect("dependency trace metadata");
    let summary = metadata
        .trace_spec_summaries
        .iter()
        .find(|summary| summary.trace_spec.last().is_some_and(|name| name == "Safe"))
        .unwrap();
    let bound = &summary.alternatives[0][0].selector_bounds[0];
    assert_eq!(bound.binding, "R");
    assert_eq!(
        bound.effect_path,
        summary.alternatives[0][0].pattern.as_ref().unwrap().effects[0].path
    );
    assert_eq!(bound.spec_path, ["first", "library", "api", "Within"]);
    assert_eq!(
        bound.spec_args,
        [etas_frontend::ProjectExternalTypeInput::Named(vec![
            "first".into(),
            "library".into(),
            "api".into(),
            "ReportsRoot".into()
        ])]
    );
    let source = &mut loaded.loaded.input.sources[0];
    let source_id = source.id;
    source.text =
        "module app.main; import first.library.api.Safe; flow main() -> unit ~ Safe { return; }"
            .into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("selector bounds require checked replay support")
                && diagnostic.primary.span.source == source_id
        }),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn dependency_overlays_keep_canonical_imports_and_type_only_metadata_across_aliases() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    for (alias, namespace) in [("first", "first.library"), ("library", "library")] {
        let root = origin_metadata::consumer(&workspace, alias)
            .package_root
            .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
import {namespace}.api.apply;
import {namespace}.types.UserId;
import std.compiler.{{compile, CompileError}};
agent Worker(value: UserId<i32>) -> UserId<i32> ![] {{ return apply(value); }}
flow main() -> i32 ![Error<CompileError>] {{
    if (^apply).body != None {{ return -1; }}
    let revised = compile(^Worker);
    return revised(UserId<i32> {{ value = 18 }}).value;
}}
"#
            ),
        )
        .unwrap();
        let compiled = compile(&root);
        assert!(
            compiled
                .loaded
                .loaded
                .sources
                .iter()
                .all(|(path, _)| !path.ends_with("library/types.es")),
            "type-only dependencies must remain metadata-only for alias {alias}"
        );
        let checked = compiled.response.output.checked.as_ref().unwrap();
        for source in &compiled.loaded.loaded.input.sources {
            if matches!(
                source.kind,
                etas_frontend::SourceKind::DependencySourceOverlay { .. }
            ) {
                let key = compiled
                    .loaded
                    .loaded
                    .input
                    .environment
                    .stable_source_key(source.id)
                    .expect("overlay source has a stable package-relative identity");
                assert_eq!(checked.sources.stable_source_key(source.id), Ok(key));
            }
        }
        execute(compiled, 3, 19);
    }
}

#[test]
fn runtime_overlay_preserves_the_publishers_trace_clause_source_identity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
public spec Safe: trace = -Console.stdout_write;
public flow lookup() -> i32 ![] ~ Safe { return 7; }
"#,
    );
    let library = workspace.0.join("library");
    let publisher = etas_driver::load_project(
        &origin_metadata::options(&library),
        etas_driver::LoadProjectRequest {
            paths: vec![library],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let publisher_checked = origin_metadata::check(&publisher);
    let identity = |checked: &etas_frontend::CheckedProgram| {
        let fact = checked
            .effects
            .trace_specs
            .conformances
            .values()
            .flatten()
            .next()
            .unwrap();
        (
            fact.source_spec_instance.as_ref().unwrap().id(),
            fact.alternatives[0][0].source_clause().unwrap(),
        )
    };
    let expected = identity(&publisher_checked);
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        "module app.main; import first.library.api.lookup; flow main() -> i32 { return lookup(); }",
    )
    .unwrap();
    let compiled = compile(&root);
    let checked = compiled.response.output.checked.as_ref().unwrap();
    assert_eq!(identity(checked), expected);
    execute(compiled, 2, 7);
}

#[test]
fn dependency_packages_with_the_same_source_module_keep_distinct_overlay_identities() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 ![] { return 7; }",
    );
    let other = workspace.0.join("other");
    std::fs::create_dir_all(other.join("src/library")).unwrap();
    std::fs::write(
        other.join("etas.toml"),
        "[package]\nname = \"other-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
    )
    .unwrap();
    std::fs::write(
        other.join("src/library/api.es"),
        "module library.api; public flow lookup() -> i32 ![] { return 11; }",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &origin_metadata::options(&other),
        etas_driver::PackageMetadataRequest {
            package_root: other,
            bin: None,
        },
    )
    .unwrap();
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("etas.toml"),
        r#"
[package]
name = "consumer-first"
version = "0.1.0"
edition = "2026"
[dependencies]
first = { package = "origin-library", version = "0.1.0", import = "first", path = "../library" }
second = { package = "other-library", version = "0.1.0", import = "second", path = "../other" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup as first;
import second.library.api.lookup as second;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return first() + second(); }
flow main() -> i32 ![Error<CompileError>] {
    if (^first).body != None || (^second).body != None { return -1; }
    let revised = compile(^Worker);
    return revised();
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&root),
        etas_driver::PackageUpdateRequest {
            package_root: root.clone(),
        },
    )
    .unwrap();
    execute(compile(&root), 3, 18);
}

#[test]
fn retained_definition_analysis_does_not_materialize_its_dependency_runtime_sources() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    let manifest = root.join("etas.toml");
    let content = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        format!("{content}\n[reflection]\nretain_bodies = [\"app.main.Worker\"]\n"),
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.apply;
import first.library.types.UserId;
agent Worker(value: UserId<i32>) -> UserId<i32> ![] { return apply(value); }
flow main() -> i32 {
    if (^Worker).body == None { return -1; }
    return 19;
}
"#,
    )
    .unwrap();
    execute(compile(&root), 1, 19);
}

#[test]
fn reflecting_an_external_flow_does_not_materialize_hidden_runtime_source() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 ![] { return 7; }",
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup;
flow main() -> i32 {
    if (^lookup).body != None { return -1; }
    return 19;
}
"#,
    )
    .unwrap();
    execute(compile(&root), 1, 19);
}

#[test]
fn package_local_namespace_bindings_do_not_leak_into_the_consumer() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import library.api.apply;
flow main() -> i32 { return 0; }
"#,
    )
    .unwrap();
    let compiled = compile(&root);
    assert_eq!(compiled.loaded.loaded.sources.len(), 1);
    let output = &compiled.response.output;
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Syntax(etas_core::SyntaxDiagnosticCode::InvalidItem)
                && diagnostic
                    .message
                    .contains("missing imported module `library.api.apply`")
                && diagnostic.primary.span.source == compiled.loaded.loaded.input.sources[0].id
        }),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn dependency_overlay_resolves_grouped_and_wildcard_imports_in_its_package_namespace() {
    let sources = SOURCES
        .iter()
        .map(|(path, source)| {
            (
                *path,
                if *path == "library/api.es" {
                    r#"module library.api;
import library.internal.{increment};
import library.types.*;
public flow apply(value: UserId<i32>) -> UserId<i32> ![] { return increment(value); }
public flow lookup() -> i32 { return 7; }
"#
                } else {
                    *source
                },
            )
        })
        .collect::<Vec<_>>();
    let workspace = origin_metadata::workspace_with_sources(&sources, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.apply;
import first.library.types.UserId;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return apply(UserId<i32> { value = 18 }).value; }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return revised();
}
"#,
    )
    .unwrap();
    execute(compile(&root), 3, 19);
}

#[test]
fn dependency_reexports_materialize_the_defining_module_without_exposing_its_body() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                "module library.api; public import library.internal.increment as apply; public flow lookup() -> i32 { return 7; }",
            ),
            (
                "library/internal.es",
                "module library.internal; public flow increment(value: i32) -> i32 ![] { return value + 1; }",
            ),
        ],
        "",
    );
    for (import, callable) in [
        ("import first.library.api.apply;", "apply"),
        ("import first.library.api.{apply};", "apply"),
        ("import first.library.api.*;", "apply"),
        ("import first.library.api as Api;", "Api.apply"),
    ] {
        let root = origin_metadata::consumer(&workspace, "first")
            .package_root
            .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
{import}
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 ![] {{ return {callable}(18); }}
flow main() -> i32 ![Error<CompileError>] {{
    if (^{callable}).body != None {{ return -1; }}
    let revised = compile(^Worker);
    return revised();
}}
"#
            ),
        )
        .unwrap();
        let compiled = compile(&root);
        assert!(
            compiled.response.output.diagnostics.is_empty(),
            "{import}: {:?}",
            compiled.response.output.diagnostics
        );
        assert!(
            compiled
                .loaded
                .loaded
                .sources
                .iter()
                .all(|(path, _)| !path.ends_with("library/api.es"))
        );
        execute(compiled, 2, 19);
    }
}
