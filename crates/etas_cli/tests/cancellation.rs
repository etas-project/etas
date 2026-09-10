#![cfg(all(unix, feature = "cli-run"))]

#[path = "cancellation/approval.rs"]
mod approval;

use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(body: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-cancellation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("main.es"),
            format!(
                r#"
module tests.cancellation;
import std.io.{{println, read_line}};
import std.runtime.limits.Iterations;
flow main(args: Array<string>) -> i32 ![Console] {{
    println("ready")?;
    {body}
    println("fallback")?;
    return 0;
}}
"#
            ),
        )
        .unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn cancelled_process(body: &str, signal: &str, expected: i32) {
    let fixture = Fixture::new(body);
    let mut child = Process(
        Command::new(env!("CARGO_BIN_EXE_etas"))
            .args(["--format", "json", "run"])
            .arg(fixture.0.join("main.es"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (ready, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        ready.send(line).unwrap();
        let mut rest = String::new();
        reader.read_to_string(&mut rest).unwrap();
        rest
    });
    let first = receiver
        .recv_timeout(Duration::from_secs(15))
        .expect("run must reach ready before signal");
    assert_eq!(first, "ready\n", "{first}");
    assert!(
        Command::new("kill")
            .args([signal, &child.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "cancelled process did not exit");
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = reader.join().unwrap();
    let mut stderr = String::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert_eq!(status.code(), Some(expected), "{output}\n{stderr}");
    assert!(!output.contains("fallback"), "{output}");
    let report: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(report["outcome"]["kind"], "cancelled", "{report}");
    assert_eq!(
        report["termination"]["local_work_settled"], true,
        "{report}"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn sigint_stops_cpu_only_loop_on_single_thread_executor() {
    cancelled_process("while true limit Iterations(1000000000) {}", "-INT", 130);
}

#[test]
fn sigterm_stops_stdin_wait_and_cannot_be_caught_by_postfix_try() {
    cancelled_process("let input = read_line()?;", "-TERM", 143);
}

fn send_signal(child: &Process, signal: &str) {
    assert!(
        Command::new("kill")
            .args([signal, &child.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
}

fn wait_exit(child: &mut Process) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(9);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "process hung during shutdown/output"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn blocked_output(second_signal: bool, host_write: bool) {
    let text = "x".repeat(1024 * 1024);
    let fixture = Fixture::new(&format!("println(\"{text}\")?;"));
    if !host_write {
        std::fs::write(fixture.0.join("main.es"), format!(
            "module tests.cancellation; import std.runtime.checkpoint; flow main(args: Array<string>) -> i32 {{ checkpoint(\"{text}\"); return 0; }}"
        )).unwrap();
    }
    let trace = fixture.0.join("trace.json");
    let profile = fixture.0.join("profile.json");
    let mut child = Process(
        Command::new(env!("CARGO_BIN_EXE_etas"))
            .args(["--format", "json", "--profile-out"])
            .arg(&profile)
            .arg("run")
            .arg(fixture.0.join("main.es"))
            .arg("--trace-out")
            .arg(&trace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        if host_write {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(line, "ready\n");
        }
        let mut first_byte = [0];
        reader.read_exact(&mut first_byte).unwrap();
        sender.send(reader).unwrap();
    });
    // Keep the pipe open but undrained: the real OS write must remain pending.
    let _blocked_pipe = receiver.recv_timeout(Duration::from_secs(30)).unwrap();
    reader.join().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    send_signal(&child, "-INT");
    if second_signal {
        std::thread::sleep(Duration::from_millis(100));
        send_signal(&child, "-TERM");
    }
    let status = wait_exit(&mut child);
    assert_eq!(status.code(), Some(130));
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(trace).unwrap()).unwrap();
    assert_eq!(report["incomplete"], true, "{report}");
    if host_write {
        assert!(report["termination"].is_null());
        assert_eq!(report["shutdown"], "pending");
        assert!(!report["pending_operations"].as_array().unwrap().is_empty());
        assert!(!report["events"].as_array().unwrap().is_empty());
        assert_eq!(
            report["trigger"],
            if second_signal {
                "second_signal"
            } else {
                "grace_expired"
            }
        );
    } else {
        // A signal in output must not rewrite the evaluator's published terminal outcome.
        assert_eq!(report["outcome"]["kind"], "completed");
        assert_eq!(report["output_shutdown"]["trigger"], "second_signal");
    }
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(profile).unwrap()).unwrap();
    assert_eq!(report["status"], "forced");
    if host_write {
        assert!(
            report["spans"]
                .as_array()
                .unwrap()
                .iter()
                .any(|span| span["name"] == "interpreter.eval" && span["status"] == "abandoned")
        );
    }
    assert!(
        report["spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["name"] == "cli.run.total" && span["status"] == "forced")
    );
}

#[test]
fn second_signal_exits_blocked_host_stdout_and_saves_incomplete_trace_profile() {
    blocked_output(true, true);
}

#[test]
fn real_host_cleanup_timeout_exits_without_waiting_for_blocked_stdout() {
    blocked_output(false, true);
}

#[test]
fn signals_remain_live_during_blocked_terminal_report() {
    blocked_output(true, false);
}

#[cfg(feature = "cli-resume")]
#[test]
fn resume_stdin_cancellation_uses_shared_profile_and_trace_finalizer() {
    let fixture = Fixture::new("");
    std::fs::write(
        fixture.0.join("main.es"),
        r#"
module tests.cancellation;
import std.io.{println, read_line};
import std.runtime.checkpoint;
flow main(args: Array<string>) -> i32 ![Console] {
    checkpoint("before-input");
    println("ready")?;
    let input = read_line()?;
    return 0;
}
"#,
    )
    .unwrap();
    let checkpoints = fixture.0.join("checkpoints");
    let mut initial = Process(
        Command::new(env!("CARGO_BIN_EXE_etas"))
            .arg("run")
            .arg(fixture.0.join("main.es"))
            .arg("--checkpoint-dir")
            .arg(&checkpoints)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    initial
        .0
        .stdin
        .take()
        .unwrap()
        .write_all(b"first\n")
        .unwrap();
    let status = wait_exit(&mut initial);
    let mut error = String::new();
    initial
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut error)
        .unwrap();
    assert!(status.success(), "{error}");
    let trace = fixture.0.join("resume-trace.json");
    let profile = fixture.0.join("resume-profile.json");
    let mut child = Process(
        Command::new(env!("CARGO_BIN_EXE_etas"))
            .args(["--format", "json", "--profile-out"])
            .arg(&profile)
            .args(["resume", "0", "--checkpoint-dir"])
            .arg(checkpoints)
            .arg("--trace-out")
            .arg(&trace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (ready, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        ready.send(line).unwrap();
        let mut rest = String::new();
        reader.read_to_string(&mut rest).unwrap();
        rest
    });
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(20)).unwrap(),
        "ready\n"
    );
    send_signal(&child, "-TERM");
    assert_eq!(wait_exit(&mut child).code(), Some(143));
    let report: serde_json::Value = serde_json::from_str(&reader.join().unwrap()).unwrap();
    assert_eq!(report["outcome"]["kind"], "cancelled");
    let trace: serde_json::Value = serde_json::from_slice(&std::fs::read(trace).unwrap()).unwrap();
    assert_eq!(trace["outcome"]["kind"], "cancelled");
    let profile: serde_json::Value =
        serde_json::from_slice(&std::fs::read(profile).unwrap()).unwrap();
    assert_eq!(profile["status"], "cancelled");
    assert!(
        profile["spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["name"] == "interpreter.eval" && span["status"] == "cancelled")
    );
    assert!(
        profile["spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["category"] == "interpreter")
    );
    assert!(
        profile["spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["category"] == "frontend")
    );
}
