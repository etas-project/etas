# Implementation Strategy

Etas should be implemented in three project phases:

```text
Phase 1: Frontend + typed HIR interpreter
Phase 2: AIR v0 + runtime execution model
Phase 3: FIR middle-end + agent workflow optimization
```

The first implementation should not target native code generation. Phase 1
validates the language frontend and typed execution semantics before the
runtime IR is stabilized. Phase 2 introduces AIR v0 as the runtime-facing
execution model. Phase 3 introduces FIR as the middle-end for analysis,
transformation, and agent workflow optimization after AIR execution semantics
are concrete.

## 1. Implementation Model

The Phase 1 pipeline is:

```text
.es source
  -> parser
  -> name resolution
  -> HIR
  -> typed HIR
  -> typed HIR interpreter
```

The Phase 2 pipeline is:

```text
.es source
  -> typed HIR
  -> AIR v0
  -> AIR interpreter / runtime
```

The Phase 3 pipeline is:

```text
.es source
  -> HIR
  -> typed HIR
  -> FIR
  -> optimized FIR
  -> AIR
  -> AIR interpreter / runtime
```

From the user's perspective, Etas should feel like a runnable application language. Internally, it should have a real compiler frontend and an explicit intermediate representation.

The first executable prototype is a typed-HIR interpreter, not a native compiler
and not a bytecode VM compiler. AIR is the Phase 2 executable runtime IR. FIR is
the Phase 3 analysis and transformation IR. A compact bytecode can be added
later as a backend generated from AIR.

## 2. Why Typed HIR First, Then AIR

Etas's primary complexity is not CPU-bound computation. Its core responsibilities are:

- model calls;
- tool dispatch;
- effect checking;
- effect/action enforcement;
- policy enforcement;
- approval gates;
- runtime-scoped effect handlers;
- checkpoint and resume;
- trace logging;
- memory versioning;
- prompt construction and taint checks;
- token, context, cost, and time budgets.

These concerns eventually belong in the runtime. Phase 1 should validate the
frontend and typed source semantics first; Phase 2 should then interpret AIR so
the runtime can mediate every meaningful operation through an explicit execution
model.

Native code generation would add early complexity around ABI, linking, code generation, and low-level optimization without addressing Etas's main semantic problems.

## 3. Typed HIR Interpreter vs AIR Interpreter

Etas should start with a typed HIR interpreter for early prototyping, but the
formal runtime target should be an AIR interpreter.

A typed HIR interpreter is useful because:

- it is faster to implement;
- it preserves resolved names, scopes, source mappings, and typed facts;
- it is convenient for validating early language semantics;
- it avoids stabilizing AIR before type/effect semantics and entry-flow behavior are understood.

However, HIR is still source-shaped, not runtime-shaped. Etas runtime
execution needs explicit representation for:

- effect checks;
- effect/action checks;
- policy checks;
- budget checks;
- approval dominance;
- prompt taint checks;
- checkpoint boundaries;
- trace node ids;
- retry paths;
- handler scopes and resume states;
- data dependencies;
- memory versions;
- concurrent scheduling for joined branches;
- graph visualization;
- resume positions.

If Etas directly interprets HIR as the long-term runtime representation, these
concerns either remain implicit or get scattered throughout the interpreter.

AIR makes them explicit. For example:

```etas
let review = Reviewer.run(draft);
```

can lower to:

```text
n1 = PromptBuild(Reviewer.run, [draft]) -> prompt
n2 = PolicyCheck(Reviewer.policy)
n3 = AgentCall(Reviewer.run, prompt, Schema<Review>) -> review
n4 = ValidateSchema(review, Review) -> review
n5 = PostProcess(Reviewer.run, draft, review) -> review
```

The runtime can then execute explicit checked nodes instead of rediscovering policy, schema, prompt, and trace behavior from source syntax.

Project staging:

```text
Phase 1: source -> typed HIR -> typed HIR interpreter
Phase 2: source -> typed HIR -> AIR v0 -> AIR interpreter/runtime
Phase 3: source -> typed HIR -> FIR -> optimized FIR -> AIR -> runtime
```

The Phase 1 typed HIR interpreter is a bootstrap execution path and semantic
validation tool, not the long-term runtime architecture.

## 4. Not Python-Style Dynamic Execution

Etas should support quick execution, but it should not behave like a dynamic "parse and eval directly" language.

Before a flow runs, Etas should check:

- names and modules resolve;
- types are valid;
- agent outputs have schemas;
- effects are declared;
- effect/action grants are available;
- trust labels do not flow into unsafe prompt channels;
- loops have required limits;
- high-impact effects are guarded by policy;
- the `main` entry flow is executable by the interpreter or runtime.

Phase 1 executes checked typed HIR, not unchecked source code. Phase 2 and later
execute checked AIR, not unchecked source code.

## 5. Program Entry

An Etas program enters through a `main` flow.

```etas
flow main(args: Array<string>) -> i32 {
    return 0;
}
```

`main` is executed directly. It returns an `i32` process status code; by convention, `0` means success and non-zero values indicate failure. The return type annotation is useful in documentation, but the compiler can still infer it when every return candidate is `i32`. Public entry metadata can record the generated type and effect signature for review and deployment.

The runtime sequence is:

1. parse and type-check the program;
2. find `main(args: Array<string>) -> i32`;
3. infer and verify `main`'s effects, handlers, action grants, limits, and policies against host policy;
4. in Phase 1, execute `main` through the typed HIR interpreter;
5. in Phase 2+, lower `main` to AIR v0, or through FIR to AIR in Phase 3;
6. execute AIR through the runtime;
7. emit trace, metrics, checkpoints, final result, and process status code.

`main` may call other flows, agents, and tools like any other flow, subject to ordinary type, effect, action boundary, policy, and limit checks. Static analysis starts from `main` and conservatively summarizes all reachable execution paths.

## 6. Compiler Responsibilities

The compiler frontend and staged lowering pipeline should own:

- parsing;
- name resolution;
- type checking;
- flow type checking;
- effect checking;
- effect/action checking;
- trust and prompt taint analysis;
- policy validation;
- loop limit validation;
- typed HIR interpretation for Phase 1;
- lowering checked HIR to AIR v0 for Phase 2;
- local AIR lowering-time simplifications;
- AIR verification;
- static flow graph extraction;
- diagnostics.

Later compiler passes may add:

- lowering checked HIR to FIR;
- FIR-based analysis and transformation;
- lowering optimized FIR to AIR;
- context slicing;
- prompt partial evaluation;
- retrieval hoisting;
- agent call caching;
- verifier insertion;
- cost-aware planning.

## 7. Runtime Responsibilities

The AIR interpreter/runtime should own:

- model provider calls;
- tool execution;
- memory reads and writes;
- policy enforcement;
- effect/action enforcement;
- approval handling;
- budget accounting;
- retries and fallback;
- checkpoint and resume;
- trace event emission;
- sandboxing;
- temporary value lifecycle management;
- persistent state retention and compaction;
- observability export.

This keeps all external effects under one runtime authority boundary.

## 8. Runtime Memory Management

Etas source assumes automatic memory management. Users should not write `free`, manage ownership, or reason about low-level allocation in ordinary Etas programs. Memory management is an implementation and runtime concern, not a source-language feature.

The implementation should distinguish two kinds of memory:

| Category | Examples | Management model |
|---|---|---|
| Temporary runtime values | records, lists, prompts, decoded agent outputs, local flow values, AIR interpreter stack values | Automatically reclaimed by the implementation |
| Persistent agent state | `MemoryRegion<S>` / `Store<K, V>` runtime resources, traces, checkpoints, conversation history, artifacts, cached model/tool results | Governed by explicit retention, compaction, archival, and deletion policies |

For compiler data structures, arena allocation is a good default. AST, typed AST, and many AIR construction objects usually share the same lifetime and can be released together after checking or lowering.

For runtime sharing, the implementation can choose the right ownership strategy:

| Strategy | Use case |
|---|---|
| Arena | Parser/compiler AST and short-lived lowering structures |
| `Rc`-style reference counting | Single-threaded prototype interpreter with shared immutable nodes |
| `Arc`-style thread-safe reference counting | Concurrent runtime sharing compiled AIR, tool registries, model configs, and policy environments |
| Host/runtime GC | If implemented in a GC language such as OCaml, or if embedding in a managed runtime |

These are implementation techniques, not Etas syntax. A Rust implementation can use arenas, `Rc`, `Arc`, object stores, or custom allocators without exposing Rust ownership to Etas users. An OCaml implementation can rely on OCaml's GC for temporary compiler/runtime values.

Etas source-level records, tuples, arrays, lists, maps, and sets have value semantics. The runtime may implement that efficiently with immutable structural sharing, reference counting, or copy-on-write. It does not need to eagerly deep-copy every argument. The only requirement is observational: passing a value to a callee must not let that callee mutate the caller's value unless the mutation goes through an explicit, analyzer-visible mutable API or a typed persistent resource such as `Store<K, V>`.

Persistent agent state is different. Garbage collection of heap objects does not decide how long an approval record, trace, checkpoint, memory version, or conversation turn remains available. Those objects need runtime policies:

```text
TraceRetention:
    keep full trace for 30 days
    compact model/tool payloads after 7 days
    redact Secret<T> fields before export
    keep approval records for audit policy duration

CheckpointRetention:
    keep latest successful checkpoint per run
    keep branch checkpoints while an approval or replay window is active
    delete expired checkpoints after durable trace commit

ConversationRetention:
    keep recent messages
    summarize older turns
    preserve message ids and provenance links
```

The runtime should make retention visible in traces and manifests where it affects replay, audit, or compliance. Compaction must preserve enough metadata for safe recovery: node ids, input hashes, memory versions, prompt hashes, model profiles, tool idempotency keys, approval ids, and session/message ids.

This gives Etas the right boundary:

```text
temporary value memory -> automatic implementation memory management
persistent agent state -> runtime retention, compaction, audit, and deletion policy
```

## 9. Toplevel And REPL

Etas should have a toplevel, but it should be analysis-first.

Useful commands:

```text
:type expr
:check file.es
:effects FlowName
:policy FlowName
:graph FlowName
:run FlowName
:run FlowName --dry-run
:resume checkpoint_id
:replay trace.json
```

The REPL may evaluate deterministic expressions:

```etas
> let x: i32 = 3
> normalize_title("Hello World")
> type Review = { score: i32, summary: string }
```

Flow execution should be explicit:

```text
> :run WritePaper --budget cost=$1.00,tokens=20000
```

The default REPL should not execute external effects unless the user starts an authorized runtime session. Agent calls, tool calls, file writes, network access, payments, and email sending should require explicit `:run` authorization or an approved runtime policy.

## 10. CLI Shape

The command-line interface can start with:

```bash
etas check app.es
etas graph app.es
etas run app.es
etas replay trace.json
etas repl
```

Later commands can include:

```bash
etas effects app.es
etas policy app.es
etas trace app.es
etas resume CHECKPOINT_ID
etas export-air app.es
```

## 11. Phased Plan

### 11.1 Phase 1: Frontend And Typed HIR Interpreter

Scope:

- parser and syntax diagnostics;
- HIR lowering with modules, symbols, scopes, source maps, and resolution diagnostics;
- type table and type checker;
- effect declarations and enough effect checking to prevent obviously unsafe execution;
- `flow`;
- return type inference for flows from `return` expressions and final expressions;
- `main`;
- typed HIR interpreter for deterministic local execution;
- minimal host tool registry for standard-library tool bindings such as `std.io.stdout.write`, `std.io.stderr.write`, and `std.io.stdin.read_line`;
- CLI commands for `check`, `dump ast`, `dump hir`, and typed-HIR interpreter smoke execution.

Goal: run small deterministic `.es` flows from typed HIR and validate source
semantics before committing to AIR execution shape.

Exit criteria:

- `etas check` validates syntax, names, types, and Phase 1 effect restrictions.
- `etas run --hir` or an equivalent explicit Phase 1 command executes local deterministic flows and the small set of registered Phase 1 host tools.
- unsupported agent/tool/runtime operations fail with stable diagnostics.
- examples include at least one checked and executed typed-HIR program.

### 11.2 Phase 2: AIR v0 And Runtime Execution Model

Scope:

- checked HIR facts needed by AIR lowering;
- AIR v0 builder;
- AIR node ids;
- executable AIR node arrays;
- value slot layouts;
- precomputed `next` successors;
- AIR interpreter;
- AIR verifier;
- runtime authority boundary;
- model provider abstraction;
- tool registry;
- structured output validation;
- trace logging;
- limit enforcement;
- effect/action, policy, approval, prompt taint, and memory hooks as explicit runtime checks.

Goal: execute real or dry-run agent/tool flows through AIR v0 with trace output
and explicit runtime mediation.

Exit criteria:

- `etas dump air` emits deterministic AIR v0 for supported examples.
- `etas run --dry-run` executes supported AIR v0 flows without real provider/tool side effects.
- runtime traces link events back to AIR node ids and source spans.
- AIR v0 unsupported constructs are rejected before execution.

### 11.3 Phase 3: FIR Middle-End And Agent Workflow Optimization

Scope:

- FIR builder from checked HIR;
- typed structured graph IR with value ids, data dependencies, regions, and effect summaries;
- FIR-to-AIR lowering;
- middle-end validation passes;
- context slicing;
- prompt partial evaluation;
- retrieval hoisting;
- agent call caching;
- verifier insertion;
- parallel scheduling for independent workflow branches;
- cost-aware planning;
- effect/action checks;
- prompt taint analysis;
- approval dominance checks;
- memory access checks;
- graph visualization based on FIR instead of AIR execution layout.

Goal: optimize and analyze agent workflows without overloading AIR v0 with
middle-end responsibilities.

Exit criteria:

- FIR dumps and golden tests are deterministic.
- optimized FIR lowers to equivalent AIR.
- workflow optimizations preserve policy, trace, replay, and checkpoint semantics.
- optimizer effects are visible in diagnostics or trace metadata where they affect runtime behavior.

### 11.4 Later Backends And Developer Tools

After the three phases stabilize, consider:

- analysis-first REPL;
- LSP completion, hover, definition, semantic tokens, and diagnostics over typed HIR/FIR facts;
- checkpoint/resume across process restart;
- replay and retention policies;
- distributed runtime backend;
- durable flow backend;
- Rust host integration;
- Python/TypeScript SDK generation;
- AIR bytecode or native backend.

Native code generation should remain a later, optional backend, not the initial implementation target.

## 12. Positioning

Etas should be:

```text
statically checked source language
executed through typed HIR in Phase 1
compiled to executable AIR v0 in Phase 2
optimized through FIR and lowered to AIR in Phase 3
with an analysis-first REPL
```

This gives Etas the ergonomics of a runnable language while preserving the static guarantees and runtime control needed for agent systems.
