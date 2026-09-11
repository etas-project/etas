use etas_interpreter::api::{RunEventObserver, WorkflowEvent, codec};
use std::sync::Mutex;

/// Session-local journal; never captures live frames, authority or raw requests.
#[derive(Debug, Default)]
pub(crate) struct EventJournal(Mutex<Vec<serde_json::Value>>);

impl RunEventObserver for EventJournal {
    fn observe(&self, event: &WorkflowEvent) {
        if let Ok(mut events) = self.0.lock() {
            events.push(codec::event_json(event));
        }
    }
}

impl EventJournal {
    pub(crate) fn snapshot(&self) -> Result<Vec<serde_json::Value>, crate::error::CliError> {
        self.0.lock().map(|events| events.clone()).map_err(|_| {
            crate::error::CliError::RuntimeState("execution event journal poisoned".into())
        })
    }
}
