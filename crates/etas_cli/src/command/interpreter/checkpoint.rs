use std::path::{Path, PathBuf};

use etas_host::replay::ReplayKeyProvider;
use etas_interpreter::api::{RunResult, codec};

use crate::error::CliError;

pub(crate) const CLI_CHECKPOINT_SCHEMA: &str = "etas.cli.checkpoint.v1";

pub(crate) struct CheckpointFiles<'a> {
    pub directory: &'a Path,
    pub keys: &'a dyn ReplayKeyProvider,
    pub lifecycle: Option<&'a super::lifecycle::CommandLifecycle>,
}

impl CheckpointFiles<'_> {
    pub(crate) fn write(
        &self,
        sources: &[PathBuf],
        all: bool,
        flow: &str,
        result: &RunResult,
        runtime_profile: &serde_json::Value,
    ) -> Result<(), CliError> {
        for checkpoint in &result.checkpoints {
            let mut payload = codec::checkpoint_artifact_json(sources, flow, checkpoint)
                .map_err(|error| CliError::RuntimeState(error.to_string()))?;
            let code_sources = etas_interpreter::api::code::checkpoint_source_revisions(
                checkpoint,
                Default::default(),
            )
            .map_err(|error| {
                CliError::RuntimeState(format!("checkpoint source export failed: {error}"))
            })?;
            let object = payload.as_object_mut().ok_or_else(|| {
                CliError::RuntimeState("checkpoint codec produced a non-object artifact".to_owned())
            })?;
            object.insert("cli_schema".to_owned(), CLI_CHECKPOINT_SCHEMA.into());
            object.insert("source_load_all".to_owned(), all.into());
            object.insert("runtime_profile".to_owned(), runtime_profile.clone());
            object.insert(
                "code_sources".to_owned(),
                serde_json::to_value(code_sources).map_err(|error| {
                    CliError::RuntimeState(format!("checkpoint source encoding failed: {error}"))
                })?,
            );
            let path = checkpoint_path(self.directory, codec::checkpoint_id(checkpoint));
            let mut bytes =
                codec::checkpoint_file_to_bytes(payload, codec::CheckpointFileLimits::default())
                    .map_err(|error| CliError::RuntimeState(error.to_string()))?;
            let key = self
                .keys
                .resolve(checkpoint.replay_key_id)
                .map_err(|error| CliError::RuntimeState(error.to_string()))?;
            let nonce = key
                .seal_checkpoint(&mut bytes)
                .map_err(|error| CliError::RuntimeState(error.to_string()))?;
            let mut file = Vec::with_capacity(bytes.len() + super::checkpoint_file::HEADER_LEN);
            file.extend_from_slice(super::checkpoint_file::MAGIC);
            file.extend_from_slice(key.id().to_hex().as_bytes());
            file.extend_from_slice(&nonce);
            file.append(&mut bytes);
            atomic_write_bytes(&path, file, self.lifecycle)?;
        }
        Ok(())
    }
}

pub fn checkpoint_path(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("checkpoint-{id}.json"))
}

fn atomic_write_bytes(
    path: &Path,
    bytes: Vec<u8>,
    lifecycle: Option<&super::lifecycle::CommandLifecycle>,
) -> Result<(), CliError> {
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
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
