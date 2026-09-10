use std::path::{Path, PathBuf};

use etas_interpreter::api::{RunResult, codec};

use crate::error::CliError;

pub(crate) fn write_checkpoint_files(
    dir: &Path,
    sources: &[PathBuf],
    flow: &str,
    result: &RunResult,
    runtime_profile: &serde_json::Value,
    lifecycle: Option<&super::lifecycle::CommandLifecycle>,
) -> Result<(), CliError> {
    for checkpoint in &result.checkpoints {
        let mut payload = codec::checkpoint_artifact_json(sources, flow, checkpoint)
            .map_err(|error| CliError::RuntimeState(error.to_string()))?;
        if let serde_json::Value::Object(object) = &mut payload {
            object.insert("runtime_profile".to_owned(), runtime_profile.clone());
        }
        let path = checkpoint_path(dir, codec::checkpoint_id(checkpoint));
        atomic_write_json(&path, &payload, lifecycle)?;
    }
    Ok(())
}

pub fn checkpoint_path(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("checkpoint-{id}.json"))
}

fn atomic_write_json(
    path: &Path,
    payload: &serde_json::Value,
    lifecycle: Option<&super::lifecycle::CommandLifecycle>,
) -> Result<(), CliError> {
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    let bytes = serde_json::to_vec_pretty(payload)
        .map_err(|error| CliError::RuntimeState(error.to_string()))?;
    let owned = path.to_owned();
    let write = move || {
        if let Some(parent) = owned.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, owned)
    };
    let result = match lifecycle {
        Some(owner) => owner.io(path.display().to_string(), true, write),
        None => write(),
    };
    result.map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })
}
