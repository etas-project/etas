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
