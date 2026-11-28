use crate::error::CliError;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Deserialize)]
struct Recording {
    trace: String,
    id: u64,
    action: String,
    session: String,
    phase: String,
    bytes: Vec<u8>,
}

struct Expected {
    event: usize,
    session: String,
    action: &'static str,
    phase: &'static str,
    length: usize,
}

fn unavailable() -> CliError {
    CliError::InvalidUsage(
        "PTY replay unavailable: a matching protected input/output recording is required".into(),
    )
}

fn payload<'a>(start: &'a Value, name: &str) -> Option<&'a str> {
    start["payload"]
        .as_array()?
        .iter()
        .find(|field| field["name"] == name)?["value"]["value"]
        .as_str()
}

pub(super) fn restore(report: &mut Value, recording: Option<&Path>) -> Result<(), CliError> {
    let Some(events) = report["events"].as_array_mut() else {
        return Ok(());
    };
    let mut reads = HashMap::new();
    let mut expected = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        if event["kind"] == "host_request_started" && event["subject_kind"] == "pty" {
            let id = event["id"].as_u64().ok_or_else(unavailable)?;
            let trace = event["trace"]["trace_id"]
                .as_str()
                .ok_or_else(unavailable)?;
            match event["qualified_action"].as_str() {
                Some("Command.write_pty") => {
                    let length = payload(event, "bytes")
                        .and_then(|s| s.parse().ok())
                        .ok_or_else(unavailable)?;
                    let session = payload(event, "session")
                        .ok_or_else(unavailable)?
                        .to_owned();
                    expected.insert(
                        (trace.to_owned(), id),
                        Expected {
                            event: index,
                            session,
                            length,
                            action: "Command.write_pty",
                            phase: "submitted",
                        },
                    );
                }
                Some("Command.read_pty") => {
                    reads.insert(id, trace.to_owned());
                }
                _ => {}
            }
        } else if event["kind"] == "host_request_finished" && event["pty"]["result"] == "output" {
            let id = event["id"].as_u64().ok_or_else(unavailable)?;
            let trace = reads.remove(&id).ok_or_else(unavailable)?;
            let length = event["pty"]["bytes"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(unavailable)?;
            let session = event["pty"]["session"]
                .as_str()
                .ok_or_else(unavailable)?
                .to_owned();
            expected.insert(
                (trace, id),
                Expected {
                    event: index,
                    session,
                    length,
                    action: "Command.read_pty",
                    phase: "returned",
                },
            );
        }
    }
    if let Some(path) = recording {
        let file = File::open(path).map_err(|_| unavailable())?;
        for line in BufReader::new(file).lines() {
            let record: Recording = serde_json::from_str(&line.map_err(|_| unavailable())?)
                .map_err(|_| unavailable())?;
            let entry = expected
                .remove(&(record.trace, record.id))
                .ok_or_else(unavailable)?;
            if entry.session != record.session
                || entry.action != record.action
                || entry.phase != record.phase
                || entry.length != record.bytes.len()
            {
                return Err(unavailable());
            }
            events[entry.event]["recorded_bytes"] = Value::from(record.bytes);
        }
    }
    if expected.values().any(|entry| entry.length != 0) {
        return Err(unavailable());
    }
    Ok(())
}
