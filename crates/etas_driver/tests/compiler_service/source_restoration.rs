use super::*;
use etas_code::definition::{CodeDefinitionOperations, DefinitionSourceBundle};
use etas_interpreter::{
    Interpreter,
    api::code::PreparedCodeDefinitions,
    api::{EntryPoint, InterpValue, RunOptions},
};

const PROGRAM: &str = r#"
module app;
import std.compiler.{AgentDef, CompileError, compile};
agent Worker() -> i32 { return helper(); }
flow helper() -> i32 { return 1; }
flow inspect() -> unit { let definition = ^Worker; return; }
flow main(draft: AgentDef<Worker>) -> i32 ![Error<CompileError>] {
    let next = compile(draft);
    return next();
}
"#;

const FLOW_PROGRAM: &str = r#"
module app;
import std.compiler.{CompileError, compile, parse_body};
flow normalize(text: string) -> string ![] { return std.text.trim(text); }
flow main(source: string) -> string ![Error<CompileError>] {
    let body = match parse_body(source) {
        Ok(body) => body,
        Err(_) => { return "parse-failed"; },
    };
    let next = compile((^normalize).with_body(body));
    return next("  Hello  ");
}
"#;

fn saved_flow_source() -> Vec<u8> {
    let (checked, definitions) = source::prepared_with_bodies(FLOW_PROGRAM, &["app.normalize"]);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_flow(expression).unwrap();
    let draft = definitions
        .with_body(
            &original,
            definitions
                .syntax_operations()
                .parse_body("let cleaned = std.text.trim(text); return cleaned;")
                .unwrap(),
        )
        .unwrap();
    serde_json::to_vec(&definitions.save_source(&draft, Default::default()).unwrap()).unwrap()
}

fn restore_flow_and_run(bytes: &[u8], allowed: bool) -> etas_interpreter::api::RunResult {
    let saved: DefinitionSourceBundle = serde_json::from_slice(bytes).unwrap();
    let (checked, definitions) = source::prepared_with_bodies(FLOW_PROGRAM, &["app.normalize"]);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_flow(expression).unwrap();
    let draft = definitions
        .restore_source_draft(&original, &saved, Default::default())
        .unwrap();
    assert_eq!(draft.revision().inputs(), &saved.revision);
    assert!(
        definitions
            .resolve(&draft)
            .unwrap()
            .checked_implementation()
            .is_none()
    );
    let prepared = Arc::new(PreparedCodeDefinitions::new(definitions.clone()));
    let value = InterpValue::String(saved.source.text.clone().into());
    let service = NativeCompilerService::new(definitions, Default::default());
    let host = CompilerHost::new(no_host::NoHost::default(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(prepared),
        ..Default::default()
    };
    if allowed {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow("Code", "compile"));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![value],
            &host,
            options,
        ))
        .unwrap();
    assert_eq!(host.inner().calls(), 0);
    result
}

#[test]
fn saved_flow_source_recompiles_in_a_fresh_run() {
    let result = restore_flow_and_run(&saved_flow_source(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::String("Hello".into())));
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
fn saved_flow_source_does_not_restore_compiler_authority() {
    let result = restore_flow_and_run(&saved_flow_source(), false);
    assert!(result.value().is_none());
    assert!(!result.diagnostics.is_empty());
    assert!(result.events.iter().all(|event| !matches!(
        event,
        etas_interpreter::api::WorkflowEvent::HostTrace(
            etas_host::TraceEvent::HostRequestStarted {
                kind: etas_host::HostRequestKind::Compiler,
                ..
            }
        )
    )));
}

#[test]
fn saved_flow_source_cannot_rebind_to_another_target() {
    let saved: DefinitionSourceBundle = serde_json::from_slice(&saved_flow_source()).unwrap();
    let other_source = FLOW_PROGRAM.replace("normalize", "other");
    let (checked, definitions) = source::prepared_with_bodies(&other_source, &["app.other"]);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let other = definitions.reflect_flow(expression).unwrap();
    assert!(matches!(
        definitions.restore_source_draft(&other, &saved, Default::default()),
        Err(
            etas_frontend::code::definition::NativeDefinitionError::SourceBundle(
                etas_code::definition::SourceBundleError::TargetMismatch
            )
        )
    ));
}

#[test]
fn self_consistent_saved_flow_source_is_rechecked() {
    let mut saved: DefinitionSourceBundle = serde_json::from_slice(&saved_flow_source()).unwrap();
    saved.source.text = "return false;".into();
    saved.source.source_start = 0;
    saved.source.source_length = saved.source.text.len() as u64;
    saved.source.source_unit =
        CodeFingerprint::of_bytes("etas.code.source-unit.v1", saved.source.text.as_bytes());
    saved.source.content =
        CodeFingerprint::of_bytes("etas.code.body-content.v1", saved.source.text.as_bytes());
    saved.revision.content = saved.source.content;
    saved.revision_key = etas_code::RevisionKey::from_inputs(&saved.revision);
    let result = restore_flow_and_run(&serde_json::to_vec(&saved).unwrap(), true);
    assert!(result.value().is_none());
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn compiled_callable_definition_parameter_has_symbolic_contract_facts() {
    let input = ProjectInput::single_source(SourceInput::anonymous(
        r#"
module app;
import std.compiler.{CallableDef, CompileError, compile};
flow main(draft: CallableDef<(string) -> string ![]>) -> string ![Error<CompileError>] {
    let next = compile(draft);
    return next("value");
}
"#,
    ));
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output.checked.is_some());
}

#[test]
fn saved_flow_source_restores_through_a_typed_definition_parameter() {
    const PROGRAM: &str = r#"
module app;
import std.compiler.{CallableDef, CompileError, compile};
flow normalize(text: string) -> string ![] { return std.text.trim(text); }
flow other(value: i32) -> i32 ![] { return value; }
flow inspect() -> unit { let definition = ^normalize; let other_definition = ^other; return; }
flow main(draft: CallableDef<(string) -> string ![]>) -> string ![Error<CompileError>] {
    let next = compile(draft);
    return next("  Hello  ");
}
"#;
    let find = |checked: &etas_frontend::CheckedProgram,
                definitions: &etas_frontend::code::definition::NativeDefinitionStore,
                name: &str| {
        checked
            .checked_reflections()
            .keys()
            .copied()
            .find(|expression| {
                matches!(definitions.catalog().static_reflection(*expression),
                Ok(etas_frontend::code::definition::StaticReflectionView::Callable(header))
                    if header.id.as_declaration().is_some_and(|id| id.declaration() == [name]))
            })
            .unwrap()
    };
    let (checked, definitions) = source::prepared_with_bodies(PROGRAM, &["app.normalize"]);
    let expression = find(&checked, &definitions, "normalize");
    let original = definitions.reflect_flow(expression).unwrap();
    let body = definitions
        .syntax_operations()
        .parse_body("let cleaned = std.text.trim(text); return cleaned;")
        .unwrap();
    let draft = definitions.with_body(&original, body).unwrap();
    let bytes =
        serde_json::to_vec(&definitions.save_source(&draft, Default::default()).unwrap()).unwrap();

    let saved: DefinitionSourceBundle = serde_json::from_slice(&bytes).unwrap();
    let (checked, definitions) = source::prepared_with_bodies(PROGRAM, &["app.normalize"]);
    let expression = find(&checked, &definitions, "normalize");
    let other_expression = find(&checked, &definitions, "other");
    let original = definitions.reflect_flow(expression).unwrap();
    let draft = definitions
        .restore_source_draft(&original, &saved, Default::default())
        .unwrap();
    let prepared = Arc::new(PreparedCodeDefinitions::new(definitions.clone()));
    assert!(
        prepared
            .callable_definition_value_at(other_expression, draft.clone())
            .is_err()
    );
    assert!(
        prepared
            .callable_definition_value_at(etas_hir::HirExprId(u32::MAX), draft.clone())
            .is_err()
    );
    let value = prepared
        .callable_definition_value_at(expression, draft)
        .unwrap();
    let foreign_value = value.clone();
    let service = NativeCompilerService::new(definitions, Default::default());
    let host = CompilerHost::new(no_host::NoHost::default(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(prepared),
        ..Default::default()
    };
    options
        .host_context
        .authority
        .grants
        .push(HostActionGrant::allow("Code", "compile"));
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![value],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::String("Hello".into())));

    let (other_checked, other_definitions) =
        source::prepared_with_bodies(PROGRAM, &["app.normalize"]);
    let other_prepared = Arc::new(PreparedCodeDefinitions::new(other_definitions.clone()));
    let other_service = NativeCompilerService::new(other_definitions, Default::default());
    let other_host = CompilerHost::new(no_host::NoHost::default(), other_service.clone())
        .with_candidate_resolver(Arc::new(other_service));
    let mut other_options = RunOptions {
        code_definitions: Some(other_prepared),
        ..Default::default()
    };
    other_options
        .host_context
        .authority
        .grants
        .push(HostActionGrant::allow("Code", "compile"));
    let rejected = runtime()
        .block_on(Interpreter.run_checked(
            &other_checked,
            EntryPoint {
                item: other_checked.entry.unwrap(),
            },
            vec![foreign_value],
            &other_host,
            other_options,
        ))
        .unwrap();
    assert!(rejected.value().is_none());
    assert!(
        rejected.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("different checked program or type identity")),
        "{:?}",
        rejected.diagnostics
    );
}

fn saved_source() -> Vec<u8> {
    let (checked, definitions) = source::prepared(PROGRAM);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let draft = definitions
        .with_body(
            &original,
            definitions
                .syntax_operations()
                .parse_body("if helper() == 1 { return 9; } return 10;")
                .unwrap(),
        )
        .unwrap();
    serde_json::to_vec(&definitions.save_source(&draft, Default::default()).unwrap()).unwrap()
}

fn restore_and_run(bytes: &[u8], allowed: bool) -> etas_interpreter::api::RunResult {
    let saved: DefinitionSourceBundle = serde_json::from_slice(bytes).unwrap();
    let (checked, definitions) = source::prepared(PROGRAM);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    let draft = definitions
        .restore_source_draft(&original, &saved, Default::default())
        .unwrap();
    assert_eq!(draft.revision().inputs(), &saved.revision);
    assert!(
        definitions
            .resolve(&draft)
            .unwrap()
            .checked_implementation()
            .is_none()
    );
    let prepared = Arc::new(PreparedCodeDefinitions::new(definitions.clone()));
    let value = prepared.definition_value(draft).unwrap();
    let service = NativeCompilerService::new(definitions, Default::default());
    let host = CompilerHost::new(no_host::NoHost::default(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(prepared),
        ..Default::default()
    };
    if allowed {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow("Code", "compile"));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![value],
            &host,
            options,
        ))
        .unwrap();
    assert_eq!(host.inner().calls(), 0);
    result
}

#[test]
fn saved_source_recompiles_in_a_fresh_run_and_executes_the_saved_body() {
    let result = restore_and_run(&saved_source(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(9)));
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
fn saved_source_cannot_restore_old_compiler_authority() {
    let result = restore_and_run(&saved_source(), false);
    assert!(result.value().is_none());
    assert!(!result.diagnostics.is_empty());
    assert!(result.events.iter().all(|event| !matches!(
        event,
        etas_interpreter::api::WorkflowEvent::HostTrace(
            etas_host::TraceEvent::HostRequestStarted {
                kind: etas_host::HostRequestKind::Compiler,
                ..
            }
        )
    )));
}

#[test]
fn self_consistent_saved_source_is_rechecked_instead_of_executing_the_original() {
    let mut saved: DefinitionSourceBundle = serde_json::from_slice(&saved_source()).unwrap();
    saved.source.text = "return false;".into();
    saved.source.source_start = 0;
    saved.source.source_length = saved.source.text.len() as u64;
    saved.source.source_unit =
        CodeFingerprint::of_bytes("etas.code.source-unit.v1", saved.source.text.as_bytes());
    saved.source.content =
        CodeFingerprint::of_bytes("etas.code.body-content.v1", saved.source.text.as_bytes());
    saved.revision.content = saved.source.content;
    saved.revision_key = etas_code::RevisionKey::from_inputs(&saved.revision);
    let result = restore_and_run(&serde_json::to_vec(&saved).unwrap(), true);
    assert!(result.value().is_none());
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn source_bundle_cannot_rebind_to_an_updated_package_environment() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    let root = loaded.package_root.clone().unwrap();
    let program = r#"
module app.main;
import dep.library.api.lookup;
agent Worker() -> i32 { return lookup(); }
flow main() -> unit { let definition = ^Worker; return; }
"#;
    std::fs::write(root.join("src/app/main.es"), program).unwrap();
    loaded.loaded.input.sources[0].text = program.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.Worker".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let target = definitions.reflect_agent(expression).unwrap();
    let draft = definitions
        .with_body(
            &target,
            definitions
                .syntax_operations()
                .parse_body("if lookup() == 7 { return 9; } return 10;")
                .unwrap(),
        )
        .unwrap();
    let saved = definitions.save_source(&draft, Default::default()).unwrap();
    assert!(!saved.dependencies.artifacts.is_empty());
    drop((draft, target, definitions, checked, loaded));

    let library = workspace.0.join("library");
    std::fs::write(
        library.join("src/library/api.es"),
        "module library.api; public flow lookup() -> i32 { return 8; }",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &origin_metadata::options(&library),
        etas_driver::PackageMetadataRequest {
            package_root: library.clone(),
            bin: None,
        },
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&root),
        etas_driver::PackageUpdateRequest {
            package_root: root.clone(),
        },
    )
    .unwrap();
    let mut fresh = etas_driver::load_project(
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
    fresh
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.Worker".into());
    let checked = origin_metadata::check(&fresh);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&fresh, &checked, Default::default())
            .unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let current = definitions.reflect_agent(expression).unwrap();
    assert!(matches!(
        definitions.restore_source_draft(&current, &saved, Default::default()),
        Err(etas_frontend::code::definition::NativeDefinitionError::SourceBundle(_))
    ));
}
