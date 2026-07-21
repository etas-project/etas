# Agents, Tools, Messages, Prompt Values, and Persistent Memory APIs

## 1. Agent

### 1.1 Agent as a First-Class Construct

An agent is a first-class nominal component. It has:

- a stable identity;
- model/runtime annotations;
- exposed model tools;
- attached policies;
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

`agent Name;` declares only the nominal component. The ergonomic form
`agent Name(input: I) -> O { ... }` declares the same component and provides a
default `run` implementation. Its body is checked like a flow body, except that
agent-scoped `perform infer(prompt)` is available. Effect contracts remain in the
`![...]` position after the output type, not in annotations.

The purpose of `agent` is not to copy SDK object configuration into source
syntax. Frameworks such as LangChain, Google ADK, and AutoGen already model
agents as host-language objects with a model, instructions, tools, and runtime
harness. Etas keeps `agent` only because the declaration creates a semantic
boundary that the compiler and runtime can analyze:

| Surface fact | Semantic use |
|---|---|
| nominal agent identity | stable trace identity, runtime profile, policy target, and replay key |
| default `run` method | stable public callable entrypoint, requested-action recording, replay, and stage composition |
| `perform infer(prompt)` | language-visible model inference boundary, schema derivation, typed decoding, trace, replay, mock/resample, and optimization |
| ordinary agent method body | context slicing, memory selection, prompt-channel trust checks, post-inference checks, and local normalization |
| `impl agent` methods/specs | reusable agent behaviors, explicit method surface, and spec evidence |
| `@tools(...)` surface | model-callable tool set, least-privilege checking, and runtime tool denial |
| inferred effects | policy, approval, deployment manifest, and replay planning |
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
agent Reviewer(input: Draft) -> Review ![FileIO] {
    let papers =
        ProjectMemory.Papers
            .related_to(input)
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
Reviewer.run : Draft -> Review ![FileIO]
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
∪ possible effects of exposed tools in config.tools
∪ context/session/typed-memory support effects
```

At each `perform infer` inside `Reviewer.run`, the compiler records
`Agentic.infer<Reviewer.run, Review>` as requested-action metadata for policy, limit,
trace, replay, and host-support checks. The agent declaration itself does not
record that action, and the metadata does not escape as a user-visible effect
obligation of ordinary `Reviewer.run(input)`.

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
| `tool ... { ... }` | Yes | Etas source | A safe model-callable wrapper around ordinary Etas logic, policy checks, validation, context narrowing, and calls to other tools |
| `tool ...;` | No | Package interface or compiler/runtime metadata | A typed imported tool signature for a standard-library primitive, host-provided binding, generated API, or precompiled package item |

Host bindings are not declared with a source-level keyword in the MVP.
Standard-library primitive symbols and host-provided package bindings are
described by compiler/runtime metadata or package manifests, then imported and
called like ordinary package symbols. Their interface metadata must include
type, effect row, determinism class, schema requirements, and runtime binding
information. Ordinary implementation source should prefer `tool ... { ... }`
or `import` an existing bodyless signature from a package interface.

This gives `tool` a clear boundary: it is not just a renamed `flow`; it is an
LLM action boundary with schema generation, effect/action/policy checks,
trace visibility, argument validation, and output-leakage restrictions.

Effect actions are the primitive runtime authority boundary. Minimal standard
operations such as console I/O, approval, command execution, memory access, and
time are standard actions. External services such as web search, workspace
writes, email sends, database queries, browser automation, and payments should
be modeled as package-defined or host-defined effect actions. Library APIs then
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
omits both `![...]` and an allowing policy, the inferred body effects must be
empty. An explicit `![...]` row is the tool's public effect boundary and an
upper-bound contract for the inferred body effects.

```etas
tool safe_web_lookup(query: string) -> Array<Trusted<Snippet>> ![AcademicSearch.search]
    ~ (
        +AcademicSearch.search
        & +Sanitized
        & (AcademicSearch.search >> Sanitized)
    )
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
policy narrowing, or output safety is needed. The runtime implementation
receives typed arguments and returns the declared result. Its effect row is
mandatory in metadata because the compiler cannot inspect the implementation.

The compiler enforces additional rules for model-callable tools:

1. tool input and output types must be schema-encodable for model tool calling;
2. body effects are inferred and must fit the explicit or policy-derived effect boundary;
3. omitted tool boundaries default to no effects;
4. high-impact effects may require approval, policy, sandbox, or limit guards;
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

Rows and policies can name roots or specific actions:

```etas
![Network]
![AcademicSearch.search, ProjectWorkspace.write<"reports/**">, CompanyEmail.send<WorkAccount>]
```

Root effects are coarse summaries; action references are deployable authority
facts. `Network` covers `AcademicSearch.search`, but production policies should
prefer fine-grained actions when possible.

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
and active policies, approval requirements, sandbox rules, and limits pass
and deployment grants allow the concrete effect/action instances
```

An agent does not receive ambient authority just because a tool is in its
`tools` list. The listed tool controls what the model may request. The tool's
body, effect row, followed policy, and runtime deployment grants control what is
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

The user-facing API should be ordinary stdlib flows exported from module
`std.io`. Their formal signatures include both the console action and the
possible host I/O error:

```text
std.io.print(text: string)    -> unit   ![Error<IOError>]
  requested_actions = [Console.stdout_write]
  default_actions = [Console.stdout_write]
std.io.println(text: string)  -> unit   ![Error<IOError>]
  requested_actions = [Console.stdout_write]
  default_actions = [Console.stdout_write]
std.io.eprint(text: string)   -> unit   ![Error<IOError>]
  requested_actions = [Console.stderr_write]
  default_actions = [Console.stderr_write]
std.io.eprintln(text: string) -> unit   ![Error<IOError>]
  requested_actions = [Console.stderr_write]
  default_actions = [Console.stderr_write]
std.io.read_line()            -> string ![Error<IOError>]
  requested_actions = [Console.stdin_read_line]
  default_actions = [Console.stdin_read_line]
std.io.read_all()             -> string ![Error<IOError>]
  requested_actions = [Console.stdin_read_all]
  default_actions = [Console.stdin_read_all]
```

These flows are not pure. They do not catch `Error<IOError>` by default; they
perform lower-level standard actions and expose the same observable failure to
callers:

```etas
effect Console extends FileIO {
    action stdout_write(text: string) -> unit;
    action stderr_write(text: string) -> unit;
    action stdin_read_line() -> string;
    action stdin_read_all() -> string;
}

enum IOError {
    Eof;
    Interrupted;
    PermissionDenied;
    InvalidUtf8;
    Host(string);
}
```

For example:

```etas
module std.io;

flow println(text: string) -> unit ![Error<IOError>]
  requested_actions = [Console.stdout_write]
  default_actions = [Console.stdout_write]
{
    perform Console.stdout_write(text);
    perform Console.stdout_write("\n");
}

flow read_line() -> string ![Error<IOError>]
  requested_actions = [Console.stdin_read_line]
  default_actions = [Console.stdin_read_line]
{
    return perform Console.stdin_read_line();
}
```

The public stdlib flows expose only residual effects such as possible host
failure. The performed console action remains visible as requested/default
action metadata for policy, trace, replay, and runtime mediation.
For example:

```text
std.io.read_line()  : string ![Error<IOError>]
std.io.read_line()? : Result<string, IOError>
std.io.println("x") : unit ![Error<IOError>]
```

The postfix `?` form captures `Error<IOError>` into a value-level `Result`; it
does not unwrap a `Result`.

This split keeps the surface API ergonomic while preserving the effect/action
discipline:

| Layer | Role |
|---|---|
| `std.io.println` | Convenient stdlib flow used by programs |
| `Console.stdout_write` | Runtime action and authority fact |
| `Console` | Observable effect category, narrower than `FileIO` |
| Host/default implementation | Platform implementation selected by the runtime |

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

The current `etas_std` does not yet provide every low-level runtime substrate
needed by the official EDK packages. The implemented surface is closer to:

| Current area | Current role |
|---|---|
| `std.io` | Console I/O wrappers backed by `Console.*` requested/default actions |
| `std.memory` | Typed memory support and compiler-known resource constructors |
| `std.host.command`, `std.host.path`, `std.host.url`, `std.host.sandbox` | Command, path, URL, and sandbox support types and wrappers |
| `std.bytes`, `std.text` | Small pure helpers such as length, split, join, and integer parsing |
| `std.security.*` | Trust, declassification, and policy support helpers |
| Effect/action registry | Broad roots such as `Network`, `FileIO`, and `Secret`, with many low-level wrappers still missing |

The SPEC should distinguish these layers:

```text
std substrate = low-level, orthogonal, policy-visible runtime primitives
EDK           = high-level production integrations written in Etas
runtime       = executes substrate actions and enforces sandbox, policy, trace,
                replay, secret redaction, timeout, and cancellation
```

The API path uses lowercase modules, while the effect/action owner is uppercase
without a `Std` prefix:

| API | Effect/action owner |
|---|---|
| `std.net.tcp.connect(...)` | `Net.tcp_connect<...>` |
| `std.stream.read(...)` | `Stream.read<...>` |
| `std.tls.connect(...)` | `Tls.handshake<...>` |
| `std.fs.read_bytes(...)` | `Fs.read<...>` |
| `std.secret.read(...)` | `Secret.read<...>` |
| `std.browser.protocol.send(...)` | `Browser.send<...>` |

`Stream` actions are tied to typed stream-handle provenance. Reading from a TLS
stream remains network authority; reading from a file stream remains file
authority. Policies can also mention `Stream.read<_>` or `Stream.write<_>`
directly when byte movement itself is the concern.

`Browser` is a standard substrate effect under `Network`, not a separate core
broad root. `std.browser.protocol` exposes session attach/create, protocol
transport, event receive, screenshot bytes, and session/origin binding; high
level browser actions such as navigate, click, and DOM extraction belong in
`edk.browser`.

EDK-facing substrate APIs should expose their authority actions directly in the
public effect row:

```etas
std.net.tcp.connect(host: Host, port: Port, options: TcpOptions)
    -> TcpStream ![Net.tcp_connect<_, _>, Error<NetworkError>]

std.stream.read<S ~ ByteStream>(stream: S, max_bytes: usize, timeout: Timeout?)
    -> StreamRead ![Stream.read<S>, Error<StreamError>]

std.stream.read_until_limit<S ~ ByteStream>(stream: S, limit: ByteLimit, timeout: Timeout?)
    -> bytes ![Stream.read<S>, Error<StreamError>]

std.tls.connect(stream: TcpStream, server_name: Host, config: TlsConfig)
    -> TlsStream ![Tls.handshake<_>, Error<TlsError>]

std.fs.read_bytes<R ~ Region>(path: WorkspacePath<R>)
    -> bytes ![Fs.read<R>, Error<IOError>]

std.fs.write_bytes<R ~ Region>(path: WorkspacePath<R>, body: bytes)
    -> unit ![Fs.write<R>, Error<IOError>]
```

`ByteStream` is a standard marker spec satisfied by opaque stream-handle
types such as `TcpStream`, `TlsStream`, `FileStream`, and `BrowserStream`. It is
not a concrete supertype, so `std.stream.*` uses spec-bound polymorphism rather
than general subtyping.

Filesystem paths use the same pattern. `WorkspacePath<R>` is indexed by a
region marker type, and specs such as `Within<Parent>` express region
relationships. This lets policy match constrained action patterns such as
`Fs.read<R ~ Within<ReportsRoot>>` by typed resource region instead of by ad-hoc
path strings.

Pure data APIs stay ordinary functions or flows with no runtime action:

```etas
std.http.codec.encode_request(req: HttpWireRequest) -> bytes
std.http.codec.decode_response_head(body: bytes) -> Result<HttpWireResponseHead, HttpCodecError>
std.codec.text.utf8_decode(body: bytes, malformed: MalformedInput)
    -> Result<string, TextCodecError>
std.crypto.sha256(body: bytes) -> Digest
std.crypto.constant_time_eq(a: bytes, b: bytes) -> bool
```

Secret-backed crypto is not pure because it consumes opaque secret material:

```etas
std.crypto.hmac_sha256<K>(key: SecretValue<K>, body: bytes)
    -> Digest ![Secret.use<K>, Error<CryptoError>]
```

`std.http.codec` owns wire-level types such as `HttpWireRequest` and
`HttpWireResponseHead`. EDK HTTP packages must translate between their
user-facing request/response records and these wire-level codec types.

`StreamRead` returns either `Data(bytes)` or `Eof`. Timeout, cancellation,
closed streams, host failures, and byte-limit violations are represented as
`Error<StreamError>`, not as EOF.

High-level APIs remain outside core `std`. For example,
`edk.http.client.request` should be implemented in Etas over `std.net`,
`std.stream`, `std.tls`, and `std.http.codec`;
`edk.workspace.files.read/write` should be implemented over `std.fs`; and
`edk.browser.navigate/click/read` should be implemented over
`std.browser.protocol`. EDK must not hide missing substrate behind
package-private host calls.

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

but it is not lowered as a plain method call. An agent call is a typed runtime pipeline:

```text
Draft
  -> AgentMethod.run
  -> Prompt
  -> Agentic.infer<Reviewer.run, Review>
  -> Validate/Decode<Review>
  -> AgentMethod.post-infer code
  -> Review
```

The corresponding AIR keeps the intermediate boundaries visible:

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

`SessionConfig` describes how conversation history is selected and retained:

```etas
type SessionConfig = {
    id: SessionId,
    context: ContextPolicy,
    retention: RetentionPolicy,
    compaction: CompactionPolicy,
}
```

Example:

```etas
flow SupportTurn(msg: Message<CustomerRequest>) -> Message<CustomerReply> {
    let session = SessionConfig {
        id = msg.session.unwrap_or(current_session()),
        context = SummaryPlusRecent(recent = 8),
        retention = Days(90),
        compaction = SummarizeWhen(ContextTokens(24_000)),
    };

    return Message.with_session(msg, session) ~> TriageAgent;
}
```

The runtime uses the session to load relevant history, append new messages, deduplicate retry writes, connect traces across runs, and enforce context-window policy. This should be recognized by the compiler, AIR, and runtime, but it should not become a `session` keyword in the MVP.

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

`Secret<T>` should not be prompt-encodable by default. It must be redacted, declassified, or explicitly revealed through a policy-checked support API.

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
@model("gpt-5.5")
agent Summarizer(input: Sanitized<string>) -> Summary {
    return perform infer<Summary>(SummaryPrompt(input));
}

let safe_text = sanitize(web_page.body);
let summary = Summarizer.run(safe_text);
```

For secret data:

```etas
let redacted = declassify(secret_doc, policy = RedactPII);

let result = Agent.run(redacted);
```

Declassification is explicit and auditable.

---

## 5. Persistent Memory APIs

### 5.1 Typed Persistent State

Persistent memory is represented by compiler-known standard/runtime support types, not dedicated source syntax. A memory schema is expressed as an ordinary type built from `MemoryRegion<S>` and `Store<K, V>`, then bound to an immutable top-level runtime resource handle.

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

type ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, PaperRecord>,
    Decisions: Store<TraceId, DecisionRecord>,
}>;

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
            .select(input)
            .limit(Tokens(20_000));

    let prompt = Prompt.new()
        .system(Trusted("Find strong evidence."))
        .data(input)
        .data(papers);

    return perform infer<Notes>(prompt);
}
```

The compiler infers corresponding effects such as `Memory.read<ProjectMemory.Papers>` and records the accessed
region in summaries, manifests, AIR, and traces. Persistent memory writes
should normally happen in surrounding `flow` code after validation and approval,
not inside an agent method unless the write is intentionally part of that
method's public behavior.

### 5.3 Versioned Persistent State

Every memory write creates a versioned update:

```text
Memory.write(agent, region, key, old_value, new_value, trace_id, timestamp)
```

This supports replay, rollback, provenance, debugging, audit, and abstract interpretation over memory effects.

---
