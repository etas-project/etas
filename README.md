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

Pure named `std.task` jobs, including generic flows and flows from dependency
source packages, can use multiple CPU workers. Pure bound methods defined in the
program and closures with freezable captures can also use workers:

```bash
etas run program.es --cpu-threads 4
etas run program.es --cpu-threads auto
etas run examples/concurrent_generic.es --cpu-threads 4
etas run examples/concurrent_methods.es --cpu-threads 4 \
  --checkpoint-dir /tmp/etas-method-checkpoints
etas run examples/concurrent_closures.es --cpu-threads 4 \
  --checkpoint-dir /tmp/etas-closure-checkpoints
```

The default is `1`; the CLI caps `auto` and explicit counts at 64. The language
`Concurrency(n)` limit applies independently to each scope and its descendants.
Effectful jobs and Handler continuations execute on the coordinator. Generic
jobs need concrete, freezable input, output and capture types. Active concurrent scopes
can checkpoint after the coordinator drains admitted CPU work; see the
checkpoint example below.

Embedding applications can share a finite CPU pool across invocations through
the interpreter API's `InterpreterRuntime` and `RunOptions::runtime`. Each run
has its own `cpu_threads` quota and cancellation scope, and ready runs take
turns in the dispatch queue. The CLI uses an isolated executor for each invocation.

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
| `etas std build --manifest-path <path> --out-dir <path>` | Check the std source package and install its matching SDK | Development migration |
| `etas std install --manifest-path <path> --prefix <path>` | Install the current CLI and matching std SDK into a relocatable prefix | Development migration |
| `etas dump air`, `etas graph` | Future AIR/analysis surfaces | Explicitly unsupported |
| `etas policy` | Legacy CLI placeholder; not a source-language Policy construct | Explicitly unsupported |
| `etas watch`, `etas repl`, `etas lsp` | Long-lived CLI service surfaces | Explicitly unsupported |

Use `etas <command> --help` for the authoritative option set. Human-readable,
text, JSON, JSONL, Mermaid, and DOT outputs are available where a command has a
corresponding renderer.

### Build the std SDK

The maintained Etas standard library sources live in the sibling `etas-core`
repository. Build them as a whole library, including private bodies, without a
`main` flow:

```bash
./target/debug/etas std build \
  --manifest-path ../etas-core/crates/etas_std/etas.toml \
  --out-dir /tmp/etas-sdk/std
```

The output contains `manifest.json`, `etas.toml`, the complete `library/` source
tree, and `artifacts/std.etasmeta`. The existing package artifact container
holds public metadata and the checked body graph together. Relative source
paths allow moving the SDK. Rebuilding replaces the installed source tree.

The driver loader verifies compiler, schema, target, toolchain ID, native binding
ABI, and content digests before exposing the bundle. Its ABI expectation is
compiled into the toolchain; loading never constructs the handwritten registry.
Release packaging can set `ETAS_TOOLCHAIN_ID` when building the driver.

Select the SDK for an entire compilation session, including dependencies:

```bash
etas check app.es --std-provider artifact --sysroot /tmp/etas-sdk
etas run app.es --std-provider artifact --sysroot /tmp/etas-sdk
```

Projects that declare a std dependency lock it with the same provider:

```bash
etas pkg lock ./project --std-provider artifact --sysroot /tmp/etas-sdk
etas run ./project --std-provider artifact --sysroot /tmp/etas-sdk
```

This explicitly upgrades old `builtin:std` records to the selected SDK's version
and native ABI. Manifest version requirements apply to that selected version.
Check/run reject a different installed version or ABI and leave the lockfile
untouched. Source body rebuilds keep the lock when the version/ABI stays the
same; the SDK's complete content identity invalidates execution and dependency
metadata caches. The legacy provider retains its original builtin lock identity.

Artifact mode loads source exports and checked bodies from the verified SDK.
Adding a public flow in a new `.es` file requires no Rust registration. Rebuild
the SDK after source changes; its complete content identity invalidates the
consumer cache. Direct calls, function values and checked generic applications
use the independently checked body graph and the caller's execution state.

Without `--sysroot`, artifact mode looks beside the actual executable:
`<prefix>/bin/etas` uses `<prefix>/lib/etas/<toolchain-id>/std`. Install both the
binary and the SDK together, including for a Cargo-built CLI:

```bash
./target/debug/etas std install \
  --manifest-path ../etas-core/crates/etas_std/etas.toml \
  --prefix /tmp/etas-toolchain
/tmp/etas-toolchain/bin/etas run app.es --std-provider artifact
```

The complete prefix can be moved and used offline. An explicit sysroot is never
replaced with a legacy fallback.

Std native declarations use an explicit checked binding, for example:

```etas
@std.binding("std.text.trim")
public flow trim(value: string) -> string ![] {}
```

Only a std source build accepts this annotation. Its signature is checked against
the primitive ABI, and generic type/effect arguments are forwarded by checked
calls. Rebuilding with `--std-provider artifact --sysroot <root>` uses that SDK's
verified primitive contract independently of its migrated public exports.

The source task module owns its opaque declarations and `run` signature:

```etas
module std.task;
@std.role("std.task.Task")
public type Task<T, effect E>;
@std.role("std.task.Scheduler")
public type Scheduler;
@std.role("std.task.Continuation")
public type Continuation<T>;
@std.binding("std.task.run")
public flow run<T, effect E>(body: (Scheduler) -> T ![E]) -> T ![E] {}
```

`std.role` is accepted only in a toolchain std build. The compiler checks the
canonical opaque declaration and its type/effect parameter shape; the SDK query
view preserves that source owner. Rust retains scheduler scopes, continuation
capture and transfer, value representation, and enforced budgets and permissions.
Public declarations and algorithms belong to Etas; native text acceleration is
optional. Source builds use the explicit bootstrap contracts described below.

Public source types and enum constructors are published from the checked SDK:

```etas
public type Parcel<T> = { value: T };
public spec Reader { flow read(self) -> string ![]; }
impl Parcel<string> ~ Reader {
    flow read(self) -> string ![] { return self.value; }
}
public flow via_reader<T ~ Reader>(value: T) -> string ![] {
    return value.read();
}
public enum Reply<T> { Empty, Item(T), Named { value: T }, }
```

Applications import these declarations from their authored module and can pass
their values through installed generic flows. Canonical owners, generic
parameters and named payload fields survive separate build/run processes and
installation relocation. Native role replacement requires its checked role
contract before publication.

Source specs and implementations use the existing checked package metadata and
canonical method identities. Imported methods and generic spec calls execute
the SDK's checked bodies. Public methods retain their checked Effect contracts
even when their implementation has no separate `public` keyword.

Applications can also implement SDK specs on their own private types. Generic
SDK flows select those implementations by canonical spec and method identity;
the linker prepares concrete generic method bodies in the defining program,
including private body types and nested SDK calls. SDK-private types can pass
through these generic methods without being exported as lexical names. Returned
SDK closures retain the checked caller method link. Calls keep the current
runtime Handler, permission, budget and trace context, including protected
filesystem methods and typed errors handled by the application.

Standard-library orchestration uses ordinary `flow` declarations. Reusable
Effect handling uses existing `let ... = handler { ... }` values applied with
`with`; there is no `controller` language declaration or migration requirement.

Std source builds predeclare the explicit compiler/host contracts in
`etas_std/src/intrinsic/bootstrap-contracts.json`; they construct no complete
legacy registry. The catalog carries primitive operand/result ABI, fixed core
Effect/action identities, opaque task roles, and budget enforcement contracts.
Its content identity participates in the source build cache, and the installed
binding ABI includes signatures and roles as well as primitive descriptors.
Migrated public std flows and their checked bodies come from `.es` files.

Public declarations, generic parameters, Effect signatures, enums and spec/impl
belong to the Etas source package. Text/collection algorithms, error conversion,
protocol parsing, task composition, retries and Handler scheduling should use
Etas wherever the language can express them. Rust provides OS threads, file and
network operations and storage primitives, plus continuation capture/resume,
value representation and enforced budgets/permissions.

`std.role` binds a checked source declaration to a compiler execution role; it
does not transfer ownership of that declaration to Rust. A `StdRegistry` query
view generated from the checked bundle is compatible with source ownership.
The migration must remove manually duplicated public API/signature/algorithm
tables, including transitional bootstrap declarations as they move to source.
Native acceleration is a performance choice. The maintained `trim` algorithm is
now Etas source; Rust supplies the Unicode scalar whitespace property. Its
public declaration is absent from the native bootstrap catalog.

Package preparation records only the builtin std dependency identity; it does
not project another Rust std interface. Driver and IDE dependency type projection
use the selected session registry, including its prelude aliases. Immutable
producer-source restoration retains the caller's std provider/cache identity.
The source build phase selects the compiler/host bootstrap before package
loading, so even package preparation does not invoke the legacy factory.
Index implementations are absent from the bootstrap catalog and come only from
checked Etas source when building and loading the SDK.

Independent source ownership requires both a new ordinary API added only in
`.es` and a std build process that avoids the complete legacy registry and uses
an explicitly reviewed minimal compiler/host contract set. Compiling out the
legacy factory verifies that dependency boundary; it does not by itself prove
that the remaining bootstrap catalog is minimal or the entire std is migrated.

Source role validation uses an explicit `StdSourceRole` compiler ABI instead of
registry declaration/signature lookup. Its sixty-seven roles define only the recognized
identities, generic parameter kinds and execution shapes. The query view uses
temporary internal binders until the checked source supplies public parameter
names. Those role declarations and their four native constructor implementations
are removed from the static catalog, which still has 55 compiler/host catalog entries
including 37 primitive contracts. Collection Array/List/Map/Set/Range/Slice/Iter declarations
now join the source roles. The SDK fixture uses different source parameter names
for Task, Continuation, Option, Result, WorkspacePath and collections to exercise this ownership
boundary. Source-bootstrap prelude references bind to their checked source exports.

The scalar/codec batch completes source declarations for `std.char`, `std.bytes`
and `std.codec.text`, including UTF-8 mode/error enums. ASCII digit and control
predicates use checked Etas bodies. Unicode tables, byte storage and codec
acceleration retain explicit native bindings; this does not establish bootstrap
minimality. The installed SDK compares strict/replacement decoding and Unicode
case/scalar behavior with legacy, including a permissioned file-read trace.

Twenty-three ordinary compiler report records are now declared in Etas. Rust
retains their 91 produced fields as 23 record layout constraints; together with
FsStat there are 24 such constraints, plus the parser/formatter primitives. Record ABI
checks use checked declaration TypeIds for nested nominal fields. The remaining
catalog and payload constraints have no whole-bootstrap minimality acceptance. Installed SDK checks include reflected report values crossing std
body boundaries.

Fourteen opaque compiler metadata declarations now live in Etas source with
minimal bindings to their existing CodeTypeRole identities. The roles keep
compiler-only construction and native value representation, without public
field/signature tables. Their mapping is included in the compiler ABI identity.
TypeInfo and CallableId values cross checked std body boundaries in the SDK
acceptance. AgentDef, CallableDef and SyntaxBlock now declare their 21 getter
fields in Etas source. Their checked bundle stores projected fields separately
from record storage; Rust retains three explicit getter ABI layouts and opaque
compiler-managed values. Generic definitions cross separately compiled std bodies
without losing their backend lease or program ownership checks. Registry and
container schemas are 6 and 49. This does not establish whole-bootstrap minimality.

All twenty builtin core type declarations now live in core/primitives.es. Trusted
roles map them to existing compiler primitive representations. Bundle signatures
and descriptors retain source declaration identities as transparent aliases;
primitive targets remain structural. Qualified and aliased imports execute from
the installed SDK. Role ABI fingerprint format is 4; the remaining 55 catalog
entries still require migration and minimum-contract review.

Eleven runtime effect declarations now live in Etas source. Source owns generic
names and inheritance, including Error<Failure> and Fs extending FileIO; Rust
retains their existing execution IDs and host capability requirements. The bundle
and generated query view preserve those declarations and canonical identities.
The SDK fixture adds an ordinary effect and a flow only in .es, then imports and
executes the installed body. Eight native action contracts and seven limit
requirement contracts remain transitional and require further migration. Current
role ABI fingerprint format is 4; this does not establish bootstrap minimality.

Array get/push/pop/extend and List head/tail/pop/push/extend methods are authored
in Etas. Array get/push retain minimal storage primitives; iteration, list
construction and the other algorithms use ordinary Etas bodies. Checked generic
payload origins survive both free flows and methods, including adding a callback
to an empty collection and invoking it under the original caller's Handler.
Unicode char_count/is_empty/trim_start/trim_end and core assert also use source
bodies. Rust retains the scalar Unicode property and underlying text storage.

Boolean parsing/rendering and exact contains/starts_with/ends_with/strip_prefix/
strip_suffix policies are also source flows. Their private character matching
helpers travel in the bundle; no additional compiler role or native signature
is needed. The text search implementation currently materializes character
arrays; whole-library performance comparison remains a release gate.

`TextError` and its constructor exports are ordinary Etas declarations. Native
numeric parsing references that canonical enum and requires only the two unit
variants it can return; the other variants and additional source variants belong
to the source package. This ABI reference grants no language role. Source enum
constructor imports/re-exports retain canonical identities, generic payloads and
checked execution layouts. The published std provider resolves every native enum
reference to its checked declaration. Partial compiler library images may retain
ABI imports until that publication step. The bundle uses body schema 12 and
registry image schema 5.

`std.fs.Region`, `WorkspacePath<R ~ Region>` and `IOError` are also source
owned. Region supplies the compiler's static authority-selector identity;
WorkspacePath binds the existing runtime path representation. Its generic
parameter name belongs to source and retains the Region ABI bound. IOError is
an ordinary opaque type with a native payload reference, without a language
role. Publication resolves that reference to its checked opaque declaration.
All filesystem API declarations now use source: path identity helpers, reading,
writing, listing, stat and atomic replacement. `ReadLimitError` and `ListLimitError`
are ordinary source enums. Their native references require only the LimitExceeded
variants and usize payloads returned by the host; extra source variants need no
Rust API registration. `FsStat` is a source record with checked is_file/is_dir/len
host payload fields, and its native import shares the source nominal identity.
The host decoder wraps the result in that checked nominal representation.
Bounded reads/listings, OS operations and permission enforcement remain host
capabilities. Eight existing primitives and four action descriptors were added
to the explicit catalog for these bindings; this does not establish whole-catalog
minimality. Native binding aliases retain the primitive identity even when the public wrapper has
the same path, preserving requested actions and caller Handler error handling. Terminal host failures keep the original permission diagnostic when no
retry surrounds the source call. An enclosing retry now unwinds the callee's
Handler and residual scopes, restores the parent model policy, and switches to
the caller's checked program before starting the next attempt. Installed SDK
acceptance compares two denied write attempts and their exhaustion diagnostic
against legacy; a separate checked-revision test verifies transient Host failure
recovery and caller continuation. Pending retry failures are portable in
checkpoint schema v78. Source std now declares both checkpoint import paths,
backed by the existing runtime checkpoint primitive. Saved std callables refer
to the current caller's verified bundle and checked type bridge; executable
code and invocation permissions are not taken from checkpoint data.

Installed acceptance saves state inside a private generic std record and while
a std flow holds an application callback. New processes restore all three save
points, preserve the caller's Handler, produce 42 and skip checkpoints already
passed. A separate compiler integration test reloads the SDK and recompiles the
caller into a fresh PreparedCodeRevision in another process before resuming a
library-owned checkpoint. Frozen body recompilation strips the previously
injected std dependency projection before namespace checks, then attaches the
same selected provider again. Whole-library compatibility and performance
validation remain migration gates.

UTF-8 byte length and boundary queries, scalar indexing, byte-position search,
non-overlapping counts, eager split/limited split, first/last partition,
replacement and ASCII case policies also use Etas bodies. Scalar encoding widths
are computed from character order; arrays use the existing storage primitives.
`List.get` delegates checked indexing through those same primitives and preserves
callback payloads under the caller's Handler. No new host primitive or language
role is introduced. Lazy text iterator APIs remain pending, alongside the
whole-library performance gate.

Verify the existing SDK process fixture with the complete legacy factory
compiled out (the ordinary configuration also runs the legacy comparison):

```bash
cargo test --locked -p etas_cli --features forbid-legacy-std --test cli \
  installed_std_executes_new_generic_flow_and_rebuilt_join_body_in_external_processes
```

The core export example refreshes the transitional ABI catalog from the retained
migration baseline during native contract development; SDK build/load processes
do not run that exporter. The catalog contains no std source algorithms. Native
text bindings can later be replaced with Etas implementations independently.

Compare the installed SDK with the existing per-module API inventory:

```bash
cargo run -p etas_driver --example std_sdk_inventory -- /tmp/etas-sdk \
  > /tmp/std-candidate.json
```

The release default remains `--std-provider legacy`. The source batch covers
text flows, opaque task roles, source Option/Result enums, constructors,
predicates and unwrap methods, Index declarations/implementations, seven
collection declarations and nine Array/List methods. Remaining collection APIs,
roles/modules, full API
and runtime parity, artifact checkpoint/resume and release gates remain required
before the default changes.

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
max_tool_rounds = 64

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

Active task scopes can also checkpoint. The coordinator drains admitted CPU
and Host work before saving task machines, queues, resource permits and parked
Handler continuations. Resume preserves consumed budgets and completed actions.

```bash
etas run examples/concurrent_checkpoint.es --cpu-threads 4 \
  --checkpoint-dir /tmp/etas-concurrent-checkpoints
etas resume 0 --checkpoint-dir /tmp/etas-concurrent-checkpoints
```

The resumed example prints `tasks completed` without repeating `starting tasks`
or the completed CPU jobs.

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
