# Phase 1 User CLI Design

Owner: `Architect`

Last updated: `2026-09-10`

## 1. Purpose

`etas` is the real user-facing CLI and distribution repository.

In Phase 1 it composes:

```text
etas source
  -> etas_driver assemble project input
  -> etas_frontend::FrontendSession check
  -> etas-interpreter run checked HIR
```

It must not own compiler or interpreter internals.

## 2. Crates

```text
etas/
  crates/
    etas_cli/
    etas_driver/
    etas_package/
    etas_integration_tests/
```

The CLI repository is the user's entry point. It should feel like one coherent
tool even though implementation is delegated to component repositories.

### 2.1 `etas_cli`

Owns:

- binary entry point;
- command-line parsing;
- config discovery;
- exit-code policy;
- terminal output mode selection;
- operating-system signals, input lifecycle, and user-visible interruption
  policy through the interpreter's public controlled execution API.

It should use `clap` or the selected CLI parser, but command execution logic
should delegate into `etas_driver`.

Recommended top-level options:

```text
--format text|json
--color auto|always|never
--quiet
--verbose
--workspace <path>
```

### 2.2 `etas_cli` / `etas_driver`

`etas_cli` owns argument parsing and human-facing output. `etas_driver` owns
project loading, package preparation, frontend session orchestration, and
interpreter handoff.

```text
check
dump ast
dump hir
run
resume
version
pkg metadata
pkg pack
pkg prepare
pkg lock
pkg update
```

Command routing:

```text
etas check <file>
  -> etas_driver::load_project
  -> etas_frontend::FrontendSession::check

etas dump ast <file>
  -> etas_driver::load_project
  -> etas_frontend::FrontendSession::check
  -> syntax dump

etas dump hir <file>
  -> etas_driver::load_project
  -> etas_frontend::FrontendSession::check
  -> HIR dump

etas run <file>
  -> etas_driver::load_project
  -> etas_frontend::FrontendSession::check
  -> Interpreter::create_run
  -> RunInvocation::execute (keep RunControl for interrupts)

etas resume <checkpoint>
  -> prepared CheckedProject and decoded checkpoint
  -> Interpreter::create_resume
  -> RunInvocation::execute (same lifecycle driver)
```

Command modules:

```text
command/
  check.rs
  dump_ast.rs
  dump_hir.rs
  run.rs
  resume.rs
  version.rs
  interpreter/
    lifecycle/
      mod.rs
      signals.rs
      run.rs
      report.rs
output/
  diagnostics.rs
  text.rs
  json.rs
workspace/
  input.rs
  files.rs
```

### 2.3 `etas_integration_tests`

Owns cross-repository public API tests:

- CLI smoke tests;
- fixture loading;
- golden output tests;
- static-check versus Phase 1 execution-readiness tests;
- checkpoint/resume tests;
- unsupported AIR/runtime behavior tests;
- diagnostic shape tests.

It must test public facades only.

### 2.4 Runtime Configuration Boundary

CLI/runtime configuration owns backend connections, managed resource ceilings,
authority, lifecycle deadlines and existing model/tool transport setup. It does
not require a default production summarizer, summary model, tokenizer, prompt or
automatic compaction policy. Those are EDK/application choices composed from
ordinary source APIs, not new `etas run` options or a hidden Session service.
Keep the existing application configuration boundary rather than adding a second
configuration framework for summarization.

The current SPEC has removed `SessionConfig.compaction`. Obsolete automatic
summary settings must now receive explicit migration diagnostics, not silent
ignoring or replacement by a test provider. Keep ordinary model adapters,
configured retention/storage maintenance and budget/authority/cancellation
enforcement. `ContextTokens` is still a budget, not a summary trigger.

`run` and `resume` consume the interpreter's typed storage outcomes and lifecycle
report. Preserve safe receipt/operation correlation in structured output without
printing stored payloads or secrets. CLI does not automatically retry unknown
writes, select a reconciliation policy or create a checkpoint merely because
the user cancelled. Source `commit`/`reconcile` APIs cover recovery; no separate
CLI reconciliation command is required for this contract. Session recovery uses
the specified `prepare_context` / `publish_context` / `reconcile_context` path;
its implementation is no longer waiting for a language-boundary decision.

Real Linux isolation acceptance follows the
[Workspace enforcement gate](../../../etas-core/docs/architect/etas-workspace-design.md#71-linux-enforcement-gate).
Unavailable required isolation rejects execution rather than falling back to an
unconfined command. Portable CLI tests do not certify Linux kernel enforcement.

## 3. User-Facing Commands

Phase 1 supported commands:

```text
etas check <file>
etas dump ast <file>
etas dump hir <file>
etas check --phase1 <file>
etas run <file> [--entry <flow>] [--arg <json>] [--fuel <n>]
etas resume <checkpoint> [--fuel <n>]
```

Phase 1 unsupported or deferred commands should fail explicitly:

```text
etas dump air <file>
etas graph <file>
etas replay <trace>
```

The CLI should never pretend that AIR execution exists before Phase 2.

`etas check <file>` is a static-correctness gate. It should not fail solely
because a statically valid program requires model, tool, network, filesystem,
command, durable-memory, checkpoint, resume, workflow orchestration, or effect
handler support.

`etas check --phase1`, `etas run`, and `etas resume` are direct checked-HIR
execution gates. They may fail when host handlers, authority grants, checkpoint
stores, or interpreter orchestration support are missing.

The portable user entry ABI is `main(args: Array[string]) -> i32`. CLI arguments
provided after the source file, or after an explicit `--` separator when that
syntax is supported, must be passed to the checked-HIR interpreter as
`Array[string]`. The CLI must not encode process arguments as `List[string]`.

## 4. Output Policy

The CLI owns presentation:

- human-readable diagnostics;
- JSON diagnostics;
- deterministic dumps;
- exit codes;
- quiet/verbose flags.

It should not alter compiler or interpreter diagnostic content. Diagnostics
come from `etas-core` data structures and are rendered by CLI output code.

Exit-code policy:

```text
0  success
1  valid command, user program diagnostics
2  invalid CLI usage
3  runtime failure (including incomplete cleanup without an OS signal)
4  internal error
130 interrupted by SIGINT
143 terminated through the SIGTERM handling path (Unix)
```

Unsupported commands remain explicit usage/availability diagnostics. The signal
exit statuses do not claim that external operations rolled back or all cleanup
succeeded; structured reports carry that evidence independently.

### 4.1 Run And Resume Lifecycle

Use the public lifecycle described in the
[Interpreter design](../../../etas-interpreter/docs/architect/phase1-interpreter-design.md#31-controlled-invocation-api)
and the common [execution contract](../../../etas-core/docs/architect/etas-execution-design.md).
Both `run` and `resume` use `command/interpreter/lifecycle/`; neither invents a
private evaluator, adapter cancellation tree or second cleanup protocol.
The CLI constructs one public `RunInvocation`, obtains its `RunControl`, and
drives `execute()` while observing signals. It does not construct or reuse a
mutable invocation scope in `RunOptions`. Borrowed project/Host inputs remain
valid through execution; no project clone or hidden background evaluator is
required. The existing simple run/resume convenience APIs use this same owner
inside the Interpreter.

File responsibilities:

| File | Responsibility |
|---|---|
| `signals.rs` | Scoped OS signal registration and input event observation; restore/release registrations when finished |
| `run.rs` | Own and drive the public invocation future while observing interrupts; request stop and observe termination through its control handle |
| `report.rs` | Render run outcome, pending cleanup, uncertain external operations and signal/exit mapping |

First SIGINT or SIGTERM requests cooperative stop and starts a bounded cleanup
wait. A second termination signal, or expiry of the configured cleanup wait,
enters forced-exit handling. Report incomplete cleanup before exit where output
is possible. Preserve 130/143 for the initiating Unix signal and expose forced
exit/cleanup status in JSON; do not label forced exit as a successful join.
Ordinary non-signal runtime failure retains exit code 3. A signal arriving after
terminal result publication does not rewrite that completed run result.

Keep the invocation future pinned and polled after the first signal while
observing the stop deadline. `RunControl::join()` does not drive execution;
switching exclusively to a join waiter can deadlock cooperative shutdown.
Dropping a temporary signal/wait future must not drop the invocation. Only the
documented forced-exit path abandons further waiting, with pending work reported
honestly and supervisor lifetime retained until process shutdown.

Use the existing runtime profile configuration path for a proposed
`execution.shutdown_grace_ms` setting: default 5000 ms, valid range 1..60000 ms.
Validate it before execution. It bounds waiting/cleanup policy, not business
execution, and cannot extend the original business budget. Keep configuration
discovery and local overrides centralized; do not add scattered environment
reads. Adapter operations must still report their actual termination guarantees.

EOF is an input result, not unconditional cancellation of the run. Console and
approval input share the process stdin broker; stopping a waiter must not consume
a subsequent request's line. Applications embedding the interpreter supply
their own signals/UI and need not depend on CLI policy.

Use explicit runtime outcome for exit decisions rather than only testing whether
diagnostics contain an error. Consume the Interpreter's final
`Completed(value) | Failed(failure) | Cancelled(cause)` outcome and required
termination report; do not reconstruct a final result from `TimedOut(pending)`.
A completed source value alone is not proof of successful cleanup. Preserve
the primary language failure or initiating signal while reporting cleanup
failures separately. Finalize trace/profile with the actual stop status;
retain confirmed external work and uncertainty. Write only checkpoints already
reported valid by the interpreter, without fabricating a checkpoint on Ctrl-C.

## 5. Dependency Direction

Allowed:

```text
etas -> etas-core
etas_cli -> etas_driver
etas_driver -> etas_package
etas_driver -> etas_frontend
etas -> etas-interpreter
```

Deferred until later phases:

```text
etas -> etas-optimizing
etas -> etas-runtime
etas -> etas-ide
```

Forbidden:

```text
etas_cli owning parser/type/interpreter logic
etas_cli or etas_driver reaching into private component modules
```

The CLI composes component facades. It does not become a monolithic
implementation repository.

## 6. Test Direction

CLI tests should include:

- `etas check` success and diagnostic cases;
- `etas dump ast` deterministic output;
- `etas dump hir` deterministic output;
- `etas run` on a supported deterministic fixture;
- `etas run` with host handler readiness diagnostics;
- `etas resume` from a Phase 1 interpreter checkpoint fixture;
- unsupported AIR/runtime command exits;
- JSON diagnostic shape;
- exit-code behavior.

Add real subprocess signal tests for CPU-only evaluation, pending stdin/approval,
loopback I/O and command execution. Cover first/repeated signals, EOF, completion
races, configured cleanup timeout, partial/uncertain operations, and JSON/text
exit reporting. Verify process cleanup within each backend's documented limits;
a fake adapter returning `Cancelled` is not an OS-signal acceptance test.
