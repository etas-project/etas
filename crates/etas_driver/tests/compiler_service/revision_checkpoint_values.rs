use super::revision_checkpoint::compile;
use super::*;
use etas_interpreter::api::codec::{checkpoint_artifact_json, checkpoint_from_json};
use etas_interpreter::api::{InterpValue, RunOptions, code::SavedCodeRevision};

const CAPTURED: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    checkpoint("selected");
    return selected();
}
"#;

fn captured_definitions(
    program: &str,
) -> (
    etas_frontend::CheckedProgram,
    Arc<NativeDefinitionStore>,
    DefinitionRef,
) {
    let (checked, definitions) = source::prepared(program);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    (checked, definitions, original)
}

pub(super) fn save_captured(
    program: &str,
    bodies: &[Option<&str>],
    expected: i32,
) -> (Vec<Vec<u8>>, serde_json::Value) {
    save_checkpoint_at(program, bodies, expected, 1, 0)
}

pub(super) fn save_checkpoint_at(
    program: &str,
    bodies: &[Option<&str>],
    expected: i32,
    checkpoints: usize,
    selected_checkpoint: usize,
) -> (Vec<Vec<u8>>, serde_json::Value) {
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, code::PreparedCodeDefinitions},
    };
    let (checked, definitions, original) = captured_definitions(program);
    let saved: Vec<Vec<u8>> = bodies
        .iter()
        .map(|body| {
            let definition = match body {
                None => original.clone(),
                Some(body) => definitions
                    .with_body(
                        &original,
                        definitions.syntax_operations().parse_body(body).unwrap(),
                    )
                    .unwrap(),
            };
            let revision = compile(definitions.clone(), definition, true).unwrap();
            serde_json::to_vec(&revision.save_source(Default::default()).unwrap()).unwrap()
        })
        .collect();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(no_host::NoHost::with_checkpoint(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
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
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(expected)));
    assert_eq!(result.checkpoints.len(), checkpoints);
    let checkpoint = result.checkpoints[selected_checkpoint].clone();
    drop(result);
    let retained =
        etas_interpreter::api::code::checkpoint_source_revisions(&checkpoint, Default::default())
            .unwrap();
    for source in &retained {
        source.validate(Default::default()).unwrap();
        assert!(
            saved.iter().any(|bytes| {
                let expected: SavedCodeRevision = serde_json::from_slice(bytes).unwrap();
                expected == *source
            }),
            "checkpoint exported an unrelated or changed revision"
        );
    }
    let mut artifact = checkpoint_artifact_json(&[], "main", &checkpoint).unwrap();
    artifact["code_sources"] = serde_json::to_value(&retained).unwrap();
    assert!(artifact["checkpoint"].get("retained_code").is_none());
    let bytes =
        etas_interpreter::api::codec::checkpoint_file_to_bytes(artifact, Default::default())
            .unwrap();
    let document =
        etas_interpreter::api::codec::checkpoint_file_from_bytes(&bytes, Default::default())
            .unwrap();
    (saved, std::ops::Deref::deref(&document).clone())
}

#[test]
fn a_captured_agent_exports_restorable_source_after_the_run_result_is_released() {
    let (_, artifact) = save_captured(CAPTURED, &[None], 19);
    let sources = artifact["code_sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    let bytes = sources
        .iter()
        .map(|source| serde_json::to_vec(source).unwrap())
        .collect::<Vec<_>>();
    let (checked, definitions, revisions) = restore_captured(CAPTURED, &bytes);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

pub(super) fn restore_captured(
    program: &str,
    saved: &[Vec<u8>],
) -> (
    etas_frontend::CheckedProgram,
    Arc<NativeDefinitionStore>,
    Arc<etas_interpreter::api::code::PreparedRevisionSet>,
) {
    let (checked, definitions, original) = captured_definitions(program);
    let mut revisions = etas_interpreter::api::code::PreparedRevisionSet::new(checked.clone());
    for bytes in saved {
        let saved: SavedCodeRevision = serde_json::from_slice(bytes).unwrap();
        let draft = definitions
            .restore_source_draft(&original, &saved.source, Default::default())
            .unwrap();
        let revision = compile(definitions.clone(), draft, true).unwrap();
        revision
            .validate_restored_source(&saved, Default::default())
            .unwrap();
        revisions.insert(revision).unwrap();
    }
    (checked, definitions, Arc::new(revisions))
}

pub(super) fn resume_captured(
    checked: &etas_frontend::CheckedProgram,
    definitions: Arc<NativeDefinitionStore>,
    revisions: Arc<etas_interpreter::api::code::PreparedRevisionSet>,
    artifact: &serde_json::Value,
) -> etas_interpreter::api::RunResult {
    use etas_interpreter::{Interpreter, api::code::PreparedCodeDefinitions};
    let checkpoint = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        artifact,
        checked,
        Some(&revisions),
    )
    .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(no_host::NoHost::with_checkpoint(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let result = runtime()
        .block_on(Interpreter.resume_checkpoint(
            checked,
            &checkpoint,
            &host,
            RunOptions {
                code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
                checkpoint_revisions: Some(revisions),
                ..Default::default()
            },
        ))
        .unwrap();
    assert_eq!(host.inner().calls(), 0);
    assert!(
        result.events.iter().all(|event| !matches!(
            event,
            etas_interpreter::api::WorkflowEvent::HostTrace(
                etas_host::TraceEvent::HostRequestStarted {
                    kind: etas_host::HostRequestKind::Compiler,
                    ..
                }
            )
        )),
        "resume must use the already rechecked revision, not re-run the old compile call"
    );
    result
}

#[test]
fn captured_agent_values_require_recompiled_native_revisions_on_checkpoint_restore() {
    use etas_interpreter::api::code::PreparedRevisionSet;
    let (source, artifact) = save_captured(CAPTURED, &[None], 19);
    let (checked, definitions, revisions) = restore_captured(CAPTURED, &source);
    assert!(
        checkpoint_from_json(&artifact, &checked)
            .unwrap_err()
            .message()
            .contains("freshly prepared revisions")
    );
    let empty = Arc::new(PreparedRevisionSet::new(checked.clone()));
    assert!(
        etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &artifact,
            &checked,
            Some(&empty)
        )
        .unwrap_err()
        .message()
        .contains("no freshly prepared revision")
    );
    let checkpoint = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&revisions),
    )
    .unwrap();
    let host = no_host::NoHost::with_checkpoint();
    let denied = runtime()
        .block_on(etas_interpreter::Interpreter.resume_checkpoint(
            &checked,
            &checkpoint,
            &host,
            RunOptions {
                code_definitions: Some(Arc::new(
                    etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions.clone()),
                )),
                ..Default::default()
            },
        ))
        .unwrap();
    assert!(denied.value().is_none());
    assert!(
        denied.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("requires freshly prepared revisions")),
        "{:?}",
        denied.diagnostics
    );
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert!(result.checkpoints.is_empty());
}

const CAPTURED_TWO: &str = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 19; }
flow revise() -> Worker ![Error<CompileError>] {
    let body = match parse_body("return 23;") { Ok(body) => body, Err(_) => { return abort("bad test body"); }, };
    return compile((^Worker).with_body(body));
}
flow main() -> i32 ![Error<CompileError>] {
    let old = compile(^Worker);
    let next = revise();
    checkpoint("both revisions");
    return old() + next();
}
"#;

#[test]
fn captured_old_and_new_agent_revisions_coexist_after_source_recompilation() {
    let (source, artifact) = save_captured(CAPTURED_TWO, &[None, Some("return 23;")], 42);
    let (checked, _, partial) = restore_captured(CAPTURED_TWO, &source[..1]);
    let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        &checked,
        Some(&partial),
    )
    .unwrap_err();
    assert!(
        error.message().contains("no freshly prepared revision"),
        "{error}"
    );
    let (checked, definitions, revisions) = restore_captured(CAPTURED_TWO, &source);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(42)));
    assert!(result.checkpoints.is_empty());
}

#[test]
fn a_captured_callback_can_invoke_an_auxiliary_method_after_restore() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
impl agent Worker { flow extra() -> i32 { return 31; } }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let method = () => selected.extra();
    checkpoint("method");
    return method();
}
"#;
    let (source, artifact) = save_captured(program, &[None], 31);
    let (checked, definitions, revisions) = restore_captured(program, &source);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(31)));
}

#[test]
fn a_generic_named_enum_payload_preserves_its_compiled_agent_reference() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 19; }
enum Holder<T> { Empty, Named { actor: T }, }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    let wrapped = Holder.Named<Worker> { actor = selected };
    checkpoint("named enum");
    return match wrapped { Holder.Named { actor } => actor(), Holder.Empty => 0, };
}
"#;
    let (source, artifact) = save_captured(program, &[None], 19);
    let (checked, definitions, revisions) = restore_captured(program, &source);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
}

#[test]
fn a_candidate_root_restores_a_captured_frozen_original_agent() {
    use etas_interpreter::api::code::PreparedRevisionSet;
    let bytes_and_artifact = {
        let (_, definitions, original) = captured_definitions(CAPTURED);
        let body = definitions
            .syntax_operations()
            .parse_body("let original = Worker; checkpoint(\"frozen\"); return original();")
            .unwrap();
        let draft = definitions.with_body(&original, body).unwrap();
        let revision = compile(definitions, draft, true).unwrap();
        let host = no_host::NoHost::with_checkpoint();
        let result = runtime()
            .block_on(
                revision
                    .create_run(vec![], &host, RunOptions::default())
                    .execute(),
            )
            .unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.value(), Some(&InterpValue::i32(19)));
        assert_eq!(result.checkpoints.len(), 1);
        let source = revision
            .save_checkpoint_source(&result.checkpoints[0], Default::default())
            .unwrap();
        let artifact = checkpoint_artifact_json(&[], "Worker", &result.checkpoints[0]).unwrap();
        (serde_json::to_vec(&source).unwrap(), artifact)
    };
    let (source, artifact) = bytes_and_artifact;
    let saved: SavedCodeRevision = serde_json::from_slice(&source).unwrap();
    let (checked, definitions, original) = captured_definitions(CAPTURED);
    let draft = definitions
        .restore_source_draft(&original, &saved.source, Default::default())
        .unwrap();
    let revision = compile(definitions, draft, true).unwrap();
    revision
        .validate_restored_source(&saved, Default::default())
        .unwrap();
    let mut revisions = PreparedRevisionSet::new(checked);
    revisions.insert(revision.clone()).unwrap();
    let revisions = Arc::new(revisions);
    let checkpoint = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        revision.program(),
        Some(&revisions),
    )
    .unwrap();
    let host = no_host::NoHost::with_checkpoint();
    let result = runtime()
        .block_on(
            revision
                .create_resume(
                    &checkpoint,
                    &host,
                    RunOptions {
                        checkpoint_revisions: Some(revisions),
                        ..Default::default()
                    },
                )
                .execute(),
        )
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert!(result.checkpoints.is_empty());
    assert_eq!(host.calls(), 0);
}

fn edit_agent_reference(
    value: &mut serde_json::Value,
    edit: &mut impl FnMut(&mut serde_json::Value),
) {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some("agent_revision") {
        edit(&mut value["reference"]);
        return;
    }
    match value {
        serde_json::Value::Object(fields) => {
            for value in fields.values_mut() {
                edit_agent_reference(value, edit);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                edit_agent_reference(value, edit);
            }
        }
        _ => {}
    }
}

#[test]
fn captured_agent_references_reject_identity_tampering_and_resource_expansion() {
    let (source, artifact) = save_captured(CAPTURED, &[None], 19);
    let (checked, _, revisions) = restore_captured(CAPTURED, &source);
    for (case, expected) in [
        (0, "differs from its freshly prepared revision"),
        (1, "absent from its prepared revision"),
        (2, "frozen method differs from its declared agent owner"),
        (3, "does not identify its agent entry or ordinary flow"),
        (4, "exceeds byte budget"),
        (5, "exceeds node"),
    ] {
        let mut artifact = artifact.clone();
        let mut edited = 0;
        edit_agent_reference(&mut artifact, &mut |reference| {
            edited += 1;
            match case {
                0 => {
                    reference["revision"]["snapshot_fingerprint"] =
                        serde_json::to_value(CodeFingerprint::of_bytes("bad", b"untrusted"))
                            .unwrap()
                }
                1 => {
                    let target = etas_code::DefinitionKey::new(
                        etas_code::PackageKey::from_canonical_fingerprint(
                            CodeFingerprint::of_bytes("bad", b"foreign"),
                        ),
                        vec!["app".into()],
                        vec!["missing".into()],
                        etas_code::DefinitionKind::Flow,
                    )
                    .unwrap();
                    reference["callable"] =
                        serde_json::to_value(etas_code::CallableId::declaration(target).unwrap())
                            .unwrap();
                }
                2 => reference["frozen_agent"] = serde_json::json!(u32::MAX),
                3 => {
                    reference["revision"]["target"] = serde_json::to_value(
                        etas_code::DefinitionKey::new(
                            etas_code::PackageKey::from_canonical_fingerprint(
                                CodeFingerprint::of_bytes("bad", b"foreign"),
                            ),
                            vec!["app".into()],
                            vec!["Other".into()],
                            etas_code::DefinitionKind::Agent,
                        )
                        .unwrap(),
                    )
                    .unwrap()
                }
                4 => reference["extra"] = serde_json::json!("x".repeat(1024 * 1024 + 1)),
                5 => reference["extra"] = serde_json::json!(vec![("p", "h"); 16_384]),
                _ => unreachable!(),
            }
        });
        assert!(edited > 0);
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &artifact,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        assert!(error.message().contains(expected), "case {case}: {error}");
    }
    let (foreign, _, _) = captured_definitions(CAPTURED);
    let (_, definitions, original) = captured_definitions(CAPTURED);
    let native = compile(definitions, original, true).unwrap();
    let mut set = etas_interpreter::api::code::PreparedRevisionSet::new(foreign);
    assert!(
        set.insert(native)
            .unwrap_err()
            .contains("different checked source universe")
    );
}

#[test]
fn a_queued_revision_call_preserves_the_checked_argument_site_at_a_direct_checkpoint() {
    let result = super::revisions::execute(
        r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker(value: unit) -> i32 { return 19; }
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return selected(checkpoint("pending call"));
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert_eq!(result.checkpoints.len(), 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        etas_interpreter::api::WorkflowEvent::CheckpointCreated(_)
    )));
}
