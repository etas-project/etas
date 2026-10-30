use crate::error::CliError;
use etas_host::replay::{
    DirectoryReplayKeyProvider, ReplayKeyProvider, default_replay_key_provider,
};
use std::{path::Path, sync::Arc};

pub(crate) fn replay_key_provider(
    explicit: Option<&Path>,
    durable: bool,
) -> Result<Arc<dyn ReplayKeyProvider>, CliError> {
    if !durable && explicit.is_none() {
        return Ok(default_replay_key_provider());
    }
    let path = super::host_config::checkpoint_key_directory(explicit)?;
    DirectoryReplayKeyProvider::open(&path)
        .map(|provider| Arc::new(provider) as Arc<dyn ReplayKeyProvider>)
        .map_err(|error| CliError::InvalidUsage(format!("checkpoint replay-key provider: {error}")))
}
