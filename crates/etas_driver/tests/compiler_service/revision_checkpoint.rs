use super::*;
use etas_interpreter::api::codec::{
    CheckpointDocument, checkpoint_artifact_json, checkpoint_from_json,
};
use etas_interpreter::api::{
    InterpValue, RunOptions,
    code::{PreparedCodeRevision, SavedCodeRevision},
};

const PROGRAM: &str = r#"
module app;
import std.runtime.checkpoint;
agent Worker() -> i32 { checkpoint("base"); return 1; }
flow inspect() -> unit { let definition = ^Worker; return; }
flow main() -> unit { return; }
"#;

fn fresh_definitions() -> (Arc<NativeDefinitionStore>, DefinitionRef) {
    let (checked, definitions) = source::prepared(PROGRAM);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
    (definitions, original)
}

pub(super) fn compile(
    definitions: Arc<NativeDefinitionStore>,
    definition: DefinitionRef,
    allowed: bool,
) -> Result<Arc<PreparedCodeRevision>, HostError> {
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let mut invocation = request(definition.clone(), CompilerOperation::Compile);
    if !allowed {
        invocation.authority = AuthorityContext::deny_all();
    }
    let expected = invocation.clone();
    let response = runtime().block_on(execute(&service, invocation))?;
    validate_compiler_response(&expected, &response, Default::default())?;
    let reference = match response.result {
        Ok(CompilerResult::Compiled(reference)) => reference,
        other => panic!("candidate compilation: {other:?}"),
    };
    InterpreterCandidatePublisher
        .publish_checked_reference(&service, &reference, definitions, &definition)
        .map_err(|error| HostError::new(HostErrorCode::InvalidResponse, error.to_string()))
}

fn save(body: &str) -> (Vec<u8>, CheckpointDocument) {
    let (definitions, original) = fresh_definitions();
    let body = definitions.syntax_operations().parse_body(body).unwrap();
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
    assert_eq!(result.checkpoints.len(), 1);
    assert_eq!(host.calls(), 0);
    let checkpoint = &result.checkpoints[0];
    let source = revision
        .save_checkpoint_source(checkpoint, Default::default())
        .unwrap();
    let artifact = checkpoint_artifact_json(&[], "Worker", checkpoint).unwrap();
    (serde_json::to_vec(&source).unwrap(), artifact)
}

fn restore(bytes: &[u8], allowed: bool) -> Result<Arc<PreparedCodeRevision>, HostError> {
    let saved: SavedCodeRevision = serde_json::from_slice(bytes).unwrap();
    saved.validate(Default::default()).unwrap();
    let (definitions, original) = fresh_definitions();
    let draft = definitions
        .restore_source_draft(&original, &saved.source, Default::default())
        .unwrap();
    let revision = compile(definitions, draft, allowed)?;
    revision
        .validate_restored_source(&saved, Default::default())
        .map_err(|error| HostError::new(HostErrorCode::InvalidResponse, error.to_string()))?;
    Ok(revision)
}

fn decode_for_revision(
    artifact: &CheckpointDocument,
    revision: &Arc<PreparedCodeRevision>,
) -> etas_interpreter::api::InterpreterCheckpoint {
    let mut set = etas_interpreter::api::code::PreparedRevisionSet::new(
        revision.type_bridge().original_program().clone(),
    );
    set.insert(revision.clone()).unwrap();
    etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
        &Default::default(),
        artifact,
        revision.program(),
        Some(&Arc::new(set)),
    )
    .unwrap()
}

#[test]
fn a_checkpoint_from_a_candidate_root_recompiles_saved_source_before_resume() {
    let (saved, artifact) = save("let before = 9; checkpoint(\"candidate\"); return before;");
    let revision = restore(&saved, true).unwrap();
    assert!(
        checkpoint_from_json(&artifact, revision.program())
            .unwrap_err()
            .message()
            .contains("requires freshly prepared revisions")
    );
    let checkpoint = decode_for_revision(&artifact, &revision);
    let host = no_host::NoHost::with_checkpoint();
    let result = runtime()
        .block_on(
            revision
                .create_resume(&checkpoint, &host, RunOptions::default())
                .execute(),
        )
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(9)));
    assert!(
        result.checkpoints.is_empty(),
        "resume must not execute the body before the saved continuation again"
    );
    assert_eq!(host.calls(), 0);
}

#[test]
fn a_candidate_checkpoint_cannot_restore_previous_compiler_authority() {
    let (saved, _) = save("checkpoint(\"candidate\"); return 9;");
    let error = restore(&saved, false).unwrap_err();
    assert_eq!(error.code, HostErrorCode::AuthorityDenied);
}

#[test]
fn a_candidate_checkpoint_rejects_a_different_freshly_compiled_body() {
    let (saved, artifact) = save("checkpoint(\"candidate\"); return 9;");
    let (definitions, original) = fresh_definitions();
    let body = definitions
        .syntax_operations()
        .parse_body("checkpoint(\"candidate\"); return 10;")
        .unwrap();
    let draft = definitions.with_body(&original, body).unwrap();
    let different = compile(definitions, draft, true).unwrap();
    let saved: SavedCodeRevision = serde_json::from_slice(&saved).unwrap();
    assert!(matches!(
        different.validate_restored_source(&saved, Default::default()),
        Err(etas_interpreter::api::code::RevisionCheckpointError::RevisionMismatch)
    ));
    assert!(checkpoint_from_json(&artifact, different.program()).is_err());
    let (_, base) = fresh_definitions();
    assert_ne!(base.revision().key(), saved.source.revision_key);
}

#[test]
fn a_candidate_checkpoint_validates_both_source_and_checked_snapshot_claims() {
    let (saved, _) = save("checkpoint(\"candidate\"); return 9;");
    let revision = restore(&saved, true).unwrap();
    let mut corrupted: SavedCodeRevision = serde_json::from_slice(&saved).unwrap();
    corrupted.snapshot_fingerprint = CodeFingerprint::of_bytes("bad", b"not checked");
    assert!(matches!(
        revision.validate_restored_source(&corrupted, Default::default()),
        Err(etas_interpreter::api::code::RevisionCheckpointError::SnapshotMismatch)
    ));
    corrupted.source.source.text = "return false;".into();
    assert!(matches!(
        revision.validate_restored_source(&corrupted, Default::default()),
        Err(etas_interpreter::api::code::RevisionCheckpointError::InvalidSource(_))
    ));
}

#[test]
fn a_candidate_checkpoint_cannot_drop_the_prepared_revision_execution_binding() {
    let (saved, artifact) = save("checkpoint(\"candidate\"); return 9;");
    let revision = restore(&saved, true).unwrap();
    let checkpoint = decode_for_revision(&artifact, &revision);
    let host = no_host::NoHost::with_checkpoint();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.resume_checkpoint(
            revision.program(),
            &checkpoint,
            &host,
            RunOptions::default(),
        ))
        .unwrap();
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("exact freshly prepared code revision")),
        "{:?}",
        result.diagnostics
    );
    let mut stripped = artifact.clone();
    stripped["checkpoint"]["code_revision"] = serde_json::Value::Null;
    assert!(checkpoint_from_json(&stripped, revision.program()).is_err());
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            revision.program(),
            revision.entry(),
            vec![],
            &host,
            RunOptions::default(),
        ))
        .unwrap();
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("requires its prepared code revision")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(host.calls(), 0);
}

#[test]
fn equal_body_text_does_not_allow_resume_under_a_different_parent_revision() {
    let (saved, artifact) = save("checkpoint(\"candidate\"); return 9;");
    let saved: SavedCodeRevision = serde_json::from_slice(&saved).unwrap();
    let (definitions, original) = fresh_definitions();
    let first = definitions
        .restore_source_draft(&original, &saved.source, Default::default())
        .unwrap();
    let prepared_first = compile(definitions.clone(), first.clone(), true).unwrap();
    let body = definitions
        .syntax_operations()
        .parse_body(&saved.source.source.text)
        .unwrap();
    let next = definitions.with_body(&first, body).unwrap();
    let next = compile(definitions, next, true).unwrap();
    assert_eq!(
        next.revision().inputs().content,
        saved.source.revision.content
    );
    assert_ne!(next.revision().key(), saved.source.revision_key);
    let checkpoint = decode_for_revision(&artifact, &prepared_first);
    let host = no_host::NoHost::with_checkpoint();
    let result = runtime()
        .block_on(
            next.create_resume(&checkpoint, &host, RunOptions::default())
                .execute(),
        )
        .unwrap();
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("exact freshly prepared code revision")),
        "{:?}",
        result.diagnostics
    );
}
