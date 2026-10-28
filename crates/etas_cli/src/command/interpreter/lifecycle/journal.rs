use etas_interpreter::api::{RunEventObserver, WorkflowEvent, codec};
use std::sync::Mutex;

/// Session-local journal; never captures live frames, authority or raw requests.
#[derive(Debug, Default)]
pub(crate) struct EventJournal(Mutex<Vec<codec::JsonDocument>>);

impl RunEventObserver for EventJournal {
    fn observe(&self, event: &WorkflowEvent) {
        if let Ok(mut events) = self.0.lock() {
            events.push(codec::event_json(event).into());
        }
    }
}

impl EventJournal {
    pub(crate) fn snapshot(&self) -> Result<Vec<codec::JsonDocument>, crate::error::CliError> {
        self.0.lock().map(|events| events.clone()).map_err(|_| {
            crate::error::CliError::RuntimeState("execution event journal poisoned".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use etas_interpreter::api::InterpValue;

    #[test]
    fn journal_snapshot_and_release_preserve_deep_event_payloads() {
        let mut value = InterpValue::Bool(true);
        for _ in 0..20_000 {
            value = InterpValue::OptionSome(value.into());
        }
        let journal = EventJournal::default();
        let event = WorkflowEvent::MessageCreated {
            id: "event".into(),
            from: None,
            to: None,
            session: None,
            role: "user".into(),
            created_at: "step-1".into(),
            payload: Box::new(value),
            provenance: None,
        };
        journal.observe(&event);
        let first = journal.snapshot().unwrap();
        journal.observe(&event);
        let second = journal.snapshot().unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 2);
        assert_eq!(first[0], second[0]);
        assert_eq!(first[0].to_string().matches("option_some").count(), 20_000);
    }
}
