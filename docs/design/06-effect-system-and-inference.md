# Effect System and Inference

Types classify values; effects classify computations. `I -> O ![E]` bounds
escaping effects after handling. Requested-action and ordered trace facts retain
handled behavior for authority and temporal checks. Rows and handlers are not
permission grants.

## 1. Core Model

An effect is a named class of action relevant to safety, cost, audit, or runtime authority. Etas does not have a separate source-level authority-token concept. Concrete resource authority is expressed with fine-grained, parameterized effect actions.

```text
CoreEffectRoot = Agentic | Network | FileIO | Command | Memory | Secret | Time | Human | Error<E>
EffectSet      = Set<EffectRef>
```

The core root set should stay small. These are standard declarations with
specified compiler/runtime contracts, not additional keywords or a requirement
to implement their entire API in the host. Effect declaration/extension, rows,
inference, and handling are language mechanisms; concrete standard names and
platform behavior follow the [library boundaries](01-overview.md#4-language-and-library-boundaries).
Roots are broad categories used for
effect-root extension, trace-spec/runtime-policy coverage, and high-level
summaries. This is not value subtyping.

| Root | Meaning |
|---|---|
| `Agentic` | Managed agent construction/requests and explicit model interaction |
| `Network` | Accesses external network resources |
| `FileIO` | Reads or writes files visible to the runtime |
| `Command` | Executes a sandboxed command or child process |
| `Memory` | Reads, writes, migrates, or performs storage compaction on typed persistent memory resources |
| `Secret` | Reads, decrypts, or reveals secret material |
| `Time` | Reads wall-clock time, sleeps, schedules, or uses timers |
| `Human` | Requests human input, approval, notification, or UI-mediated decisions |
| `Error<E>` | Raises a typed recoverable failure |

Session history reads and context publication are memory operations. Generating
a summary through a model is an explicit application/EDK computation that records
its own `Agentic.infer` requested action under the ordinary default-handler
rules; it must not be hidden inside a memory read, publication, or retention
operation. Selecting an existing summary does not itself request inference.
See [Sessions And Conversations](03-agents-tools-prompts-memory.md#33-sessions-and-conversations).

Fine-grained authority is represented by effect actions. Some actions are
defined by the core prelude, while most external authorities are defined by
packages, host bindings, or user code. In an effect row, trace spec, or runtime
policy, action references are effect facts:

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
row, trace spec, or runtime-policy action pattern unless it resolves to a
compile-time constant. Concrete runtime values are ordinary action payload
fields and are recorded in trace events for exact runtime checks.

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
wildcard syntax in effect rows, trace specs, or runtime policies; write the tag
for the whole effect, or write an action/action-instance pattern for a narrower
boundary.

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

The standard library should define only orthogonal actions needed by the
language runtime and common local programs. These names are not keywords; they
are standard-library effect/action names, possibly re-exported by a prelude.
Their declarations, wrappers, and expressible handler logic can be Etas source;
host execution is needed only for the specified primitive endpoints.

| Root / effect | Minimal standard actions | Typical authority |
|---|---|---|
| `Console extends FileIO` | `stdin_read_line`, `stdin_read_all`, `stdout_write`, `stderr_write` | CLI input/output used by `std.io` |
| `Approval extends Human` | `request` | Human approval and UI-mediated decisions |
| `Clock extends Time` | `now`, `monotonic`, `sleep` | Wall-clock observations, monotonic samples and timers |
| `Random` | `entropy` | Protected platform entropy; deterministic seeded generators do not perform this action |
| `Memory` | `read<R>`, `write<R>` | Typed memory regions and stores |
| `Session extends Memory` | `read`, `publish`, `reconcile`, `acquire`, `handoff`, `release` | Session history/context and scoped exclusive routing leases; not agent-instance ownership |
| `Secret` | `read<K>`, `use<K>` | Host-mediated secret handles and non-revealing secret-backed operations |
| `Command` | `run<S>` | Sandboxed process execution scoped by sandbox profile |
| `Agentic` | `infer<C, O>`, `step<C, O>` | Full-loop and single-generation model interaction requested actions, where `C` is an agent method identity |
| `Error<E>` | `raise` | Typed recoverable failure |
| `Async` | `yield`, `await<T, E>` | Scope-bound task scheduling control, not an external-authority grant |
| `Inject<T>` | `get(name: string)` | Named, typed dependency request; no default provider or authority grant |

`Async` is exported by `std.task`. Default handlers support ordinary managed
execution; custom handlers can bind `as k` and use checked scheduler transfer.
Handling these control actions does not hide child-work effects or create a
new broad authority root. See [Task And Scheduling APIs](19-standard-library.md#2-task-and-scheduling-apis).

`Random` is a standard action owner outside the minimal broad-root hierarchy;
it is not implicitly covered by Time or Secret authority. All three clock
actions and entropy retain requested/default metadata under standard handling.
See [Time And Randomness](19-standard-library.md#13-time-and-randomness) for
their value, error, determinism and replay contracts.

The following modules define the standard substrate contracts. Their inclusion
in the SPEC does not imply that every backend already implements them:

| API module | Standard effect/action owner | Intended substrate |
|---|---|---|
| `std.task` | `Async` and recognized task-execution contracts | Structured start/wait, scoped scheduler handles, single-shot transfer, cancellation, and cleanup |
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
by `FileIO`. Trace specs and runtime policies may also target `Stream.read<_>`
or `Stream.write<_>` directly when they need to constrain byte movement
independent of origin.

`Browser` is a standard substrate effect that extends `Network`; it is not a new
core broad root. `Network` trace specs and runtime policies cover browser
protocol transport by default, while `Browser.attach<_>`, `Browser.send<_>`,
`Browser.recv<_>`, and `Browser.screenshot<_>` give trace specs a narrower
action-family surface for browser sessions. Concrete profile and session
handles are runtime action payloads, not values captured inside `<...>`. High-level actions such as
`EdkBrowser.navigate`, `EdkBrowser.click`, and `EdkBrowser.read_dom` belong in
EDK.

The source naming convention is:

| Layer | Naming style | Example |
|---|---|---|
| API path | lowercase module/function path | `std.net.tcp.connect(...)` |
| Effect/action owner | uppercase effect name, no `Std` prefix | `Net.tcp_connect<_, _>` |
| Types | uppercase type names | `TcpStream`, `TlsStream`, `WorkspacePath<ReportsRoot>` |

Public wrapper contracts distinguish returned values/errors from requested
authority. The rows below assume the standard handlers: the listed actions
remain in requested/default metadata; residual errors remain in `![...]`.
Without a covering handler, the corresponding action also escapes. Neither
case erases authority checks or makes I/O pure.

| API | Return Type Under Standard Handling | Requested Action | Contract |
|---|---|---|---|
| `std.net.tcp.connect(host: Host, port: Port, options: TcpOptions = DefaultTcpOptions)` | `TcpStream ![Error<NetworkError>]` | `Net.tcp_connect<_, _>` | Connect to the concrete host/port payload under the active network boundary. |
| `std.tls.connect(stream: TcpStream, server_name: Host, config: TlsConfig = DefaultTlsConfig)` | `TlsStream ![Error<TlsError>]` | `Tls.handshake<_>` | Validate the peer/server name; return a precise TLS stream, not an untyped byte handle. |
| `std.stream.read<S ~ ByteStream>(stream: S, max_bytes: usize, timeout: Option<Timeout> = None)` | `StreamRead ![Error<StreamError>]` | `Stream.read<S>` | At most `max_bytes`; partial reads are normal. EOF is a distinct value. |
| `std.stream.read_until_limit<S ~ ByteStream>(stream: S, byte_limit: ByteLimit, timeout: Option<Timeout> = None)` | `bytes ![Error<StreamError>]` | `Stream.read<S>` | Read through EOF with an inclusive byte limit and one whole-operation timeout. Not an HTTP message reader. |
| `std.stream.write_all<S ~ ByteStream>(stream: S, body: bytes, timeout: Option<Timeout> = None)` | `unit ![Error<StreamError>]` | `Stream.write<S>` | Complete only after all bytes are accepted, or fail; an error may follow a partial write. |
| `std.stream.flush<S ~ ByteStream>(stream: S, timeout: Option<Timeout> = None)` | `unit ![Error<StreamError>]` | `Stream.flush<S>` | Flush adapter buffering; not proof of remote receipt or durable storage. |
| `std.stream.close<S ~ ByteStream>(stream: S)` | `unit ![Error<StreamError>]` | `Stream.close<S>` | Close the resource; repeated close is harmless. Aliases observe closure, not a copied connection. |
| `std.fs.path<R ~ Region>(relative: string)` | `Result<WorkspacePath<R>, IOError> ![]` | None | Validate a relative path's lexical form; no filesystem lookup or authority grant. |
| `std.fs.root<R ~ Region>()` | `WorkspacePath<R> ![]` | None | Describe the runtime-bound region root; construction does not grant access. |
| `std.fs.relative_name<R ~ Region>(path: WorkspacePath<R>)` | `string ![]` | None | Return the region-relative name, not a host absolute-path escape. |
| `std.fs.open<R ~ Region>(path: WorkspacePath<R>, mode: FileMode)` | `FileStream<R> ![Error<IOError>]` | `Fs.open<R>` | Dispatch one typed request. The mode determines the required region read/write grants under the [file open admission rules](19-standard-library.md#file-open-action-handler-matching-and-admission). |
| `std.fs.read_bytes<R ~ Region>(path: WorkspacePath<R>)` | `bytes ![Error<IOError>]` | `Fs.read<R>` | Read file bytes; no implicit text decoding. |
| `std.fs.write_bytes<R ~ Region>(path: WorkspacePath<R>, body: bytes)` | `unit ![Error<IOError>]` | `Fs.write<R>` | Replace contents; failure can leave partial output. No atomicity/durability promise. |
| `std.fs.list<R ~ Region>(path: WorkspacePath<R>)` | `List<WorkspacePath<R>> ![Error<IOError>]` | `Fs.list<R>` | Direct children, not recursive traversal; enumeration order is unspecified. |
| `std.fs.stat<R ~ Region>(path: WorkspacePath<R>)` | `FsStat ![Error<IOError>]` | `Fs.stat<R>` | Observe current file metadata; not a promise that a later operation sees the same file. |
| `std.fs.canonicalize<R ~ Region>(path: WorkspacePath<R>)` | `WorkspacePath<R> ![Error<IOError>]` | `Fs.canonicalize<R>` | Resolve an existing path within its region; reject escape. Subsequent operations must recheck access safely. |
| `std.fs.atomic_replace<R ~ Region>(path: WorkspacePath<R>, body: bytes)` | `unit ![Error<IOError>]` | `Fs.atomic_replace<R>` | Replace as one visibility change on supported backends; report unsupported operation rather than silently use a non-atomic write. Crash durability is a separate contract. |
| `std.secret.read<K>(key: SecretKey<K>)` | `SecretValue<K> ![Error<SecretError>]` | `Secret.read<K>` | Return a redaction-safe secret handle, not inspectable bytes. |
| `std.crypto.hmac_sha256<K>(key: SecretValue<K>, body: bytes)` | `Digest ![Error<CryptoError>]` | `Secret.use<K>` | Authorize secret use without exporting key bytes. |

| `std.browser.protocol` API | Return Type Under Standard Handling | Requested Action | Contract |
|---|---|---|---|
| `attach(profile: BrowserProfile)` | `BrowserSession ![Error<BrowserError>]` | `Browser.attach<_>` | Attach under the concrete endpoint/profile policy; the profile is data, not a grant. |
| `create(profile: BrowserProfile)` | `BrowserSession ![Error<BrowserError>]` | `Browser.create<_>` | Create an owned browser session under runtime enforcement. |
| `send(session: BrowserSession, message: BrowserMessage)` | `unit ![Error<BrowserError>]` | `Browser.send<_>` | Send protocol data, not a high-level click/navigation API. Recheck the concrete session and requested operation. |
| `recv(session: BrowserSession)` | `BrowserMessage ![Error<BrowserError>]` | `Browser.recv<_>` | Receive a protocol response/event; obey inherited cancellation and limits. |
| `screenshot(session: BrowserSession)` | `BrowserScreenshot ![Error<BrowserError>]` | `Browser.screenshot<_>` | Return image bytes and format metadata from this session. |
| `close(session: BrowserSession)` | `unit ![Error<BrowserError>]` | `Browser.close<_>` | Release this session; repeated close is harmless. Detaching must not silently terminate an externally owned browser. |

| Browser Support Type | Shape / Meaning |
|---|---|
| `BrowserMessage` | Alias of `std.json.JsonValue`, not a second JSON tree. |
| `BrowserProfile` | Record: `name: string`, `endpoint: Option<std.host.url.Url>`, `headless: bool`. |
| `BrowserSession` | Opaque live connection; copying a handle does not copy a browser or grant new authority. |
| `BrowserScreenshot` | Record: `content: bytes`, `mime_type: string`, `width: Option<usize>`, `height: Option<usize>`. |
| `BrowserError` | Record: `code: string`, `message: string`; diagnostics must not disclose session secrets. |

Every filesystem operation enforces the concrete region binding and canonical
path containment at execution, including symlinks and rename races. Calling
`canonicalize` first is not a substitute for that enforcement. A static `R`
cannot grant an arbitrary runtime root.

Configuration values, exact/bounded reads, file modes, conditional cleanup and
canonical error identities are specified in
[Managed Resource Lifecycles](19-standard-library.md#14-managed-resource-lifecycles).
Filesystem extensions use the same `Fs extends FileIO` owner; none is a private
untracked host call. Generic stream wrappers retain origin/mode provenance.

`ByteStream` is a standard marker spec, not a concrete stream supertype.
Concrete opaque handles such as `TcpStream`, `TlsStream`, `FileStream<R>`, and
`BrowserStream` satisfy `ByteStream`. This keeps stream APIs generic without
introducing general subtyping:

```etas
impl TcpStream ~ ByteStream;
impl TlsStream ~ ByteStream;
impl FileStream<R ~ Region> ~ ByteStream;
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
trace specs and runtime policies do not depend only on string patterns.

Pure helpers have `![]`; malformed input is ordinary `Result`/decode-state
data, not an implicit host request or an `Error` action.

| API | Return Type | Contract |
|---|---|---|
| `std.http.codec.encode_request(req: HttpWireRequest)` | `Result<bytes, HttpCodecError>` | Validate and encode HTTP/1 wire bytes. Invalid request/header/framing data returns `Err`; never emit it unchecked. |
| `std.http.codec.decode_response_head(input: bytes)` | `Result<HttpWireResponseHead, HttpCodecError>` | Decode a complete header prefix; this convenience API does not return the body offset. |
| `std.http.codec.decode_response_head_incremental(input: bytes, end_of_stream: bool)` | `HttpDecodeStep<HttpWireResponseHead>` | Return a complete head and consumed offset, need more input, or a malformed-input diagnostic. |
| `std.http.codec.decode_response(input: bytes)` | `Result<HttpWireResponse, HttpCodecError>` | Decode one complete ordinary response; reject trailing unconsumed data. |
| `std.http.codec.decode_response_incremental(input: bytes, end_of_stream: bool)` | `HttpDecodeStep<HttpWireResponse>` | Complete according to framing, not merely socket EOF; retain trailing bytes using the consumed offset. |
| `std.codec.text.utf8_encode(text: string)` | `bytes` | Exact UTF-8 bytes. |
| `std.codec.text.utf8_decode(body: bytes, malformed: MalformedInput)` | `Result<string, TextCodecError>` | `Strict` rejects malformed UTF-8; `Replace` explicitly permits replacement. |
| `std.crypto.sha256(body: bytes)` | `Digest` | Deterministic hashing of public bytes. |
| `std.crypto.constant_time_eq(a: bytes, b: bytes)` | `bool` | Compare public byte values without content-dependent early exit; different lengths may be observed. Physical timing guarantees require backend verification. |

| `std.http.codec` Type | Shape / Meaning |
|---|---|
| `HttpHeader` | Record: `name: string`, `value: string`; duplicate header entries are retained. |
| `HttpWireRequest` | Record: `method: string`, `target: string`, `version: string`, `headers: List<HttpHeader>`, `body: bytes`. |
| `HttpWireResponseHead` | Record: `version: string`, `status: i32`, `reason: string`, `headers: List<HttpHeader>`. |
| `HttpWireResponse` | Record: `head: HttpWireResponseHead`, `body: bytes`. |
| `HttpDecodeStep<T>` | Enum: `NeedMore`, `Complete(T, usize)`, `Malformed(HttpDecodeFailure)`; the `usize` is consumed bytes from the start of this input. |
| `HttpDecodeFailure` | Record: `kind: HttpCodecFailureKind`, `offset: usize`; byte offset in the supplied input. |
| `HttpCodecError` | Coarse convenience-codec error enum: `MalformedMessage`. Detailed incremental failures use `HttpDecodeFailure`. |
| `HttpCodecFailureKind` | Closed failure-kind enum defined below; callers can distinguish incomplete, malformed, unsupported and over-limit input. |

```etas
public enum HttpCodecFailureKind {
    InvalidLineEnding, InvalidStatusLine, InvalidStatusCode, InvalidHeader,
    InvalidContentLength, ConflictingContentLength, ConflictingMessageFraming,
    InvalidChunkSize, InvalidChunkExtension, InvalidChunkTerminator,
    InvalidTrailer, ForbiddenResponseBody, UnsupportedHttpVersion,
    UnsupportedTransferEncoding, UnexpectedEof, LimitExceeded,
}
```

| Decode Boundary | Rule |
|---|---|
| Incomplete input | `NeedMore` only when more bytes could complete the message. At `end_of_stream = true`, incompleteness is `UnexpectedEof`. |
| `Complete(value, consumed)` | Keep `input[consumed, input.len())` for the next message; do not silently discard over-read bytes. |
| Resource bounds | Callers bound accumulated input/header/body sizes; codec allocations obey inherited limits. Repeated whole-prefix decoding does not promise linear cumulative work. |
| HTTP scope | These are wire-level values, not EDK request/response types. Request-aware HEAD/CONNECT, upgrades, streaming state and bounded decoder options remain open contracts in the API coverage table. |

EDK explicitly translates its high-level values to/from these wire types.

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
    UnexpectedEof,
    Busy,
    UnsupportedOperation,
    Host(string),
}
```

| Stream Boundary | Rule |
|---|---|
| Positive-size `read` | `Data` contains `1..max_bytes` bytes; an empty transient read must not masquerade as EOF or cause a busy loop. |
| Zero-size `read` | On a valid open stream, return `Data(empty bytes)` without consuming input; do not probe EOF. |
| EOF | `Eof` is normal completion, not `Error<StreamError>`. |
| Exact byte limit | EOF at exactly the limit succeeds. To distinguish it from oversized input, `read_until_limit` may consume at most one additional byte, then raise `LimitExceeded`. The error does not restore consumed data. |
| Timeout | `None` adds no local timeout but retains inherited deadlines. `Some` bounds the whole call, not each chunk afresh; never reset it on partial progress. |
| Failure / cancellation | Local failures raise the typed error; consumed input or accepted output is not rolled back. Scope cancellation still unwinds the scope: a Cancelled diagnostic cannot make it resumable. Budget errors are delivered outside the exhausted boundary under the [limit outcome rules](04-flows-human-gates-and-protocols.md#1-flow). |
| Concurrent aliases | Same-direction overlap fails Busy; TCP/TLS permit one reader plus one writer, while a shared file cursor does not. Compound operations retain admission until completion. See [stream admission](19-standard-library.md#142-stream-and-transport-values). |
| Closing | Closing a stream wakes pending operations; later reads/writes fail with `Closed`. Resource cleanup must not depend on receiving remote EOF. |
| HTTP keep-alive | `read_until_limit` waits for EOF; use incremental HTTP framing to stop at a complete message on a still-open connection. |

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
actions for inference, trace-spec checking, runtime-policy checking, trace,
replay, and handler dispatch. The difference is ownership: domain and platform
actions are package-defined authority boundaries, not core standard-library
commitments.

### 1.2 Agentic Runtime Actions

Declaring an agent performs no inference. The standard model operations are:

```etas
effect Agentic {
    action infer<C, O>(prompt: Prompt, schema: Schema<O>) -> O;
    action step<C, O>(conversation: Conversation, schema: Schema<O>) -> StepResult<C, O>;
}
```

| Boundary | Contract |
|---|---|
| `Agentic.create<A>` | Managed construction event; include supplied/injected initializer behavior and Autonomous startup separately. Its default handling does not consume dependency-provider actions. Not user-forgeable. |
| `Agentic.call<A.method>` | Typed request dispatch; retains callee/role behavior, receiver mode and request context. |
| `Agentic.infer<C, O>` | Full controller: model steps, checked tools and further steps until a validated output or failure/limit outcome. |
| `Agentic.step<C, O>` | Exactly one generation, returning `Final(O)`, `CallTools(...)` or `Stopped(...)`; no tool execution, retries or hidden format-repair calls. |
| Identity | `C` is a resolved method; concrete instance/request ids, snapshots and accounts are protected runtime context, not dynamic type arguments. |
| Default handling | These boundaries remain requested/default metadata without ordinary escaping obligations. Callee/tool/handler-produced effects and errors still propagate. |
| Authorization | Current grants, trace specs, sandbox, trust, secrecy and budgets apply even to default-handled actions. |
| Analysis | Preserve conservative transitive summaries through tool delegation and erased roles. Action sets do not encode temporal order. |

Inside an executing agent method, `perform infer<O>(prompt)` uses that method's
checked identity/configuration; O can be inferred. It is default-handled but
remains requested behavior. An ordinary record or metadata value cannot forge
that context. Whole construction/call expressions additionally record
`Agentic.create/call`; state dependencies remain even with an empty escaping row.

`infer` runs the full controller; `step` performs one generation and returns
tool requests without executing them. Both preserve trust, schema, tool-surface,
authority, budget and cancellation checks. Model/tool failures still propagate.
A mock handler may supply an inference result but cannot manufacture approval
or erase child actions. Full API/result contracts are in
[Single-Step Interaction](19-standard-library.md#4-single-step-agent-interaction).

### 1.3 Parameterized Effects

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
type ProjectMemorySchema = {
    Papers: Store<PaperId, PaperRecord>,
    Drafts: Store<Topic, Draft>,
};

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );

flow LoadPaper(id: PaperId) {
    return ProjectMemory.Papers.get(id);
}
```

Under the standard memory handler, the compiler infers:

```text
LoadPaper : PaperId -> Option<PaperRecord> ![Error<StorageError>]
requested/default actions = [Memory.read<ProjectMemory.Papers>]
```

Memory effect coverage is hierarchical:

```text
Memory.read<ProjectMemory> covers Memory.read<ProjectMemory.Papers>
Memory.write<ProjectMemory> covers Memory.write<ProjectMemory.Drafts>

Memory.read<ProjectMemory.Papers> does not cover Memory.read<ProjectMemory.Drafts>
Memory.read<R> does not cover Memory.write<R>
Memory.write<R> does not cover Memory.read<R>
```

Trace specs can choose a broad region or a narrow store without misusing the
residual error row as an authority declaration:

```etas
spec ReadAnyProjectMemory: trace = +Memory.read<ProjectMemory>;
spec ReadOnlyDrafts: trace = +Memory.read<ProjectMemory.Drafts>;
```

Store APIs preserve these requested-action facts even when standard handlers
consume the actions. Actual backend errors remain `Error<StorageError>`:

| API family | Requested / Latent Action |
|---|---|
| `get`, `get_entry`, `contains`, `keys`, `page`, `reconcile` | `Memory.read<R>` |
| `select`, `query`, `related_to`, `scan` | Deferred selection retains latent `Memory.read<R>`; collect/prompt materialization executes it. |
| Prepared `commit`, `put`, `upsert`, `insert`, `delete`, `update`, versioned mutations | `Memory.write<R>`; conditional validation is part of that atomic mutation. |
| `clear` or explicit read/modify/write composition | Both read and write; not an atomic transaction. |

Full signatures, preparation, receipt and unknown-outcome rules are in
[Persistent Stores](19-standard-library.md#15-persistent-stores-and-session-publication).
Public summaries preserve both residual errors and requested/latent actions;
ordinary source declarations may continue to infer them.

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
`StripePayment extends Network`, then a trace spec or runtime policy over
`Network` also covers `StripePayment.charge<A>`, while one over
`StripePayment.charge<A>` remains specific to that payment action instance.

Approval gates use `Approval.request >> Target.action`. They require matching
accepted evidence, not merely an attempted approval. The local permission
expansion and enclosing restrictions follow [Trace Specs](05-type-system-and-errors.md#154-trace-specs).

## 2. Effect Actions And Runtime-Scoped Handlers

### 2.1 Declarations And Tools

```etas
effect Error<E> {
    action raise(error: E) -> never;
}

effect Choice {
    action choose(left: string, right: string) -> string;
}
```

| Construct | Contract |
|---|---|
| `effect E;` | Tag without operations; a tag itself cannot be performed. |
| `effect E { action ...; }` | Tag plus owned action signatures. |
| `impl E { action ...; }` | Add action signatures to that effect, not executable bodies. |
| `perform E.action(args)` | Typed request; record the concrete action, not every action under E. |
| `tool` | Explicit model-callable source boundary, separate from effect declarations. |
| Handler/default implementation | Supplies action behavior; any actions it performs remain checked. |

`impl` is kind-checked: passive types contain flow methods, agents may also
contain member tools, effects contain action signatures. Tools may delegate to
agents while preserving transitive restrictions; only agent methods directly
use agent-scoped `infer`. See [Tool Rules](03-agents-tools-prompts-memory.md#2-tool).

### 2.2 Handler Scope

Handlers are dynamically scoped and single-shot. Values/arms capture lexically
under [Capture Rules](20-lexical-capture.md). Agent requests inherit the caller's
applicable handlers, not the receiver creator's; no caller-local mutable
capture may cross requests. Resumption targets the exact suspended request.

| Arm completion | Value type | Behavior |
|---|---|---|
| `resume value;` | Action's declared result B | Continue at the suspended `perform`; `resume;` supplies unit. |
| `finish value;` | Handled expression's answer R | End that delimited computation with R. |
| Scheduler transfer | `never` | Consume `as k`, park/requeue the original task; no detached or duplicate continuation. |
| `abort(...)` or another non-returning expression | `never` | Leave the path without producing a value. |

Ordinary action arms must explicitly complete by one of these forms.
The [dependency sugar](02-general-programming-constructs.md#72-constructor-injection)
expands field arms to `Inject<T>.get("name") => { resume expr; }`,
type arms to `Inject<T>.get(_) => { resume expr; }`.
`Action(...) => expr` is not an implicit fallback. A direct arm `return` is
rejected; a nested flow has its own ordinary return target. Actions returning
`never`, including `Error.raise`, cannot resume or bind resumable k.

`resume` is a control keyword, not a function value. Resumption reinstalls the
original handler context (deep handling); actions performed by an arm use its
enclosing handlers. `as k` exposes one scope-bound, non-copyable continuation;
transfer invalidates the local handle. It cannot be serialized, placed in an
ordinary copying container, resumed twice or detached from its task.

Delayed resumption preserves restrictions, cancellation and budget accounts.
`finish` unwinds only the delimited subtree and pending receiver updates,
not unrelated clients or already committed external actions. Scheduler and
child-task completion rules are in [Concurrency](16-concurrency.md#11-handlers-in-concurrent-scopes).

### 2.3 Handler Values And Default Handling

```etas
let first = handler {
    Choice.choose(left, right) => resume left;
};

flow choose(h: ![Choice]) -> string {
    perform Choice.choose("a", "b") with h
}
```

| Handler type | Meaning |
|---|---|
| `![H]` | Handle H; infer produced effects. |
| `![H for R]` | Also constrain the handled expression's answer type R. |
| `![H => E]` | Bound produced effects by E. |
| `![H => E for R]` | Bound both produced effects and answer type. |

Omitted produced effects mean `_`, not `[]`. Action result B comes from its
signature; R comes from the handled computation and `finish` constraints.
They need not be equal, and actions in one handler can return different types.

Handlers are first-class parameters/results/fields. Use `handler { ... }` as
a value; after `with`, either that value or a bare arm block is accepted.
`expr with h` and `handle expr with h` are equivalent. A trailing
`flow ... { body } with h` handles the entire body. A top-level handler value
is permitted; a top-level executed `handle` expression is not a module item.

```etas
flow fallback<E, R>(recover: E -> R) -> ![Error<E> for R] {
    return handler {
        Error<E>.raise(error) => {
            finish recover(error);
        }
    };
}
```

```text
escaping(expr with h) =
    (escaping(expr) - fully handled actions) union effects(handler arms)
```

Each request records attempted behavior and uses the nearest matching scoped
handler, otherwise its registered default. Resuming does not also execute the
default. `std.runtime.default(Action, args...)` explicitly invokes the registered
default while skipping the current arm; it retains all authority checks and
produced effects. Missing handling propagates an obligation, not a hidden grant.

Standard handling of `Agentic.infer/step` and `Console.*` consumes those
actions, not unrelated tool effects or typed failures. For example,
`std.io.println` still exposes `Error<IOError>`. Requested/default facts and
ordered evidence remain visible independently of the escaping row.

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

Callable types use `I -> O ![E]`; multiple inputs use `(A, B) -> O`.
An omitted local row is inferred. There is no separate function construct.

### 4.1 Effect Row Polymorphism

`effect E` binds a row parameter, not an ordinary value type. Rows can combine
fixed entries and parameters, and handlers remove fully handled obligations
from instantiated rows.

```etas
flow twice<T, effect E>(f: () -> T ![E]) -> (T, T) ![E] {
    return (f(), f());
}
```

Instantiation preserves the callback's latent effects, requested actions,
captures and scope restrictions. Explicit rows remain upper bounds, not grants.

## 5. Effects And Determinism

| Class | Behavior |
|---|---|
| `Deterministic` | Depends only on explicit inputs and deterministic computation. |
| `NonDeterministic` | May observe/change external state, infer, read time/randomness, ask a human, depend on scheduling or call behavior with these properties. |

Composition takes the maximum under `Deterministic < NonDeterministic`.
An agent's nominal identity alone does not determine the class. Model inference
is non-deterministic even when default handling leaves an empty escaping row.

Receiver mutation, captured state, state versions and non-suspension are separate
facts. `![]` does not prove purity, memoizability or safe concurrency.
Passive `var self` requires checked transitive non-suspension; active updating
requests can suspend while retaining exclusive pending state. Neither ordinary
local writeback nor captured-var assignment implicitly performs `Memory.write`.

## 6. Declaration Defaults

| Construct | Omitted effect boundary |
|---|---|
| Flow, main, lambda or agent method | Infer escaping behavior; explicit rows are checked upper bounds. |
| Tool | Infer its body, then check the empty model-callable boundary unless an explicit row or applicable positive trace constraint supplies it. |
| External primitive | Requires explicit signature/behavior metadata. |
| Agent declaration | Declares state/configuration; construction and methods have separate behavior. Model-call authority comes only from checked listed tools. |

Exports preserve inferred or explicit contracts for callers. Default handling
cannot hide requested actions at a tool or authority boundary.
Action-free parameter defaults do not request dependencies; field
[`@inject`](02-general-programming-constructs.md#72-constructor-injection)
deliberately contributes omitted provider actions at construction.

## 7. Anonymous Flows And Latent Effects

Creating a flow value does not run its body. Calling it makes its latent
effects immediate. Returning/storing/composing it preserves its effect row,
requested behavior, captures and lifetimes. Collections of callbacks must
unify compatible rows; `join` includes branch effects. Type abstraction cannot
discard these obligations or grant execution authority.

## 8. Inference Rules

Effect inference computes an upper bound over effects that may occur during execution.

| Construct | Inferred effect |
|---|---|
| Pure expression | `{}`; constructing a record is pure only when its evaluated initializers are pure. |
| Nominal record/agent construction | Supplied initializer effects, then omitted `@inject` actions and their handling behavior; active construction additionally retains `Agentic.create<A>` and startup summaries. |
| Stored-field access | Receiver-expression effects only; never re-request an injected field. Active privacy/snapshot rules still apply. |
| Flow call | Callee effect set |
| Etas tool call | Tool boundary effects plus inferred body effects |
| Runtime-provided primitive call | Declared effect metadata |
| Agent call | Callee agent method summary: ordinary method-body effects, declared model-callable tool effects, typed memory API effects, other runtime effects, and requested actions from `perform infer` such as `Agentic.infer<Agent.method, O>` |
| Typed memory read | Request `Memory.read<R>`; standard handling retains `Error<StorageError>` rather than hiding failure. |
| Typed memory write | Request `Memory.write<R>`; standard handling retains storage errors/outcome data and the operation reference. |
| Support flow call such as `approve(...)` | Callee effect set, usually `Approval.request` |
| Branch / `match` | Union of branch effects |
| `for` / `while` | Iterable/condition and body effects, plus boundary errors from any resource annotation |
| `join([...])` | Union of joined branch effects |
| `retry` | Retried body effects, plus boundary errors from any resource annotation |
| <code>A &#124; B</code> | Union of composed stage effects; result is a `flow` |
| `x ~> Stage` | Stage summary; agent stages include requested actions from their method bodies, usually `Agentic.infer<Agent.method, O>` |
| <code>x ~&gt; (A &#124; B)</code> | Effect set of the composed flow |
| `@limits(bounds)` on a declaration, loop, retry, or block | Preserve the computation's effects and requested actions; add `Error<BudgetExceeded>` when the budget may be exhausted. Local bound expressions must be deterministic, non-suspending and action-free. Bounded-prefix `for` completion alone does not add this error. |
| `perform E.action_name(...)` | Action reference `E.action_name` or `E.action_name<_>` unless a static selector is known, plus its root coverage |
| `e?` | Effects of `e` minus the captured `Error<E>`, result type becomes `Result<T, E>` |
| `expr with h` / `handle expr with h` | Expression effects minus fully handled effects, plus handler arm effects |

Constructor injection follows ordinary action handling:

```text
initialize(C { supplied }) =
    supplied expressions in source order
    ; perform omitted fields' provider requests in declaration order
    ; publish the completed record/instance

escaping(construction) = effects of that checked sequence after handling
requested(construction) includes each performed provider action
trace(construction) preserves initialization and handler execution order
```

Explicit fields suppress only their own provider requests. Provider effects
are included transitively; resolving a reference never grants authority to use
it. A missing provider is an ordinary escaping demand, rejected at a closed
boundary or runnable entry without coverage, not an implicit global lookup.
Later field reads/copies add no provider action. Handler substitution, rather
than an unchecked annotation, determines whether construction is deterministic.

## 9. Checking Rules

An explicit row must cover inferred escaping effects under the tag/selector
coverage relation. Trace constraints and runtime authority independently
restrict requested behavior. Static rejection versus exact dynamic checks is
defined in [Safety Contracts](14-safety-analysis-static-dynamic.md).

## 10. Execution And Recovery

All actual actions, including handler-produced operations, obey active authority,
trace, approval, sandbox and budget checks before protected execution.
Model requests must also belong to the checked tool surface.

| Operation | Meaning |
|---|---|
| Recompute | Execute again only with evidence of equivalent deterministic behavior. |
| Replay | Reuse a valid recorded result without repeating its action. |
| Resample | Deliberately execute again as a new outcome/branch. |
| Deduplicate | Reconcile an external write using its stable operation/idempotency contract. |

External reads may change, model results may differ and completed writes are
not rolled back. Recovery must not blindly repeat non-idempotent actions.
Reused approval must still match the input, request and validity constraints;
changing a draft can invalidate its prior approval.

## 11. Boundary Violations

No denied operation executes. Repair, a handler fallback or approval routing
may change the continuation, never the authority boundary.

| Runtime event | Default behavior | Recoverability |
|---|---|---|
| Agent requests a tool outside its declared tool set | Do not execute. Return structured `ToolDenied` feedback to the agent. | May allow bounded repair if runtime policy permits. |
| Agent requests a declared tool whose effect is outside the active run's effect boundary | Do not execute. Raise `EffectBoundaryViolation`. | Fatal for the run or deployment boundary; not a model retry. |
| Tool or action effect is not covered by the active boundary | Do not execute. Raise `EffectBoundaryViolation`. | Handler may fall back, but cannot grant authority. |
| Runtime policy denies the operation | Do not execute. Raise or return `PolicyDenied`. | If runtime policy declares an override path, request approval; otherwise fail. |
| Approval is required but missing | Pause and perform/request `Approval.request`. | Approved resumes; rejected returns structured denial or raises `HumanRejected`. |
| Command or tool attempts sandbox escape | Sandbox denies before side effect. Raise `SandboxViolation`. | Usually fatal; retry only if the next attempt changes arguments within the sandbox. |
| Tool timeout or transient host failure | Raise tool error. | `retry` may apply because this is failure, not authority escalation. |
| Tool implementation performs an undeclared effect | Treat as runtime integrity failure. Abort and mark tool binding invalid. | Fatal until the tool declaration or implementation is fixed. |

A handler may return a fallback or perform an independently allowed action.
It cannot mutate grants, sandbox profiles, tool surfaces, accepted approval
evidence or remaining budgets. Tool feedback and retries remain bounded.
May-action summaries describe possible behavior; traces describe what occurred.
