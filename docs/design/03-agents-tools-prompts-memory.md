# Agents, Tools, Messages, Prompt Values, and Persistent Memory APIs

## 1. Agent

### 1.1 Agent as a First-Class Construct

An agent is a first-class nominal component. It has:

- a stable identity;
- model/runtime annotations;
- exposed model tools;
- attached trace specs and runtime-policy metadata;
- retry, limit, trace, and replay behavior;
- optional methods implemented with `impl agent`;
- an optional default `run` method generated from the ergonomic agent-body form.

The source shape is:

```text
@model(...)
@tools([...])
agent Name;

agent Name(input: I) -> O {
    ...
}
```

Annotations hold static metadata such as model choice, model-visible tools,
default limits, trace mode, retention profile, optimization hints, and
documentation tags. These names are annotation paths, not language keywords.
`@limits(...)` is a mandatory resource contract: an agent's declaration bounds
apply per method invocation, including generated `run`, not when the nominal
component is declared. A method's own resource annotation conjoins with the
agent and caller bounds; it cannot override them. Parameter-dependent budgets
use local annotated blocks. See [Resource Contracts](04-flows-human-gates-and-protocols.md#18-resource-contracts-and-budgeted-loops).

`agent Name;` declares only the nominal component. The ergonomic form
`agent Name(input: I) -> O { ... }` declares the same component and provides a
default `run` implementation. Its body is checked like a flow body, except that
agent-scoped `perform infer(prompt)` is available. Effect contracts remain in the
`![...]` position after the output type, not in annotations.

`infer` is the full-loop convenience action. An agent method or a handler for
its inference request may instead control individual generations through
`std.agent.step`, receive tool requests without executing them, and dispatch
selected calls with `std.tool.invoke`. This uses the same agent identity, tool
surface, effects, policy, limits, and trace checks, not a second untyped tool
registry. The transparent protocol types and APIs are specified in
[Single-Step Agent Interaction](19-standard-library.md#4-single-step-agent-interaction).

The purpose of `agent` is not to copy SDK object configuration into source
syntax. Frameworks such as LangChain, Google ADK, and AutoGen already model
agents as host-language objects with a model, instructions, tools, and runtime
harness. Etas keeps `agent` only because the declaration creates a semantic
boundary that the compiler and runtime can analyze:

| Surface fact | Semantic use |
|---|---|
| nominal agent identity | stable trace identity, runtime-profile target, runtime-policy target, and replay key |
| default `run` method | stable public callable entrypoint, requested-action recording, replay, and stage composition |
| `perform infer(prompt)` | language-visible model inference boundary, schema derivation, typed decoding, trace, replay, mock/resample, and optimization |
| ordinary agent method body | context slicing, memory selection, prompt-channel trust checks, post-inference checks, and local normalization |
| `impl agent` methods/specs | reusable agent behaviors, explicit method surface, and spec evidence |
| `@tools(...)` surface | model-callable tool set, least-privilege checking, and runtime tool denial |
| inferred effects | trace-spec checks, runtime-policy checks, approval, deployment manifest, and replay planning |
| `Agentic.infer<A.method, O>` requested action | agent-call trace, mock/replay/resample, provider routing, and agent fusion |
| limits and trace metadata | bounded non-deterministic execution and audit |

If an `agent` declaration cannot produce these summaries, it is only a framework
configuration object and does not justify a language construct.

Example:

```etas
type Review = {
    summary: string,
    strengths: Array<string>,
    weaknesses: Array<string>,
    required_revisions: Array<string>,
    score: i32,
    confidence: f64
}

@model("gpt-5.5-thinking")
@tools([latex.check])
@limits([ContextTokens(32_000), Tokens(8_000)])
agent Reviewer(input: Draft) -> Review ![FileIO, Error<StorageError>, Error<BudgetExceeded>] {
    let papers =
        ProjectMemory.Papers
            .related_to(std.memory.query("project.draft-similarity", 1, input))
            .limit(Tokens(16_000));

    let prompt = Prompt.new()
        .system(Trusted(
            "You are a strict POPL reviewer. Focus on soundness, novelty, formal clarity, and evaluation."
        ))
        .data(input)
        .data(papers);

    return perform infer<Review>(prompt);
}
```

Output range checks should be expressed with schema metadata or ordinary
validation flows:

```etas
flow validate_review(review: Review) -> Result<Review, ValidationError> {
    if review.score < 1 || review.score > 10 {
        return Err(ValidationError.InvalidScore);
    }

    if review.confidence < 0.0 || review.confidence > 1.0 {
        return Err(ValidationError.InvalidConfidence);
    }

    return Ok(review);
}
```

### 1.2 Agent Type

The compiler represents an agent declaration as a nominal component. If the
declaration uses the ergonomic body form, the compiler also generates a public
callable `run` method:

```text
Reviewer : AgentComponent<ReviewerConfig>
Reviewer.run : Draft -> Review ![FileIO, Error<StorageError>, Error<BudgetExceeded>]
```

The generated `run` entrypoint is what participates in ordinary calls and stage
composition. It is also the value that can satisfy callable specs such as
`Stage<I, O, E>`:

```text
Reviewer.run ~ Stage<Draft, Review, FileIO>
```

This representation is not intended to dominate source-level syntax. Users
normally declare agents with annotations and either an ergonomic body or explicit
`impl agent` blocks; the compiler infers the corresponding component summary,
method summaries, and generated `run` type.

The ergonomic declaration:

```etas
agent Reviewer(input: Draft) -> Review {
    return perform infer<Review>(ReviewPrompt(input));
}
```

desugars to:

```etas
agent Reviewer;

impl agent Reviewer ~ Stage<Draft, Review, E> {
    flow run(input: Draft) -> Review ![E] {
        return perform infer<Review>(ReviewPrompt(input));
    }
}
```

`impl agent` may define inherent methods or implement additional specs:

```etas
impl agent Reviewer {
    flow explain(input: Draft) -> Explanation {
        return perform infer<Explanation>(ExplainPrompt(input));
    }
}
```

Unlike a pure deterministic computation:

```text
f : A -> B
```

an agent method is better understood as a checked computation with ordinary
escaping effects, requested actions, determinism metadata, and trace semantics:

```text
A.run : Input -> AgentRun<Output, Effects, RequestedActions, Determinism, Trace>
```

Most useful agents perform at least one model inference and are therefore
`NonDeterministic`, but that fact is inferred from `perform infer` and the method
body rather than from the mere presence of the `agent` keyword.

### 1.3 Operational Intuition

An agent call may:

1. run an agent method body such as `Reviewer.run(input)`;
2. read typed memory APIs or conversation state while building context;
3. perform one or more agent-scoped `perform infer(prompt)` operations;
4. allow the model to request exposed tools from `@tools(...)`;
5. decode and validate structured model output;
6. run ordinary post-inference code in the method body;
7. emit a trace;
8. retry or fail.

These describe the complete-loop path. A single `std.agent.step` stops after
one model generation; the Etas controller owns tool execution, continuation,
validation/repair strategy, and delegation. Returned `ToolCall` and `ToolResult`
records are ordinary data, not permission tokens or execution evidence.

Therefore, an agent is not semantically equivalent to a plain SDK object. Its
methods still use the same effect and determinism analysis as flows, with
agent-specific requested actions added at explicit model-inference boundaries.

An `agent` method body is flow-like code scoped to the current agent component.
It may select, summarize, sanitize, and encode context, perform model inference,
and normalize or repair the returned typed value. Ordinary multi-agent
orchestration, approval gates, high-impact tool calls, persistent memory writes,
publishing, and other durable business side effects should normally remain in
surrounding `flow` code unless they are intentionally part of a specific agent
method's public behavior.

The inferred source-level effect of `Reviewer.run(input)` is:

```text
effects(agent run body)
∪ possible effects of tools exposed by @tools(...)
∪ context/session/typed-memory support effects
```

Explicit inference/step operations contribute their corresponding
`Agentic.infer<C, O>` or `Agentic.step<C, O>` requested actions for analysis,
trace, replay, and runtime checks. Declaring an agent alone contributes neither
action. The default-handled action does not become an escaping effect obligation
of ordinary callers.

If an agent declaration includes an explicit `![...]` effect contract after the output type, the inferred ordinary escaping effect set must be a subset of that declared upper bound. If no explicit effect contract is present, the compiler infers the ordinary escaping effects and public package metadata records them.

---

## 2. Tool

### 2.1 Tool Declaration

A tool is a model-callable action boundary. It is the only source construct that
can be listed in an agent's `@tools(...)` annotation and exposed to the model as a
callable schema. A `flow` is ordinary Etas logic and cannot be exposed to the
model directly.

Tool implementations and tool interfaces use the same `tool` name:

| Form | Body | Implementation | Purpose |
|---|---:|---|---|
| `tool ... { ... }` | Yes | Etas source | A safe model-callable wrapper around ordinary Etas logic, trace-spec/runtime-policy checks, validation, context narrowing, and calls to other tools |
| `tool ...;` | No | Package interface or compiler/runtime metadata | A typed imported tool signature for a standard-library primitive, host-provided binding, generated API, or precompiled package item |

Host bindings are not declared with a source-level keyword in the MVP.
Standard-library primitive symbols and host-provided package bindings are
described by compiler/runtime metadata or package manifests, then imported and
called like ordinary package symbols. Their interface metadata must include
type, effect row, determinism class, schema requirements, and runtime binding
information. Ordinary implementation source should prefer `tool ... { ... }`
or `import` an existing bodyless signature from a package interface.

This gives `tool` a clear boundary: it is not just a renamed `flow`; it is an
LLM action boundary with schema generation, effect/action/trace-spec/runtime-policy checks,
trace visibility, argument validation, and output-leakage restrictions.

Effect actions are the primitive runtime authority boundary. Minimal standard
operations such as console I/O, approval, command execution, memory access, and
time are standard actions. High-level integrations such as web search, workspace
workflows, email, databases, browser automation, and payments use package-defined
actions over that substrate. Library APIs then
provide ergonomic flow wrappers over `perform`.

```etas
effect AcademicSearch extends Network {
    action search(q: string) -> Array<Untrusted<WebPage>>;
}

flow academic.search(q: string) -> Array<Untrusted<WebPage>> ![AcademicSearch.search] {
    return perform AcademicSearch.search(q);
}
```

Etas-implemented tools use ordinary Etas bodies. Their effects are inferred
from the body. Because tools are callable by models, they fail closed: if a tool
omits `![...]` and has no trace spec or runtime-policy context that allows the
body effects, the inferred body effects must be empty. An explicit `![...]` row
is the tool's public effect boundary and an upper-bound contract for the
inferred body effects.

```etas
tool safe_web_lookup(query: string) -> Array<Trusted<Snippet>> ![AcademicSearch.search]
    ~ (AcademicSearch.search >> Sanitized)
{
    let pages = perform AcademicSearch.search(query);
    let snippets = extract_snippets(pages);
    return sanitize(snippets);
}
```

This tool can be exposed to a model, while helper flows such as
`extract_snippets` and `sanitize` remain ordinary source functions that are not
directly callable by the model.

Operations implemented outside Etas should be exposed through package metadata,
host-binding metadata, or compiler-known minimal standard-library metadata, then
wrapped by Etas `flow` or `tool` declarations when model-callable validation,
trace-spec/runtime-policy narrowing, or output safety is needed. The runtime implementation
receives typed arguments and returns the declared result. Its effect row is
mandatory in metadata because the compiler cannot inspect the implementation.

The compiler enforces additional rules for model-callable tools:

1. tool input and output types must be schema-encodable; optional defaulted inputs are filled and the full effective arguments validated at the checked invocation entry, not by assuming the model applied a JSON Schema default;
2. body effects are inferred and must fit the explicit or trace-spec/runtime-policy-derived effect boundary;
3. omitted tool boundaries default to no effects;
4. high-impact effects may require approval, trace-spec, runtime-policy, sandbox, or limit guards;
5. outputs must not leak forbidden trust, secret, or internal authority values;
6. only Etas `tool` declarations and imported tool symbols with tool metadata may appear in `@tools(...)`.

### 2.2 Tool Effects And Authority Actions

Etas-implemented tools infer effects from their body, but model-callable
execution is governed by the tool boundary. Flows are ergonomic and infer by
default; tools are conservative and deny effects by default unless the boundary
is explicit.

Runtime-provided primitive symbols and host-provided package bindings must carry
explicit effect metadata because the compiler cannot inspect their
implementation. The built-in core effect roots are intentionally broad:

| Effect | Meaning |
|---|---|
| `Network` | Accesses external network |
| `FileIO` | Reads or writes files visible to the runtime |
| `Command` | Executes a sandboxed command or child process |
| `Memory` | Reads or writes typed persistent memory through standard memory APIs |
| `Human` | Requests human input, approval, notification, or UI-mediated decisions |
| `Agentic` | Internal runtime support for agentic model-mediated reasoning, primarily recorded as requested action `Agentic.infer<A.method, O>` |
| `Secret` | Reads, decrypts, or reveals secret material |
| `Time` | Reads time, sleeps, schedules, or uses timers |
| `Error<E>` | Raises a typed recoverable error |

Fine-grained authority should normally use actions attached to those roots or to
application/library effects:

```etas
effect CompanyEmail extends Network {
    action send(account: EmailAccount, draft: EmailDraft) -> Receipt;
    action read(account: EmailAccount, query: EmailQuery) -> Array<EmailMessage>;
}

effect ProjectWorkspace extends FileIO {
    action read<P>(path: Path) -> bytes;
    action write<P>(path: Path, body: bytes) -> unit;
}

effect Memory {
    action read<R, K, V>(store: Store<K, V>, key: K) -> Option<V>;
    action write<R, K, V>(store: Store<K, V>, key: K, value: V) -> unit;
}
```

Rows, trace specs, and runtime policies can name roots or specific actions:

```etas
![Network]
![AcademicSearch.search, ProjectWorkspace.write<"reports/**">, CompanyEmail.send<WorkAccount>]
```

Root effects are coarse summaries; action references are deployable authority
facts. `Network` covers `AcademicSearch.search`, but production runtime policies
and trace specs should prefer fine-grained actions when possible.

`Command` is always sandboxed in source Etas. If no sandbox profile is supplied
to a command wrapper, the standard library uses the support value
`DefaultCommandSandbox`.

```etas
flow std.command.run(cmd: Command, sandbox: SandboxProfile = DefaultCommandSandbox)
    -> CommandResult ![Command.run<_>]
{
    return perform Command.run(cmd, sandbox);
}
```

### 2.3 Tool Boundary Rules

A tool becomes executable only when all of these checks pass:

```text
the tool is listed for the calling agent or directly visible to the calling flow
and the inferred body effects are covered by the tool boundary
and active trace specs, runtime policies, approval requirements, sandbox rules, and limits pass
and deployment grants allow the concrete effect/action instances
```

An agent does not receive ambient authority just because a tool is in its
`tools` list. The listed tool controls what the model may request. The tool's
body, effect row, active trace specs, and runtime deployment grants control what is
actually executable.

The MVP also keeps the recursion boundary simple: a `flow` may call tools, and a
tool may call private flows, other tools, imported tool symbols, or effect actions. A
tool must not call an agent in the MVP. If a model needs another model, expose
that orchestration as a surrounding `flow` instead of hiding it inside a
model-callable tool.

### 2.4 Standard IO

Standard input/output belongs in the standard library. Its low-level authority
is modeled as effect actions, while the user-facing API is ordinary stdlib
flows.

All APIs below belong to `std.io`. Under the standard console handler they
retain `Error<IOError>`; the handled Console action remains in requested/default
metadata. That metadata is not an additional source declaration clause.

| API | Return Type | Requested / Default Action | Behavior |
|---|---|---|---|
| `print(text: string)` | `unit ![Error<IOError>]` | `Console.stdout_write` | Write exactly the supplied text to stdout. |
| `println(text: string)` | `unit ![Error<IOError>]` | `Console.stdout_write` | Write text followed by LF; no atomicity guarantee across concurrent calls. |
| `eprint(text: string)` | `unit ![Error<IOError>]` | `Console.stderr_write` | Write exactly the supplied text to stderr. |
| `eprintln(text: string)` | `unit ![Error<IOError>]` | `Console.stderr_write` | Write text followed by LF to stderr. |
| `read_line()` | `string ![Error<IOError>]` | `Console.stdin_read_line` | Read through LF or EOF; omit LF and a preceding CR. Empty line returns `""`; EOF before any data raises `Eof`; a final unterminated line is returned once. |
| `read_all()` | `string ![Error<IOError>]` | `Console.stdin_read_all` | Read through EOF, preserving line endings; immediate EOF returns `""`. Obey inherited size/resource limits. |

Input uses strict UTF-8 decoding; malformed input raises `InvalidUtf8`.
I/O failure may occur after consuming input or writing a prefix; neither errors
nor cancellation promise rollback. These flows are not pure and do not catch
their I/O errors by default.

```etas
effect Console extends FileIO {
    action stdout_write(text: string) -> unit;
    action stderr_write(text: string) -> unit;
    action stdin_read_line() -> string;
    action stdin_read_all() -> string;
}

public enum IOError {
    Eof, Interrupted, PermissionDenied, InvalidUtf8,
    NotFound, AlreadyExists, InvalidInput, NotDirectory, IsDirectory,
    DirectoryNotEmpty, Closed, TimedOut, LimitExceeded, Unsupported,
    Host(string),
}
```

This is the canonical `std.io.IOError`, re-exported rather than redeclared by
filesystem/runtime modules. File and stream lifecycle APIs are tabulated in
[Managed Resource Lifecycles](19-standard-library.md#15-managed-resource-lifecycles).

For example, these are Etas implementations under the standard default console
handler. The wrapper bodies are source code; only the final platform write/read
requires a runtime endpoint:

```etas
module std.io;

flow println(text: string) -> unit ![Error<IOError>]
{
    perform Console.stdout_write(text);
    perform Console.stdout_write("\n");
}

flow read_line() -> string ![Error<IOError>]
{
    return perform Console.stdin_read_line();
}
```

The requested actions in the table remain visible to trace-spec checking,
runtime policy, trace and replay. `std.io.read_line()?` has type
`Result<string, IOError>`: postfix `?` captures the error, not unwraps a result.

This split keeps the surface API ergonomic while preserving the effect/action
discipline:

| Layer | Role |
|---|---|
| `std.io.println` | Convenient stdlib flow used by programs |
| `Console.stdout_write` | Standard action declaration with a runtime execution contract; remains visible as a requested action |
| `Console` | Standard effect category, narrower than `FileIO`, not a source keyword |
| Default handler / platform endpoint | Handles the standard action and performs the actual I/O under runtime enforcement |

This follows the [five-layer model](01-overview.md#53-feature-map-by-layer).
Neither standard-library membership nor installation of a default handler grants
authority, and compiler recognition does not require hard-coding the wrapper.

The MVP should keep these APIs string-based. Generic printing through `Display`, `Format`, or `ToString` can be added after the spec story is stable.

Suggested default host policy:

| Stream | CLI default | Reason |
|---|---:|---|
| stdout write | Granted | Normal program output |
| stderr write | Granted | Diagnostics and logs |
| stdin read | Interactive or explicit grant | It may block and consumes user input |
| raw terminal mode | Denied | Terminal control is outside the MVP |

Standard IO should not become source syntax:

```etas
print "hello"; // not part of the MVP
```

It also should not require a model-callable tool unless an agent model needs to
request console I/O directly. Ordinary programs use `std.io` flows; agent tools
wrap those flows only when console access should be exposed to the model.

### 2.5 Standard Substrate API Shape

Std supplies orthogonal substrate; EDK supplies high-level integrations written
in Etas over that substrate. Runtime execution enforces the same action, policy,
sandbox, limit, cancellation, and trace contracts for both layers.

The authoritative substrate module table, signatures, stream provenance,
`ByteStream` bounds, wire-level HTTP types, and secret-backed crypto rules are in
[Minimal Standard Actions](06-effect-system-and-inference.md#11-minimal-standard-actions).
Public text, JSON, task, and single-step model APIs are in
[Standard Library](19-standard-library.md). These are design contracts, not an
implementation-status inventory.

API paths use lowercase names such as `std.net.tcp.connect`; effect/action
owners use uppercase names such as `Net.tcp_connect`, without a `Std` prefix.
EDK wrappers must use the public substrate rather than private host escape
hatches. See [EDK Packages](17-edk-official-packages.md).

---

## 3. Messages And Conversations

### 3.1 Message as the Agent Communication Value

Multi-agent systems are mostly about typed messages moving between agents, tools, humans, persistent stores, and runtime control points. `Prompt` is only the model-call input package. The semantic communication value is `Message<T>`.

`Message<T>`, `SessionConfig`, and `Conversation` are Agent/runtime support types, not keywords. Etas does not add `msg` or `message` declarations in the MVP.

```etas
type Message<T> = {
    id: MessageId,
    from: Participant,
    to: Participant,
    role: Role,
    content: T,
    session: SessionId?,
    provenance: Provenance,
    created_at: Time,
}
```

The exact record shape is a runtime support contract; user code should normally construct messages through support APIs so ids, provenance, session links, and trace metadata remain consistent.

### 3.2 Agent Inputs And Outputs

An agent can consume and produce ordinary typed values:

```etas
@model("gpt-5.5")
agent Reviewer(input: Draft) -> Review {
    return perform infer<Review>(ReviewPrompt(input));
}
```

The generated public entrypoint has the ordinary typed shape:

```etas
Reviewer.run : Draft -> Review
```

and executes the checked method body in the agent's context. For the example
above, the model-interaction path is:

```text
Draft
  -> AgentMethod.run
  -> Prompt
  -> Agentic.infer<Reviewer.run, Review>
  -> Validate/Decode<Review>
  -> AgentMethod.post-infer code
  -> Review
```

The corresponding lowering keeps these boundaries visible when they occur in
the body; arbitrary agent methods are not restricted to this fixed pipeline:

```text
PromptBuild(Reviewer.run, draft) -> Prompt
Perform Agentic.infer<Reviewer.run, Review>(prompt, Schema<Review>) -> Review
PostProcess(Reviewer.run, Review) -> Review
```

`Prompt` is therefore not the input type's parent class, and `ModelResponse` is
not the agent output type. The input type `I`, method body, model-call prompt,
provider response, schema validation, post-inference code, and output type `O`
remain separate semantic steps.

Users normally do not write an output type argument on `run`. The output type and
`Schema<O>` are inferred from the agent declaration:

```etas
let review = Reviewer.run(draft); // Review, from `agent Reviewer(input: Draft) -> Review`
```

The agent-scoped inference operation has this source-level shape:

```text
perform infer<O>(prompt: Prompt) -> O
```

The output type argument may be inferred from context. If prompt construction is
simple, the agent body can delegate to a prompt-builder flow:

```etas
flow ReviewPrompt(draft: Draft) -> Prompt {
    return Prompt.new()
        .system(Trusted("Review the draft for correctness and risk."))
        .data(draft);
}

@model("gpt-5.5")
agent Reviewer(input: Draft) -> Review {
    return perform infer<Review>(ReviewPrompt(input));
}
```

`perform infer(prompt)` is not a normal stdlib function call and not a new global
keyword. It is an agent-scoped action shorthand. It is accepted only inside an
agent body or `impl agent` method and elaborates to
`perform Agentic.infer<CurrentAgent.method, O>(prompt, Schema<O>)`.

For explicit controller code, `std.agent.step(conversation, schema)` requests
the corresponding single-generation `Agentic.step<C, O>` action. Its `Final(O)`
case is schema-validated; `CallTools` returns native-tool requests that still
require checked invocation. `Conversation` assembly does not implicitly read,
append, or publish persistent session history. See the
[standard protocol APIs](19-standard-library.md#43-public-apis).

The runtime must not automatically place user input into the system channel. If
the input should be included, the body should place it explicitly through APIs
that require the input type to satisfy `PromptEncode`.

Agent output type `O` requires a schema and decoder. The compiler/runtime should automatically derive these for ordinary Etas types such as primitives, records, enums, lists, maps, and options. Opaque or host-native output types require an explicit support implementation.

When conversation continuity, handoff, or sender/receiver provenance matters, agents should use `Message<T>`:

```etas
@model("gpt-5.5")
@tools([orders.lookup, refund.request])
agent RefundAgent(input: Message<RefundRequest>) -> Message<RefundReply> {
    return perform infer<Message<RefundReply>>(RefundPrompt(input));
}

flow RouteCustomerMessage(msg: Message<CustomerRequest>) -> Message<CustomerReply> {
    let route = msg ~> TriageAgent;

    match route.target {
        Refund => return msg.cast<RefundRequest>() ~> RefundAgent;
        Sales => return msg.cast<SalesRequest>() ~> SalesAgent;
        Support => return msg.cast<SupportRequest>() ~> SupportAgent;
    }
}
```

The `~>` operator does not need special surface syntax for messages. When the value being passed is `Message<T>`, lowering and runtime execution preserve message metadata, session linkage, provenance, trace events, and handoff information.

### 3.3 Sessions And Conversations

`SessionConfig` describes how existing conversation history and published
context are selected and retained. It does not schedule summary generation:

```etas
type SessionConfig = {
    id: SessionId,
    context: ContextPolicy,
    retention: RetentionPolicy,
}
```

Example:

```etas
flow SupportTurn(msg: Message<CustomerRequest>) -> Message<CustomerReply> {
    let session = SessionConfig {
        id = msg.session.unwrap_or(current_session()),
        context = SummaryPlusRecent(recent = 8),
        retention = Days(90),
    };

    return Message.with_session(msg, session) ~> TriageAgent;
}
```

The runtime uses the session to load relevant history, append new messages, deduplicate retry writes, connect traces across runs, and enforce context-window policy. This should be recognized by the compiler, AIR, and runtime, but it should not become a `session` keyword in the MVP.

`SummaryPlusRecent(recent = 8)` selects an already published summary and recent
turns; it never invokes a model or creates a summary. With no published summary,
it selects recent turns only, with the absence of a summary visible to the
caller. The selected context is a view, not a claim to contain the full history.
Selecting a view does not delete the underlying messages.

Applications or EDK packages explicitly read history, generate and validate a
summary, and publish the resulting context through `std.agent.session`:

| API | Contract |
|---|---|
| `history_page` | Read a bounded history page with a backend-issued cursor and `SessionHistoryFence`, plus any published context. |
| `prepare_context` | Bind caller-produced `SessionContextContent` and its history fence to a `StorageOperationRef`; preparing does not publish or generate content. |
| `publish_context` | Conditionally publish that content, checking its session generation, history fence, and context version; return a confirmed outcome or an explicitly unknown outcome. |
| `reconcile_context` | Resolve an uncertain publication using the same operation reference and backend evidence. |

The application/EDK owns the trigger, model/provider choice, model-specific
token counting, prompt, validation, and retry strategy. Summary generation uses
ordinary Etas flows/agents; model calls remain visible as requested inference
actions and obey normal trace-spec, runtime-policy, cancellation, and budget
checks. `ContextTokens(...)` remains a context budget bound, not an instruction
to summarize. Exceeding it follows the limit failure rules; session loading or
publication must not silently launch an inference call to make the input fit.

Publication must preserve these guarantees:

- Context content carries provenance for its source history and producer. A
  summary is derived content; publication does not certify its semantic accuracy
  or upgrade its trust level.
- Publication atomically validates the expected history/context state and
  installs a new context version. A conflicting append, replacement, or session
  generation change rejects a stale publication without overwriting newer
  context. The caller may explicitly reload and regenerate.
- `SessionContextReceipt` identifies the operation, session generation,
  published context version, and backend-confirmed durability. Retrying the
  same operation must reuse its confirmed outcome rather than publish twice.
- Failure or cancellation during generation leaves published context unchanged.
  A confirmed publication rejection also leaves it unchanged. If publication
  may already have committed, timeout or cancellation is not proof of rollback:
  reconcile the operation before deciding whether to retry.
- Publishing a summary does not remove source messages. Retention, archival,
  deletion, and storage compaction remain separate runtime/storage operations,
  preserving required provenance and making any loss of replay data explicit.

This contract requires history access, version validation, context publication,
and receipts from the standard substrate. It does not require a built-in
production summarizer or a host callback that generates summaries during
session maintenance.

### 3.4 Handoff

Handoff is a runtime/AIR/trace event, not a source keyword. Source code expresses handoff as typed routing between agents or flows:

```etas
flow CustomerTurn(msg: Message<CustomerRequest>) -> Message<CustomerReply> {
    let route = msg ~> TriageAgent;

    match route.target {
        Refund => return msg.cast<RefundRequest>() ~> RefundAgent;
        Sales => return msg.cast<SalesRequest>() ~> SalesAgent;
        Support => return msg.cast<SupportRequest>() ~> SupportAgent;
    }
}
```

The compiler/runtime can infer a `Handoff` event when a message moves from one agent-owned stage to another:

```text
Handoff {
  from: AgentId,
  to: AgentId,
  message: MessageId,
  session: SessionId,
  reason: string?,
}
```

This gives handoff first-class trace semantics without adding `handoff`, `msg`, or `message` keywords to the source language.

---

## 4. Prompt Values

### 4.1 Prompt as a Typed Agent Support Value

Prompts are not raw strings. They are typed Agent/runtime support values with trust boundaries. `Prompt` is not the primary multi-agent communication construct; it is the model-call input package used by the agent runtime. Etas does not need a dedicated `prompt` declaration in the MVP; use deterministic flows that return `Prompt`.

```etas
flow ReviewPrompt(input: { draft: Draft, criteria: ReviewCriteria }) -> Prompt {
    return Prompt.new()
        .system(Trusted("You are a programming languages conference reviewer."))
        .data(input);
}
```

`system`, `user`, `assistant`, and `data` are method names or `Role` enum values from Agent/runtime support, not language keywords.

### 4.2 Prompt Parts And Prompt Encoding

Etas should not use subclassing or `toString` as the main prompt serialization protocol. The model should be:

```text
Prompt          = complete model-call input package
PromptPart      = content that can be appended to one prompt channel
T ~ PromptEncode = evidence that T can be encoded into PromptPart
```

`Prompt.new().data(x)` and `Prompt.new().user(x)` require the type of `x` to
satisfy `PromptEncode`. This is spec polymorphism, not OO subtyping. Etas MVP
does not need general value subtyping for prompt construction.

Default `PromptEncode` instances should be derivable for:

```text
bool, integer, f32, f64, string, bytes
record, enum
Array<T>, List<T>, Map<K, V>, Set<T>, Range<I>, Slice<T>, Option<T>
Message<T>
Trusted<T>, Untrusted<T>, Sanitized<T>, Public<T>
```

`Secret<T>` should not be prompt-encodable by default. It must be redacted, declassified, or explicitly revealed through a trace/runtime-policy-checked support API.

Trust and channel rules are part of prompt encoding:

```text
Prompt.system(x) requires trusted instruction content.
Prompt.user(x) and Prompt.data(x) may accept untrusted data with provenance.
Untrusted<T> must not flow into the system channel.
Sanitized<T> may flow into user/data channels.
```

Users should customize prompt serialization by writing an explicit prompt builder or a support implementation, not by relying on `toString`:

```etas
flow DraftPromptPart(draft: Draft) -> PromptPart {
    return PromptPart.data({
        title = draft.title,
        body = draft.body,
        citations = draft.citations
    });
}

flow ReviewPrompt(draft: Draft) -> Prompt {
    return Prompt.new()
        .system(Trusted("Review the draft."))
        .data(DraftPromptPart(draft));
}
```

`toString` may exist for debugging or display, but it is not the safe prompt construction protocol because it erases structure, trust labels, provenance, channel placement, and schema information.

### 4.3 Control Plane vs Data Plane

Etas distinguishes trusted instructions from untrusted content.

```etas
Trusted<string>
Untrusted<string>
Secret<string>
```

The system prompt belongs to the control plane. User input, web pages, PDFs, emails, and retrieved documents are usually data-plane content.

Therefore:

```etas
flow BadPrompt(x: Untrusted<string>) -> Prompt {
    return Prompt.new()
        .system(x);   // compile-time error
}
```

The compiler rejects this because untrusted data flows into the trusted instruction channel.

### 4.4 Declassification

Sometimes secret or untrusted content must be sent to an agent after sanitization or redaction.

```etas
import std.security.declassify.{sanitize, declassify};

@model("gpt-5.5")
agent Summarizer(input: Sanitized<string>) -> Summary {
    return perform infer<Summary>(SummaryPrompt(input));
}

let safe_text = sanitize(web_page.body, strategy = WebText);
let summary = Summarizer.run(safe_text);
```

For secret data:

```etas
let redacted = declassify(
    secret_doc,
    strategy = RedactPII,
    reason = "prepare approved material for summarization"
);

let result = Agent.run(redacted);
```

`WebText` is a configured `SanitizationStrategy<string, string>` and
`web_page.body` has type `Untrusted<string>`. `RedactPII` is a configured typed
`DeclassificationStrategy` for the secret document. Both strategies perform
their declared validation or transformation. Secret release checks the current
authority and records its outcome. The complete contracts are defined in
[Sanitization And Declassification](19-standard-library.md#8-sanitization-and-declassification).

---

## 5. Persistent Memory APIs

### 5.1 Typed Persistent State

Persistent memory uses compiler-known std support types, not dedicated syntax.
The schema `S` is an ordinary record of `Store<K, V>` fields;
`std.memory.region<S>` returns one `MemoryRegion<S>` handle. Do not wrap the
schema in `MemoryRegion` again, which would introduce a second region layer.

```etas
type PaperRecord = {
    title: string,
    authors: Array<string>,
    year: i32,
    venue: string,
    claims: Array<Claim>,
    embedding: Vector<1536>,
    provenance: Array<Citation>
}

type DecisionRecord = {
    time: Time,
    agent: AgentId,
    decision: string,
    rationale: string,
    trace: TraceId
}

type ProjectMemorySchema = {
    Papers: Store<PaperId, PaperRecord>,
    Decisions: Store<TraceId, DecisionRecord>,
};

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );
```

`Map<K, V>` remains an ordinary in-memory value type. `Store<K, V>` is a typed persistent store field inside a `MemoryRegion<...>`. `ProjectMemory` is an immutable resource handle, not a mutable global variable. Constructing the handle is declarative; it does not connect to or mutate the backend at module load. The compiler recognizes these standard support types and resource constructors to generate memory schema metadata, infer region-sensitive memory effects, and check replay/version safety. The concrete backend is bound by the manifest/runtime, for example SQLite in tests, Postgres in production, or a vector store for retrieval.

### 5.2 Scoped Access

Agent memory access should be visible in the agent method body and the typed
memory APIs it calls. A read used to assemble prompt context is ordinary Etas
code before `perform infer`:

```etas
@model("gpt-5.5-thinking")
agent Researcher(input: Topic) -> Notes {
    let papers =
        ProjectMemory.Papers
            .select(std.memory.query("project.topic", 1, input))
            .limit(Tokens(20_000));

    let prompt = Prompt.new()
        .system(Trusted("Find strong evidence."))
        .data(input)
        .data(papers);

    return perform infer<Notes>(prompt);
}
```

The project supplies the query language/version adapters used above; these names
are not universal database query formats. See [storage encoding and query contracts](19-standard-library.md#165-storage-encoding-and-query-contracts).

The compiler retains requested actions such as `Memory.read<ProjectMemory.Papers>` and records the accessed
region in summaries, manifests, AIR, and traces. Persistent memory writes
should normally happen in surrounding `flow` code after validation and approval,
not inside an agent method unless the write is intentionally part of that
method's public behavior.

### 5.3 Versioned Persistent State

Every committed memory change creates a versioned update; a confirmed no-op
need not create a new version:

```text
Memory.write(agent, region, key, old_value, new_value, trace_id, timestamp)
```

This supports replay, provenance, debugging, audit and memory analysis.
It does not imply automatic rollback after cancellation or unknown commitment.
The authoritative CRUD, condition, receipt, reconciliation and session API
tables are in [Persistent Stores And Session Publication](19-standard-library.md#16-persistent-stores-and-session-publication).

---
