use std::path::{Path, PathBuf};

use etas_interpreter::api::{RunResult, codec};

use crate::error::CliError;

pub fn write_checkpoint_files(
    dir: &Path,
    sources: &[PathBuf],
    flow: &str,
    result: &RunResult,
    runtime_profile: &serde_json::Value,
) -> Result<(), CliError> {
    std::fs::create_dir_all(dir).map_err(|source| CliError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for checkpoint in &result.checkpoints {
        let mut payload = codec::checkpoint_artifact_json(sources, flow, checkpoint);
        if let serde_json::Value::Object(object) = &mut payload {
            object.insert("runtime_profile".to_owned(), runtime_profile.clone());
        }
        let path = checkpoint_path(dir, codec::checkpoint_id(checkpoint));
        atomic_write_json(&path, &payload)?;
    }
    Ok(())
}

pub fn checkpoint_path(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("checkpoint-{id}.json"))
}

fn atomic_write_json(path: &Path, payload: &serde_json::Value) -> Result<(), CliError> {
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    std::fs::write(
        &tmp,
        serde_json::to_vec_pretty(payload).expect("json serializes"),
    )
    .map_err(|source| CliError::Io {
        path: tmp.clone(),
        source,
    })?;
    std::fs::rename(&tmp, path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}
