use super::revision_checkpoint_values::{restore_captured, resume_captured, save_checkpoint_at};
use etas_interpreter::api::InterpValue;

fn program(pipeline: &str) -> String {
    format!(
        r#"
module app;
import std.compiler.{{compile, CompileError}};
import std.runtime.checkpoint;
import std.runtime.limits.Tokens;
agent Worker(value: i32) -> i32 {{ checkpoint("native stage"); return value; }}
flow main() -> i32 {{
    let selected = compile(^Worker);
    let value = {pipeline};
    checkpoint("caller");
    return value;
}}
"#
    )
}

#[test]
fn a_checkpoint_in_a_compiled_limited_agent_restores_the_policy_and_then_leaves_its_scope() {
    let program = program("19 ~> selected limit Tokens(23)");
    let (saved, artifact) = save_checkpoint_at(&program, &[None], 19, 2, 0);
    assert_eq!(
        artifact["checkpoint"]["model_policy"]["options"]["max_output_tokens"],
        23
    );
    let (checked, definitions, revisions) = restore_captured(&program, &saved);
    let result = resume_captured(&checked, definitions, revisions, &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert_eq!(result.checkpoints.len(), 1);
    let caller =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    assert!(caller["checkpoint"]["model_policy"]["options"]["max_output_tokens"].is_null());
}

fn find_kind<'a>(
    value: &'a mut serde_json::Value,
    kind: &str,
) -> Option<&'a mut serde_json::Value> {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some(kind) {
        return Some(value);
    }
    match value {
        serde_json::Value::Object(fields) => {
            fields.values_mut().find_map(|value| find_kind(value, kind))
        }
        serde_json::Value::Array(values) => {
            values.iter_mut().find_map(|value| find_kind(value, kind))
        }
        _ => None,
    }
}

#[test]
fn remaining_native_pipeline_stages_keep_their_checked_invocation_cursor_and_limits() {
    let program = program("19 ~> selected limit Tokens(23) ~> selected limit Tokens(11)");
    let (saved, artifact) = save_checkpoint_at(&program, &[None], 19, 3, 0);
    let (checked, definitions, revisions) = restore_captured(&program, &saved);
    let result = resume_captured(&checked, definitions, revisions.clone(), &artifact);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(19)));
    assert_eq!(result.checkpoints.len(), 2);
    let stage =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    assert_eq!(
        stage["checkpoint"]["model_policy"]["options"]["max_output_tokens"],
        11
    );
    for damage in ["owner", "index", "frame", "suffix"] {
        let mut damaged = artifact.clone();
        let machine = &mut damaged["checkpoint"]["machine"];
        if damage == "suffix" {
            find_kind(machine, "composed_call").unwrap()["remaining"] = serde_json::json!([]);
        } else {
            let boundary = find_kind(machine, "program_return").unwrap();
            match damage {
                "owner" => boundary["call_site"]["origin"]["expr"] = serde_json::json!(u32::MAX),
                "index" => boundary["call_site"]["origin"]["index"] = serde_json::json!(2),
                "frame" => {
                    boundary["call_site"]
                        .as_object_mut()
                        .unwrap()
                        .remove("frame");
                }
                _ => unreachable!(),
            }
        }
        let error = etas_interpreter::api::codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &damaged,
            &checked,
            Some(&revisions),
        )
        .unwrap_err();
        let expected = match damage {
            "owner" => "checked owner",
            "index" => "out of bounds",
            "frame" => "frame",
            "suffix" => "remaining suffix",
            _ => unreachable!(),
        };
        assert!(error.message().contains(expected), "{damage}: {error}");
    }
}
