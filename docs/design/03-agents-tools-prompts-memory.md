# Agents, Tools, Messages, Prompt Values, and Persistent Memory APIs

## 1. Agent

### 1.1 Agent as a First-Class Construct

An `agent` declares a nominal managed type with isolated state and a checked
execution context. Methods execute in the caller's logical task and may suspend.
An instance is not a task, OS thread or process; it has no implicit mailbox or
worker loop. An expression of agent type yields a scoped instance reference.

| Construct | Value / execution contract |
|---|---|
| `type T = {... }` | Passive value; ordinary methods execute in the caller's task. |
| `agent A = {... };` | Declare a managed type with isolated instances, not a task per instance. |
| `a.method(args)` | Execute and return in the current logical task; suspension does not block a scheduler thread. |
| `std.task.start(() => a.method(args))` | Explicit child task; instance-access checks still apply. |
| `perform infer<O>(prompt)` | Inference in the executing agent method's checked context. |

Fields are private even when the agent type is public. External field access,
record deconstruction, schema encoding of live instances, and direct state
mutation are rejected. Ordinary records, containers and flows may hold or pass
agent references, preserving their scope bounds. Copying a reference shares an
instance; it does not clone its state or transfer lifecycle ownership.

### 1.2 Agent Type

```etas
agent Coder = {
    history: Array<string>,
    attempts: i32,
};

impl Coder {
    public flow implement(var self, task: string) -> string {
        self.attempts = self.attempts + 1;
        let prompt = Prompt.new().data(self.history).user(task);
        return perform infer<string>(prompt);
    }

    public flow stats(self) -> i32 ![] {
        return self.attempts;
    }
}
```

Type parameters and spec bounds use the ordinary type/impl mechanisms. Model,
tool and resource configuration stays in annotations, not state-field keywords.
State history is not automatically inserted into a prompt, trusted, summarized,
or made durable. Session and Store APIs retain their persistence contracts.

| Form | Meaning |
|---|---|
| `agent A = { fields };` | Private state; `{}` means no fields. Neither form generates methods. |
| `impl A { public flow run(self, x: I) -> O {... } }` | Explicit method; inputs, output and effects belong to it, not the agent declaration. |
| `A { field = value,... }` | Fresh scoped instance; construction/injection follows section 1.5. |
| `let b = a;` | Copy an existing scoped reference, not its state or ownership. |
| `a.run(x)` / `a.run` | Invoke/extract a bound method; retain instance, access mode and scope. Binding does not acquire access. |
| `x ~> A` / `A \| B` | Existing stage-operator adapter for empty-state types with explicitly defined public `run`; construct per invocation. No generated method or singleton. |
| Stateful stage | Construct an instance first, then use it or its bound method; injected fields still count as state. |
| `^A` | Reflect the declaration without construction; a bare type name is not an instance. |

Type/callable/trace specs constrain the type, methods and behavior respectively.
No function-shaped agent declaration or generated `run` exists. `A.run` identifies
a method in metadata; execution requires a receiver. Instance, declaration and
revision identities remain distinct. Metadata cannot forge references; private
state does not derive structural Eq/Hash.

Stored fields may use [`@inject` or `@inject(Action)`](02-general-programming-constructs.md#72-constructor-injection)
for construction-time resolution through `with`. A field then retains the
returned value, not its provider handler or authority. For per-call routing,
perform the dependency action inside the method; ordinary field reads remain
reads. Injection does not expose private state or convert dependency edges into
lifecycle ownership edges.

### 1.3 Operational Intuition

Resolve the receiver once and evaluate arguments/defaults before acquiring
instance access. A call then enters the checked agent context in the same task.
Different instance identities isolate local state, not external resources.

| Rule | Contract |
|---|---|
| Updating receiver | `var self` exclusively owns pending state for the whole call, including inference/await. No other activation may read/write that instance. Whole-`self` assignment cannot replace identity. |
| Read receiver | `self` holds shared read access for the whole call; other readers may overlap, writers may not. There is no implicit old-state query during an update; explicitly return ordinary snapshot data before concurrent work. |
| Commit / failure | Normal return, including `Err(...)`, commits once before exposing the result. Error escape, cancellation or `finish` unwinding discards pending state. Release access on every exit; completed external effects are not rolled back. |
| Suspension | Yield scheduler execution, not instance access. Passive `var self` remains non-suspending; agent access may suspend without creating a new task. |
| Helpers / reentry | Direct `self.method(...)` helper calls reuse the authorized receiver/pending state; only the outer updating call publishes it. A read receiver cannot upgrade to a writer. Aliased or callback reentry gets no such privilege, even in the same task. |
| Static access | Track instance aliases and read/write use through calls, captures, tasks and imports. Reject known overlapping use of the same instance when either use writes. Distinct variable/field names do not prove distinct instances. |
| Dynamic access | Where concrete identity is unknown, use an atomic pre-body guard; Strict mode requires proof. Conflict is a non-resumable access fault under ordinary task-failure propagation, not a queued call or implicit retry. Reject if no exact guard is possible. |
| References | Copies keep identity/owner, not independent update rights. No live receiver or pending-state alias may escape; cross-task captures follow their ordinary rules. |
| Determinism | Inferred from reachable computation and state/capture observations. `![]` alone proves neither purity, non-suspension nor access independence. |

`var self` on ordinary types retains its local-binding, non-suspending writeback
rules. On agents, the caller need not use a local `var`: mutation is of instance
state, not the reference binding. Access evidence preserves this distinction.

Agent methods may use ordinary control flow, call flows/tools/agents, construct
context, and explicitly invoke inference or single-step model interaction.
Inference is not mandatory. Default-handled `Agentic.create<A>`,
`Agentic.call<A.method>`, `Agentic.infer<C, O>` and `Agentic.step<C, O>` remain
requested-action metadata; their handling does not erase callee errors,
requested actions, trace obligations or cancellation.

### 1.4 Roles, Tools And Call Context

The standard marker spec `std.agent.Active` is compiler-provided evidence for
active types; user `impl` cannot give a passive type active execution semantics.
Updating agent roles entail it; `Active` does not start a worker:

```etas
spec Reviewer ~ Active {
    flow review(var self, patch: string) -> string ![];
}

agent TechLead = {
    coder: Coder,
    reviewers: Array<? ~ Reviewer>,
};
```

`? ~ Reviewer` erases the concrete agent type, not identity, lifetime,
receiver/access mode or method summaries. A deterministic mock can be an empty-state
agent implementing the same role. A passive updating value cannot be packed
as that active role.

| Boundary | Contract |
|---|---|
| Model tools | `@tools([TechLead.ask_coder])` names a declared member tool; dispatch binds the current instance. The model supplies only ordinary schema-encodable arguments, never `self`. |
| Member tools | Optional `self` is a checked immutable copy of the method state at the model/step boundary, not live receiver access. No reacquisition for this copy; calls through retained agent references still undergo access checks. |
| Delegation | Tools may call other agents, directly or through helpers. Transitive effects, requested actions, lifetimes and budgets must be checked; a tool cannot directly use agent-scoped `infer`. |
| Call context | Inherit caller handlers, restrictions, budgets, cancellation and trace links, conjoined with receiver restrictions. References grant no authority. |
| Upward effects | Use ordinary enclosing-handler lookup along the call chain, not the instance creator's environment. Handler code retains its checked lexical agent context. |
| Handler captures | An agent call alone adds no task boundary. Same-task local-var capture remains valid; explicit child tasks enforce task affinity. A handler cannot bypass a live receiver's access/escape restrictions. |
| Trace specs | `impl A ~ Guard {... }` applies a trace spec to every callable in that block; type-spec impls still require their declared methods. Operators name actions, not tool names. |

`std.agent.step` and `std.tool.invoke` provide explicit controller code under
the same invocation context; see [Single-Step Agent Interaction](19-standard-library.md#4-single-step-agent-interaction).

### 1.5 Ownership, Budgets And Supervision

Lifecycle ownership is a scope tree; calls may form a graph. Ownership does not
require the creator to execute every method: child tasks may borrow use while
the owner lives, subject to instance-access and capture checks.

| Operation | Contract |
|---|---|
| Construction | Create the instance scope first. Evaluate supplied fields once in source order, then omitted `@inject` fields in declaration order. Fresh children belong here; existing references keep their owners. Publish/start Autonomous only after success; failure cleans up new children. |
| Method-local construction | Owned by that invocation scope; cannot escape into longer-lived state/results. |
| Scope exit | Prevent new use, cancel/join owned tasks and active calls, clean up descendants, then release instances. No idle worker to keep alive. |
| Sharing | References may circulate within owner bounds. Aggregates, injected fields, role erasure and closures preserve aliases; fields alone do not prove ownership or disjointness. |
| Limits | Declaration/method bounds apply per invocation; construction-and-use scopes can bound aggregate lifetime work. Charge caller and applicable owner accounts once each; no quota reset or transferred approval. |
| Supervision | Default is structured failure propagation and cleanup, not restart. Explicit supervising flows can construct a replacement and choose retry/state recovery; old references do not silently point to it. |
| Autonomous entry | Explicit `Autonomous` conformance starts one structured `main` task after initialization under the owner context, cancelled/joined on exit. An updating main excludes other instance access throughout its activation; no waiting clients or implicit snapshot service. |
| Handoff | Routing a request is not consuming a session. Exclusive handoff uses the checked session-lease API; plain `SessionConfig` and `Message<T>` remain ordinary data. |

Shared-state services require explicit library protocols. No durable state or
remote transport is implied; external retries require their recovery contract.

## 2. Tool

### 2.1 Tool Declaration

Only a `tool` can be exposed through `@tools(...)`; ordinary flows remain
helpers. Tool signatures generate model schemas, while their bodies use
ordinary Etas logic.

| Form | Contract |
|---|---|
| `tool name(args) -> O ![E] { body }` | Source implementation; declared boundary must cover inferred escaping behavior and applicable requested-action restrictions. |
| `tool name(args) -> O;` | Imported package/interface signature with a resolvable implementation and behavior metadata; not an unbound source implementation. |
| Member tool in `impl A` | Optional snapshot `self` is bound by the runtime, omitted from the model schema. Never an updating receiver. |
| Omitted tool effect boundary | Empty boundary, unlike flow/agent-method inference; default handling does not hide requested actions from authority checks. |

Arguments/results must support the model schema/encoding contract. Secrets,
live instance references and receiver state cannot be exposed as tool results.
A registered tool is an allowed model request target, not an authority grant.

### 2.2 Tools, Actions And Flow Wrappers

| Construct | Responsibility |
|---|---|
| Effect action | Typed operation and policy-visible behavior boundary. |
| Default handler | Implements the action through checked std substrate; does not grant permission. |
| Flow wrapper | Convenient API over actions or other ordinary logic. |
| Tool wrapper | Explicit model-callable entry with schemas, validation, checked behavior and trace attribution. |

```etas
effect AcademicSearch extends Network {
    action search(query: string) -> Array<string>;
}

flow search(query: string) -> Array<string> {
    return perform AcademicSearch.search(query);
}

tool search_papers(query: string) -> Array<string> ![AcademicSearch.search] {
    return search(query);
}
```

Applications supply the action handler unless a package registers a default.
Effect roots can abbreviate action sets; temporal constraints name actions,
not the tool wrapper. Substrate and sandbox contracts are centralized in
[Effect System](06-effect-system-and-inference.md#11-minimal-standard-actions).

### 2.3 Tool Boundary Rules

| Check | Requirement |
|---|---|
| Reachability | Model requests only declared tools; direct source calls require normal symbol visibility. |
| Transitive behavior | Include helper flows, tool calls and delegated agent methods; erasure cannot discard conservative summaries. |
| Runtime checks | Validate arguments, current authority, trace constraints, sandbox and resource limits before dispatch. |
| Delegation | A tool may call an agent; recursion must remain within enforceable inherited budgets and cancellation scopes. No fresh budget or implicit grant. |
| Inference context | Agent-scoped `infer` is not available directly in a tool. A bound member tool may delegate through its snapshot's subagent references. |
| Effects | Installing a handler never removes requested-action history or deployment requirements. |
| Failure | Preserve typed errors and cancellation; no implicit retry of externally visible work. |

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
[Managed Resource Lifecycles](19-standard-library.md#14-managed-resource-lifecycles).



The requested actions in the table remain visible to trace-spec checking,
runtime policy, trace and replay. `std.io.read_line()?` has type
`Result<string, IOError>`: postfix `?` captures the error, not unwraps a result.

The [five-layer model](01-overview.md#4-language-and-library-boundaries) applies:
wrappers are Etas flows, Console actions are runtime-mediated, and neither
default handling nor std membership grants authority. A tool wrapper is needed
only when a model should request console I/O.

| Stream | CLI default |
|---|---|
| stdout / stderr | Granted for ordinary output/diagnostics. |
| stdin | Interactive or explicitly granted; consumes user input. |
| raw terminal mode | Denied; outside the specified console API. |

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
hatches.

---

## 3. Messages And Conversations

### 3.1 Message as the Agent Communication Value

`Message<T>` carries communication data and provenance between agents, tools,
humans and stores. It implies no mailbox or delivery mechanism; `Prompt`
packages model-call input.

`Message<T>`, `SessionConfig`, and `Conversation` are Agent/runtime support types, not keywords. Etas does not add `msg` or `message` declarations.

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

Method inputs and outputs are ordinary types; only model interactions require
encoding/decoding. An agent can accept a scoped agent reference for peer calls
without making that reference model-serializable.

```etas
@model("gpt-5.5")
agent Reviewer = {};
impl Reviewer {
    public flow run(self, input: Draft) -> Review {
        let prompt = Prompt.new()
.system(Trusted("Review the draft for correctness and risk."))
.data(input);
        return perform infer<Review>(prompt);
    }
}
```

| Boundary | Contract |
|---|---|
| `(Reviewer {}).run(draft)` | Explicitly constructs an empty-state instance and invokes its declared method. |
| `reviewer.run(draft)` | Calls an existing instance; does not reconstruct it. |
| `perform infer<O>(prompt)` | Agent-method shorthand for `Agentic.infer<CurrentAgent.method, O>`; `O` can be inferred. |
| Prompt assembly | Explicit `PromptEncode` use; input is never implicitly trusted or placed in the system channel. |
| Model response | `Schema<O>` and `ResponseDecode` validate/decode the response, independently of the method's eventual result. Field `@inject` does not make schema fields optional or run providers during derived decoding. |
| Controller code | `std.agent.step` returns one generation; `std.tool.invoke` checks requested tools. Conversation assembly alone does not persist history. |
| Message-aware calls | `Message<T>` retains session/provenance metadata; ordinary method calls need no message wrapper. |

Agent methods are not restricted to a build/infer/postprocess pipeline.
Model-inference typing is defined in
[Agentic Runtime Actions](06-effect-system-and-inference.md#12-agentic-runtime-actions);
single-step data and APIs are defined in
[Standard Library](19-standard-library.md#4-single-step-agent-interaction).

### 3.3 Sessions And Conversations

Active state is not durable history. Session configuration selects existing
history/context and retention; it never schedules summarization.

```etas
type SessionConfig = {
    id: SessionId,
    context: ContextPolicy,
    retention: RetentionPolicy,
}
let selection = SessionConfig {
    id = session_id,
    context = SummaryPlusRecent(recent = 8),
    retention = Days(90),
};
```

| Contract | Rule |
|---|---|
| Selection | SummaryPlusRecent reads an existing summary plus recent turns. Without a summary, return recent turns and expose its absence. Selection never deletes source messages. |
| Explicit APIs | `history_page` reads bounded history/fences; `prepare_context` binds caller content to an operation; `publish_context` conditionally installs it; `reconcile_context` resolves uncertain outcomes. |
| Generation | Application/EDK owns triggers, model/tokenizer, prompt, validation and retries. Inference remains checked and accounted; no host maintenance summarizer. |
| Context bound | `ContextTokens` is a budget, not an instruction to generate a summary. Loading/publishing cannot silently invoke a model to fit it. |
| Atomic publication | Validate session generation, history fence and context version together; reject stale content without overwriting newer state. |
| Evidence | Preserve producer/history provenance and backend-confirmed receipt/durability. Publishing neither proves semantic accuracy nor upgrades trust. |
| Failure / retry | Generation failure or confirmed rejection leaves publication unchanged. An uncertain commit requires reconciliation; reuse the same operation/outcome instead of publishing twice. |
| Retention | Archival/deletion/storage compaction remain separate operations; preserve required provenance and expose loss of replay data. |

Reading/appending persistent history uses explicit session APIs and their
effects. Attaching configuration or assembling Conversation data alone performs
no persistence. [Standard Library](19-standard-library.md#154-session-history-and-context-publication)
owns the complete types, signatures and outcome rules.

### 3.4 Handoff

Routing a `Message<T>` to another method records a communication event; it does
not itself consume session ownership. Exclusive handoff uses
[SessionLease](19-standard-library.md#154-session-history-and-context-publication):

```etas
flow Forward(session: SessionConfig, desk: ? ~ SupportDesk, text: string) {
    std.agent.session.with_lease(session, lease =>
        std.agent.session.handoff(lease, next => desk.take(next, text))
    )
}

spec SupportDesk ~ Active {
    flow take(self, lease: SessionLease, text: string) -> string;
}
```

The lease moves once; the previous generation cannot continue the conversation.
Its scope cannot escape the owning call, and failure/uncertain backend outcome
does not justify duplicate delivery. Lease possession does not grant I/O rights.
`Handoff` trace metadata links source/target request, session and lease generation.
Agent-instance lifecycle ownership is unchanged. No handoff/message keyword is added.

---

## 4. Prompt Values

### 4.1 Prompt as a Typed Agent Support Value

Prompts are not raw strings. They are typed Agent/runtime support values with trust boundaries. `Prompt` is not the primary multi-agent communication construct; it is the model-call input package used by the agent runtime. Etas does not need a dedicated `prompt` declaration ; use deterministic flows that return `Prompt`.

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
satisfy `PromptEncode`. This is spec polymorphism, not OO subtyping. Etas
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
agent Summarizer = {};
impl Summarizer {
    public flow run(self, input: Sanitized<string>) -> Summary {
        return perform infer<Summary>(SummaryPrompt(input));
    }
}

let safe_text = sanitize(web_page.body, strategy = WebText);
let summary = (Summarizer {}).run(safe_text);
```

For secret data:

```etas
let redacted = declassify(
    secret_doc,
    strategy = RedactPII,
    reason = "prepare approved material for summarization"
);

let result = agent_instance.run(redacted);
```

`WebText` is a configured `SanitizationStrategy<string, string>` and
`web_page.body` has type `Untrusted<string>`. `RedactPII` is a configured typed
`DeclassificationStrategy` for the secret document. Both strategies perform
their declared validation or transformation. Secret release checks the current
authority and records its outcome. The complete contracts are defined in
[Sanitization And Declassification](19-standard-library.md#8-sanitization-and-declassification).

---

## 5. Persistent Memory APIs

| Value | Contract |
|---|---|
| `Map<K, V>` | Ordinary in-memory value; updates do not mutate copies. |
| `Store<K, V>` | Typed persistent store field within a `MemoryRegion<S>`; access requests region-indexed actions. |
| `MemoryRegion<S>` | Immutable resource descriptor with stable identity; construction is declarative and does not connect/write at module load. |
| Commit | Creates a versioned update; confirmed no-ops need not create a version. Cancellation or uncertain commitment does not imply rollback. |

```etas
type ProjectSchema = {
    Notes: Store<string, string>,
};

let ProjectMemory = std.memory.region<ProjectSchema>(
    stable_id = "project_memory",
    store = "project-main"
);
```

Backend bindings do not change the stable region identity. Reads used for prompt
assembly remain explicit code before inference; writes remain visible requests.
Deferred selections perform reads when materialized, not when described.

Region scopes are specified in [Parameterized Effects](06-effect-system-and-inference.md#13-parameterized-effects).
CRUD, encoding, commit/reconciliation and session contracts are tabulated in
[Persistent Stores And Session Publication](19-standard-library.md#15-persistent-stores-and-session-publication).
