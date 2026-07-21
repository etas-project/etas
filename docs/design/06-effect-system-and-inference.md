# Effect System and Inference

Etas effects describe observable inference, runtime, and external actions. They are part of the static contract of flows, agents, tools, and selected runtime operations.

The effect system is a separate semantic layer from the type system:

```text
Types   classify values.
Effects classify computations.
```

They are nevertheless checked together. Effect inference runs over typed AST/AIR, uses flow, agent, tool, typed memory API, and support-flow signatures from the type environment, and feeds its results back into flow signatures:

```text
Input -> Output ![Effects]
```

Here `Input` and `Output` are type-system facts, while `Effects` is an effect-system fact attached to the same callable signature.

The effect system has three jobs:

1. infer possible effects from source code;
2. check inferred effects against signatures, policies, limits, and approval requirements;
3. give the runtime an enforceable authority boundary.

## 1. Core Model

An effect is a named class of action relevant to safety, cost, audit, or runtime authority. Etas does not have a separate source-level authority-token concept. Concrete resource authority is expressed with fine-grained, parameterized effect actions.

```text
CoreEffectRoot = Agentic | Network | FileIO | Command | Memory | Secret | Time | Human | Error<E>
EffectSet      = Set<EffectRef>
```

The core root set should stay small. Roots are broad categories used for
effect-root extension, policy coverage, and high-level summaries. This is not
value subtyping.

| Root | Meaning |
|---|---|
| `Agentic` | Performs agentic reasoning or model-mediated agent execution |
| `Network` | Accesses external network resources |
| `FileIO` | Reads or writes files visible to the runtime |
| `Command` | Executes a sandboxed command or child process |
| `Memory` | Reads, writes, migrates, or compacts typed persistent memory resources |
| `Secret` | Reads, decrypts, or reveals secret material |
| `Time` | Reads wall-clock time, sleeps, schedules, or uses timers |
| `Human` | Requests human input, approval, notification, or UI-mediated decisions |
| `Error<E>` | Raises a typed recoverable failure |

Fine-grained authority is represented by effect actions. Some actions are
defined by the core prelude, while most external authorities are defined by
packages, host bindings, or user code. In an effect row or policy, action
references are effect facts:

```text
EffectRef =
    EffectTag
  | EffectTag "." ActionName
  | EffectTag "." ActionName "<" StaticEffectArgList ">"

StaticEffectArg =
    TypeName
  | Literal
  | StableResourceName
  | StableResourcePath
  | CompileTimeConstant
  | "_"
```

Action references are static effect facts. Their `<...>` arguments are static
selectors, not runtime argument captures. A runtime local such as `path`,
`stream`, `topic`, `account`, or `req.host` must not appear inside an effect
row or policy action pattern unless it resolves to a compile-time constant.
Concrete runtime values are ordinary action payload fields and are recorded in
trace events for exact runtime policy checks.

For example:

```etas
![Network]
![AcademicSearch.search]
![Http.request<"github.com">]
![ProjectWorkspace.write<"reports/**">]
![Memory.read<ProjectMemory.Papers>]
![CompanyEmail.send<WorkAccount>]
![Error<IOError>]
```

`AcademicSearch.search` is covered by `Network`,
`ProjectWorkspace.write<"reports/**">` is covered by `FileIO`, and
`Memory.read<ProjectMemory.Papers>` is covered by `Memory`. Trace specs may
target either broad roots or narrow action instances.

A tag name is the only source-level shorthand for "all actions under this tag".
For example, `![Http]` allows every current and future action under the imported
`Http` effect tag, while `![Http.request<_>]` allows only the `request` action
for any argument. Etas does not support `Http.*` or other action namespace
wildcard syntax in effect rows or policies; write the tag for the whole effect,
or write an action/action-instance pattern for a narrower boundary.

Effect rows are positive summaries and upper bounds only. They do not contain
denial entries:

```etas
![Http, ProjectWorkspace.read<"docs/**">]      // valid
![Http, -Http.request<_>]                      // invalid
```

Use trace specs for rejection and contextual constraints:

```etas
spec ReadOnlyWeb: trace =
    +Http
    & -Http.request<"POST", _>
    & -Http.upload<_>;
```

### 1.1 Minimal Standard Actions

The core standard library should define only orthogonal actions needed by the
language runtime and common local programs. These names are not keywords; they
are prelude or standard-library effect/action names.

| Root / effect | Minimal standard actions | Typical authority |
|---|---|---|
| `Console extends FileIO` | `stdin_read_line`, `stdin_read_all`, `stdout_write`, `stderr_write` | CLI input/output used by `std.io` |
| `Approval extends Human` | `request` | Human approval and UI-mediated decisions |
| `Clock extends Time` | `now`, `sleep` | Time reads and timers |
| `Memory` | `read<R>`, `write<R>` | Typed memory regions and stores |
| `Secret` | `read<K>`, `use<K>` | Host-mediated secret handles and non-revealing secret-backed operations |
| `Command` | `run<S>` | Sandboxed process execution scoped by sandbox profile |
| `Agentic` | `infer<C, O>` | Agent-scoped model inference requested action, where `C` is an agent method identity |
| `Error<E>` | `raise` | Typed recoverable failure |

The current implementation-facing `etas_std` does not yet provide every
substrate API needed by EDK. It currently covers console I/O, typed memory
support, host command/path/url/sandbox support, thin bytes/text helpers,
security/trust helpers, and an effect/action registry for broad roots such as
`Network`, `FileIO`, and `Secret`. The following low-level modules are intended
standard-library substrate, but should be treated as design targets until they
exist in the implementation:

| API module | Standard effect/action owner | Intended substrate |
|---|---|---|
| `std.net.tcp` | `Net extends Network` | TCP connect, socket identity, cancellation, timeout metadata |
| `std.stream` | `Stream` | Bounded byte-stream read, write, flush, close, EOF, timeout, and cancellation over typed stream handles |
| `std.tls` | `Tls extends Network` | TLS client session setup, server-name binding, certificate validation, TLS errors |
| `std.fs` | `Fs extends FileIO` | Project-root-scoped read, write, list, stat, canonicalize, and atomic replace |
| `std.http.codec` | none | Deterministic wire-level HTTP request/response framing and parsing; no network access |
| `std.codec.text` | none | UTF-8 and charset encode/decode with explicit malformed-input behavior |
| `std.secret` | `Secret` | Opaque secret key/value reads and redaction-safe secret values |
| `std.crypto` | none for public deterministic operations; `Secret.use<K>` for secret-backed operations | Hashes, HMAC/signatures over opaque secrets, digest encoding, constant-time comparison; randomness would need an action |
| `std.browser.protocol` | `Browser extends Network` | Browser session attach/create, protocol transport, event receive, screenshot bytes, and session/origin binding |

`Stream` is an origin-indexed substrate effect, not a new broad authority root.
Typed stream handles carry provenance from the action that created them. A
`Stream.read<S>` or `Stream.write<S>` over a TLS/TCP stream is still covered by
the active `Network` authority, while a stream over a workspace file is covered
by `FileIO`. Policies may also target `Stream.read<_>` or `Stream.write<_>`
directly when they need to constrain byte movement independent of origin.

`Browser` is a standard substrate effect that extends `Network`; it is not a new
core broad root. `Network` policies cover browser protocol transport by default,
while `Browser.attach<_>`, `Browser.send<_>`, `Browser.recv<_>`, and
`Browser.screenshot<_>` give policies a narrower action-family surface for
browser sessions. Concrete profile and session handles are runtime action
payloads, not values captured inside `<...>`. High-level actions such as
`EdkBrowser.navigate`, `EdkBrowser.click`, and `EdkBrowser.read_dom` belong in
EDK.

The source naming convention is:

| Layer | Naming style | Example |
|---|---|---|
| API path | lowercase module/function path | `std.net.tcp.connect(...)` |
| Effect/action owner | uppercase effect name, no `Std` prefix | `Net.tcp_connect<_, _>` |
| Types | uppercase type names | `TcpStream`, `TlsStream`, `WorkspacePath<ReportsRoot>` |

EDK-facing substrate wrappers should expose their authority action in the public
effect row. For example:

```etas
std.net.tcp.connect(host: Host, port: Port, options: TcpOptions)
    -> TcpStream ![Net.tcp_connect<_, _>, Error<NetworkError>]

std.stream.read<S ~ ByteStream>(stream: S, max_bytes: usize, timeout: Timeout?)
    -> StreamRead ![Stream.read<S>, Error<StreamError>]

std.stream.read_until_limit<S ~ ByteStream>(stream: S, limit: ByteLimit, timeout: Timeout?)
    -> bytes ![Stream.read<S>, Error<StreamError>]

std.stream.write_all<S ~ ByteStream>(stream: S, body: bytes)
    -> unit ![Stream.write<S>, Error<StreamError>]

std.tls.connect(stream: TcpStream, server_name: Host, config: TlsConfig)
    -> TlsStream ![Tls.handshake<_>, Error<TlsError>]

std.fs.read_bytes<R ~ Region>(path: WorkspacePath<R>)
    -> bytes ![Fs.read<R>, Error<IOError>]

std.fs.write_bytes<R ~ Region>(path: WorkspacePath<R>, body: bytes)
    -> unit ![Fs.write<R>, Error<IOError>]

std.secret.read<K>(key: SecretKey<K>)
    -> SecretValue<K> ![Secret.read<K>, Error<SecretError>]

std.crypto.hmac_sha256<K>(key: SecretValue<K>, body: bytes)
    -> Digest ![Secret.use<K>, Error<CryptoError>]
```

`ByteStream` is a standard marker spec, not a concrete stream supertype.
Concrete opaque handles such as `TcpStream`, `TlsStream`, `FileStream`, and
`BrowserStream` satisfy `ByteStream`. This keeps stream APIs generic without
introducing general subtyping:

```etas
impl TcpStream ~ ByteStream;
impl TlsStream ~ ByteStream;
impl FileStream ~ ByteStream;
impl BrowserStream ~ ByteStream;
```

Therefore `std.tls.connect(...)` can preserve the precise return type
`TlsStream`, while `std.stream.write_all` and `std.stream.read_until_limit` can
accept it through the spec bound `S ~ ByteStream`.

When code needs a heterogeneous collection of stream handles, it should use an
existential spec object:

```etas
let streams: Array<? ~ ByteStream> = [tcp, tls, file];
```

Calling `std.stream.*` on a value of type `? ~ ByteStream` opens the
existential package for that call. The concrete stream type remains hidden, so
the inferred stream action selector is conservative, for example
`Stream.read<_>` rather than `Stream.read<TlsStream>`.

Filesystem authority should use the same spec mechanism for resource regions.
`WorkspacePath<R>` is indexed by a marker region type, `Region` classifies valid
regions, and relation specs such as `Within<Parent>` express containment
without introducing subtyping:

```etas
spec Region;
spec Within<Parent>;

type ReportsRoot = unit;
type DraftsRoot = unit;

impl ReportsRoot ~ Region;
impl DraftsRoot ~ Region;
impl DraftsRoot ~ Within<ReportsRoot>;

spec ReportsOnly: trace =
    +Fs.read<R ~ Within<ReportsRoot>>;
```

The runtime still records concrete paths in traces and enforces canonical
project-root path checks. The static effect row exposes the region boundary so
policies do not depend only on string patterns.

Pure substrate helpers should not introduce effects:

```etas
std.http.codec.encode_request(req: HttpWireRequest) -> bytes
std.http.codec.decode_response_head(bytes: bytes) -> Result<HttpWireResponseHead, HttpCodecError>
std.codec.text.utf8_decode(body: bytes, malformed: MalformedInput)
    -> Result<string, TextCodecError>
std.crypto.sha256(body: bytes) -> Digest
std.crypto.constant_time_eq(a: bytes, b: bytes) -> bool
```

`HttpWireRequest`, `HttpWireResponseHead`, header blocks, transfer-encoding
state, and body-limit values are `std.http.codec` wire-level types. They are not
EDK's high-level `edk.http.HttpRequest` or `edk.http.HttpResponse` types. EDK
packages should explicitly translate between their user-facing request/response
types and the standard wire-level codec types.

`StreamRead` distinguishes ordinary EOF from data and errors:

```etas
enum StreamRead {
    Data(bytes),
    Eof,
}

enum StreamError {
    TimedOut,
    Cancelled,
    Closed,
    Interrupted,
    LimitExceeded,
    Host(string),
}
```

Timeout, cancellation, closed streams, and body-limit violations are typed
errors. EOF is a normal stream result, not an error. `read_until_limit` returns
the accumulated bytes when the stream reaches EOF before the limit, and raises
`Error<StreamError>` if the configured limit, timeout, cancellation, or host
failure occurs first.

`SecretValue<K>` is opaque and redaction-safe. Pure code cannot reveal or
inspect its bytes. Operations such as HMAC that use secret material are ordinary
std API calls, but they are effectful through `Secret.use<K>`; public
deterministic crypto over non-secret `bytes`, such as hashing or constant-time
comparison, remains pure.

High-level production integrations should still be provided by EDK or other
packages, not by core `std`. This includes HTTP clients, web search, workspace
convenience APIs, databases, vector indexes, browser automation, email,
calendars, queues, object stores, payments, identity, deployment, package
registries, observability backends, and business APIs. Such packages may still
define effects that extend core roots:

```etas
effect CompanyEmail extends Network {
    action send<A>(account: EmailAccount, draft: EmailDraft) -> Receipt;
}

effect ProjectWorkspace extends FileIO {
    action read<P>(path: Path) -> bytes;
    action write<P>(path: Path, body: bytes) -> unit;
}
```

The language treats package-defined actions exactly like minimal standard
actions for inference, policy, trace, replay, and handler dispatch. The
difference is ownership: domain and platform actions are package-defined
authority boundaries, not core standard-library commitments.

### 1.2 Agentic Runtime Actions

Agent calls are represented internally by the `Agentic` effect root and concrete
runtime actions under it. `Agentic` is not a generic "some LLM happened" tag.
It is the agent-runtime boundary where prompt/context construction, model
provider dispatch, schema validation, trace, replay, resampling, policy, and
limits meet.

`Agentic.infer<C, O>` is the elaborated action behind source-level
`perform infer<O>(prompt)` inside an agent body or `impl agent` method. It is not
a source-level escaping effect of ordinary generated `Agent.run` calls. The
action must remain visible to analysis, policy, limits, observability, replay,
and AIR lowering, but the agent runtime installs a default handler that consumes
it through the runtime agent provider. Users should not have to write
`![Agentic.infer<C, O>]` on normal flow signatures just because a flow runs an
agent.

The standard declaration should be equivalent to:

```etas
effect Agentic {
    action infer<C, O>(prompt: Prompt, schema: Schema<O>) -> O;
}
```

`C` identifies the resolved agent method, such as `Reviewer.run` or
`Reviewer.explain`. `O` is the expected decoded output type. The agent runtime
constructs the full provider request from the prompt plus the current agent
method's model profile, tool surface, context and session metadata, limits,
policy context, trace parent id, and stable hashes needed for replay and caching.

Users normally write:

```etas
agent Reviewer(input: Draft) -> Review {
    let prompt = ReviewPrompt(input);
    return perform infer<Review>(prompt);
}
```

The compiler elaborates the `perform infer` expression to an explicit agentic
runtime boundary:

```text
PromptBuild(Reviewer.run, draft) -> Prompt
perform Agentic.infer<Reviewer.run, Review>(prompt, Schema<Review>) -> Review
PostProcess(Reviewer.run, draft, Review) -> Review
```

The generated source entrypoint remains typed as an ordinary callable:

```text
Reviewer.run : Draft -> Review
```

The requested-action metadata still records the semantic boundary:

```text
requested actions = [Agentic.infer<Reviewer.run, Review>]
escaping effects  = []
runtime support   = [Agentic]
determinism       = NonDeterministic
```

If the agent method body reads memory, calls support flows, or exposes model-callable
tools with ordinary authority actions, those effects are inferred normally and
may escape through `run`. `Agentic.infer<C, O>` itself is consumed by the standard
agent runtime boundary:

```text
Reviewer.run : Draft -> Review ![Memory.read<ProjectMemory>, AcademicSearch.search]
requested actions include [Agentic.infer<Reviewer.run, Review>, Memory.read<ProjectMemory>, AcademicSearch.search]
escaping effects exclude [Agentic.infer<Reviewer.run, Review>]
```

The broad `Agentic` root can be used as a runtime support summary when the exact
agent action is intentionally abstracted, but analysis, policy, trace, and
optimization should prefer the concrete requested action `Agentic.infer<C, O>`.

#### Runtime Agent Provider

`Agentic.infer<C, O>` has a runtime-provided default handler behind agent
methods. The provider interprets the internal action:

```text
Agentic.infer<C, O>
  -> resolve agent method C
  -> check effect boundary, policy, limits, secret flow, and prompt trust
  -> check replay/resample mode
  -> build provider payload
  -> call provider or local model runtime
  -> decode and validate O
  -> record trace, usage, latency, hashes, and finish reason
  -> return O
```

The runtime provider is not a grant of authority. It runs only after active
effect boundaries, policies, deployment grants, limits, sandbox constraints,
secret-flow checks, and prompt-trust checks allow the call. Tool calls exposed
to the model remain ordinary requested actions and effect obligations; hiding
`Agentic.infer<C, O>` does not hide package-defined actions such as
`AcademicSearch.search`, `ProjectWorkspace.write<P>`, `CompanyEmail.send<A>`, or
any other authority action the agent may request.

Test harnesses, replay systems, and local-model adapters may override the agent
provider through runtime configuration. They should not require public flow
signatures to expose `Agentic.infer<C, O>`.

For advanced source-level experiments, a scoped handler may interpret the
internal action. Such handlers are type-checked against requested-action
metadata, not the ordinary escaping effect row, so they do not force public
`run` signatures to expose `Agentic.infer<C, O>`:

```etas
let MockReviewer: ![Agentic.infer<Reviewer.run, Review> => []] = handler {
    Agentic.infer<Reviewer.run, Review>(prompt, schema) => {
        resume Review {
            score = 8,
            comment = "acceptable",
        };
    }
};

flow TestReview() -> Review {
    handle {
        Reviewer.run(sample_draft())
    } with MockReviewer
}
```

Such handlers make the same source code usable for deterministic CI mocks, trace
replay, cheap-model routing, local-only execution, dry runs, and red-team
resampling. A handler may interpret or route `Agentic.infer<C, O>`, but it must
not bypass policy, limits, secret checks, or trace recording.

Some minimal standard actions have default action implementations supplied by
the runtime or standard library. A default implementation is still subject to
the active effect boundary, policy, approval, sandbox, and limit checks. It is
not a grant of authority by itself.

Standard library APIs are thin flow wrappers over minimal standard actions:

```etas
flow std.io.println(text: string) -> unit ![Error<IOError>]
  requested_actions = [Console.stdout_write]
  default_actions = [Console.stdout_write]
{
    perform Console.stdout_write(text + "\n");
}
```

Package APIs use the same shape, but the actions are owned by the package:

```etas
flow academic.search(q: string) -> Array<Untrusted<WebPage>>
    ![AcademicSearch.search, Error<SearchError>]
{
    return perform AcademicSearch.search(q);
}
```

For host-backed actions, the action return type describes the successful return
value. The runtime default action implementation may still raise a typed host failure such as
`Error<IOError>`. Standard library wrappers expose that error in their effect
row unless they explicitly handle it. A lossy or value-level wrapper can be
provided separately, but `std.io.println` and similar core I/O flows do not
swallow errors by default.

The wrapper is the ergonomic API; the action is the semantic authority boundary visible to policy, trace, replay, and handlers.

### 1.2 Parameterized Effects

Etas supports parameterized effects and action references. A parameterized effect is still an effect-system fact, not an ordinary generic value type. The parameter narrows the observable authority scope:

```etas
![Error<ParseError>]
![Memory.read<ProjectMemory.Papers>]
![ProjectWorkspace.write<"reports/**">]
![CompanyEmail.send<WorkAccount>]
```

For memory effects, `R` must be a compiler-resolved persistent memory place:

```text
R =
    MemoryRegionHandle
  | MemoryRegionHandle.StoreField
  | MemoryRegionHandle.StoreField.NestedStoreField
```

The leftmost name must resolve to an immutable top-level resource handle created by a compiler-known standard resource constructor such as `std.memory.region<...>`. Import aliases do not create a new memory identity; they resolve to the same region handle.

```etas
type ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, PaperRecord>,
    Drafts: Store<Topic, Draft>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );

flow LoadPaper(id: PaperId) {
    return ProjectMemory.Papers.get(id);
}
```

The compiler infers:

```text
LoadPaper : PaperId -> Option<PaperRecord> ![Memory.read<ProjectMemory.Papers>]
```

Memory effect coverage is hierarchical:

```text
Memory.read<ProjectMemory> covers Memory.read<ProjectMemory.Papers>
Memory.write<ProjectMemory> covers Memory.write<ProjectMemory.Drafts>

Memory.read<ProjectMemory.Papers> does not cover Memory.read<ProjectMemory.Drafts>
Memory.read<R> does not cover Memory.write<R>
Memory.write<R> does not cover Memory.read<R>
```

This lets public boundaries choose either a broad region-level contract or a narrow store-level contract:

```etas
flow ReadAnyProjectMemory(topic: Topic) -> Option<Draft> ![Memory.read<ProjectMemory>] {
    return ProjectMemory.Drafts.get(topic);
}

flow ReadOnlyDrafts(topic: Topic) -> Option<Draft> ![Memory.read<ProjectMemory.Drafts>] {
    return ProjectMemory.Drafts.get(topic);
}
```

Store API calls contribute memory effects by rule:

| API family | Inferred effect |
|---|---|
| `get`, `select`, `query`, `contains`, `keys`, `scan` | `Memory.read<R>` |
| `put`, `insert`, `delete`, `update`, `clear` | `Memory.write<R>` |
| `upsert`, transactional read-modify-write APIs | `Memory.read<R>` and `Memory.write<R>` |

The exact API names are owned by the standard memory library, but their signatures must expose whether they read, write, or do both. User code should normally omit these effects and let inference attach them to the enclosing flow, agent, or tool signature.

Libraries and applications may define narrower effects and actions by extending a root effect:

```etas
effect ReadPDF extends FileIO {
    action fetch(citation: Citation) -> Paper;
}

effect GitHub extends Network {
    action issue_create(repo: Repo, title: string, body: Markdown) -> Issue;
}
```

Effect checking uses the extension relation. If a package defines
`StripePayment extends Network`, then a policy over `Network` also covers
`StripePayment.charge<A>`, while a policy over `StripePayment.charge<A>`
remains specific to that payment action instance.

Human approval is a standard runtime action under the human-interaction root:

```etas
type ApprovalRequest = {
    title: string,
    content: Value,
    risk: Risk
}

enum ApprovalDecision {
    Accepted;
    Rejected;
}

effect Approval extends Human {
    action request(req: ApprovalRequest) -> ApprovalDecision;
}

flow approve<T>(title: string, content: T, risk: Risk) -> bool ![Approval.request]
{
    let decision = perform Approval.request(ApprovalRequest {
        title = title,
        content = content,
        risk = risk
    });

    return decision == Accepted;
}
```

The public `approve(...)` flow is the ergonomic wrapper. The semantic boundary is the `Approval.request` action.

Approval requirements use `>>`, a trace-spec temporal operator:

```etas
Approval.request >> CompanyEmail.send<WorkAccount>;
Approval.request >> StripePayment.charge<BillingAccount>;
```

Internally this normalizes to a typed trace-spec term:

```etas
trace.before(Approval.request, CompanyEmail.send<WorkAccount>)
trace.before(Approval.request, StripePayment.charge<BillingAccount>)
```

The same operator can express non-approval guards:

```etas
Sanitized >> PromptSystemWrite;
HumanReview >> PublicPublish;
```

The intended trace-spec primitive forms are:

```text
ActionPattern = EffectRef | TrustPredicate | ReviewGate | PromptChannel | PublishAction | ...
+ActionPattern -> TraceSpec
-ActionPattern -> TraceSpec
ActionPattern >> ActionPattern -> TraceSpec
```

## 2. Effect Actions And Runtime-Scoped Handlers

An effect row is primarily a static authority and audit summary. Etas separates effect tags from effect actions:

1. an `effect` declaration defines a tag;
2. an optional `effect` block defines the actions attached to that tag;
3. an `impl EffectName` block may also add actions to the tag;
4. each `action` is semantically an effect operation boundary, not an ordinary flow implementation.

```etas
effect Error<E> {
    action raise(err: E) -> never;
}

effect Approval {
    action request(req: ApprovalRequest) -> ApprovalDecision;
}
```

This keeps the tag/action split while making the common declaration compact. The `effect` name is the static tag. Each `action` inside the block is the action interface owned by that tag. The action body is not written in Etas source; it is interpreted by a handler or by a runtime-provided implementation.

An empty tag remains valid, and its actions may be declared later with `impl`:

```etas
effect CompanyEmail extends Network;

effect Approval extends Human;

impl CompanyEmail {
    action send(account: EmailAccount, draft: EmailDraft) -> Receipt;
}
```

The same effect tag is usable in two ways:

```etas
![Approval, Error<AppError>]
```

as a static effect row, and:

```etas
let decision = perform Approval.request(req);
```

as a runtime operation that can be captured by a nearby handler.

The rules are:

1. `effect Name;` defines a tag with no source-level actions.
2. `effect Name { action op(...) -> ...; }` defines a tag plus action signatures owned by that tag.
3. `impl Name { action op(...) -> ...; }` is the separated form for declaring actions on an existing effect tag.
4. `impl` targets are kind-checked. `impl TypeName` may contain only `flow` methods, while `impl EffectName` may contain only `action` signatures.
5. An `action` signature has no body. It is interpreted by a handler or by a runtime-provided implementation.
6. `perform EffectName.actionName(...)` invokes an effect operation boundary and contributes the owning effect tag to inference.
7. Handler arms match concrete actions such as `Approval.request(req)`, not bare effect tags.
8. Tag effects appear in effect rows, manifests, trace-spec summaries, and audit summaries.
9. Action boundaries appear in AIR, trace, replay, resample, and handler dispatch.
10. Minimal runtime boundaries such as memory, sandboxed command execution, model use, approval, console I/O, and time are standard effect actions with default action implementations where appropriate.
11. Web search, HTTP, workspace I/O, email, payment, databases, browsers, object stores, deployment, identity, and other domain/platform adapters are package-defined or host-defined effect actions. Their declared effects use the same fine-grained effect/action vocabulary.
12. A handler may recover from a denied operation, but it cannot bypass the active effect boundary, policy, sandbox, approval, or limit checks.

### 2.1 Effect Actions Versus Tools

Effect actions and tools are deliberately separate:

```text
effect action = runtime operation boundary, with an optional default implementation
tool          = model-callable action boundary
```

A tool is not part of an effect. A tool is the model-callable boundary through
which an agent may request a typed action. Etas-implemented tools infer effects
from their bodies and are checked against an explicit tool boundary. Standard
and package-defined external operations are represented as effect actions with
runtime implementations:

```etas
effect AcademicSearch extends Network {
    action search(q: string) -> Array<Untrusted<WebPage>>;
}

tool safe_search(q: string) -> Array<Trusted<Snippet>> ![AcademicSearch.search]
    ~ (
        +AcademicSearch.search
        & +Sanitized
        & (AcademicSearch.search >> Sanitized)
    )
{
    let pages = perform AcademicSearch.search(q);
    return sanitize(extract_snippets(pages));
}
```

In this example, `AcademicSearch.search` is the authority-carrying runtime action and
`safe_search` is the model-callable wrapper. A trace spec can target the action:

```etas
Approval.request >> AcademicSearch.search;
```

while trace records both the model-callable tool boundary and the underlying action:

```text
ToolCall(safe_search, args)
Perform(AcademicSearch.search, q)
```

Non-standard third-party APIs and generated adapters should be described by
package metadata and wrapped by ordinary Etas tools when they need to be
model-callable:

```etas
effect GitHub extends Network {
    action issue_create(repo: Repo, title: string, body: Markdown) -> Issue;
}

tool create_github_issue(repo: Repo, title: string, body: Markdown)
    -> Issue ![Http.request<"api.github.com">, GitHub.issue_create<_>]
{
    return perform GitHub.issue_create(repo, title, body);
}
```

Effect actions are appropriate for runtime-mediated points whose behavior is
supplied by a scoped handler or by a registered default action implementation:

```etas
effect Approval {
    action request(req: ApprovalRequest) -> ApprovalDecision;
}

effect Error<E> {
    action raise(err: E) -> never;
}
```

The boundary rule is:

```text
Use `action` for standard runtime operations:
  approval, typed recoverable error, interrupt, pause/resume, human input,
  command execution, console I/O, memory, time, secrets, and model use.

Use package-defined actions for domain or platform operations:
  web search/fetch, project workspace I/O, email, payment, database access,
  browser automation, queueing, object storage, deployment, and identity.

Use `tool` for model-callable wrappers:
  safe search, safe refund, safe SQL query, safe file export, domain-specific actions.

Use package metadata or runtime bindings for non-standard host adapters:
  third-party APIs, generated OpenAPI/MCP bindings, company-internal services.
```

If a handler for an action needs to touch the outside world, it must perform allowed actions or call declared tools. Those operations still go through normal effect, policy, sandbox, approval, limit, and trace checks.

Tag granularity and action granularity are intentionally different. A tag describes a class of authority or observable behavior, such as `Approval` or `Error<E>`. An action describes a specific runtime interaction inside that class, such as `Approval.request` or `Error.raise`.

Etas handlers are runtime-scoped effect handlers, not unrestricted algebraic effects. A handler can intercept selected actions performed inside its dynamic body:

```etas
flow SafeEmail(request: string) -> EmailResult ![CompanyEmail.send<WorkAccount>]
{
    handle {
        DraftAndSendEmail(request);
        EmailResult { sent = true, message = "sent" }
    } with {
        Approval.request(req) => {
            let decision = ui.approve(req);
            resume decision;
        }

        Error<AppError>.raise(err) => {
            finish EmailResult { sent = false, message = err.message };
        }
    }
}
```

Inside `DraftAndSendEmail`, this expression:

```etas
let decision = perform Approval.request(req);
```

does not call an ordinary flow. It performs an effect action. If an enclosing `handle` has an `Approval.request` arm, the runtime pauses the current flow state, runs the handler arm, and `resume decision;` supplies the value of the `perform` expression.

`resume` is a control-flow keyword statement, not a standard-library function. It cannot be called, passed as a value, imported, shadowed, or stored. It is valid only inside a handler arm.

The first version intentionally restricts handlers:

1. `resume` is available only inside a handler arm for an action whose return type is not `never`.
2. A handler arm may resume at most once.
3. A resume continuation cannot be stored, returned, copied, or passed as a value.
4. A resume continuation cannot cross `join`, detached task, or runtime thread boundaries.
5. Actions returning `never`, such as `Error.raise`, cannot resume and must either `finish` the handled expression, abort, or perform another effect.

### 2.2 Handler Scope

Etas borrows the surface shape of algebraic effect handlers, but it does not aim to provide a general-purpose algebraic-effects calculus like Koka. Etas handlers are runtime-scoped: they make selected runtime actions visible, enforceable, recoverable, traceable, and replayable in an agent runtime.

| Dimension | Koka-style algebraic effects | Etas runtime-scoped effect actions |
|---|---|---|
| Main goal | General-purpose control abstraction | Agent runtime safety, approval, recovery, trace, replay |
| Operation/action role | A programmable semantic operation | A runtime-observable action boundary |
| Handler role | Interprets effects and can encode many control patterns | Handles approval, fallback, recovery, pause/resume, and runtime mediation |
| Continuation model | Can support richer delimited continuation behavior | Single-resume, scoped continuation only |
| Multi-shot behavior | May be supported by language design | Not supported in the MVP |
| External authority | Usually outside the core effect-handler model | Central: effect actions, sandbox, policy, approval, and limits are checked before execution |
| Durability | Not the primary semantic target | Core target: checkpoint, replay, deduplication, and audit |
| Trace | Optional implementation concern | Semantic runtime artifact |

Relative to general-purpose effect-handler languages:

| Feature | Etas current design | Koka | Effekt | Multicore OCaml | Unison |
|---|---|---|---|---|---|
| Primary goal | Agent runtime recovery, safety, trace, replay, and tool mediation | General-purpose functional effect system | General-purpose effect and ability language | Effect-handler runtime support for OCaml | Functional language with abilities |
| Effects in types | Yes, as inferred rows such as `![E]` | Yes, row-polymorphic effects | Yes | Mostly no full effect row in function types | Yes, abilities appear in types |
| Effect shape | Tag plus optional `action` declarations | Algebraic operations | Effects and operations | Extensible effect constructors | Abilities and requests |
| Handler form | `handler { ... }` values, `expr with h`, and `expr with { ... }` | Language-level handlers | Language-level handlers | `match ... with effect ...` style handling | Ability handlers |
| First-class handlers | Yes: handlers can be parameters, return values, and record fields | Supported through ordinary abstraction patterns | Supported through language abstraction patterns | More limited by OCaml's surface model | Supported through ability abstraction patterns |
| Continuation model | Scoped `resume`, single-use, not a value | General delimited-continuation model | Structured continuation model | Explicit captured continuations | Request handling with continuation semantics |
| Multi-shot continuation | Not supported in the MVP | Supported or expressible in richer designs | Supported or controlled by the design | Usually one-shot | More general than Etas, depending on ability use |
| Host-provided actions | Standard authority is modeled as effect actions; non-standard host adapters use package metadata with action-based effect rows | Usually modeled as language effects or library effects | Usually modeled as effects/abilities | Usually library/runtime concern | Can be modeled as abilities |
| Agentic inference action | `Agentic.infer<C, O>(Prompt, Schema<O>)` carries agent method identity, model profile, prompt/messages, tool surface, schema, limits, trace, replay, and policy context | Can encode an operation, but the language does not know agent-runtime semantics | Can encode an operation, but agent semantics are library-level | Can encode via effects/callbacks, but agent semantics are library-level | Can encode via abilities, but agent semantics are library-level |
| Authority model | Effect actions plus sandbox, policy, approval, limits, and deployment grants over effect instances | Usually orthogonal to handlers | Ability-oriented, but not agent-runtime specific | Mostly outside the core effect mechanism | Abilities constrain programs, but are not an agent sandbox model |
| Error handling | `Error<E>` is an effect action; `?` lowers effectful errors to `Result<T, E>` at value boundaries | Exceptions/effects can both be modeled | Effect handlers can model errors | Exceptions and effects are separate mechanisms | Abilities can model errors |
| Effect inference | Default inference; explicit rows are optional public contracts | Strong inference | Static effect information | No complete static effect-row inference | Static ability information |
| Parameterized effects | Yes, for example `Memory.read<R>` and `Error<E>` | Yes, through parametric effect rows | Yes, through parametric abstractions | Can be simulated with ordinary types | Yes, through parametric abilities |
| Replay, trace, audit | First-class runtime design goal | Not a core language goal | Not a core language goal | Not a core language goal | Not a core language goal |
| Agent nondeterminism | First-class semantic property of agent/flow execution | Not a core concept | Not a core concept | Not a core concept | Not a core concept |
| Runtime enforcement | Handlers plus effect boundaries, policy, limits, sandbox, approval, and trace | Primarily language semantics | Primarily language semantics and ability discipline | Primarily runtime control flow | Primarily ability semantics |
| Expressiveness | Deliberately weaker than unrestricted algebraic effects | High | High | Strong control-flow support, weaker static effect typing | High |
| Implementation complexity | Moderate; suitable for an interpreter and agent runtime | High | High | Runtime-level complexity | Medium to high |
| Agent-system fit | High: designed around agents, tools, approval, limits, trace, and replay | Generic; not agent-specific | Generic; not agent-specific | Generic; not agent-specific | Generic; not agent-specific |

The distinction from a generic effect language is not that Etas has handlers
and other languages cannot encode them. A generic language can define an
operation similar to `infer(prompt)`. Etas makes `Agentic.infer<C, O>` a
compiler/runtime-recognized boundary with structured agent payload: prompt trust,
context fingerprints, output schema, tool surface, token and cost accounting,
provider routing, policy context, trace ids, replay/resample metadata, and
deployment grants. This is what lets Etas statically summarize agent calls,
mock or replay them using handlers, check policy and secret flow, plan caches,
coordinate limits, and preserve audit semantics without treating model calls as
opaque library callbacks.

Etas treats `resume` as a scoped control statement, not as a first-class continuation value. Agent programs need predictable runtime mediation more than maximal control-flow expressiveness:

```text
Allowed:
  - recover from Error.raise
  - pause and resume Approval.request
  - finish the handled expression with a fallback value
  - route an action to a default action implementation
  - record the action in trace for replay or audit

Not MVP goals:
  - use handlers as generators
  - encode arbitrary coroutines
  - store continuations
  - resume a continuation multiple times
  - use handlers to grant denied effect actions
  - use handlers to bypass sandbox, policy, approval, or limit checks
```

This makes Etas less expressive than Koka-style handlers, but more aligned with static effect inference, runtime authorization, checkpointing, replay, and auditability.

Handlers affect inferred effects. If a handler fully handles `Error<AppError>`
and `Approval`, those effects do not escape the handled expression. Effects
performed by the handler body itself still count:

```text
effects(expr with h)
  = (effects(expr) - handled_effects)
    union effects(handler_arms)
```

Handler arms have two explicit completion forms:

| Form | Target | Type requirement | Control flow |
|---|---|---|---|
| `resume value;` | Current effect action | `value` has the action's declared return type | Continue the handled expression at the suspended `perform` site |
| `finish value;` | Whole handled expression | `value` has the handled expression's answer type `R` | Stop the handled expression and make `expr with h` evaluate to `value` |
| `abort(...)` or another `never` expression | Current control path | Expression type is `never`, accepted for any required arm result | Terminate the path; no value is produced |
| `return value;` | Current `flow` | `value` has the flow's return type | Flow-level return; not a handler completion form |

To avoid two meanings of `return`, a `return` statement directly inside a
handler arm is rejected unless it belongs to a nested flow declaration or flow
literal. Handler arms use `resume` to continue and `finish` to produce the
answer of the handled expression.

`Action(...) => expr` is not a handler fallback form. A handler arm must use
`resume`, `finish`, or a non-returning expression of type `never`. If the arm
uses a block, all reachable paths through that block must complete in one of
those ways. This keeps handler control flow explicit and makes effect analysis
and AIR lowering straightforward.

For example, if `Cache.get : string -> Option<string>`, then a
`Cache.get(key)` arm must resume with `Option<string>`. If the handled
expression is `BuildReport(id) : Report`, then an `Error.raise` arm that cannot
resume can either `finish` with a `Report` or terminate with `never`:

```etas
BuildReport(id) with {
    Cache.get(key) => {
        resume Some("cached text");
    }

    Error<IOError>.raise(err) => {
        finish Report {
            title = "Fallback",
            body = err.message
        };
    }
}
```

Because `never` is the bottom type, this is also valid:

```etas
BuildReport(id) with {
    Error<IOError>.raise(err) => {
        abort(err.message);
    }
}
```

Authority is not handled away. Handling `Approval.request` or `Error.raise` does not grant permission to send email, write files, execute commands, or access the network. A performed action or tool call must still be covered by the active effect boundary, policy, sandbox, approval, and limit checks.

### 2.3 Handling Actions And Default Implementations

Actions may be handled by user code even when the standard library, a package,
or the runtime provides a default action implementation. The default
implementation is the fallback used when no matching user handler is active; it
is not a special permission grant.

For a performed action:

```etas
perform ProjectWorkspace.write(path, body);
```

runtime dispatch is:

```text
1. Create a typed action request and record an attempted-action trace event.
2. Find the nearest matching scoped handler.
3. If a handler exists, run the handler arm.
4. If no handler exists, run the registered default action implementation.
5. Record whether the action was handled by user code or executed by the default implementation.
```

A handler interprets the action. If the handler resumes with a value, the default
implementation is not automatically executed:

```etas
let DryRunWorkspace: ![ProjectWorkspace.write<_> => Log.write] = handler {
    ProjectWorkspace.write(path, body) => {
        perform Log.write("would write " + path.to_string());
        resume;
    }
};

flow PreviewSave(path: Path, body: string) -> unit ![Log.write] {
    handle {
        perform ProjectWorkspace.write(path, body);
    } with DryRunWorkspace;
}
```

This is a dry run: the source body requested `ProjectWorkspace.write`, but the enclosing
handler interpreted the request and produced only `Log.write` as an escaping
effect. The trace must still include the attempted `ProjectWorkspace.write` request, so
policy review and audit can see what would have happened.

Handlers may also wrap the default implementation, but that must be explicit and
must still pass normal authority checks. The exact forwarding API is a runtime
support API rather than syntax; the design intent is:

```etas
let AuditedWorkspace: ![ProjectWorkspace.write<_> => Log.write, ProjectWorkspace.write<_>] = handler {
    ProjectWorkspace.write(path, body) => {
        perform Log.write("writing " + path.to_string());
        std.runtime.default(ProjectWorkspace.write, path, body);
        resume;
    }
};
```

`std.runtime.default(...)` means "invoke the registered default implementation
for this action, skipping the current handler arm." It is not a bypass: executing
the default workspace write still requires the matching action, such as
`ProjectWorkspace.write<_>` or `ProjectWorkspace.write<"reports/**">`, to be
allowed by the active effect boundary, policy, approval, sandbox, and limits.

The effect analysis therefore tracks two related facts:

```text
requested actions = actions performed before handler interpretation
                    plus compiler-generated runtime actions such as
                    Agentic.infer<C, O>
escaping effects  = effects that remain after handled actions are removed
                    plus effects produced by handler arms
```

For ordinary type signatures, `![...]` describes the escaping effect row, not
the full requested-action set. AIR and trace metadata should also preserve
attempted, compiler-generated, handled, and default-action execution events.
Production policies may choose to constrain requested actions as well as
escaping effects. In particular, `Agentic.infer<C, O>` is recorded as a
requested action for agent inference, but generated `Agent.run` consumes it
through the runtime agent provider and does not expose it as an escaping
source-level effect.

Support flows can be thin wrappers over actions. For example:

```etas
flow approve<T>(title: string, content: T, risk: Risk) -> bool ![Approval.request]
{
    let decision = perform Approval.request(ApprovalRequest {
        title = title,
        content = content,
        risk = risk
    });

    return decision == Accepted;
}
```

This preserves the ergonomic `approve(...)` API while giving the runtime a typed action boundary that can be handled, traced, replayed, or suspended for human input.

Handlers can also be named and reused as ordinary values whose type records the actions they handle. A standalone handler value uses the `handler` expression keyword. The shortest form records the handled effects/actions and lets the compiler infer effects produced by the handler arms:

```etas
let HumanApproval: ![Approval] = handler {
    Approval.request(req) => {
        let decision = ui.approve(req);
        resume decision;
    }
};

flow PublishWithApproval(doc: Draft) -> PublishResult {
    handle {
        Publish(doc)
    } with HumanApproval;
}
```

For example, given a simple choice action:

```etas
effect Choose<T> {
    action choose(options: List<T>) -> T;
}
```

top-level handler bindings are allowed because they create immutable handler
values; they do not run the handled computation. A top-level `let` initializer
must still be effect-free, so this is valid:

```etas
let ChooseFirst: ![Choose<i32> => []] = handler {
    Choose<i32>.choose(options) => {
        resume List.first(options);
    }
};
```

but this is not a valid top-level item, because it would execute a flow while
loading the module:

```etas
// Not a module item.
handle Search() with ChooseFirst;
```

To publish a handled computation, declare a named flow. Expression-bodied flows
are the concise form:

```etas
flow SearchOnce() -> Answer =
    Search() with ChooseFirst;
```

The handler type syntax has four forms:

```text
![H]             == ![H => _]
![H for R]       == ![H => _ for R]
![H => E]        == ![H => E]
![H => E for R]  == ![H => E for R]
```

`H` is the handled action/effect set. `_` means the handler's produced effects
are inferred from the handler arms. `E` is an explicit upper bound on produced
effects. `R` is the answer type of the handled expression, needed when a handler
arm uses `finish` rather than resuming the captured action.

Each action's return type still comes from its `action` declaration. If
`action op(...) -> B`, then `resume value;` requires `value : B`. If the arm
uses `finish value;`, then `value : R`, where `R` is the result type of the
whole `expr with h`.

For example, this handler handles approval and infers its produced effects:

```etas
let HumanApproval: ![Approval] = handler {
    Approval.request(req) => {
        std.io.println(req.title);
        let decision = ui.approve(req);
        resume decision;
    }
};
```

The compiler may infer:

```text
HumanApproval : ![Approval => Console.stdout_write, UI.approve]
```

If the public boundary should explicitly bound produced effects, write them:

```etas
let HumanApproval: ![Approval => Console.stdout_write, UI.approve] = handler {
    Approval.request(req) => {
        std.io.println(req.title);
        let decision = ui.approve(req);
        resume decision;
    }
};
```

If the handler must produce no extra effects, write an empty produced row:

```etas
let AutoApproval: ![Approval => []] = handler {
    Approval.request(req) => {
        resume Accepted;
    }
};
```

Non-resumable actions such as `Error<E>.raise(...) -> never` often need an explicit result type:

```etas
let ReportFallback: ![Error<AppError> for Report] = handler {
    Error<AppError>.raise(err) => {
        std.io.eprintln(err.message);
        finish Report {
            title = "Fallback",
            body = err.message
        };
    }
};
```

The compiler infers the produced effects:

```text
ReportFallback : ![Error<AppError> => Console.stderr_write for Report]
```

To require a pure fallback:

```etas
let PureReportFallback: ![Error<AppError> => [] for Report] = handler {
    Error<AppError>.raise(err) => {
        finish Report {
            title = "Fallback",
            body = err.message
        };
    }
};
```

Handler types can be parameterized through ordinary type parameters, usually by returning a handler value from a flow:

```etas
flow fallback<E, R>(recover: E -> R) -> ![Error<E> for R] {
    return handler {
        Error<E>.raise(err) => {
            finish recover(err);
        }
    };
}
```

They can also map one error effect to another:

```etas
let MapIOError: ![Error<IOError> => Error<AppError>] = handler {
    Error<IOError>.raise(err) => {
        perform Error<AppError>.raise(AppError.IO(err));
    }
};
```

Handler values can be passed as parameters and returned like other first-class values:

```etas
flow PublishWithApproval(
    doc: Draft,
    approval_handler: ![Approval],
) -> PublishResult {
    Publish(doc) with approval_handler
}

flow choose_approval_handler(auto: bool) -> ![Approval] {
    if auto {
        return AutoApproval;
    }

    return HumanApproval;
}
```

Anonymous handler blocks can be applied directly after `with`. The `with` form
is handler application:

```etas
let result =
    Publish(doc) with {
        Approval.request(req) => {
            resume Accepted;
        }
    };
```

Use `finish` when an arm should end the handled expression instead of resuming
the current action:

```etas
let DefaultName: ![Error<IOError> => [] for string] = handler {
    Error<IOError>.raise(err) => {
        finish "anonymous";
    }
};
```

It is equivalent to:

```etas
let result =
    handle Publish(doc) with {
        Approval.request(req) => {
            resume Accepted;
        }
    };
```

A trailing handler on a `flow` declaration applies to the whole flow body:

```etas
flow PublishWithAutoApproval(doc: Draft) -> PublishResult {
    Publish(doc)
} with {
    Approval.request(req) => {
        resume Accepted;
    }
}
```

This is equivalent to wrapping the body in a `handle` expression.

`handler` is an expression keyword, not a declaration keyword. The bare
handler-arm block `{ Action(...) => ... }` is not a general expression. Use
`handler { ... }` when binding, passing, returning, or storing an anonymous
handler value. After `with`, a handler-arm block is parsed as a handler block. A
reusable handler value does not grant authority by itself; it only supplies
interpretation for actions that are otherwise already allowed by the effect
boundary, policy, sandbox, approval, and limit checks.

## 3. Command Sandboxing

`Command` is authority-amplifying because a process can try to read files, write files, access the network, inspect environment variables, or spawn other processes.

Etas source code cannot declare an unsandboxed command. `Command.run<S>` and
`Command.spawn<S>` are parameterized by a sandbox profile, and the runtime
applies `DefaultCommandSandbox` when a wrapper omits the profile. `SandboxProfile`
and `DefaultCommandSandbox` are support values, not source syntax keywords.

Persistent state is represented by compiler-known standard APIs. Calls such as `Store.get`, `Store.put`, `Store.query`, or top-level resource constructors such as `std.memory.region<...>` contribute or expose region-sensitive effects such as `Memory.read<ProjectMemory.Papers>` and `Memory.write<ProjectMemory.Decisions>`.

```text
SandboxProfile = {
  cwd: Path,
  read: Array<PathPattern>,
  write: Array<PathPattern>,
  network: AccessMode,
  secrets: AccessMode,
  timeout: Duration
}

DefaultCommandSandbox : SandboxProfile
```

```etas
flow std.command.run(cmd: Command, sandbox: SandboxProfile) -> CommandResult ![Command.run<_>] {
    return perform Command.run(cmd, sandbox);
}
```

A project can wrap command execution as a model-callable tool, but the tool must
make the sandbox-scoped action explicit:

```etas
tool run_tests(cmd: Command) -> TestResult ![Command.run<DefaultCommandSandbox>]
{
    return test.decode(perform Command.run(cmd, DefaultCommandSandbox));
}
```

The default sandbox is intentionally restrictive. Projects may provide named `SandboxProfile` values, but command execution remains sandboxed. Unsandboxed command execution is a runtime/admin escape hatch, not normal Etas source.

## 4. Flow Type Syntax

Source-level flow type annotations may include an effect row after the output type:

```etas
Topic -> Report ![Network]
(TenantId, Query) -> Answer ![Network, Memory.read<TenantMemory>]
Topic -> Report
```

Single-input flow types may omit parentheses. Multiple-input flow types must use parentheses.

The compiler infers flow types in ordinary code and normalizes explicit annotations or inferred signatures to the core representation:

```text
I -> O ![E]  ~~>  Flow<I, O, E>
I -> O       ~~>  Flow<I, O, inferred(E)>
```

There is no separate source-level function type. Deterministic callable code is still written as `flow`; the compiler may lower it to an internal function representation.

### 4.1 Effect Row Polymorphism

Etas supports effect row polymorphism for reusable higher-order flows and
stdlib combinators. An effect row variable ranges over a finite set of effect
tags, action references, and parameterized action references.

```etas
flow twice<T, effect E>(f: () -> T ![E]) -> (T, T) ![E] {
    let a = f();
    let b = f();
    return (a, b);
}
```

`effect E` is a kinded type parameter whose values are effect rows, not value
types. Calling `twice` instantiates `E` with the latent effects of the supplied
flow. The caller sees those effects unchanged:

```etas
let r = twice(() => {
    return std.io.read_line();
});

// inferred effect row includes Console.stdin_read and Error<IOError>
```

Effect row variables can be combined with fixed effects:

```etas
flow log_then<T, effect E>(label: string, f: () -> T ![E])
    -> T ![Console.stdout_write, E]
{
    std.io.println(label);
    return f();
}
```

Explicit rows remain upper-bound contracts. If a polymorphic flow declares
`![Console.stdout_write, E]`, its body may perform console output plus whatever
effects are contributed by `E`, but no additional effects. Handlers may remove
handled actions from an instantiated row in the same way they remove concrete
actions.

## 5. Effects And Determinism

Effects do not decide whether a flow is deterministic. A flow may have effects without being non-deterministic:

```etas
flow read_paper(url: Url) -> Paper {
    return pdf.parse(http.get(url));
}
```

The compiler infers:

```text
effects = [Http.request<_>, ReadPDF.fetch]
determinism = NonDeterministic
runtime = Effectful
```

Etas uses two determinism classes:

```text
Deterministic        = depends only on explicit inputs and deterministic computation
NonDeterministic     = may observe or change external state, call runtime-provided primitives,
                       call Etas tools whose bodies are non-deterministic,
                       read time or randomness,
                       use typed memory APIs, call agents, perform model inference, request human approval,
                       call a non-deterministic subflow, or use runtime choice
```

For stage composition, determinism is ordered and combined by maximum:

```text
Deterministic < NonDeterministic
determinism(A | B) = max(determinism(A), determinism(B))
```

The composed value is always a flow:

```text
A ~ Stage<I, M, E1>
B ~ Stage<M, O, E2>
A | B : I -> O ![E1 + E2]
```

An agent method is non-deterministic when its body performs model inference,
even if it does not use tools, network access, or persistent memory directly.
The compiler records requested action `Agentic.infer<Agent.method, O>` at each
agent-scoped `perform infer` because model-mediated execution is
non-deterministic, trace-producing, schema-checked, and cost-bearing. That
requested action is not an escaping source-level effect of generated `run`:

```etas
flow Draft(topic: Topic) -> Draft {
    return Writer.run(topic);
}
```

The inferred metadata is:

```text
escaping effects  = []
requested actions = [Agentic.infer<Writer.run, Draft>]
determinism       = NonDeterministic
runtime support   = [Agentic]
```

Pure deterministic helpers also use `flow`:

```etas
flow normalize_title(title: string) {
    trim(lowercase(title))
}
```

```text
effects = []
determinism = Deterministic
runtime = Direct
```

The source language uses `flow` for ordinary callables and `tool` for
model-callable action boundaries. The implementation may still use multiple
lowering targets for deterministic flows, orchestrated flows, Etas-implemented
tools, and host-provided primitive symbols.

## 6. Declaration Defaults

Effects are inferred for every body written in Etas, but the default boundary
differs by construct:

| Construct | Omitted `![...]` means | Rationale |
|---|---|---|
| `flow` | infer and expose the body effects | Ordinary program logic should stay ergonomic |
| `tool` | infer the body effects, then check them against an empty boundary unless `![...]` or a `~ (+Action ...)` trace spec is present | Model-callable boundaries should fail closed |
| Runtime-provided primitive symbol | metadata must include explicit effect row | Compiler cannot inspect the host implementation |
| `agent` | infer harness effects; model-call authority comes only from listed tools | Agent reasoning and tool authority are separate |

For `flow`, this rule applies to `main`, expression-bodied flows, local helper
flows, exported flows, and anonymous flows. The compiler computes the effect set
from calls to agents, tools, memory APIs, support flows, performed effect
actions, handlers, loops, retries, and pipeline stages.

For non-deterministic helper flows:

```etas
flow read_paper(url: Url) -> Paper {
    return pdf.parse(http.get(url));
}
```

The compiler infers:

```etas
![Http.request<_>, ReadPDF.fetch]
```

For non-deterministic flows:

```etas
type ResearchMemorySchema = MemoryRegion<{
    Reports: Store<string, Draft>,
}>;

let ResearchMemory =
    std.memory.region<ResearchMemorySchema>(
        stable_id = "research_memory",
        store = "research"
    );

flow Research(topic: string) -> Report {
    let papers = academic.search(topic);
    let draft = Writer.run(papers);
    ResearchMemory.Reports.put(topic, draft);
    return finalize(draft);
}
```

The compiler infers:

```text
escaping effects  = [AcademicSearch.search, Memory.write<ResearchMemory.Reports>]
requested actions = [Agentic.infer<Writer.run, Draft>, AcademicSearch.search, Memory.write<ResearchMemory.Reports>]
```

A declaration may still write an explicit effect set when it is useful as a
contract. The explicit row is an upper-bound constraint, not a request to turn
off inference:

```etas
flow Research(topic: string) -> Report ![AcademicSearch.search, Memory.write<ResearchMemory.Reports>]
{
    ...
}
```

The compiler still infers the body effects and checks:

```text
inferred_effects(body) <= declared_effects(declaration)
```

For example, `main` uses the same rule as every other flow:

```etas
flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    let input = std.io.read_line();
    std.io.println(solve(input));
    return 0;
}
```

If the body additionally performs `Network`, the program is rejected unless the
declared row is widened. If the declared row contains an effect the body no
longer produces, the program remains valid; implementations may warn that the
public contract is broader than the current implementation.

For Etas-implemented tools, inference still runs, but omitted authority means
`[]`:

```etas
tool unsafe_search(q: string) -> Array<Snippet> {
    let pages = perform AcademicSearch.search(q);
    return extract_snippets(pages);
}
```

This is rejected because `AcademicSearch.search` is inferred but not allowed at the
model-callable boundary. The user must make the authority visible:

```etas
tool safe_search(q: string) -> Array<Trusted<Snippet>> ![AcademicSearch.search]
{
    let pages = perform AcademicSearch.search(q);
    return sanitize(extract_snippets(pages));
}
```

or bind an inline trace spec:

```etas
tool safe_search(q: string) -> Array<Trusted<Snippet>>
    ~ (
        +AcademicSearch.search
        & +Sanitized
        & (AcademicSearch.search >> Sanitized)
    )
{
    let pages = perform AcademicSearch.search(q);
    return sanitize(extract_snippets(pages));
}
```

For exported flows, exported agents, and public package APIs, the implementation should require either an explicit effect signature or a generated signature that is checked into package metadata. Public effect contracts should be visible during review.

High-impact actions such as `Command.run<S>`, `StripePayment.charge<A>`,
`CompanyEmail.send<A>`, `ProjectWorkspace.write<P>`, and `Secret.read<K>` should require
explicit acknowledgement at a boundary. This can be an explicit `![...]`
contract, a checked generated signature, or a policy review artifact. The goal
is not to make every local declaration verbose; it is to keep authority-changing
behavior visible at API and deployment boundaries.

Runtime-provided primitive symbols and host-provided package bindings are
different. Their package or compiler metadata must explicitly declare effects
because the compiler cannot inspect the host implementation. Etas-implemented
`tool` declarations infer body effects, but omitted `![...]` means an empty
model-callable effect boundary.

```etas
tool email.send(to: EmailAddress, subject: string, body: string) -> unit ![CompanyEmail.send<WorkAccount>];
```

## 7. Anonymous Flows And Latent Effects

Anonymous flows use `=>` and are delayed computations. Creating one does not execute body effects; the effects are latent in the arrow type and become immediate only when the value is called or executed by a combinator such as `join`.

```etas
let fetch = (url: Url) => http.get(url);
let page = fetch(url);
```

If `http.get : Url -> Page ![Network]`, then `fetch : Url -> Page ![Network]`. The `fetch` binding has no immediate effect; the `page` binding has `Network`.

Higher-order flows must preserve latent effects:

```etas
flow map_docs<A, B, E>(docs: Array<A>, f: A -> B ![E]) -> Array<B> {
    var out: Array<B> = [];

    for doc in docs
        limit Iterations(docs.len())
    {
        out = out.push(f(doc));
    }

    return out;
}
```

Returned or stored flow values expose their latent effects through type, manifest, and runtime policy:

```text
immediate_effects
latent_effects_returned
latent_effects_stored
```

Core rules:

1. `I -> O` in inferred/local positions means the effect row is inferred, not necessarily empty.
2. `I -> O ![]` explicitly means no effects escape.
3. Calling a flow value turns its latent effects into immediate effects of the caller.
4. `join([...])` unions branch latent effects and combines determinism by maximum.
5. Flow values in containers unify their effect rows.
6. Creating a flow value never grants authority; runtime checks still happen at execution.

## 8. Inference Rules

Effect inference computes an upper bound over effects that may occur during execution.

| Construct | Inferred effect |
|---|---|
| Pure expression | `{}` |
| Flow call | Callee effect set |
| Etas tool call | Tool boundary effects plus inferred body effects |
| Runtime-provided primitive call | Declared effect metadata |
| Agent call | Callee agent method summary: ordinary method-body effects, declared model-callable tool effects, typed memory API effects, other runtime effects, and requested actions from `perform infer` such as `Agentic.infer<Agent.method, O>` |
| Typed memory read | `Memory.read<R>` |
| Typed memory write | `Memory.write<R>` |
| Support flow call such as `approve(...)` | Callee effect set, usually `Approval.request` |
| Branch / `match` | Union of branch effects |
| `for` / `while` | Body effects, plus required limit checks |
| `join([...])` | Union of joined branch effects |
| `retry` | Retried body effects |
| <code>A &#124; B</code> | Union of composed stage effects; result is a `flow` |
| `x ~> Stage` | Stage summary; agent stages include requested actions from their method bodies, usually `Agentic.infer<Agent.method, O>` |
| <code>x ~&gt; (A &#124; B)</code> | Effect set of the composed flow |
| `x ~> Stage limit ...` | Stage effect plus effects from limit expressions; limits become stage-local checks |
| `perform E.action_name(...)` | Action reference `E.action_name` or `E.action_name<_>` unless a static selector is known, plus its root coverage |
| `e?` | Effects of `e` minus the captured `Error<E>`, result type becomes `Result<T, E>` |
| `expr with h` / `handle expr with h` | Expression effects minus fully handled effects, plus handler arm effects |

Example:

```etas
flow DraftReport(topic: string) -> Report {
    let papers = academic.search(topic);
    let draft = Writer.run(papers);

    if approve("Publish?", draft, risk = Medium) {
        perform CompanyEmail.send(WorkAccount, "team@example.com", "Report", draft.body);
    }

    return Report { body: draft.body };
}
```

The inferred metadata is:

```text
escaping effects  = [AcademicSearch.search, Approval.request, CompanyEmail.send<WorkAccount>]
requested actions = [Agentic.infer<Writer.run, Draft>, AcademicSearch.search, Approval.request, CompanyEmail.send<WorkAccount>]
```

## 9. Checking Rules

After inference, the compiler checks the inferred effect set against declarations and policies.

If a declaration provides an explicit effect set, the inferred set must be a
subset. Explicit effect rows are checked upper bounds for the implementation:

```text
inferred_effects(body) <= declared_effects(declaration)
```

This catches missing declarations:

```etas
flow SendReport(topic: string) -> unit ![AcademicSearch.search]
{
    perform CompanyEmail.send(WorkAccount, "team@example.com", "Report", "...");
    // compile-time error: CompanyEmail.send<WorkAccount> is inferred but not declared
}
```

Trace specs can impose additional constraints:

```etas
spec Production: trace =
    +AcademicSearch.search
    & +CompanyEmail.send<WorkAccount>
    & -StripePayment.charge<_>
    & -Command.run<_>
    & +Approval.request
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

Effect declarations and policies are the authority model. A flow may declare
`FileIO`, but a production policy can still deny `ProjectWorkspace.write<"src/**">`
or require approval before `ProjectWorkspace.write<"reports/**">`.

## 10. Runtime Enforcement

Static effect checking is not a substitute for runtime mediation.

The runtime receives the active entry flow, usually `main`, its normalized
escaping effect set, and the active policy. Every effectful operation, whether it
originates from a flow, tool, or agent, goes through runtime mediation:

1. actions that escape to runtime execution must be in the active entry flow's effect set;
2. the action must satisfy active policy;
3. high-impact effects may require a dominating approval event;
4. model-callable tools must be in the agent's declared tool set;
5. loops and retries must stay within declared limits;
6. `perform` operations must dispatch to a valid scoped handler or to a default action implementation;
7. trace events must record attempted actions, handler-dispatched actions, default-action executions, and resulting effects.

This gives Etas a three-layer discipline:

```text
Infer   = compute possible effects
Check   = compare effects against declarations and policy
Enforce = mediate actual runtime operations
```

### 10.1 Recovery Guidance From Effects

The effect system also gives the runtime recovery guidance. After a crash, checkpoint resume, or manual branch operation, the runtime must decide whether each prior step can be recomputed, replayed from trace, or resampled.

These are different operations:

```text
recompute = execute again because the step is deterministic and equivalent
replay    = do not execute again; reuse the recorded trace result
resample  = execute again intentionally to create a new result or branch
```

Effects and determinism drive the default:

| Node class | Typical effects | Default recovery | Reason |
|---|---|---|---|
| Pure deterministic flow | `[]` | Recompute | Same input should produce same output |
| Validation or formatting | `[]` | Recompute | Local deterministic work |
| External state read | `Http.request<D>`, `ProjectWorkspace.read<P>`, `Memory.read<R>`, `Clock.now` | Replay during crash recovery, refresh only by policy | Re-reading may observe different external state |
| Agent/model call | requested action `Agentic.infer<C, O>` | Replay during crash recovery, resample only by explicit policy | Model output is non-deterministic |
| Human decision | `Approval.request` | Replay if inputs are unchanged | Re-asking may change the run and user burden |
| Non-idempotent write | `CompanyEmail.send<A>`, `StripePayment.charge<A>`, `ProjectWorkspace.write<P>` | Deduplicate or replay receipt; never blind retry | Re-execution may duplicate side effects |
| Command | `Command.run<S>` | Replay output or rerun only if sandbox, policy, and idempotency allow | Commands may mutate state even in sandbox |

Example:

```etas
flow Publish(topic: Topic) -> Receipt {
    let normalized = Normalize(topic);
    let draft = normalized ~> Writer;
    let approved = approve(draft);

    if approved {
        return perform CompanyEmail.send(WorkAccount, draft);
    }

    abort("rejected");
}
```

The inferred behavior is:

```text
Normalize  : Deterministic, effects = []
Writer     : NonDeterministic, requested actions = [Agentic.infer<Writer.run, Draft>], escaping effects = []
approve    : NonDeterministic, effects = [Approval.request]
email.send : NonDeterministic, effects = [CompanyEmail.send<WorkAccount>]
```

For crash recovery after `email.send`, the safe policy is:

```text
Normalize  -> recompute
Writer     -> replay recorded draft
approve    -> replay recorded decision if draft is unchanged
email.send -> reuse receipt or deduplicate by idempotency key
```

For an explicit "regenerate draft" operation, the policy changes:

```text
Normalize  -> recompute
Writer     -> resample
approve    -> request again
email.send -> cannot reuse old side effect
```

This is a concrete contribution of the effect system to an Agent PL. Without effect and determinism summaries, the runtime must rely on ad-hoc retry rules attached to individual framework nodes. With summaries, Etas can derive conservative defaults, generate safer manifests, and reject recovery plans that would duplicate high-impact effects.

## 11. Boundary Violations And Recovery

Existing agent frameworks generally treat unsafe or unauthorized tool use as a runtime mediation problem:

| System | Relevant behavior |
|---|---|
| [OpenAI Agents SDK tool guardrails](https://openai.github.io/openai-agents-python/ref/tool_guardrails/) | Tool guardrails can allow a call, reject it with content returned to the model, or raise an exception to halt execution. |
| [OpenAI Agents SDK human-in-the-loop](https://openai.github.io/openai-agents-python/human_in_the_loop/) | Tools can require approval. Runs surface pending approvals as interruptions, serialize state, then resume after approval or rejection. |
| [LangChain human-in-the-loop middleware](https://docs.langchain.com/oss/python/langchain/human-in-the-loop) | Tool calls can pause execution for human review; decisions include approve, edit, reject, or respond, with checkpoint-backed resume. |
| [Semantic Kernel function invocation](https://learn.microsoft.com/en-us/semantic-kernel/concepts/ai-services/chat-completion/function-calling/function-invocation) and [filters](https://learn.microsoft.com/en-us/semantic-kernel/concepts/enterprise-readiness/filters) | Automatic function invocation can be disabled, and invocation filters can intercept or terminate function calls before execution. |

Etas should make this policy explicit in the language/runtime contract. The default rule is:

```text
No effectful operation executes until its effect boundary, policy, approval, sandbox, and limit checks pass.
```

This is a fail-closed boundary and a runtime conformance requirement, not a style guideline. The runtime may give the agent feedback, pause for approval, invoke a handler, or retry a transient failure, but it must not execute a denied operation.

### 11.1 Runtime Response Matrix

| Runtime event | Default behavior | Recoverability |
|---|---|---|
| Agent requests a tool outside its declared tool set | Do not execute. Return structured `ToolDenied` feedback to the agent. | May allow bounded repair if policy permits. |
| Agent requests a declared tool whose effect is outside the active run's effect boundary | Do not execute. Raise `EffectBoundaryViolation`. | Fatal for the run or deployment boundary; not a model retry. |
| Tool or action effect is not covered by the active boundary | Do not execute. Raise `EffectBoundaryViolation`. | Handler may fall back, but cannot grant authority. |
| Policy denies the operation | Do not execute. Raise or return `PolicyDenied`. | If policy declares an override path, request approval; otherwise fail. |
| Approval is required but missing | Pause and perform/request `Approval.request`. | Approved resumes; rejected returns structured denial or raises `HumanRejected`. |
| Command or tool attempts sandbox escape | Sandbox denies before side effect. Raise `SandboxViolation`. | Usually fatal; retry only if the next attempt changes arguments within the sandbox. |
| Tool timeout or transient host failure | Raise tool error. | `retry` may apply because this is failure, not authority escalation. |
| Tool implementation performs an undeclared effect | Treat as runtime integrity failure. Abort and mark tool binding invalid. | Fatal until the tool declaration or implementation is fixed. |

### 11.2 Repair Is Not Authority Escalation

Retry and model repair can help when the model called the wrong tool name, passed invalid arguments, or selected an unavailable safe alternative. They must not be used to turn a denied authority into an allowed one.

For example:

```text
ToolDenied:
  requested = email.send
  reason = effect_not_allowed CompanyEmail.send<WorkAccount>
  valid_alternatives = [email.draft, support.note]
```

The runtime may send this denial back to the agent for a bounded repair attempt. The agent may choose `email.draft` or `support.note`, but the runtime still denies `email.send` until the active effect boundary and policy allow `CompanyEmail.send<WorkAccount>`.

### 11.3 Handlers Can Recover, Not Grant Authority

Handlers can recover from denied operations by returning a fallback value, escalating to a human, or recording a pending task. They cannot bypass runtime authority checks. This is a language/runtime rule, not a recommendation: a handler cannot mutate the active effect boundary, sandbox profile, policy environment, approval state, tool allowlist, or limit budget.

```etas
flow NotifyCustomer(req: Request) -> NotifyResult {
    let receipt = perform CompanyEmail.send(
        WorkAccount,
        req.email,
        "Update",
        req.body
    );

    return NotifyResult.sent(receipt.id);
}

flow SafeNotify(req: Request) -> NotifyResult {
    handle {
        NotifyCustomer(req)
    } with {
        Error<EffectBoundaryViolation>.raise(err) => {
            perform ProjectWorkspace.write(
                "support/pending-email.md",
                err.message
            );

            finish NotifyResult {
                sent = false,
                reason = err.message,
                next = "human_support"
            };
        }
    }
}
```

If the active boundary or policy does not allow `CompanyEmail.send<WorkAccount>`, the runtime denies `CompanyEmail.send(...)` before the external side effect happens. The handler above can turn that denial into a typed fallback result and write a pending support note if `ProjectWorkspace.write<"support/**">` is allowed. It cannot make `CompanyEmail.send` execute without `CompanyEmail.send<WorkAccount>`, active policy approval, and any other runtime checks.

Human approval is similar. A trace spec may require approval before email is
sent:

```etas
spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

The runtime may then perform an approval action:

```etas
Approval.request(req) => {
    let decision = ui.approve(req);
    resume decision;
}
```

That handler supplies an `ApprovalDecision`; it does not grant email authority. After approval, the runtime still checks the effect boundary, policy, sandbox, and limits before executing `CompanyEmail.send`.

The rule is:

```text
Handler answers: what value or fallback should this action produce?
Runtime answers: is the effect action authorized to execute?
```

A conforming runtime must reject or ignore any handler path that attempts to grant authority directly. Authority can only come from the active effect boundary, selected policy, sandbox profile, approval records accepted by policy, declared tool set, and active limit budget.

### 11.4 Static Upper Bound, Runtime Trace Fact

For agents, the compiler knows the upper bound of possible effects from the declared tool set, typed memory API calls, model profile, support calls, and nested flows:

```etas
@model("gpt-5.5")
@tools([crm.lookup, email.send])
agent SupportAgent(input: Ticket) -> Reply {
    let prompt = Prompt.new()
        .system(Trusted("Draft a support reply."))
        .data(input);

    return perform infer<Reply>(prompt);
}
```

The inferred effect set is:

```text
escaping effects(SupportAgent.run)  = [CustomerDb.read<CustomerDB>, CompanyEmail.send<WorkAccount>]
requested actions(SupportAgent.run) = [
    Agentic.infer<SupportAgent.run, Reply>,
    CustomerDb.read<CustomerDB>,
    CompanyEmail.send<WorkAccount>
]
```

This does not mean every run sends email. It means the agent may send email, so deployment, policy, and approval checks must account for that possibility. The trace records the actual effects that occurred in one run.
