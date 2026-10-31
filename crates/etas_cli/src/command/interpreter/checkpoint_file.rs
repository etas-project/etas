use std::{io::Read, path::Path, sync::Arc};

use etas_host::replay::{ReplayKeyId, ReplayKeyProvider};
use etas_interpreter::api::codec;

use crate::error::CliError;

pub(super) const MAGIC: &[u8; 8] = b"ETASCP01";
pub(super) const HEADER_LEN: usize = 8 + 64 + 12;
const TAG_LEN: usize = 16;

/// Read the sealed CLI file, returning the same provider for verified execution replay.
pub fn read(
    path: &Path,
    key_directory: Option<&Path>,
) -> Result<(codec::CheckpointDocument, Arc<dyn ReplayKeyProvider>), CliError> {
    let limits = codec::CheckpointFileLimits::default();
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| {
            file.take((limits.max_bytes + HEADER_LEN + TAG_LEN + 1) as u64)
                .read_to_end(&mut bytes)
        })
        .map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        })?;
    if bytes.len() > limits.max_bytes + HEADER_LEN + TAG_LEN {
        return Err(CliError::InvalidUsage(
            "checkpoint file exceeds byte limit".into(),
        ));
    }
    if !bytes.starts_with(MAGIC) {
        return Err(CliError::InvalidUsage("unsupported checkpoint file format; recreate legacy plaintext checkpoints with the current CLI".into()));
    }
    if bytes.len() < HEADER_LEN + TAG_LEN {
        return Err(CliError::InvalidUsage(
            "truncated sealed checkpoint file".into(),
        ));
    }
    let id: ReplayKeyId = std::str::from_utf8(&bytes[8..72])
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or_else(|| CliError::InvalidUsage("invalid checkpoint replay-key identifier".into()))?;
    let mut nonce = [0; 12];
    nonce.copy_from_slice(&bytes[72..HEADER_LEN]);
    let keys = super::replay_key_provider(key_directory, true)?;
    let key = keys
        .resolve(id)
        .map_err(|error| CliError::RuntimeState(error.to_string()))?;
    bytes.drain(..HEADER_LEN);
    key.open_checkpoint(nonce, &mut bytes)
        .map_err(|error| CliError::RuntimeState(error.to_string()))?;
    let document = codec::checkpoint_file_from_bytes(&bytes, limits)
        .map_err(|error| CliError::InvalidUsage(error.to_string()))?;
    if document["checkpoint"]["replay_key_id"].as_str() != Some(id.to_hex().as_str()) {
        return Err(CliError::InvalidUsage(
            "checkpoint replay-key identity differs from its sealed file".into(),
        ));
    }
    Ok((document, keys))
}
