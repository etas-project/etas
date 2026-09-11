use super::*;

#[test]
fn cancelled_approval_input_leaves_next_console_line_and_eof_intact() {
    const CHILD: &str = "ETAS_TEST_APPROVAL_INPUT_CHILD";
    if std::env::var_os(CHILD).is_some() {
        use etas_host::{
            AuthorityContext, ExecutionBudget, HostActionGrant, HostErrorCode, HostRequestId,
            SandboxPolicy, TraceContext, TraceId,
            console::{
                ConsoleClient, ConsoleOperation, ConsoleRequest, ConsoleResult, LocalStdioClient,
            },
            execution::{CancellationReason, ExecutionScope, ExternalOutcome},
        };
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
            let client = LocalStdioClient::new();
            let scope = ExecutionScope::new_owned();
            let approval = scope.register(None, Some(HostRequestId(1)), TraceContext::root(TraceId(1))).unwrap();
            approval.begin_dispatch().unwrap();
            let error = {
                let prompt = client.read_prompt_line_scoped(approval.context());
                tokio::pin!(prompt);
                tokio::select! {
                    biased;
                    result = &mut prompt => panic!("approval unexpectedly received input: {result:?}"),
                    _ = tokio::task::yield_now() => {}
                }
                println!("approval-pending");
                signal.recv().await.unwrap();
                scope.cancel_source().stop(CancellationReason::Terminate).unwrap();
                let error = prompt.await.unwrap_err();
                assert_eq!(error.code, HostErrorCode::Cancelled);
                error
            };
            approval.complete(ExternalOutcome::Failed(error), vec![]).unwrap();
            scope.finish_body(true).unwrap();
            assert!(scope.termination().unwrap().is_some());
            println!("console-pending");
            for (id, expected) in [(2, "next-console-line\n"), (3, "")] {
                let response = client.execute(ConsoleRequest {
                    id: HostRequestId(id), operation: ConsoleOperation::ReadLineStdin,
                    authority: AuthorityContext {
                        grants: vec![HostActionGrant::allow("Console", "stdin_read_line")],
                        approvals: vec![], sandbox: SandboxPolicy::deny_all(), policy: Default::default(),
                    }, trace: TraceContext::root(TraceId(2)), budget: ExecutionBudget::default(),
                }).await.unwrap();
                assert_eq!(response.result, ConsoleResult::Input(expected.into()));
            }
        });
        return;
    }
    let mut child = Process(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "approval::cancelled_approval_input_leaves_next_console_line_and_eof_intact",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            sender.send(line.unwrap()).unwrap();
        }
    });
    let wait_line = |expected: &str| {
        loop {
            let line = receiver.recv_timeout(Duration::from_secs(15)).unwrap();
            if line == expected {
                break;
            }
        }
    };
    wait_line("approval-pending");
    send_signal(&child, "-TERM");
    wait_line("console-pending");
    child
        .0
        .stdin
        .take()
        .unwrap()
        .write_all(b"next-console-line\n")
        .unwrap();
    assert!(wait_exit(&mut child).success());
    reader.join().unwrap();
}
