use super::*;
use etas_code::{
    descriptor::BodyAccess,
    syntax::{CodeSyntaxOperations, RetainedBodyForm},
};
use etas_driver::compiler::prepare_native_definitions;
use etas_frontend::code::definition::StaticReflectionView;

const LIBRARY: &str = r#"
module library.api;
public agent Worker(value: i32) -> i32 { return value; }
impl agent Worker { public flow explain(value: i32) -> i32 { return value; } }
public flow lookup() -> i32 { return 37; }
public flow expression(value: i32) -> i32 = value + 1;
public flow handled(value: i32) -> i32 { return value; } with { Error<i32>.raise(_) => { finish 0; } }
public flow handled_expression(value: i32) -> i32 = value; with { Error<i32>.raise(_) => { finish 0; } }
private flow secret() -> i32 { /* private-retention */ return 99; }
"#;

#[test]
fn an_exported_body_can_be_rechecked_for_a_local_target_without_dependency_source() {
    let workspace = origin_metadata::workspace_with_source_config(
        "module library.api; public agent Remote(value: bool) -> bool ![] { return !value; } public flow lookup() -> i32 { return 7; }",
        "[reflection]\nexport_bodies = [\"library.api.Remote\"]\n",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Remote;
import std.compiler.{compile, CompileError};
agent Target(value: bool) -> bool ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match (^Remote).body {
        Some(body) => body,
        None => { return false; },
    };
    let next = compile((^Target).with_body(body));
    return next(false);
}
"#
    .into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                etas_interpreter::api::WorkflowEvent::HostTrace(
                    etas_host::TraceEvent::HostRequestStarted {
                        kind: etas_host::HostRequestKind::Compiler,
                        ..
                    }
                )
            ))
            .count(),
        1
    );
}

#[test]
fn malformed_body_source_is_rejected_after_binary_decode_at_the_real_import_span() {
    for malformed in [
        "let = ;".to_owned(),
        format!("{}1{}", "(".repeat(8192), ")".repeat(8192)),
    ] {
        let workspace = origin_metadata::workspace_with_source_config(
            "module library.api; public flow lookup() -> i32 { return 7; }",
            "[reflection]\nexport_bodies = [\"library.api.lookup\"]\n",
        );
        let initial = origin_metadata::consumer(&workspace, "first");
        let root = initial.package_root.unwrap();
        let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
        let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
            &artifact,
            &std::fs::read(&artifact).unwrap(),
        )
        .unwrap();
        let body = &mut metadata
            .public_metadata
            .definition_descriptors
            .as_mut()
            .unwrap()
            .headers
            .as_mut()
            .unwrap()
            .bodies
            .as_mut()
            .unwrap()
            .entries[0];
        body.source.text = malformed;
        body.source.content = etas_code::CodeFingerprint::of_bytes(
            "etas.code.body-content.v1",
            body.source.text.as_bytes(),
        );
        let unit = format!(
            "{}{}",
            " ".repeat(body.source.source_start as usize),
            body.source.text
        );
        body.source.source_length = unit.len() as u64;
        body.source.source_unit =
            etas_code::CodeFingerprint::of_bytes("etas.code.source-unit.v1", unit.as_bytes());
        let callable = body.callable.clone();
        let content = body.source.content;
        let snapshot = metadata
            .public_metadata
            .definition_descriptors
            .as_mut()
            .unwrap()
            .headers
            .as_mut()
            .unwrap()
            .snapshots
            .as_mut()
            .unwrap()
            .flows
            .iter_mut()
            .find(|snapshot| snapshot.revision.definition == callable)
            .unwrap();
        snapshot.revision.content = content;
        let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
        let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
        etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: root.clone(),
            source_config: Some(etas_package::PackageSourceConfig::read_project(&root).unwrap()),
        })
        .unwrap();
        etas_package::lock_package(etas_package::LockPackageOptions {
            package_root: root.clone(),
        })
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
        let output = Frontend.check_project(loaded.loaded.input.clone());
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("retained body syntax"))
            .expect("malformed syntax must not enter a validated external environment");
        let source = &loaded.loaded.input.sources[0];
        assert_eq!(diagnostic.primary.span.source, source.id);
        let range = diagnostic.primary.span.range;
        assert!(
            source.text[range.start.0 as usize..range.end.0 as usize].contains("first.library.api")
        );
        assert!(output.checked.is_none());
    }
}

#[test]
fn published_retention_and_export_policy_controls_metadata_only_body_reflection() {
    for mode in ["none", "retained", "exported"] {
        let public = r#"["library.api.Worker", "library.api.Worker.explain", "library.api.lookup", "library.api.expression", "library.api.handled", "library.api.handled_expression"]"#;
        let configuration = match mode {
            "none" => "[reflection]\nretain_bodies = [\"library.api.secret\"]\n".into(),
            "retained" => format!("[reflection]\nretain_bodies = {public}\n"),
            "exported" => format!(
                "[reflection]\nretain_bodies = [\"library.api.secret\"]\nexport_bodies = {public}\n"
            ),
            _ => unreachable!(),
        };
        let workspace = origin_metadata::workspace_with_source_config(LIBRARY, &configuration);
        let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
            .unwrap()
            .unwrap();
        let descriptors = metadata
            .public_metadata
            .definition_descriptors
            .as_ref()
            .unwrap();
        let headers = descriptors.headers.as_ref().unwrap();
        let bodies = headers.bodies.as_ref().unwrap();
        assert!(!headers.callables.iter().any(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|key| key.declaration() == ["secret"])
        }));
        assert_eq!(
            bodies.entries.len(),
            if mode == "none" {
                1
            } else if mode == "retained" {
                6
            } else {
                7
            }
        );
        if mode != "retained" {
            let private = bodies
                .entries
                .iter()
                .find(|body| {
                    body.callable
                        .as_declaration()
                        .is_some_and(|key| key.declaration() == ["secret"])
                })
                .unwrap();
            assert_eq!(private.access, BodyAccess::Package);
            assert!(private.source.text.contains("private-retention"));
        }
        for alias in ["first", "second"] {
            let mut loaded = origin_metadata::consumer(&workspace, alias);
            let exported = mode == "exported";
            loaded.loaded.input.sources[0].text = format!(
                r#"
module app.main;
import {alias}.library.api.{{Worker as Remote, lookup, expression, handled, handled_expression}};
flow main() -> bool {{
    let agent = ^Remote;
    let method = match agent.methods.get(0) {{ Some(method) => method, None => {{ return false; }}, }};
    return (agent.body != None) == {exported} && (method.body != None) == {exported}
        && ((^lookup).body != None) == {exported} && ((^expression).body != None) == {exported}
        && ((^handled).body != None) == {exported} && ((^handled_expression).body != None) == {exported};
}}
"#
            );
            assert_eq!(loaded.loaded.input.sources.len(), 1);
            let checked = origin_metadata::check(&loaded);
            let definitions =
                prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
            for expression in checked.types.reflections.keys() {
                let (id, body) = match definitions
                    .catalog()
                    .static_reflection(*expression)
                    .unwrap()
                {
                    StaticReflectionView::Agent(header) => {
                        let reference = definitions.reflect_agent(*expression).unwrap();
                        let agent = definitions.resolve(&reference).unwrap();
                        let methods = definitions.methods(&reference).unwrap();
                        assert_eq!(methods[0].body.is_some(), exported);
                        assert_eq!(agent.body().is_some(), exported);
                        (header.default_entry.unwrap(), agent.body().cloned())
                    }
                    StaticReflectionView::Callable(header) => (
                        header.id.clone(),
                        definitions
                            .catalog()
                            .retained_body(*expression, definitions.syntax_operations())
                            .unwrap(),
                    ),
                };
                assert_eq!(body.is_some(), exported);
                if let Some(body) = body {
                    let source = &bodies
                        .entries
                        .iter()
                        .find(|body| body.callable == id)
                        .unwrap()
                        .source;
                    assert_eq!(body.identity(), source.identity(id.fingerprint()).unwrap());
                    let rendered = definitions.syntax_operations().format(&body).unwrap();
                    assert!(
                        etas_syntax::BodySyntax::parse(etas_syntax::SourceFile::anonymous(
                            rendered.clone()
                        ))
                        .is_ok(),
                        "{rendered}"
                    );
                    if matches!(
                        source.form,
                        RetainedBodyForm::HandledBlock | RetainedBodyForm::HandledExpression
                    ) {
                        assert!(rendered.starts_with("handle {"), "{rendered}");
                        assert!(rendered.contains("finish 0;"), "{rendered}");
                    }
                }
            }
            let host = no_host::NoHost::default();
            let result = runtime()
                .block_on(etas_interpreter::Interpreter.run_checked(
                    &checked,
                    etas_interpreter::api::EntryPoint {
                        item: checked.entry.unwrap(),
                    },
                    vec![],
                    &host,
                    etas_interpreter::api::RunOptions {
                        code_definitions: Some(Arc::new(
                            etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions),
                        )),
                        ..Default::default()
                    },
                ))
                .unwrap();
            assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
            assert_eq!(
                result.value(),
                Some(&etas_interpreter::api::InterpValue::Bool(true))
            );
            assert_eq!(host.calls(), 0);
            assert!(
                matches!(
                    result.events.as_slice(),
                    [
                        etas_interpreter::api::WorkflowEvent::StepStarted(_),
                        etas_interpreter::api::WorkflowEvent::StepCompleted(_),
                    ]
                ),
                "{:?}",
                result.events
            );
            if mode == "retained" {
                let mut override_attempt = loaded.loaded.input.clone();
                override_attempt.options.reflection.export_bodies =
                    vec![format!("{alias}.library.api.Worker")];
                let rejected = Frontend.check_project(override_attempt);
                assert!(rejected.checked.is_none());
                assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Type(
                        etas_core::TypeDiagnosticCode::InvalidReflectionConfiguration
                    )));
            }
        }
    }
}
