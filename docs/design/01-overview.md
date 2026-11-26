# Etas Language Specification

Etas combines ordinary value-oriented programming with typed active agents,
effect actions, scoped handlers and trace constraints. This SPEC defines source
syntax and observable behavior, not compiler architecture or implementation
status. The [index](README.md) identifies each contract's owning chapter.

## 1. Core Constructs

| Construct | Contract |
|---|---|
| `type`, `alias`, `enum` | Nominal/aliased values, products and recursive sums. Record construction supports field defaults and recursive type defaults. |
| `let`, `var` | Immutable or rebindable bindings; records/collections retain value semantics. Module bindings are read-only. |
| `flow` | Ordinary callable; return type and effects are inferred unless constrained. |
| `agent` | Nominal type with isolated state, scoped identity and checked, suspendable methods; no implicit worker or generated `run`. |
| `tool` | Explicit model-callable boundary; ordinary source bodies may delegate to flows or agents. |
| `effect`, `action`, `perform` | Effect tags, operation signatures and checked requests. |
| `handler`, `handle`, `with` | First-class handlers; tail values implicitly `resume`, `finish` exits early. Resumption is single-shot; `as k` exposes a scoped continuation. |
| `spec`, `impl`, `~` | Type, callable and trace constraints; generic evidence and explicit implementations. |
| `? ~ Spec` | Existential value retaining checked spec evidence and scope restrictions. |
| `?` | Convert exactly one residual `Error<E>` family to `Result<T, E>`. |
| `A \| B`, `~>`, `+` | Compose/apply stages; Handler `h1 ~> h2` nests inner-to-outer, `h1 + h2` merges one layer. Install with `with`. |
| `@limits(...)` | Scoped resource bounds; nested and concurrent work share applicable parent accounts. |
| `@inject` / `@inject(Action)` | Resolve omitted fields at construction, parameters at calls; explicit values win. Tool injected inputs are model-hidden. |
| `protocol` | Role/message declaration; not a replacement for trace specs. |
| `^` | Reflection extension over retained definitions; checked compilation preserves the original contract. |

## 2. Execution Boundaries

| Boundary | Rule |
|---|---|
| Program entry | Execute `main(args: Array<string>) -> i32`; do not execute arbitrary expressions at module load. |
| Ordinary call | Execute in the calling task. Passive `var self` is a restricted, non-suspending local update. |
| Agent call | Execute in the caller's task with shared-read/exclusive-write access, including across suspension; no mailbox or conflict queue. |
| Agent identity | Copying a reference shares the instance, not its private state or lifecycle ownership. No OS-process or dedicated-thread requirement. |
| Effects | `![...]` bounds escaping effects. Requested-action and trace facts retain handled behavior; neither a row nor a handler grants authority. |
| Determinism | Infer `Deterministic` or `NonDeterministic` from reachable behavior and state dependencies. An empty escaping row alone proves neither purity nor safe parallelism. |
| Trace constraints | Scope constraints narrow monotonically across calls and delegation; handling cannot erase an attempted action or grant approval. |
| Failure | Typed recoverable failures use `Error<E>`; cancellation, budget exhaustion and non-returning termination retain their specified boundaries. |

## 3. Keywords

| Group | Keywords | Count |
|---|---|---:|
| Modules/imports | `module`, `import`, `as` | 3 |
| Declarations/visibility | `alias`, `type`, `enum`, `spec`, `impl`, `private`, `public` | 7 |
| Effects/callables | `flow`, `effect`, `extends`, `action`, `perform`, `handle`, `handler`, `with`, `resume`, `finish` | 10 |
| Domain constructs | `tool`, `agent`, `protocol` | 3 |
| Bindings/control | `let`, `var`, `if`, `else`, `match`, `for`, `in`, `while`, `retry`, `return`, `break`, `continue` | 12 |
| Total | Excludes primitive type names and boolean literals | 35 |

`self` is contextual. Annotation names, action names, `infer`, `join`,
`Async`, `Prompt`, `Session` and collection names are not additional keywords.
Primitive type names are listed in [General Constructs](02-general-programming-constructs.md#2-primitive-types).

## 4. Language And Library Boundaries

| Layer | Owns |
|---|---|
| Language | Grammar, kinds/types, binding/value rules, effects/handlers, agent access, scope and constraint semantics. |
| Recognized standard contracts | Symbol-specific rules for literals/indexing, `Result`/`Error` with `?`, `Index`, `Eq`, numeric specs, annotations, resource identities, active/inference boundaries, structured tasks and reflection descriptors. |
| Ordinary standard APIs | Public types, flows, codecs, collections and wrappers; their names alone confer no special privilege. |
| Execution support | Observable action dispatch, resource cleanup, cancellation, budgets, sandbox checks and protected evidence. This SPEC does not prescribe internal modules, IR or memory representation. |
| Application packages | Domain actions, provider handlers, HTTP clients, browser automation and other integrations built from public contracts. |

Recognition uses resolved symbol identity, preserved through imports and aliases,
not matching text. In particular:

| Contract | Required distinction |
|---|---|
| `Result<T, E>` / `Error<E>` | `?` is bound to their standard identities; unrelated same-named types do not participate. |
| `@inject` / `@limits` | Checked dependency/resource semantics, not optional metadata. |
| `Agentic` / `std.agent.Active` | Protected method/instance identity and dispatch; ordinary records cannot forge them. |
| `std.task` / resource handles | Preserve task ownership, scope, capture and one-use transfer obligations through abstraction. |
| Reflection descriptors | Runtime signature data is descriptive; it cannot forge a static callable type or bypass checked compilation. |

An API's inclusion in this specification does not assert its availability in a
particular implementation. Public signatures and failure semantics are defined
in [Standard Library](19-standard-library.md).
