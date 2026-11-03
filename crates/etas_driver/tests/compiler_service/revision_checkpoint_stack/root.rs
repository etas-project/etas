use std::sync::Arc;

use etas_interpreter::api::codec::{checkpoint_artifact_json, checkpoint_from_json_with_revisions};
use etas_interpreter::api::{
    InterpValue, RunOptions,
    code::{PreparedRevisionSet, SavedCodeRevision},
};

use super::super::{no_host::NoHost, revision_checkpoint::compile, runtime, source};

#[test]
fn a_candidate_root_preserves_its_revision_while_a_frozen_agent_call_is_active() {
    let (saved, artifact) = {
        let (checked, definitions) = source::prepared(super::NESTED);
        let expression = *checked.types.reflections.keys().next().unwrap();
        let definition = definitions.reflect_agent(expression).unwrap();
        let revision = compile(definitions, definition, true).unwrap();
        let host = NoHost::with_checkpoint();
        let result = runtime()
            .block_on(
                revision
                    .create_run(vec![InterpValue::i32(1)], &host, RunOptions::default())
                    .execute(),
            )
            .unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.value(), Some(&InterpValue::i32(19)));
        assert_eq!(result.checkpoints.len(), 1);
        let saved = revision
            .save_checkpoint_source(&result.checkpoints[0], Default::default())
            .unwrap();
        let artifact = checkpoint_artifact_json(&[], "Worker", &result.checkpoints[0]).unwrap();
        assert!(!artifact["checkpoint"]["code_revision"].is_null());
        (serde_json::to_vec(&saved).unwrap(), artifact)
    };
    let saved: SavedCodeRevision = serde_json::from_slice(&saved).unwrap();
    let (checked, definitions) = source::prepared(super::NESTED);
    let expression = *checked.types.reflections.keys().next().unwrap();
    let original = definitions.reflect_agent(expression).unwrap();
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
    let checkpoint = checkpoint_from_json_with_revisions(
        &Default::default(),
        &artifact,
        revision.program(),
        Some(&revisions),
    )
    .unwrap();
    let host = NoHost::with_checkpoint();
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

#[test]
fn a_resumed_candidate_error_unwinds_into_the_original_program_handler() {
    let program = r#"
module app;
import std.runtime.checkpoint;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![Error<i32>] {
    checkpoint("before error");
    return perform Error<i32>.raise(19);
}
flow main() -> i32 ![Error<CompileError>] {
    let selected = compile(^Worker);
    return handle {
        return selected();
    } with { Error<i32>.raise(code) => { finish code + 1; } };
}
"#;
    let (sources, artifact) = super::save_captured(program, &[None], 20);
    let (checked, definitions, revisions) = super::restore_captured(program, &sources);
    let result = super::resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(20)));
}
