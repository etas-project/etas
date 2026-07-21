# Phase 1 User CLI Design

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
- terminal output mode selection.

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
  -> etas_interpreter::run_checked

etas resume <checkpoint>
  -> etas_interpreter::resume_checked
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
3  unsupported Phase 1 command
4  internal error
```

Unsupported Phase 1 commands should be stable and explicit so users can
distinguish "not implemented yet" from "program failed."

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
