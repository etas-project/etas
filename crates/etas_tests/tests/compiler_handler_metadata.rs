mod support;

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use support::{fixture, path_str, run_cli};

struct Project(PathBuf);

impl Project {
    fn prepare() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-request-template-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let project = Self(root);
        copy(
            &fixture("compiler/reflection/packages/request_templates"),
            &project.0,
        );
        let (code, stdout, stderr) = run_cli(["etas", "pkg", "update", path_str(&project.0)]);
        assert_eq!(
            code, 0,
            "producer metadata publication failed: {stdout}\n{stderr}"
        );
        project
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn copy(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".etas" {
            continue;
        }
        let to = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}

#[test]
fn request_payload_types_preserve_private_dependency_identity_without_exporting_it() {
    let project = Project::prepare();
    let loaded = etas_driver::load_project(
        &etas_driver::DriverOptions {
            cache_mode: etas_driver::DriverCacheMode::Off,
            ..Default::default()
        },
        etas_driver::LoadProjectRequest {
            paths: vec![project.0.clone()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    std::fs::remove_dir_all(project.0.join("deps/requests/src")).unwrap();
    let mut input = loaded.loaded.input;
    input.sources[0].text = r#"module app.main;
import requests.api.{Node, build_internal};
import std.compiler.CompileError;
flow main() -> unit ![Error<CompileError>] {
    let node: Node = Node { next = None };
    build_internal(); return;
}
"#
    .into();
    let result = etas_frontend::Frontend.check_project(input.clone());
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(result.checked.is_some());
    input.sources[0].text =
        "module app.main; import requests.api.Internal; flow main() -> unit { return; }".into();
    let result = etas_frontend::Frontend.check_project(input);
    assert!(
        result.checked.is_none(),
        "private request type became public"
    );
    assert!(
        result.diagnostics.iter().all(|diagnostic| diagnostic.code
            != etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn published_request_templates_infer_custom_handlers_without_dependency_source() {
    let project = Project::prepare();
    let loaded = etas_driver::load_project(
        &etas_driver::DriverOptions {
            cache_mode: etas_driver::DriverCacheMode::Off,
            ..Default::default()
        },
        etas_driver::LoadProjectRequest {
            paths: vec![project.0.clone()],
            all: true,
            entry_policy: etas_frontend::EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    // After artifact loading no producer source is available to the checker.
    std::fs::remove_dir_all(project.0.join("deps/requests/src")).unwrap();
    for (body, output, accepted) in [
        ("build(^Worker)", "Worker", true),
        ("apply(() => build(^Worker))", "Worker", true),
        (
            "let thunk = wrap(() => build(^Worker)); apply(thunk)",
            "Worker",
            true,
        ),
        (
            "shield(() => build(^Other)); build(^Worker)",
            "Worker",
            true,
        ),
        ("build(^Other)", "Other", false),
        (
            "let first = compile(^Worker); build(^Other)",
            "Other",
            false,
        ),
    ] {
        let mut input = loaded.loaded.input.clone();
        assert_eq!(input.sources.len(), 1, "dependency must stay metadata-only");
        let metadata = input
            .environment
            .external_public_metadata
            .iter()
            .find(|metadata| {
                metadata
                    .flows
                    .iter()
                    .any(|flow| flow.path.last().is_some_and(|name| name == "build"))
            })
            .unwrap();
        let summaries = metadata
            .effect_summaries
            .iter()
            .filter(|summary| {
                summary
                    .item
                    .last()
                    .is_some_and(|name| ["build", "apply", "wrap"].contains(&name.as_str()))
            })
            .collect::<Vec<_>>();
        assert_eq!(summaries.len(), 3);
        assert!(
            summaries
                .iter()
                .all(|summary| summary.handler_requests.is_some())
        );
        let shield = metadata
            .effect_summaries
            .iter()
            .find(|summary| summary.item.last().is_some_and(|name| name == "shield"))
            .and_then(|summary| summary.handler_requests.as_ref())
            .unwrap();
        assert!(!shield.parameters.is_empty());
        assert!(shield.parameters.iter().all(|parameter| {
            parameter
                .excluded
                .iter()
                .any(|action| action.path.ends_with(&["Code".into(), "compile".into()]))
        }));
        input.sources[0].text = format!(
            r#"
module app.main;
import std.compiler.{{CompileError, compile}};
import requests.api.{{build, apply, wrap, shield}};
agent Worker() -> i32 {{ return 7; }}
agent Other() -> i32 {{ return 9; }}
flow main() -> {output} ![Error<CompileError>] {{
    return handle {{ {body} }} with {{ Code.compile(_) => {{ resume Worker; }} }};
}}
"#
        );
        let mut missing = input.clone();
        for metadata in &mut missing.environment.external_public_metadata {
            for summary in &mut metadata.effect_summaries {
                summary.handler_requests = None;
            }
        }
        let missing = etas_frontend::Frontend.check_project(missing);
        assert!(
            missing.checked.is_none(),
            "missing request evidence: {body}"
        );
        assert!(
            missing.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts
                )),
            "missing request evidence: {body}: {:?}",
            missing.diagnostics
        );
        let result = etas_frontend::Frontend.check_project(input);
        if accepted {
            assert!(
                result.diagnostics.is_empty(),
                "{body}: {:?}",
                result.diagnostics
            );
            assert!(result.checked.is_some(), "{body}");
        } else {
            assert!(
                result.checked.is_none(),
                "accepted mismatched request: {body}"
            );
            assert!(
                result.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Type(
                        etas_core::TypeDiagnosticCode::TypeMismatch
                    )),
                "{body}: {:?}",
                result.diagnostics
            );
        }
    }
}
