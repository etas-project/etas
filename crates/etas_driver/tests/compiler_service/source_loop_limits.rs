use super::*;
use etas_interpreter::{
    Interpreter,
    api::{EntryPoint, InterpValue, RunOptions},
};

#[tokio::test(flavor = "current_thread")]
async fn dynamic_loop_budget_is_evaluated_once_and_survives_checkpoint_resume() {
    let output = Frontend.check_project(ProjectInput::single_source(SourceInput::anonymous(
        r#"
module app;
import std.runtime.checkpoint;
flow budget(value: usize) -> usize ![] {
    checkpoint("evaluating loop budget");
    return value;
}
flow main() -> i32 ![] {
    let config = { maximum= 3 };
    var rounds: i32 = 0;
    while true limit Iterations(budget(config.maximum)) {
        rounds = rounds + 1;
    }
    let zero: usize = 0;
    for value in [1, 2] limit Iterations(zero) {
        rounds = rounds + value;
    }
    return rounds;
}
"#,
    )));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let host = no_host::NoHost::with_checkpoint();
    let result = Interpreter
        .run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            Vec::new(),
            &host,
            RunOptions::default(),
        )
        .await
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(3)));
    assert_eq!(result.checkpoints.len(), 1);
    let artifact =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    let bytes =
        etas_interpreter::api::codec::checkpoint_file_to_bytes(artifact, Default::default())
            .unwrap();
    let document =
        etas_interpreter::api::codec::checkpoint_file_from_bytes(&bytes, Default::default())
            .unwrap();
    let checkpoint =
        etas_interpreter::api::codec::checkpoint_from_json(&document, &checked).unwrap();
    let resumed = Interpreter
        .resume_checkpoint(&checked, &checkpoint, &host, RunOptions::default())
        .await
        .unwrap();
    assert!(resumed.diagnostics.is_empty(), "{:?}", resumed.diagnostics);
    assert_eq!(resumed.value(), Some(&InterpValue::i32(3)));
    assert!(resumed.checkpoints.is_empty());
    assert_eq!(host.calls(), 0);
}
