# Etas

> An effect-typed programming language for agent systems.

> [!WARNING]
> **Etas is highly experimental.** It is an early-stage research prototype, not
> a production-ready language or runtime. Syntax, semantics, package formats,
> compiler and interpreter behavior, CLI interfaces, and public APIs may change
> without notice. Do not use Etas for production or security-critical systems.

Etas makes agents, tools, actions, approvals, persistent memory, typed
messages, handlers, and multi-step flows explicit parts of a program. The goal
is to make agent systems statically checkable, inspectable, resumable, and
optimizable without hiding authority or orchestration behind framework code.

Etas is under active development. The current implementation is focused on
**Phase 1: the language frontend and the checked-HIR interpreter**. There are
currently no stability, backward-compatibility, migration, or long-term support
guarantees.

## Current Status

The following paths are implemented and exercised by the repository tests:

- lexing, parsing, AST construction, module assembly, and import resolution;
- HIR lowering, symbols, scopes, source maps, and structured HIR views;
- nominal and alias types, specs, generic constraints, and checked type facts;
- effect/action inference, handlers, trace specs, and interpreter-readiness
  facts;
- single-file and manifest-based project compilation;
- package manifests, lockfiles, path/GitHub dependencies, package metadata,
  materialization, and `.etaspkg` archives;
- direct checked-HIR execution through the Phase 1 interpreter;
- host boundaries for console, files, commands, TCP/TLS/streams, memory,
  sessions, tools, models, approvals, secrets, and browsers;
- trace output, profiling, replay, checkpoints, and resume.

AIR execution, the AIR-backed production runtime, and FIR optimization are not
part of the current user execution path. `etas run` executes checked HIR
directly. The interpreter is the Phase 1 lightweight runtime.

Important incomplete areas are tracked in the project documents and tests.
Protocol-state analysis, the full concurrency surface, and the Phase 2 AIR/FIR
path must not be inferred from the existence of scaffold repositories or CLI
placeholders.

## Language Example

Etas source files use the `.es` extension.

```etas
module examples.hello;

import std.io.println;

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    println("hello from Etas");
    return 0;
}
```

`std.io.println` requests the runtime action `Console.stdout_write`; its
standard implementation handles that action by default. The possible
`Error<IOError>` remains in the public effect row unless source code handles or
captures it.

## Repository Map

Etas is split into repositories by ownership boundary:

| Repository | Responsibility | State |
|---|---|---|
| [`etas`](https://github.com/etas-project/etas) | CLI, driver, package manager, integration fixtures, language design | Public, experimental |
| [`etas-core`](https://github.com/etas-project/etas-core) | Shared primitives, std metadata, host contracts, cache, package metadata | Public, experimental |
| [`etas-frontend`](https://github.com/etas-project/etas-frontend) | Syntax, HIR, shared analyses, type/effect checking, frontend sessions | Public, experimental |
| [`etas-interpreter`](https://github.com/etas-project/etas-interpreter) | Direct checked-HIR planning and execution | Public, experimental |
| [`etas-edk`](https://github.com/etas-project/etas-edk) | Official Etas Development Kit packages | Public, experimental; package-specific blockers remain |
| [`etas-ide`](https://github.com/etas-project/etas-ide) | Language intelligence, LSP, and VS Code integration | Public, experimental |

The active Phase 1 dependency direction is:

```text
etas-core
   |
   v
etas-frontend
   |
   v
etas-interpreter
   |
   v
etas (CLI + driver + package manager)

etas-ide -> etas + etas-frontend + etas-core
etas-edk -> Etas packages compiled and run through the etas CLI
```

Dependencies must remain acyclic. A foundational repository must not import a
higher-level repository to make local development easier.

## Prerequisites

- Git. The active Phase 1 repositories are public and do not require GitHub
  authentication to clone.
- Rust `1.85` or newer with Cargo.
- `rustfmt` and `clippy` for the full verification gates.
- A C/C++ build toolchain supported by Rust dependencies. On macOS, install the
  Xcode Command Line Tools.
- Node.js and npm only when building the VS Code extension in `etas-ide`.

Install or update the Rust toolchain with rustup:

```bash
rustup update stable
rustup component add rustfmt clippy
rustc --version
cargo --version
```

On macOS, initialize the native toolchain if necessary:

```bash
xcode-select --install
```

## Multi-Repository Checkout

The multi-repository development workflow uses sibling checkouts. Keep the
directory names unchanged because Cargo patch paths and local tooling refer to
them directly.

```text
etas-workspace/
  etas/
  etas-core/
  etas-frontend/
  etas-interpreter/
  etas-edk/
  etas-ide/
```

Create the workspace and clone the repositories:

```bash
mkdir etas-workspace
cd etas-workspace

git clone https://github.com/etas-project/etas-core.git
git clone https://github.com/etas-project/etas-frontend.git
git clone https://github.com/etas-project/etas-interpreter.git
git clone https://github.com/etas-project/etas.git
git clone https://github.com/etas-project/etas-edk.git
git clone https://github.com/etas-project/etas-ide.git
```

Each repository is an independent Git repository and Cargo workspace. There is
no parent Cargo workspace spanning all repositories.

### Local source patches

Committed Cargo dependencies retain their GitHub identities. For coherent
multi-repository development, the top-level workspace can patch all Etas crates
to the sibling checkouts through an ignored Cargo configuration:

```bash
cd etas
cp .cargo/config.example.toml .cargo/config.toml
cargo metadata --format-version 1 --no-deps
```

`.cargo/config.toml` is intentionally ignored by Git. Do not commit developer
machine paths. A build intended to verify published Git dependencies must run
without local patch overrides and against the revisions recorded by its lock
file.

When a change spans repositories, update and verify them in dependency order:

```text
etas-core -> etas-frontend -> etas-interpreter -> etas -> etas-ide/etas-edk
```

Do not use a downstream workaround for an upstream contract mismatch. Align the
upstream public API, update the dependent revision, and then continue.

## Build the CLI

Fetch dependencies once while network access is available:

```bash
cd etas
cargo fetch
```

Build a debug binary for local iteration:

```bash
cargo build -p etas_cli --all-features
./target/debug/etas --help
```

Build the optimized binary:

```bash
cargo build -p etas_cli --release --all-features
./target/release/etas --help
```

The binary is `target/debug/etas` or `target/release/etas`. To make the release
binary available in the current shell without installing it globally:

```bash
export PATH="$PWD/target/release:$PATH"
etas --version
```

The default feature set currently includes implemented commands and explicit
placeholder commands. A smaller frontend-oriented build is available when the
interpreter is not needed:

```bash
cargo build -p etas_cli --no-default-features \
  --features cli-check,cli-dump-ast,cli-dump-hir,cli-effects,cli-pkg
```

## Verify the Build

Run the user-level smoke path from the `etas` repository:

```bash
ETAS=./target/debug/etas

$ETAS check examples/minimal.es
$ETAS check --phase1 examples/minimal.es
$ETAS run examples/minimal.es
$ETAS check examples/hello.es
$ETAS run examples/hello.es
$ETAS dump ast examples/hello.es --tokens --spans
$ETAS dump hir examples/hello.es --symbols --scopes --source-map
$ETAS effects examples/hello.es
```

`check` validates static language semantics. `check --phase1` additionally
checks whether the selected entry is supported by the checked-HIR interpreter
and whether its required host services are configured.

Pass source-level program arguments after `--args`:

```bash
$ETAS run examples/args.es --args alpha beta
```

Pipe standard input into a console program:

```bash
printf 'hello from stdin\n' | $ETAS run examples/stdin_stdout.es
```

## Command Guide

| Command | Purpose | Current state |
|---|---|---|
| `etas check <input>` | Parse, resolve, type-check, and effect-check a file or project | Available |
| `etas check --phase1 <input>` | Also verify checked-HIR interpreter readiness | Available |
| `etas dump ast <file>` | Inspect AST, tokens, spans, and syntax diagnostics | Available |
| `etas dump hir <input>` | Inspect HIR, symbols, scopes, source maps, and diagnostics | Available |
| `etas effects <input>` | Report public effects, requested actions, and support facts | Available |
| `etas run <input>` | Execute the selected entry through checked HIR | Available |
| `etas replay <trace>` | Replay a Phase 1 interpreter trace | Available |
| `etas resume <checkpoint-id>` | Resume a persisted interpreter checkpoint | Available |
| `etas pkg lock|update|prepare` | Resolve and materialize dependencies | Available |
| `etas pkg metadata|pack` | Produce package metadata or an `.etaspkg` archive | Available |
| `etas dump air`, `etas graph` | Future AIR/analysis surfaces | Explicitly unsupported |
| `etas policy` | Legacy CLI placeholder; not a source-language Policy construct | Explicitly unsupported |
| `etas watch`, `etas repl`, `etas lsp` | Long-lived CLI service surfaces | Explicitly unsupported |

Use `etas <command> --help` for the authoritative option set. Human-readable,
text, JSON, JSONL, Mermaid, and DOT outputs are available where a command has a
corresponding renderer.

## Etas Projects

An Etas project is rooted at `etas.toml`. A minimal package contains:

```text
hello-project/
  etas.toml
  src/
    app/
      main.es
```

Example manifest:

```toml
[package]
name = "hello-project"
version = "0.1.0"

[source]
root = "src"

[[bin]]
name = "hello"
module = "app.main"
flow = "main"
```

The module declaration must match the path relative to the configured source
root:

```etas
module app.main;

flow main(args: Array<string>) -> i32 {
    return 0;
}
```

Check and run the project:

```bash
etas check .
etas check --all .
etas run .
```

`check --all` checks every package target. `run` selects the requested bin or
the manifest default entry.

## Packages

The package manager owns manifests, lockfiles, dependency resolution, package
storage, metadata, vendoring, and archives. It does not own parsing or static
language semantics.

The normal dependency workflow is:

```bash
etas pkg update .
etas pkg lock .
etas pkg prepare .
etas check --all .
```

Generated `.etas/` stores and indexes are local materialization artifacts. Do
not treat generated package metadata as source code or edit it by hand.

## Runtime Profiles

Source effects describe program behavior; they do not grant host authority.
Host-backed execution is configured by a named runtime profile in `etas.toml`
and, when needed, local overrides in `etas.local.toml` or `--runtime-config`.

Example local OMLX profile:

```toml
[runtime]
default_profile = "local-omlx"

[runtime.execution]
max_call_depth = 4096
max_steps = 1000000

[runtime.profiles.local-omlx.model]
adapter = "omlx-openai"
model = "Qwen3.5-0.8B-MLX-4bit"
base_url = "http://127.0.0.1:8848/v1"
api_key_env = "ETAS_HOST_OMLX_API_KEY"

[runtime.profiles.local-omlx.network]
allow = ["127.0.0.1:8848"]

[runtime.profiles.local-omlx.memory]
backend = "memory"

[runtime.profiles.local-omlx.session]
backend = "memory"
id = "local-session"
```

Run with the selected profile:

```bash
export ETAS_HOST_OMLX_API_KEY='<local-key>'
etas run . --profile local-omlx
etas run . --profile local-omlx --print-runtime-profile
```

Secrets should be referenced by environment-variable name or a configured
secret provider. Do not commit secret values to `etas.toml`, traces, fixtures,
or runtime profiles.

The legacy `ETAS_HOST_*` environment path and `--allow-effects` remain available
for compatibility. New project documentation should prefer named runtime
profiles because they make the selected adapters, authority, limits, and
fingerprint explicit.

## Trace, Profile, Replay, and Resume

Record and replay an interpreter trace:

```bash
etas run examples/checkpoint_resume.es --trace-out /tmp/etas-trace.json
etas replay /tmp/etas-trace.json
```

Create and resume checkpoints:

```bash
etas run examples/checkpoint_resume.es \
  --checkpoint-dir /tmp/etas-checkpoints
etas resume 0 --checkpoint-dir /tmp/etas-checkpoints
```

Inspect phase and pass timing:

```bash
etas run . --profile-tree
etas run . --profile-tree --profile-detail --profile-pass-timing
etas run . --profile-out /tmp/etas-profile.json
```

Trace artifacts describe execution events and replay identity. Profile artifacts
describe timing. They are separate contracts and should not be conflated.

## Development Gates

Run checks in the repository that owns the change. The standard gates are:

| Repository | Verification |
|---|---|
| `etas-core` | `cargo fmt --all -- --check`; `cargo test --workspace --offline`; `cargo clippy --workspace --offline -- -D warnings` |
| `etas-frontend` | `cargo fmt --all -- --check`; `cargo test --workspace --offline`; `cargo clippy --workspace --offline -- -D warnings` |
| `etas-interpreter` | `cargo fmt --all -- --check`; `cargo test -p etas_interpreter --offline`; `cargo clippy -p etas_interpreter --offline -- -D warnings` |
| `etas` | `cargo fmt --all -- --check`; `cargo test -p etas_cli -p etas_tests --offline`; `cargo clippy -p etas_cli -p etas_tests --offline -- -D warnings` |
| `etas-ide` | `cargo test --workspace --offline`; `npm run compile` and `npm run build` in `editors/vscode` |
| `etas-edk` | `etas pkg update`, `etas check --all`, and package-specific verifier scripts |

`--offline` assumes dependencies were fetched previously. If Cargo reports a
missing package or Git revision, run `cargo fetch` with network access and then
repeat the offline gate.

Cross-repository changes are complete only when upstream tests pass first and
the affected downstream repositories pass against the intended revisions.

## Architecture

```text
Etas source / etas.toml
          |
          v
      etas_cli                 command parsing and user-facing output
          |
          v
     etas_driver               project discovery and toolchain orchestration
       /      \
      v        v
etas_package  etas_frontend    dependency environment + checked HIR
                   |
                   v
             etas_interpreter  Phase 1 lightweight runtime
                   |
                   v
               etas_host       explicit host service boundaries
```

This repository contains four Rust crates:

| Crate | Role |
|---|---|
| `etas_cli` | CLI parsing, dispatch, diagnostics, output, profiling, and runtime-profile selection |
| `etas_driver` | Project discovery, package/frontend assembly, compilation, and run preparation |
| `etas_package` | Manifest, lockfile, resolver, store, source, metadata, vendor, and pack logic |
| `etas_tests` | Cross-repository language, compiler, package, and interpreter fixtures |

The interpreter runs source calls on the heap-backed `EvalMachine` frame stack.
The default call-depth budget is 4,096 and the configurable hard cap is 65,536.
Exhaustion produces an Etas diagnostic rather than overflowing the native Rust
stack.

## Troubleshooting

**Cargo cannot fetch an Etas Git dependency**

Verify network and proxy access independently with `git ls-remote`, then run
`cargo fetch`. The active Phase 1 dependencies are public and should not require
GitHub authentication.

**Cargo uses a Git crate when a sibling source change was expected**

Confirm that `.cargo/config.toml` exists in `etas`, contains the current Etas
patch entries, and that sibling directory names match the documented layout.
Inspect the resolved graph with `cargo tree -i <crate-name>`.

**An offline build reports a missing dependency**

Run `cargo fetch` once online. `--offline` is a reproducibility and CI gate, not
a dependency downloader.

**A module declaration does not match its path**

Compute the module name relative to `[source].root`. For example,
`src/app/main.es` declares `module app.main;`, not `module src.app.main;`.

**`etas run` reports a missing host service**

Static checking succeeded, but the selected runtime profile does not provide a
required adapter or authority grant. Inspect `etas check --phase1`,
`etas effects`, and `etas run --print-runtime-profile`; do not add an empty
handler or no-op host fallback.

**A package dependency is present in `etas.toml` but cannot be imported**

Run `etas pkg update` and `etas pkg prepare`, then inspect `etas.lock` and the
generated package index. The frontend consumes the prepared environment; it
does not download dependencies during type checking.

## Documentation

- [Language design and specification index](docs/design/README.md)
- [Language overview](docs/design/01-overview.md)
- [Agent, tool, prompt, memory, and message design](docs/design/03-agents-tools-prompts-memory.md)
- [Flow, handler, and protocol design](docs/design/04-flows-human-gates-and-protocols.md)
- [Type system and errors](docs/design/05-type-system-and-errors.md)
- [Effect system and inference](docs/design/06-effect-system-and-inference.md)
- [Package management](docs/design/15-package-management.md)
- [Concurrency](docs/design/16-concurrency.md)
- [EDK package design](docs/design/17-edk-official-packages.md)
- [CLI architecture](docs/architect/phase1-cli-design.md)
- [Repository boundary](docs/architect/repository-boundary.md)

The language design documents are normative for intended semantics. README
status claims must remain tied to implemented code and tests; planned behavior
must be labeled as planned.

## Citation

If you use Etas in research, please cite
[ETAS: An Effect-Typed Language for Agent Systems](https://arxiv.org/abs/2607.17780).
Machine-readable citation metadata, including the preferred paper citation, is
available in [`CITATION.cff`](CITATION.cff).

## License

Etas is distributed under the terms of both the
[MIT License](LICENSE-MIT) and the
[Apache License (Version 2.0)](LICENSE-APACHE). You may choose either license.
