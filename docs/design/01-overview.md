# Overview

## 1. Motivation

Modern multi-agent systems are often built as ad-hoc compositions of LLM calls, prompt templates, tools, memory stores, and flow frameworks. This style is flexible but has several fundamental problems:

1. **Agent behavior is difficult to reason about.**  
   The core computation is often hidden inside natural-language prompts and runtime callbacks.

2. **Tool use is dangerous without a formal effect model.**  
   An agent with access to filesystem, shell, browser, email, or payment APIs can perform high-impact actions.

3. **Prompt injection is a language-level problem.**  
   Untrusted text can flow into trusted instruction channels unless the system explicitly distinguishes control-plane prompts from data-plane content.

4. **Memory is usually implicit.**  
   Many frameworks treat memory as a magic context, making provenance, reproducibility, and debugging difficult.

5. **Multi-agent interaction is rarely typed.**  
   Agents pass strings to each other rather than typed values flowing through explicit orchestration.

6. **Flows are hard to verify.**  
   It is difficult to prove properties such as “no email is sent before human approval” or “no untrusted web content can modify system instructions.”

7. **Agent systems need compiler techniques.**  
   Context slicing, prompt specialization, effect checking, flow verification, trace abstraction, caching, partial evaluation, and optimization are all naturally compiler problems.

Therefore, Etas is designed as:

> **A general-purpose, effect-typed, trace-aware programming language where deterministic code, flows, agents, prompt values, tools, typed persistent memory APIs, trace specs, approvals, and traces are explicit, statically analyzable program elements.**

---

## 2. Design Philosophy

Etas is not merely a DSL for writing prompts. It is also not only a wrapper around Python classes. Its goal is to treat multi-agent systems as programs with explicit syntax, types, effects, and semantics.

The central idea is:

> Deterministic computation should be written as ordinary code.  
> Non-deterministic reasoning should be expressed through typed agents.  
> External actions should be mediated through typed tools and effects.  
> Multi-agent collaboration should be expressed as typed flows.  
> The entire system should produce traces that can be analyzed, replayed, audited, and abstracted.

Etas separates five kinds of computation:

| Computation form | Description |
|---|---|
| Pure deterministic code | Deterministic flows, expressions, data transformations |
| Effectful external computation | File I/O, network access, database access, shell execution, typed memory API access, time, and other runtime-mediated observations |
| Agent reasoning | Non-deterministic LLM-backed reasoning with structured output |
| Tool-mediated action | Model-callable operations performed under effect/action and trace-spec control |
| Human-mediated decision | Explicit approval or intervention points |

Etas's PL-level value should be judged along three main dimensions:

| Dimension | Core question | Etas's unique value | Gap in typical agent frameworks |
|---|---|---|---|
| **Safety** | What is this agent system allowed to do, and can it bypass a trace spec, leak data, or act without approval? | Effects, fine-grained actions, trace specs, approval gates, sandbox requirements, prompt trust labels, and typed tool declarations are compiler/runtime-visible semantics. Etas can reject or block flows whose possible behavior exceeds the declared authority boundary. | Python/TypeScript frameworks can add runtime guardrails and hooks, but behavior often hides inside callbacks, imports, closures, dynamic tool selection, or ordinary SDK clients. They usually cannot prove authority, approval, or prompt-trust properties across the whole flow unless they rebuild a restricted language layer. |
| **Reliability** | When a run crashes, retries, resumes, regenerates, or loops through agents, will it finish predictably and stay within its resource contract? | Determinism inference, typed `limit` contracts, AIR nodes, trace events, checkpoint metadata, memory versions, approval records, remaining budgets, and tool idempotency data give Etas explicit recovery semantics for non-deterministic agent execution. | Framework checkpoints can persist state, but they often treat nodes as opaque callbacks and resource limits as scattered runtime options. Without language-level determinism/effect/limit summaries, recovery policies are manual and can accidentally repeat model calls, re-request approvals, reset budgets after retry, or duplicate side effects such as email, payment, file writes, or publishing. |
| **Optimization** | How can an agent flow be optimized like a program, reducing token use, latency, duplicate retrieval, repeated inference, and unnecessary context? | Typed flows lower to AIR with data dependencies, effect summaries, prompt boundaries, typed memory API accesses, and composition structure. This enables context slicing, retrieval hoisting, prompt partial evaluation, cache planning, parallel scheduling, and cost-aware execution while preserving trace-spec and trace semantics. | Ordinary frameworks can hand-optimize individual chains, but composed agents often become opaque runtime objects. Without a compiler-visible IR and effect/determinism model, optimizations are local conventions rather than systematic transformations checked against safety and replay semantics. |

The static/dynamic boundary behind these dimensions is summarized in [Safety, Analysis, and Static-Dynamic Boundaries](14-safety-analysis-static-dynamic.md).

---

## 3. Why First-Class Non-Determinism Matters

Treating non-determinism as a first-class semantic property means the compiler and runtime can reason about agent behavior before execution. In a host-language DSL, an agent call is often just another callback or method call. Etas instead models agent inference, human approval, tool authority, effects, and traces as typed flow semantics.

### 3.1 Approval Before High-Impact Actions

Etas can reject unsafe flows before deployment:

```etas
spec SafeEmail: trace =
    Approval.request >> CompanyEmail.send<WorkAccount>;

flow NotifyCustomer(req: Request) ~ SafeEmail {
    let draft = req ~> SupportAgent ~> EmailWriter;
    return perform CompanyEmail.send(WorkAccount, draft);
}
```

The compiler can infer:

```text
NotifyCustomer:
  determinism = NonDeterministic
  effects = [CompanyEmail.send<WorkAccount>]
  requested_actions = [Agentic.infer<_, _>, CompanyEmail.send<WorkAccount>]
```

Because every path to `CompanyEmail.send<WorkAccount>` lacks a dominating approval, the program can be rejected or blocked from deployment. `CompanyEmail` is a package-defined authority effect, not a core standard-library effect. A Python or TypeScript DSL can intercept a specific runtime call, but it usually cannot prove this property across arbitrary host-language callbacks, closures, dynamic imports, and branch paths.

### 3.2 Bounded Non-Deterministic Loops

Agent loops need explicit limits because they can consume unbounded tokens, context, money, wall time, and tool calls:

```etas
flow Refine(draft: Draft) -> Draft {
    while not accepted(draft) {
        let review = draft ~> Reviewer;
        draft = { draft, review } ~> Rewriter;
    }

    return draft;
}
```

Etas can infer that the loop body is `NonDeterministic` and require:

```etas
while not accepted(draft)
    limit Iterations(5), Tokens(50_000), Cost(usd(3.00))
{
    ...
}
```

A host-language DSL can implement runtime budgets, but it usually cannot statically require every non-deterministic loop to carry a budget unless it heavily restricts the host language.

### 3.3 Deployment Manifests From Source Semantics

For a composed pipeline:

```etas
let publish = Researcher | Writer | Publisher;

flow Publish(topic: Topic) {
    let post = topic ~> publish;
    return blog.publish(post);
}
```

Etas can produce a deployment manifest before execution:

```text
flow Publish:
  determinism = NonDeterministic
  effects = [AcademicSearch.search, Blog.publish<Site>]
  requested_actions = [Agentic.infer<_, _>, AcademicSearch.search, Blog.publish<Site>]
  allowed_effects = [AcademicSearch.search, Blog.publish<Site>]
```

The deployment environment can deny missing action grants or require additional policy review before the flow runs. A DSL can log what happened during one run, but it is much harder to produce a reliable summary of all possible effects and authorities of a future run.

### 3.4 Safe Replay And Recovery

First-class non-determinism lets the runtime distinguish recomputable work from operations that must be replayed from trace:

```text
n1 = FunCall(normalize_topic)    Deterministic
n2 = AgentCall(Researcher)       NonDeterministic
n3 = ToolCall(web.search)        NonDeterministic
n4 = AgentCall(Writer)           NonDeterministic
n5 = Approval(...)
n6 = ToolCall(email.send)
```

On failure recovery, the runtime can:

```text
n1 recompute directly
n2 reuse the recorded model output unless the user explicitly requests resampling
n3 reuse or refresh according to replay policy
n5 reuse the recorded approval decision
n6 avoid sending the same email twice if the prior tool call succeeded
```

This is different from rerunning a normal host-language function, where retries can accidentally repeat non-idempotent actions such as sending email, charging payment, applying patches, or publishing content.

### 3.5 Precise Caching And Optimization

Determinism inference enables different caching and optimization rules:

```etas
flow Normalize(title: string) {
    trim(lowercase(title))
}

flow Research(topic: Topic) {
    return topic ~> Researcher;
}
```

The compiler can infer:

```text
Normalize:
  determinism = Deterministic
  effects = []

Research:
  determinism = NonDeterministic
  effects = []
  requested_actions = [Agentic.infer<Writer.run, _>]
```

`Normalize` can be inlined, memoized, constant-folded, or executed in a simple direct runtime. `Research` cannot be treated as a pure function; if cached, the key must include model, prompt, tools, memory snapshot, active trace spec, and trace mode.

### 3.6 Composed Pipelines Do Not Hide Risk

Composition remains analyzable:

```etas
let pipeline =
    Researcher
    | Writer
    | LegalReviewer
    | Publisher
    | email.send;
```

Etas can infer:

```text
pipeline: Topic -> Receipt
determinism = NonDeterministic
effects = [AcademicSearch.search, CompanyEmail.send<WorkAccount>]
requested_actions = [Agentic.infer<_, _>, AcademicSearch.search, CompanyEmail.send<WorkAccount>]
allowed_effects = [AcademicSearch.search, CompanyEmail.send<WorkAccount>]
```

The main benefit over a host-language DSL is not shorter syntax. It is that composition does not collapse into an opaque runtime object. The composed value still carries a statically visible behavior summary that can drive trace-spec checks, approval requirements, deployment gates, audit planning, caching, recovery, and optimization.

---

## 4. Why First-Class Agents Matter

An agent implemented as a Python or TypeScript class can still have typed inputs, typed outputs, tools, and callbacks. That is useful, but it is not the same as making `agent` a language construct. Etas's claim is that an agent is not just an object with a `run` method. It is a compiler-visible semantic node:

```text
typed input/output
+ model inference
+ non-determinism
+ tool authority
+ typed memory API access
+ message/session semantics
+ trace-spec boundary
+ trace event
+ effect upper bound
```

A host-language type checker usually sees only:

```text
run(input: I) -> O
```

It does not know whether that call may send email, write files, execute shell commands, read a persistent store, request human approval, perform a handoff, or move untrusted text into a trusted prompt channel. A framework can add runtime checks, but unless it heavily restricts the host language, arbitrary callbacks, closures, dynamic imports, and global clients can hide behavior from static analysis.

### 4.1 Agents Produce Manifests

Given:

```etas
@model("gpt-5.5")
@tools([web.search, paper.read])
agent Researcher(input: Topic) -> Notes {
    return perform infer<Notes>(ResearchPrompt(input));
}

@model("gpt-5.5")
agent Writer(input: Notes) -> Draft {
    return perform infer<Draft>(WritingPrompt(input));
}

flow Publish(topic: Topic) {
    let draft = topic ~> Researcher ~> Writer;
    return blog.publish(draft);
}
```

Etas can derive a deployment manifest before execution:

```text
flow Publish:
  determinism = NonDeterministic
  effects = [AcademicSearch.search, Blog.publish<Site>]
  requested_actions = [Agentic.infer<_, _>, AcademicSearch.search, Blog.publish<Site>]
  @tools([web.search, paper.read, blog.publish])
  allowed_effects = [AcademicSearch.search, Blog.publish<Site>]
```

This is stronger than logging one observed run. The manifest summarizes what future runs may do, so deployment can deny missing action grants, require review for high-impact effects, or choose a sandboxed runtime plan before the flow starts.

### 4.2 Agents Have Tool Boundaries

An agent declaration is an authority boundary:

```etas
@model("gpt-5.5")
@tools([orders.lookup, refund.request])
agent RefundAgent(input: Message<RefundRequest>) -> Message<RefundReply> {
    return perform infer<Message<RefundReply>>(RefundPrompt(input));
}
```

The runtime can deny any undeclared tool request before execution:

```text
requested tool = email.send
declared tool surface = @tools([orders.lookup, refund.request])
decision = ToolDenied
```

In a host-language class, this guarantee depends on whether all external clients are routed through the framework. If the implementation can access arbitrary globals, imports, environment variables, or SDK clients, the framework's tool table is a convention rather than a complete authority boundary.

### 4.3 Agents Carry Message And Handoff Semantics

Agent systems are not only function calls. They pass typed messages through conversations:

```etas
flow CustomerTurn(msg: Message<CustomerRequest>) -> Message<CustomerReply> {
    let route = msg ~> TriageAgent;

    match route.target {
        Refund => return msg.cast<RefundRequest>() ~> RefundAgent;
        Sales => return msg.cast<SalesRequest>() ~> SalesAgent;
    }
}
```

Because `agent`, `Message<T>`, and `~>` are visible to the compiler and AIR, Etas can preserve:

```text
MessageCreate
MessageAppend
Handoff(from = TriageAgent, to = RefundAgent, message = msg)
AgentCall(RefundAgent)
```

The trace can record sender, receiver, session id, role, payload type, provenance, and handoff metadata. A class-based framework can emit similar runtime events, but the host language usually cannot prove that all communication went through those events.

### 4.4 Agents Support Prompt Trust Checking

Prompt construction is a support type, but agent declarations give the compiler a place to check prompt boundaries:

```etas
flow BadPrompt(x: Untrusted<string>) -> Prompt {
    return Prompt.new()
        .system(x); // rejected: untrusted data enters control plane
}

@model("gpt-5.5")
agent Summarizer(input: Untrusted<string>) -> Summary {
    return perform infer<Summary>(SafeSummaryPrompt(input));
}
```

Because agent calls lower through `PromptBuild` and `AgentCall`, the compiler can analyze whether untrusted data is placed into system instructions, user content, tool arguments, or persistent stores. A host-language framework can validate prompt objects at runtime, but static information-flow checking across arbitrary string operations is much weaker.

### 4.5 Agents Improve Replay, Cache, And Recovery

Etas can distinguish ordinary deterministic work from agent inference:

```etas
flow Normalize(x: string) {
    trim(lowercase(x))
}

flow Draft(topic: Topic) {
    topic ~> Writer
}
```

The compiler infers:

```text
Normalize = Deterministic, effects = []
Draft     = NonDeterministic, effects = [], requested_actions = [Agentic.infer<Writer.run, _>]
```

Recovery can recompute `Normalize`, but `Agentic.infer<Writer.run, Draft>` may
need to be replayed from trace, resampled intentionally, or cached using a key
that includes model, prompt, tools, memory snapshot, active trace spec, and session state.
Treating both as ordinary method calls makes this distinction a manual
convention.

### 4.6 The Boundary

Other frameworks can approximate some of these benefits with restricted DSLs, decorators, schema metadata, mypy plugins, tracing hooks, or runtime guardrails. At that point, however, they are rebuilding a small language layer inside the host language.

Etas makes the layer explicit:

| Feature | Agent class in a host language | Etas `agent` |
|---|---|---|
| Input/output type checking | Usually possible | Built into declarations |
| Structured output validation | Usually possible | Built into lowering/runtime |
| Effect inference | Usually missing | Core semantic analysis |
| Effect/action manifest | Usually partial or runtime-derived | Compiler output |
| Approval dominance checks | Hard across arbitrary callbacks | AIR/control-flow analysis |
| Non-determinism classification | Usually manual | Inferred |
| Prompt trust checking | Runtime convention or custom lint | Type/effect analysis target |
| Message/handoff trace semantics | Framework event | AIR/trace semantic event |
| Hidden tool prevention | Depends on framework encapsulation | Language/runtime boundary |
| Replay/cache/checkpoint classification | Manual strategy | Derived from semantics |

The benefit of first-class `agent` is therefore not shorter syntax. It is that agent behavior becomes part of the program's analyzable semantics before execution.

---

## 5. Language Layers And Keyword Budget

Etas should stay small at the core. The five-layer classification in section 5.3
governs the terminology used throughout this SPEC. A name being outside the
keyword set does not imply that it is an ordinary library definition with no
compiler or runtime contract. API ownership and implementation support are
separate questions.

### 5.1 Surface Syntax Keywords

The current source grammar has **36 surface syntax keywords** if primitive type names are not counted as keywords.

| Group | Keywords | Count |
|---|---|---:|
| Modules and imports | `module`, `import`, `as` | 3 |
| Declarations and visibility | `alias`, `type`, `enum`, `spec`, `impl`, `private`, `public` | 7 |
| Flows, effects, handlers | `flow`, `effect`, `extends`, `action`, `perform`, `handle`, `handler`, `with`, `resume`, `finish` | 10 |
| Tools | `tool` | 1 |
| Agents | `agent` | 1 |
| Protocols | `protocol` | 1 |
| Bindings and control flow | `let`, `var`, `if`, `else`, `match`, `for`, `in`, `while`, `limit`, `retry`, `return`, `break`, `continue` | 13 |
| **Total** |  | **36** |

Primitive type names add another **20 reserved type names** if the lexer treats them as keywords: `bool`, signed and unsigned integer widths, `f32`, `f64`, `char`, `string`, `bytes`, `unit`, and `never`. Boolean literals `true` and `false` add 2 more literal words. Pascal-case names such as `Prompt`, `Message`, `SessionConfig`, `Conversation`, `SandboxProfile`, and `ApprovalRequest` should be Agent/runtime support names, not syntax keywords.

Inside `agent` declarations, model, tools, limits, tracing, and optimization
metadata are expressed with annotations such as `@model(...)`, `@tools(...)`, and
`@limits(...)`, not config-row keywords. The agent output type is already
declared by `agent Name(input: I) -> O`, so there is no separate `output` clause.
The ergonomic agent body is a flow-like default `run` implementation. Model
inference is written with agent-scoped `perform infer<T>(prompt)`, which
elaborates to `Agentic.infer<Agent.method, T>` and is handled by the runtime
agent provider.

### 5.2 Semantic And Type-System Names

This is a representative inventory, not a fixed count of language constructs or
an exhaustive std API catalog. Standard names may have compiler-recognized
contracts, as listed in section 5.4; their presence in type checking does not
make each name a core-language construct.

| Category | Names | Ownership |
|---|---|---|
| Primitive types | `bool`, integer widths, `f32`, `f64`, `char`, `string`, `bytes`, `unit`, `never` | Language-defined types; representation and execution need backend support. |
| Standard data and collection types | `Array<T>`, `List<T>`, `Map<K, V>`, `Set<T>`, `Range<I>`, `Slice<T>`, `Deque<T>`, `Queue<T>`, `Stack<T>`, `PriorityQueue<T, P>`, `OrderedMap<K, V>`, `OrderedSet<T>`, `Option<T>`, `Result<T, E>` | std definitions and APIs; literal, indexing, and error-conversion integrations have specified compiler contracts. |
| Standard specs | `Index`, `ByteStream`, `Region`, `Within<Parent>`, `PromptEncode`, `Schema`, `ResponseDecode`, `Limit`, `Stage<I, O, effect E>`, `Pure<I, O>` | std spec declarations, using the language's spec/evidence mechanism; selected integrations are compiler-recognized. |
| Persistent resource support | `MemoryRegion<S>`, `Store<K, V>`, `std.memory.region<S>(...)` | std API with compiler-recognized resource identities and runtime storage support. |
| Trust/security wrappers | `Trusted<T>`, `Untrusted<T>`, `Secret<T>`, `Public<T>`, `Sanitized<T>` | std support types with the specified trust/secret analysis and runtime enforcement contracts. |
| Agent support types | `Prompt`, `PromptPart`, `ModelResponse`, `Message<T>`, `MessageContent`, `Role`, `SessionConfig`, `Conversation`, `SandboxProfile`, `ApprovalRequest`, `ApprovalDecision` | std definitions and APIs used by agent and runtime contracts, not independent source constructs. |
| Standard support values | `Sandbox`, `DefaultCommandSandbox`, `Iterations`, `Tokens`, `ContextTokens`, `Cost`, `WallTime`, `Attempts`, `Concurrency` | std values and constructors; sandbox, limit, and scheduling enforcement belongs to the compiler/runtime contract. |
| Standard support flows | `std.ui.approve`, `abort` | std APIs, with approval-action and non-returning behavior respectively. |
| Standard effect roots | `Agentic`, `Network`, `FileIO`, `Command`, `Memory`, `Secret`, `Time`, `Human`, `Error<E>` | Standard effect declarations and registered contracts; effect declaration, extension, rows, and handling are language mechanisms. |
| Reflection support | `AgentDef<A>`, `CallableDef<F>`, `CallableDef`, `SyntaxBlock`, `CheckReport` | std support contracts in the reflection extension; not a claim of current implementation support. |
| Runtime event names | `Handoff` | Runtime/trace vocabulary, not a declaration keyword. |
| Internal type constructors | `Flow<I, O, E>`, `AgentComponent<G>`, `AgentMethod<I, O, E>` | Compiler/runtime representation, not public std APIs. |

`Flow<...>`, `AgentComponent<...>`, and `AgentMethod<...>` are compiler/runtime
representations. `AgentComponent<G>` carries stable nominal agent identity and
checked annotation metadata, while `AgentMethod<I, O, E>` summarizes callable
agent methods such as generated `run`. Users should normally write `flow`,
`agent`, and `impl agent` declarations and let the compiler infer these internal
representations.

### 5.3 Feature Map By Layer

| Layer | Examples | What The SPEC Defines |
|---|---|---|
| Language mechanisms | Modules/imports/visibility; `type`, `alias`, `enum`, records, generics and existential spec objects; `spec`/`impl` and `~`; `flow`, `agent`, `tool`; `effect`, `action`, effect rows, `perform` and handlers; bindings/control flow, `?`, `limit`, `retry`, stage composition/application, and annotation syntax | Grammar, typing/effect rules, evaluation, control flow, conformance, and scope propagation. `protocol` remains an advanced construct; `^` belongs to the separately specified reflection extension. |
| Compiler-recognized std contracts | `Index`, the `Result`/`Error` integration with `?`, `Limit`, region/resource identities, `infer`, recognized annotations, structured-concurrency APIs, and typed reflection descriptors | Standard symbol identities and the checking/elaboration obligations that cannot be obtained merely by declaring an ordinary same-named type or flow. These contracts do not prescribe that the entire implementation live inside the compiler. |
| Ordinary std APIs | Collection algorithms, text processing, HTTP/text codecs, prompt/message construction helpers, `std.io.println`, and flow wrappers over standard actions | Public types, signatures, effects, errors, and behavior implemented through ordinary Etas definitions and the specified substrate. A wrapper inherits the contracts of what it calls; its name alone adds no magic or authority. |
| Runtime support | External I/O, model providers, host endpoints for standard actions, task/handler execution, memory/resource management, structured-output validation, session/trace events, sandboxing, cancellation, budgets, and replay | Concrete execution and enforcement of the language and standard contracts across target platforms. Default handlers may combine Etas code with these primitive endpoints. |
| EDK / application libraries | HTTP clients, browser automation, business tools, evaluation harnesses, tenant context, domain events, outbox, and application state machines | Ordinary packages built on public std APIs, effects, tools, and specs; no extra compiler privileges or hidden host escape hatches. |

These layers distinguish responsibilities, not disjoint implementation modules.
For example, `join` is a std API with a compiler-recognized contract and runtime
scheduling support. `Prompt` is a std type used at an agent boundary, not another
language declaration form. A prelude is an import/export convenience for standard
names, not a sixth semantic layer or a source of authority. Internal HIR/FIR/AIR
representations are implementation details, not additional source constructs.
Feature status remains defined in its own chapter; this table does not imply
that every listed design target is implemented.

The main static classification is:

```text
flow + Deterministic          = lowerable ordinary computation
flow + NonDeterministic       = external state, tools, memory APIs, time, command, agent, approval,
                                model inference, or nondeterministic subflow orchestration
```

`flow` is the ordinary user-defined callable declaration in the source language. The compiler infers each flow's determinism class and effect row. Deterministic flows can lower to ordinary internal functions. Non-deterministic flows require runtime mediation for effects, trace, trace-spec checks, replay, and checkpointing as needed. A `tool` with a body is also implemented in Etas, but it is a model-callable boundary rather than a general helper callable. In type positions, a flow value is written with arrow syntax such as `I -> O` or `I -> O ![E]`; the compiler should infer that type in most ordinary code. The compiler normalizes flow types to the canonical internal representation `Flow<I, O, E>`. Agent communication is represented with typed Agent/runtime support values such as `Message<T>`, `SessionConfig`, and `Conversation`, not with `msg` or `message` keywords. Program execution starts when the runtime calls `main(args: Array<string>) -> i32`; the returned `i32` is the process status code.

### 5.4 Compiler-Recognized Standard Contracts

The following inventory identifies the integrations specified across the design
chapters. Recognition attaches to the resolved standard declaration and its
contract, not to an unqualified spelling. Import aliases and re-exports preserve
that identity; unrelated same-named declarations do not acquire its special
behavior. Adding ordinary std APIs does not automatically extend this inventory.

| Standard Contract | Compiler / Runtime Obligation | What Can Remain Etas Code |
|---|---|---|
| `Result<T, E>`, its `Ok`/`Err` variants, and `Error<E>.raise` | `?` uses these resolved standard definitions and captures exactly one residual `Error<E>` as specified in the error chapter. `never` is the language bottom type; `abort` is a non-returning support API, not a keyword. | ADT and effect declarations, Result methods, error mapping, and handler composition. |
| Collection literal targets, checked indexing, `Index` | Give literals their specified standard types, preserve value semantics, and enforce checked indexing without general implicit integer conversion. | Collection algorithms and methods over the defined storage primitives; not every collection method is intrinsic. |
| `Limit` and standard limit constructors | Check the `limit` clause and enforce its scope, accounting, exhaustion, and non-resetting nested budgets. | Value definitions, budget configuration, and ordinary helper flows. |
| `MemoryRegion<S>`, `Store<K, V>`, `std.memory.region`, and resource-indexed actions | Preserve stable resource identity, region-sensitive effect facts, and the top-level resource-initialization exception. | Schemas, storage helpers, and source-level wrappers; persistence itself uses the declared substrate. |
| Agent contracts: `infer`/`Agentic.infer<C, O>`, schema/encoding specs, trust/secret wrappers, and recognized annotations | Elaborate agent-scoped inference with method identity and output schema; preserve tool boundaries, trust checks, requested actions, and runtime enforcement. User metadata annotations do not create these obligations or permissions. | Prompt/message construction, codecs, validators, agent bodies, and ordinary routing/handler logic. |
| Structured-concurrency combinators and checkpoint APIs | Preserve branch effects, structured lifetimes, handler/spec/limit propagation, and recovery metadata; do not treat them as arbitrary opaque callbacks. | Combinator and recovery policy logic over the specified scheduling/checkpoint primitives. |
| Reflection extension: `AgentDef<A>`, `CallableDef<F>`, erased `CallableDef`, and `Code.check/compile` | Preserve static target types, descriptor-view rules, and checked frozen-contract compatibility. Runtime signature data alone cannot establish a static callable type. | Definition-processing helpers, source transformations, diagnostics, and checker/compiler algorithms, with the validated execution boundary preserved. |

The relevant chapters define each contract; this inventory introduces no new
keywords, coercions, or authority. Standard-library implementations must obey
normal type, effect, handler, trace-spec, and runtime-policy checks. Requested
actions do not disappear merely because their callers or handlers live in std.

### 5.5 Implementing Std In Etas

The design target is to implement standard-library definitions and expressible
algorithms in Etas itself. Compiler recognition specifies identity and semantic
contracts, not a requirement to hard-code whole APIs in the compiler or host.
For example, `Result` can be an Etas enum recognized by `?`, and `std.io.println`
can be an Etas flow over `Console.stdout_write`.

Runtime primitives remain necessary for platform I/O, execution and allocation,
resource handles, continuation/task machinery, and protected service boundaries.
Their public contracts and backend bindings must be explicit in standard/package
metadata. This uses the existing binding mechanism, not a new source keyword.
Ordinary algorithms must not become private host calls solely to compensate for
missing Etas expressiveness. A required primitive must be identified as such,
with its types, observable actions, errors, and enforcement contract.

Pure codecs and collection algorithms are not effectful simply because a
backend optimizes their execution. Conversely, an I/O or secret operation must
not lose its action visibility when its wrapper is implemented in Etas or
optimized natively. Library default handlers can compose lower-level actions;
only the actual platform boundary requires a platform implementation. Neither
std membership nor compiler recognition is an authorization grant.

Standard-library self-implementation and compiler self-hosting are separate
goals. A future Etas-written compiler may implement the algorithms behind
`std.compiler.check/compile` without changing those APIs or their validation
obligations. This layering makes that possible; it does not establish that the
current implementation or SPEC already covers every needed library. Writing
and testing std and EDK in Etas is how missing language/substrate capabilities
should be exposed and resolved.

---

## 6. Core Language Overview

Etas keeps ordinary programming and agent orchestration in one source language:

```etas
type SurveyInput = { topic: string, papers: Array<Paper>, critique: Review }

@model("gpt-5.5-thinking")
@tools([web.search, paper.read])
agent Researcher(input: string) -> Array<Paper> {
    let prompt = Prompt.new()
        .system(Trusted("Find relevant papers and return structured metadata."))
        .data(input);
    return perform infer<Array<Paper>>(prompt);
}

@model("gpt-5.5-thinking")
agent Reviewer(input: Array<Paper>) -> Review {
    let prompt = Prompt.new()
        .system(Trusted("Critique citation quality and novelty."))
        .data(input);
    return perform infer<Review>(prompt);
}

@model("gpt-5.5")
agent Synthesizer(input: SurveyInput) -> Survey {
    let prompt = Prompt.new()
        .system(Trusted("Write a concise literature survey."))
        .data(input);
    return perform infer<Survey>(prompt);
}

flow LiteratureSurvey(topic: string) -> Survey {
    let papers = Researcher.run(topic);
    let critique = Reviewer.run(papers);
    return Synthesizer.run({ topic, papers, critique });
}
```

The important point is that the agent call is not merely a function call. It is an effectful, trace-producing, potentially non-deterministic computation.

---

## 7. Core Abstractions

The core language abstractions include `flow`, `agent`, `tool`, `spec`
constraints including trace specs, effects/actions, and handlers. They operate
with standard types such as `MemoryRegion<S>`, `Store<K, V>`, `Prompt`,
`Message<T>`, `SessionConfig`, and `ApprovalRequest`, plus runtime trace events.
These are cooperating layers, not a list of additional language constructs.
`Prompt` builds model-call inputs; message and session support types carry
agent-to-agent communication. Explicit `protocol` declarations are useful for
advanced decentralized communication, but they are not required for the minimal
language because typed flows and typed messages already define an analyzable
interaction graph.

---
