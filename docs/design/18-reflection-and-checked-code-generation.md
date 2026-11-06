# Reflection and Checked Code Generation

## 1. Scope And Status

This document records the proposed reflection extension for Etas. It is a
language-design target, not a claim that the current parser, checker, standard
library, or interpreter implements it. The support types and `Code.compile`
action below are part of this proposal.

In the [five-layer model](01-overview.md#53-feature-map-by-layer), `^` is a
language mechanism, typed descriptor relationships are compiler-recognized std
contracts, and `std.compiler` is a standard API over a checked execution service.
Source transforms and checker/compiler algorithms may be written in Etas;
neither that implementation choice nor ordinary descriptor field access can
forge the static target type or bypass the validated compilation boundary.

The motivating use case is an agent inspecting its current implementation,
proposing a revision, and running a checked candidate. A compiler-testing agent,
for example, can revise its test generator using feedback about invalid tests
and uncovered language features. Evaluation and promotion remain separate:
passing the compiler does not establish that a revision is an improvement.

The reflection and compilation boundaries are:

```text
agent A -> reflect -> AgentDef<A>
ordinary flow f: F -> reflect -> CallableDef<F>
generated string -> parse_body -> SyntaxBlock
AgentDef<A> + SyntaxBlock -> with_body -> AgentDef<A> -> compile -> agent A
CallableDef<F> + SyntaxBlock -> with_body -> CallableDef<F> -> compile -> flow F
```

The final value has the original agent type `A` or ordinary callable type `F`,
not a separate compilation-stage wrapper. A different implementation is
identified by revision metadata. The ordinary-flow path lets an agent revise a
helper or test generator without first turning that computation into an agent.

It builds on the [nominal agent model](03-agents-tools-prompts-memory.md#12-agent-type),
[effect and handler semantics](06-effect-system-and-inference.md), and
[package metadata](15-package-management.md). It does not introduce unrestricted
reflection over private object state, secrets, or runtime-policy internals.
The existing static-first annotation reflection remains valid; this extension
adds bounded runtime access to retained, accessible code definitions.

## 2. Reflection Syntax

Prefix `^` reflects a definition without executing its body:

```etas
let definition: AgentDef<Generator> = ^Generator;
let signature = definition.signature;
let body = definition.body;
let effects = definition.effects;
```

Member selection binds more tightly than reflection. `^Generator.run` reflects
the selected method; `(^Generator).body` selects a field of the reflection
result. The agent's signature and body are primary reflection fields, not
convenience projections requiring users to find a generated `run` method.

Agent and flow definitions expose the same callable-field conventions:

```etas
let agent_signature = (^Generator).signature;
let agent_body = (^Generator).body;
let flow_definition = ^normalize;
let flow_signature = flow_definition.signature;
let flow_body = flow_definition.body;
```

If `normalize` has type `(string) -> string ![]`, `flow_definition` has type
`CallableDef<(string) -> string ![]>`. Its type parameter is inferred from the
reflected callable, not read from the runtime `signature` field.

The initial target forms are agent and flow declarations, accessible agent
methods, and values referring to those definitions or their compiled revisions.
This includes a local `current` binding used to select a revision. Arbitrary
expressions, object layouts, and live stack frames are not implicitly
reflectable. In particular,
`^Generator.run(input)` must not silently execute the call or be treated as
quotation of that call expression.

`^candidate` reflects that agent or flow's implementation, not always the
original source declaration. Retained snapshots are immutable and identify a
specific revision. Inspecting one does not switch existing references or active
invocations to another revision.

`^` is the reflection operator proposed here. No additional reflection keyword
or revision operator is introduced.

## 3. Definition Types

### 3.1 Agent Definition

`AgentDef<A>` is an immutable code definition associated with nominal agent `A`.
Reflection produces a snapshot of an existing implementation; editing produces
another `AgentDef<A>`, not an executable agent or evidence that the edited code
is well typed. `A` is not a version number. Revisions that preserve the frozen
public contract remain associated with that agent identity; revision metadata
identifies their different implementations.

An agent declared with a signature and body is a source-level callable with
agent-specific identity and execution context. Its reflection object describes
that computation directly. Lowering it into a nominal component plus a `run`
method must not force source reflection through an `entry`/`methods` lookup.

The descriptor fields are listed below. Optional callable information represents
an agent with no default entry. Editing replaces the body and invalidates the
old analysis as a description of the new body, as specified in section 5.

| Field | Type | Meaning |
|---|---|---|
| `signature` | `Option<SignatureInfo>` | The default entry's generic binders, named inputs, and output type; `None` when there is no default entry. |
| `body` | `Option<SyntaxBlock>` | The agent's execution body, when retained and accessible. |
| `effects` | `EffectSummary` | Preserved declared bound and optional current inferred behavior of the default invocation. |
| `determinism` | `Option<Determinism>` | Current inferred determinism of that invocation, or `None` when unavailable. |
| `trace` | `Option<TraceSummary>` | Current static behavior summary of that invocation, or `None` when unavailable. |
| `identity` | `AgentId` | Stable identity of the nominal agent. |
| `revision` | `RevisionInfo` | Content fingerprint and available parent-revision metadata. |
| `annotations` | `Array<AnnotationInfo>` | Static declaration metadata, such as model and default limits. |
| `tools` | `Array<ToolRef>` | Declared model-callable tool surface, not granted execution authority. |
| `specs` | `Array<SpecInfo>` | Descriptions of attached constraints, not runtime spec evidence. |
| `dependencies` | `DependencyInfo` | Stable symbol bindings, lexical-scope information, and locked package dependencies. |
| `methods` | `Array<CallableDef>` | Read-only, type-erased descriptors of accessible additional methods introduced through `impl agent`, not a second editable copy of the default invocation. |

`AgentId`, `CallableId`, and `ToolRef` identify resolved declarations, rather than
unqualified display names. Callable identities use the existing resolved-item
identity model and distinguish methods selected through different specs.
Dependency information does not embed secret values, live connections, or
arbitrary captured execution state.

The primary summaries describe the default invocation and its transitive calls,
not the union of every auxiliary method whether called or not. Auxiliary methods
retain their own signatures and summaries. When a default entry exists,
`(^A).signature = Some((^A.run).signature)` and the other callable fields agree.
They describe one implementation, not two independently editable bodies.

The following cases describe reflected existing agents. An edited definition
can be invalid, including a replacement body without a callable signature, and
must pass checking before it can become executable.

| Agent Definition | `signature` | `body` |
|---|---|---|
| Default entry with accessible retained source | `Some(...)` | `Some(...)` |
| Default entry without accessible retained source | `Some(...)` | `None` |
| No default entry | `None` | `None` |

Missing analysis is independent of these cases. A bare agent can still expose
identity and accessible methods. `with_body` preserves its absent signature;
it does not invent a callable interface. A proposed default-body replacement
without a signature is rejected by `check` and `compile` with a diagnostic for
the missing default-entry contract. Reflection itself is not rejected merely
for being bare.

### 3.2 Shared Callable Fields And Additional Methods

Agents, flows, and additional methods share the meaning of `signature`, `body`,
`effects`, `determinism`, and `trace`. This is a consistent reflection API, not
class inheritance, value subtyping, or a claim that an agent is an ordinary flow.
An agent additionally supplies identity, annotations, the tool surface, and the
agent-scoped inference environment.

Ordinary flow reflection returns `CallableDef<F>`, where `F` is the reflected
flow's static callable type, including its escaping-effect contract. Editing
preserves `F`; successful compilation returns a callable of type `F`. The
descriptor itself is not callable and does not prove that an edited body is
valid. `CallableId` identifies the resolved definition; it does not turn the
flow into a nominal agent type.

The bare spelling `CallableDef` denotes the read-only, type-erased descriptor
view used for heterogeneous inspection. This is a specified support-type view,
not a general rule that omitted generic arguments erase types. A typed
`CallableDef<F>` can be projected to this view when the expected type is
`CallableDef`, including as an element of `Array<CallableDef>`. Its descriptive
fields and revision identity remain available, but its static connection to `F`
is no longer available to the caller. There is no inverse implicit conversion.

| Descriptor Form | Intended Use | Editing And Compilation |
|---|---|---|
| `CallableDef<F>` | Reflect an ordinary flow while retaining its static callable contract. | Supports `with_body`, `check`, and `compile`; compilation returns `F`. |
| `CallableDef` | Inspect heterogeneous flow or method definitions, including `AgentDef<A>.methods`. | Read-only metadata; not accepted by `with_body`, `check`, or `compile`. |

Method enumeration and direct method reflection retain the read-only
`CallableDef` view. Reflecting an additional method selects its own definition,
not the enclosing agent's default body. This extension adds ordinary-flow
compilation; it does not install methods, detach an agent-scoped method into a
free flow, or expose a flow as a model-callable tool. Default agent-body revision
continues to use `AgentDef<A>`.

A callable always has a signature, so `signature` is not optional on either
callable descriptor form. `AgentDef<A>.signature` is optional because an agent
need not have a default callable entry. Both callable descriptor forms expose:

| Field | Type | Meaning |
|---|---|---|
| `id` | `CallableId` | Resolved flow or method identity, including implementation ownership where applicable. |
| `signature` | `SignatureInfo` | Generic binders, named parameters, parameter types, and return type. |
| `body` | `Option<SyntaxBlock>` | Accessible retained code body; absent when implementation details are unavailable. |
| `effects` | `EffectSummary` | Preserved declared bound and optional current inferred effect/action information. |
| `determinism` | `Option<Determinism>` | `Some(Deterministic)` or `Some(NonDeterministic)` when analyzed; otherwise `None`. |
| `trace` | `Option<TraceSummary>` | Available current temporal summary, not an execution log; otherwise `None`. |
| `revision` | `RevisionInfo` | The immutable source revision and available parent-revision metadata. |
| `dependencies` | `DependencyInfo` | Permitted lexical bindings and locked dependencies, not captured execution state. |

For example, these are distinct uses of reflection:

```etas
let editable = ^normalize;
let info: CallableDef = editable;
let definitions: Array<CallableDef> = [^normalize, ^count];
```

`normalize` and `count` may have different signatures. `editable` retains the
type needed for compilation; `info` and `definitions` expose only descriptive
views. `compile(info)` is rejected even if a result type annotation is supplied.
Reflecting a statically resolved flow again can produce its typed descriptor;
reading `info.signature` cannot recover that type or authorize an unchecked cast.
`CallableDef<F>` cannot be forged by constructing a record with matching field
names, and its preserved signature cannot be changed to claim another `F`.

Generic instantiation follows ordinary flow typing. `F` may mention in-scope
type and effect parameters; reflection does not introduce first-class polymorphic
schemes or reconstruct static binders from runtime descriptor data.

`SignatureInfo` contains type and effect-parameter binder descriptions,
`Array<ParameterInfo>`, and a return-type descriptor. `ParameterInfo` records the
parameter name and its `TypeInfo`. Resolved types must remain distinguishable
from any original annotation, including when the return type was inferred.

`TypeInfo`, `SpecInfo`, and effect-row descriptors are ordinary descriptive data.
Reading a return-type descriptor does not make it legal to use that runtime
value as a source-level type annotation. Constructing or copying a descriptor
does not establish type equality, spec conformance, or permission evidence.

### 3.3 Code Representation And Visibility

`SyntaxBlock` is an immutable structured representation with source information
and any available binding information. Parsing generated text does not resolve
its names; checking resolves them in the target definition's environment.
Its `.text` projection renders the represented body as source;
exact preservation of comments and formatting is not required by this design.
Editing syntax creates new code data, not an in-place change to executable code.

Reflection must retain lexical binding information, not reconstruct dependencies
by resolving pretty-printed names in an unrelated caller scope. Runtime-generated
text is checked in the draft's permitted definition environment. It cannot
capture unrelated caller locals or acquire access merely by spelling a name.

An imported public agent or flow can expose a public signature without exposing
its body. Bodies unavailable to the reflecting scope are `None`, not empty
implementations.
Callability, source retention, and permission to inspect source are separate:
`public agent` or `public flow` does not automatically export its body.
Cross-package body access requires the package author's explicit body export and retained code in the
published artifact; retaining code alone does not make private bodies visible.
Source visibility does not grant execution authority or expose live captured
values. Retention and export are configured in the owning package's `etas.toml`:

```toml
[reflection]
retain_bodies = ["app.Generator"]
export_bodies = ["library.Rewriter"]
```

Both fields default to empty lists and name fully qualified items owned by the
package, not filesystem paths. `retain_bodies` includes source in its artifact
for access under normal visibility rules; `export_bodies` makes a public item's
body available across packages and implies retention. Entries must resolve;
exporting a non-public item is an error. A consuming package cannot use its own
configuration to expose a dependency's unexported bodies. Package metadata records
resolved item identities, body availability/export status, and source fingerprints.

Runtime-compiled revisions inherit the target definition's retention/export
settings, so a retained RSI target does not lose its body on the next `^current`. These
settings neither publish a new version nor change the current bindings. See the
[package manifest](15-package-management.md#4-package-manifest).

## 4. Effect And Trace Descriptors

`EffectSummary` must not collapse all behavior into one set:

| Field | Type | Meaning |
|---|---|---|
| `declared_bound` | `Option<EffectRowInfo>` | Explicit escaping-effect upper bound, when supplied. `None` differs from an explicit empty row. |
| `escaping` | `Option<EffectRowInfo>` | Current inferred effects after handler interpretation, including handler-produced effects; `None` when unavailable. |
| `requested` | `Option<ActionSummary>` | Current may-action summary, including requests consumed by handlers; `None` when unavailable. |
| `default_handled` | `Option<ActionSummary>` | Current summary of requests handled by default providers; `None` when unavailable. |

`None` in an inferred field means no current analysis, not an empty effect row,
no action requests, or deterministic execution. It differs from a known empty
summary such as `Some([])`. `declared_bound = None` instead means that the user
omitted an explicit bound; body editing does not change that declaration.

Descriptors preserve parameterized actions and open row variables where
applicable; they are not just arrays of effect-name strings. Requested actions
are may-information, not a record of what occurred during a particular run.

For a method whose only request is model inference, with its default handler
introducing no escaping effect, a summary can be:

```text
escaping       = Some([])
requested      = Some([Agentic.infer<Generator.run, string>])
default_handled = Some([Agentic.infer<Generator.run, string>])
determinism    = Some(NonDeterministic)
```

The action is recorded because the body performs inference, not because an
`agent` declaration exists. Requested-action sets do not encode order.
`TraceSummary` separately represents or references the analysis information for
ordering, branching, and temporal constraints; its representation must agree
with the [safety analysis](14-safety-analysis-static-dynamic.md), not invent a
second policy semantics. Copying it does not transfer a proof to modified code.

## 5. Editing And Compilation Results

`AgentDef<A>` and `CallableDef<F>` describe code; neither is executable. Editing
does not execute the code or establish that it is well typed. Successful
compilation returns an agent of type `A` or an ordinary callable of type `F`,
invoked in the same way as the original. Revision metadata distinguishes the
implementation. No extra public type is needed solely to say that compilation
has succeeded.

The editing and compiler APIs preserve the descriptor family and target type:

```text
std.compiler.parse_body(source: string) -> Result<SyntaxBlock, ParseError>
agent_definition.with_body(body: SyntaxBlock) -> AgentDef<A>
flow_definition.with_body(body: SyntaxBlock) -> CallableDef<F>
std.compiler.check<A>(definition: AgentDef<A>) -> CheckReport ![Error<CompileError>]
std.compiler.compile<A>(definition: AgentDef<A>) -> A ![Error<CompileError>]
std.compiler.check<F>(definition: CallableDef<F>) -> CheckReport ![Error<CompileError>]
std.compiler.compile<F>(definition: CallableDef<F>) -> F ![Error<CompileError>]
```

Here `agent_definition: AgentDef<A>` and `flow_definition: CallableDef<F>`;
`F` must be an ordinary callable type, not an arbitrary result type chosen by
the caller. The compiler-service overload is selected from the descriptor's
static type. Thus `std.compiler.compile(^normalize)` is valid when the retained
definition and its contract can be checked. Its result has the original flow
type, without an explicit generic argument. Missing required body information
produces a rejection diagnostic rather than an executable placeholder.

`parse_body` accepts the source contents of a body, without its enclosing braces.
It is pure and returns `Err(...)` containing a `ParseError` for invalid syntax.
A successful parse does not establish name resolution, typing, or spec
conformance. Its result is already a `Result`; no effect-to-`Result` `?` is needed
to inspect it.

`with_body` is a pure operation that returns a new definition with
`body = Some(body)`, retaining the base revision's identity, signature, contract,
and permitted dependency environment. It neither mutates the original definition
or executable value nor runs the replacement. It accepts a `SyntaxBlock`, not a
source `string`; model output must be parsed explicitly first. `edited` or `draft` is
just a variable name for the resulting `AgentDef<A>` or `CallableDef<F>`, not
another public type.

### 5.1 Analysis After Editing

Compiler-derived summaries are read-only. `with_body` preserves the explicit
declarations and frozen contract, but sets the edited definition's current
`effects.escaping`, `effects.requested`, `effects.default_handled`, `determinism`,
and `trace` to `None`. The original definition keeps its old analysis; edited
fields must not return those old values as if they described the replacement.
Any auxiliary-method summaries depending on the changed body are invalidated too.

`check(edited)` returns available new analysis in `CheckReport`, tied to the
edited source/dependency fingerprint. It does not mutate the immutable `edited`
value. Compilation performs the necessary validation independently, reusing
analysis only when its fingerprints match. `^candidate` exposes analysis for the
successfully compiled implementation. Explicit constraints remain obligations
even when all current inferred fields are `None`.

### 5.2 Frozen Compatibility

Returning type `A` or `F` requires compatibility with the frozen caller-visible
contract, not merely the same nominal name or input/output types. A flow's arrow
type alone does not encode all requested-action and temporal obligations; its
descriptor must retain the frozen analysis contract as well. Editing and
successive revisions preserve that contract; candidate inference cannot widen
it or replace it with whatever the new body happens to do.

| Contract Component | Required Compatibility |
|---|---|
| Callable interface | Preserve the input/output interface, generic constraints, and promised spec conformance. |
| Escaping effects | Newly inferred effects must fit the frozen interface's escaping-effect upper bound. |
| Requested actions | New requests must fit its caller-visible action upper bound, including handled requests and relevant parameter scopes. An escaping row is not that bound. |
| Temporal behavior | Preserve the trace-state assumptions and guarantees used in caller-side composition, including behavior before and after the call. Merely checking the candidate's own attached specs is insufficient. |
| Determinism | Preserve a promised `Deterministic` classification; such a computation cannot be replaced by a `NonDeterministic` one. |

This freezes a contract, not the old body's exact execution sequence. A candidate
may change its algorithm and use any behavior permitted by that contract. With
an inferred interface, the established caller-visible summary must be retained
as the reference for compatibility, rather than recomputed into a broader
interface after editing. Summary comparison follows the existing
[caller-side trace analysis](14-safety-analysis-static-dynamic.md#573-caller-side-trace-spec-checking).
For example, these implementations can have identical input/output types and
action sets but are not interchangeable under an approval-before-send contract:

```text
old: approval succeeds -> send
new: send -> request approval
```

Compatibility uses conservative inclusion checks over the established abstract
summaries. For each permitted abstract entry state, candidate behavior must fit
the frozen allowed behavior, including caller-visible outcome distinctions and
intermediate violations. Merely comparing final monitor states is insufficient:
a bad intermediate action cannot be erased by a later approval or return.

Compilation must establish compatibility before returning `A` or `F`. An unproved
relation is a rejection diagnostic, not acceptance and not an instruction to
silently recompile all callers. Explicit residual checks can discharge dynamic
obligations only under the existing static/dynamic safety rules, with their
enforcement included in the validated contract. A changed public contract needs
an explicit boundary change and caller revalidation outside this replacement
API. The check is deliberately sound but incomplete; it need not recognize
every behaviorally compatible rewrite.

Body-specific optimization assumptions require additional revision/dependency
validation. Contract compatibility alone does not authorize reusing old inlined
code or cached analysis for the new implementation. Such results must be
invalidated or revalidated before reuse. More precise analyses may prove more
rewrites compatible, but cannot weaken these acceptance conditions.

### 5.3 Adopting A Revision

Adopting a compiled implementation is separate from editing and compilation:
`current = candidate` replaces a local `var` binding of type `A` or `F`; subsequent
calls through `current` execute the new implementation. Other bindings, the
original declaration, and in-progress calls do not change. No additional
creation, replacement, or execution API is needed for this local update.

## 6. Effectful Compilation

Reading retained immutable metadata and constructing drafts can be pure.
Loading source from disk or the network uses the ordinary I/O actions instead
of hiding that work behind `^`.

The proposed compiler-service actions are `Code.check` and `Code.compile`.
Both support the same two typed descriptor families as their standard wrappers:

| Input | `Code.check` Result | `Code.compile` Result |
|---|---|---|
| `AgentDef<A>` | `CheckReport` | Executable agent of type `A`. |
| `CallableDef<F>` | `CheckReport` | Ordinary callable of type `F`. |

The result is not a definition descriptor. `A` or `F` is inferred from the input,
so ordinary calls need no explicit type argument. Overload selection does not
depend on inspecting a runtime signature string. The standard flow wrappers
preserve these input and result types; neither action accepts the erased
`CallableDef` view as a typed compilation target.

The default handlers invoke the checker/compiler service. Invalid candidate code
is a normal rejected `CheckReport` from `check`; `compile` instead raises
`Error<CompileError>` with rejection diagnostics. Operational failure of either
service uses the same `Error<CompileError>`, classified as a service failure.
Neither action invokes the generated agent or flow or performs that candidate's
model/tool actions. The candidate's effects are latent until it is called.

The standard boundary handles `Code.compile` while preserving it in requested
actions and recording default handling. Under that default handler, the ordinary
escaping error obligation is `Error<CompileError>`; an overriding handler can
introduce other effects. Any implementation-side substrate requests must also
remain visible according to ordinary handler rules.

Custom handlers can log, cache, reject, or supply a compatible previously checked
agent of type `A` or callable of type `F` for `Code.compile`. They cannot forge
checked artifacts, bypass revision/dependency validation, or grant the generated
code new authority.
Compilation and candidate execution remain subject to their active trace specs,
runtime policy, and limits.

`?` keeps its existing effect-to-`Result` meaning:

```etas
let outcome = (perform Code.compile(edited))?;
// Result<A, CompileError> for AgentDef<A>, or Result<F, CompileError>
// for CallableDef<F>, under the standard compile handler.
```

This only captures the one residual `Error<CompileError>`. It is not an unwrap,
an aborting default handler, or a catch-all for multiple unrelated error types.
Running the returned candidate has that candidate's checked behavior contract;
its effects must not disappear behind a generic "run code" operation.

### 6.1 Standard Library Surface

Public APIs use the lowercase `std.compiler` namespace. Effect actions remain
owned by `Code`; this proposal does not introduce a parallel `Compiler` effect.
The API and support-type names below describe the design surface, not existing
implemented standard-library symbols.

| API | Result | Purpose And Effect Boundary |
|---|---|---|
| `std.compiler.parse_body(source: string)` | `Result<SyntaxBlock, ParseError>` | Parse body contents without enclosing braces and locate syntax errors. Pure for in-memory input; parsing alone proves neither name resolution nor typing. |
| `std.compiler.format(syntax)` | `string` | Produce stable readable source from syntax. Pure; formatting does not authorize execution or resolve new dependencies. |
| `std.compiler.check<A>(definition: AgentDef<A>)` | `CheckReport` | Check names, visibility, types, effects, handlers, specs, limits, and determinism without producing an executable candidate. Invalid code is a rejected report; service failure raises `Error<CompileError>`. Wraps `Code.check`; `A` is inferred. |
| `std.compiler.compile<A>(definition: AgentDef<A>)` | `A` | Check and compile a new implementation of agent type `A` under its contract. Wraps `Code.compile`; checking/compilation failures use `Error<CompileError>`, and `A` is inferred. |
| `std.compiler.check<F>(definition: CallableDef<F>)` | `CheckReport` | Check an edited ordinary flow against its frozen contract without executing it. Wraps `Code.check`; rejection/report and service-error rules are the same as for agents. |
| `std.compiler.compile<F>(definition: CallableDef<F>)` | `F` | Check and compile an ordinary flow under its frozen contract. Wraps `Code.compile`; checking/compilation failures use `Error<CompileError>`, and `F` is inferred. |
| `std.compiler.diff(old, new)` | `ChangeReport` | Compare available source and analysis metadata of immutable snapshots. Pure; it is not a proof of behavioral equivalence or improvement. |

Reflection already supplies definition inspection, and
`definition.with_body(syntax)` constructs an edited `AgentDef<A>` or
`CallableDef<F>`. Neither needs a duplicate compiler-service request. `A` and
`F` retain the original target's static contract; neither is a new type chosen
by generated text or a version-indexed wrapper. The erased `CallableDef` view
does not provide editing, checking, or compilation overloads.
Persistence uses source and locked-environment restoration as specified in
section 6.5. Binary artifact export/loading is not part of this public API.

Calling `check` before `compile` is optional. Compilation must perform all
necessary validation itself; it may reuse a matching check result, but cannot
accept an arbitrary user-constructed report as evidence. Default handling of
`Code.check`, like `Code.compile`, preserves requested-action metadata and any
handler-produced effects.

### 6.2 Structured Diagnostics

RSI requires actionable feedback, not only a boolean or an error-message string.
The minimum public fields of `CheckReport` are:

| Field | Contract |
|---|---|
| `status` | Stable string tag: `rejected`, `accepted`, or `guarded` (accepted with residual runtime checks). |
| `diagnostics` | Array of structured diagnostic records described below, including warnings on accepted code. |
| `revision` | `RevisionInfo` identifying the checked source revision. |
| `dependencies` | `DependencyInfo` identifying the locked environment. |
| `snapshot_fingerprint` | `string` fingerprint covering the source, dependency environment, frozen contract, and compiler configuration used by this check. |
| `effects` | `EffectSummary` for this revision, with `Option` for unavailable inferred fields. |
| `determinism` | `Option<Determinism>` for this revision. |
| `trace` | `Option<TraceSummary>` for this revision. |
| `runtime_checks` | Descriptions of the residual obligations, linked to source locations and applicable spec metadata. They describe required enforcement, not executable authority or transferable proofs. |

`accepted` has no residual runtime obligations; `guarded` records the checks
that compilation must install before execution. Neither status may be used when
compatibility is unproved. Reports may contain partial analysis on rejection,
but unavailable fields remain `None` rather than fabricated successful results.

Each diagnostic record has `code: string`, `severity: string` (`error`, `warning`,
or `note`), `message: string`, and an optional `location`. A present location
records a source identifier and revision plus `byte_start: usize` and
`byte_end: usize`, a zero-based, end-exclusive byte range in the identified
source. Line/column renderings are supplementary, not the sole location format.
Applicable diagnostics also carry structured expected/actual descriptors,
violated-constraint information, related locations, and analysis paths using
the existing reflection metadata. Abstract paths are not necessarily concrete
execution witnesses. Stable codes, not human-readable messages, identify cases.

For example, if a revised generator calls an auxiliary flow that requests
`Command.run` outside its frozen contract, diagnostics should identify the call
chain and the conflicting constraint, rather than only report "effect mismatch".
If an early error prevents analysis, missing summaries must be marked unavailable,
not reported as empty effects or deterministic behavior.

The existing support types have distinct responsibilities:

| Result Or Error | Required Public Contract |
|---|---|
| `ParseError` | Has `diagnostics`, using the shared diagnostic records; returned by `parse_body` in `Err`. |
| `CheckReport` | Has the fields above. Invalid source or unproved compatibility produces `status = "rejected"`; it is not a service failure. |
| `CompileError` | Has `kind: string` (`rejected` or `service`) and `diagnostics`. `compile` uses `rejected` for candidate rejection; both `check` and `compile` use `service` for compiler-service failure. |

Diagnostic details that cannot be computed must be explicitly absent, not empty
effects or fabricated source locations. Expected/actual descriptors are supplied
where relevant, without requiring agents to parse human-readable messages.
Checking-service failure does not produce an accepted or rejected code report;
it raises `Error<CompileError>` with `kind = "service"`. Thus `check(edited)?`
can produce `Result<CheckReport, CompileError>`, while `compile(edited)?` produces
`Result<A, CompileError>` or `Result<F, CompileError>`, according to the input
descriptor, under the standard compiler-service error boundary.
The distinction uses the existing support types, not additional lifecycle types.

Runtime authority denial, cancellation, and limit exhaustion must retain their
own enforcement semantics. They cannot silently become a successful report or
be disguised as a syntax/type error that encourages automatic repair and retry.
The `Error<CompileError>` boundary and its `?` conversion do not claim to handle
every error of an enclosing limited computation.

Acceptance with runtime checks follows the existing
[static/dynamic safety boundary](14-safety-analysis-static-dynamic.md): supported
residual checks must actually be installed and enforced. An unknown result is
not blanket permission to execute. Unsupported or unguardable obligations remain
rejections, even if a model claims the candidate is safe.

### 6.3 Snapshot Consistency

Checking and compilation must refer to the same immutable draft, frozen
contract, locked dependencies, and compiler configuration. Cache keys and reports
must include their fingerprints. Changing any relevant input invalidates the
result unless the checker establishes that reuse is sound.

Candidate execution uses the implementation that was validated, not fresh source
read from a mutable path with the same name. Restoring saved source requires
compilation against the expected contract and current execution authority.
Source loading and dependency acquisition must not silently access disk or the
network during an otherwise pure metadata operation.

### 6.4 Compiler Resource Accounting

Checking and compilation execute within the applicable caller limits and
cancellation scope. Repeated requests, cache misses, retries, or a new candidate
revision do not create a fresh enclosing budget. Wall-clock time spent in the
compiler, including service waiting time, counts toward an active `WallTime`
limit. Pure parsing/formatting is not exempt from execution resource limits
merely because it performs no external action.

Ordinary compilation consumes CPU and memory, not model tokens. `Tokens` and
`ContextTokens` apply to actual model use; `Cost` accounts for applicable actual
paid work. This design does not add a compiler-specific limit keyword or pretend
that model-token limits bound compiler memory consumption.

Exhaustion follows the existing [limit semantics](04-flows-human-gates-and-protocols.md),
including `Error<BudgetExceeded>` where applicable; cancellation must propagate
rather than restart the work with a new allowance. Additional error obligations
at a limited expression remain visible. If a `?` operand has multiple residual
Error instances, the existing single-Error rule still applies; explicit handling
or normalization is needed rather than silently catching them all.

Compiler services must enforce CPU and memory ceilings as service/runtime
configuration. Wall-clock and CPU usage are measured as time durations with
explicit units; memory ceilings and usage are measured in bytes. The wall-clock
timer measures elapsed time, not just active CPU work. Numeric defaults belong
to runtime configuration rather than new source-language keywords.

Operational compiler failure raises `Error<CompileError>` with service
diagnostics. Exhausting an applicable limit reports through that limit's
`Error<BudgetExceeded>` boundary; cancellation and authority denial retain their
existing runtime semantics. None may reset the parent budget or be converted
into a misleading source-repair diagnostic.

### 6.5 Source Persistence

The supported restoration path saves source and its locked environment, then
parses and compiles again. Applications use ordinary authorized storage APIs to
save the body source, resolved target agent or flow identity,
source/contract/dependency fingerprints, and the required locked package/compiler configuration. The target
declaration and its unchanged surrounding definitions must remain available;
a body string alone cannot reconstruct the target's lexical environment.

Restoration resolves the expected target, validates the saved fingerprints and
locked environment, parses the saved body, applies `with_body`, and calls
`compile` again. A contract or dependency mismatch must not silently select a
different target or widen its interface. Restoration starts from the statically
known target's typed reflection, not from casting a persisted `CallableDef`
view to a type guessed from its signature data. Stored summaries are not proof
that can replace checking. The restored agent or flow runs under current
authority and limits.

Saving source does not serialize an executable object, secrets, live connections,
permissions, or session state. Binary artifact export/loading can be designed
separately when format compatibility and validation are specified together;
there is no binary `load` API in the current compiler surface.

## 7. Completing The RSI Loop

The compiler decides whether code is valid under a contract and can become a
callable candidate. It does not decide which candidate is better, own the
evaluation suite, or automatically deploy a successful compilation.

```text
reflect current -> propose revision -> check / compile
                       ^                    |
                       |              isolated evaluation
                       |                    |
                       +--- feedback --- accept / reject
                                            |
                                   select next implementation
```

Check failures feed the repair loop without executing the draft. Successful
compilation permits evaluation, not automatic promotion. The remaining
responsibilities are:

| Capability | Owner | Required Boundary |
|---|---|---|
| Isolated execution | Runtime / execution library | Independent test state, timeout, cancellation, resource limits, and normal action authorization. A broad execution wrapper must not hide candidate effects. |
| Runtime observation | Runtime / observability library | Outputs, failures, latency, resource/token use, and actual action events. Coverage requires suitable compiler/interpreter or target instrumentation. |
| Evaluation and comparison | EDK / application | Independent scoring, fixed tests, comparable budgets, and an explicit acceptance rule. Non-deterministic candidates need repeated evaluation rather than trusting one favorable sample. |
| Source and version storage | Storage libraries | Preserve restorable source bundles, locked dependencies, configuration, diagnostics, and evaluation evidence, not just a version label. |
| Selection, rollback, and publication | Application / EDK | Local selection can rebind an ordinary variable. Shared publication requires atomic updates, stale-base conflict handling, and explicit activation semantics. |

This division follows the [EDK evaluation-library role](17-edk-official-packages.md).
It does not invent a compiler-owned scoring function or a privileged RSI runtime.
Persistence uses ordinary authorized storage facilities; switching code does
not implicitly migrate session or memory state. Rolling back a code selection
does not undo external actions that an earlier run already performed.

For the compiler-testing case, compilation validates the revised generator's
types and safety contract. Evaluation measures generated-test validity, target
compiler coverage, independently confirmed and deduplicated defects, regressions,
and cost. The candidate cannot rewrite the evaluator, hide failed runs, or alter
the protected acceptance criteria. Persisting those results supplies feedback
for later revisions and evidence for accepting or rejecting a candidate.

## 8. Example: Revising A Test Generator

This design example assumes retained editable definitions, the proposed compiler
service, and a compatible frozen candidate contract. It is not an example that
the current implementation is claimed to accept. Runtime model configuration is
supplied through the normal agent configuration mechanism. Retain the target in
the package's `etas.toml`:

```toml
[reflection]
retain_bodies = ["app.Generator"]
```

The source below belongs to module `app`. It shows body revision and checked
invocation, not the isolated evaluation and promotion loop required by section 7.

```etas
module app;

agent Generator(task: string) -> string {
    return perform infer(
        Prompt.new()
            .system(Trusted("Generate a valid Etas compiler test program."))
            .data(task)
    );
}

agent Rewriter(source: string) -> string {
    return perform infer(
        Prompt.new()
            .system(Trusted(
                "Revise this generator body to exercise recursive ADTs. "
                + "Keep the string parameter task and string output. "
                + "Use only its existing dependencies. "
                + "Return the replacement body without enclosing braces."
            ))
            .data(source)
    );
}

flow RunAfterRevision(task: string) -> string {
    var current = Generator;
    let definition = ^current;

    match definition.body {
        Some(body) => {
            let replacement: string = body.text ~> Rewriter;

            match std.compiler.parse_body(replacement) {
                Ok(syntax) => {
                    let edited = definition.with_body(syntax);

                    match std.compiler.compile(edited)? {
                        Ok(candidate) => { current = candidate; },
                        Err(_) => {
                            // Compilation failed; retain the current agent.
                        },
                    };
                },
                Err(_) => {
                    // Parsing failed; retain the current agent.
                },
            };
        },
        None => {
            // Source unavailable; retain the current agent.
        },
    };

    return task ~> current;
}
```

Here `replacement` is `string`, `syntax` is `SyntaxBlock`, `edited` is
`AgentDef<Generator>`, and `candidate` and `current` have type `Generator`.
Parsing errors are matched directly; `?` captures the compilation boundary's
`Error<CompileError>` as `Result<Generator, CompileError>`. Only the successful
branch adopts the new agent. The final invocation uses the selected version.

The prompt is an editing request, not a safety guarantee. The compiler checks
the candidate independently. For repeated revisions within one run, keep
`current` outside the iteration loop and reflect `^current` each time. Separate
calls to this example start again from `Generator`; persistence is not implicit.
Evaluation against independent tests still belongs before adoption in a real
RSI loop; compiling successfully alone does not establish an improvement.

Compilation does not publish a version, change global bindings, migrate session
state, or alter active invocations. Publication/hot replacement, if introduced,
needs a separate effectful operation and explicit concurrency semantics.

## 9. Example: Revising An Ordinary Flow

The same compiler service can revise an ordinary flow without wrapping it in an
agent. This standalone design example retains `app.normalize`:

```toml
[reflection]
retain_bodies = ["app.normalize"]
```

```etas
module app;

flow normalize(text: string) -> string ![] {
    return std.text.trim(text);
}

flow RunAfterFlowRevision(text: string) -> string {
    var current = normalize;
    let definition = ^current;
    let replacement: string =
        "let cleaned = std.text.trim(text); return cleaned;";

    match std.compiler.parse_body(replacement) {
        Ok(syntax) => {
            let edited = definition.with_body(syntax);

            match std.compiler.compile(edited)? {
                Ok(candidate) => { current = candidate; },
                Err(_) => {
                    // Compilation failed; retain the current flow.
                },
            };
        },
        Err(_) => {
            // Parsing failed; retain the current flow.
        },
    };

    return current(text);
}
```

`definition` and `edited` have type `CallableDef<(string) -> string ![]>`;
`candidate` and `current` have type `(string) -> string ![]`. The compilation
expression with `?` returns `Result<(string) -> string ![], CompileError>`.
`RunAfterFlowRevision` still requests compilation; a pure resulting callable
does not make its construction pure. Calling `std.compiler.check(edited)` first
is optional and uses the same structured reports as agent compilation.

The replacement is a string here to isolate the reflection and compiler APIs;
an agent can generate that string instead. Parsing is required in either case.
A replacement returning `i32`, introducing `std.io.println`, or violating a
frozen trace contract is rejected. Handling a new action's error does not erase
its requested-action obligation. On rejection the example retains `normalize`;
successful adoption changes only `current`, not other callers of `normalize`.
Compilation proves contract compatibility, not behavioral equivalence or quality.

## 10. Precedents

| Precedent | Lesson for Etas |
|---|---|
| [Pharo reflection](https://books.pharo.org/pharo-by-example9/pdf/2022-03-26-index.pdf), chapter 18 | Existing methods can be inspected as objects and new methods installed dynamically. Etas adopts inspectable definitions, not implicit mutation of a global method table. |
| [Scala 3 staging](https://docs.scala-lang.org/scala3/reference/metaprogramming/staging.html) and [reflection](https://docs.scala-lang.org/scala3/reference/metaprogramming/reflection.html) | Distinguish typed code construction, low-level AST transformation, and runtime compilation. Arbitrary AST edits need checking; quotation alone is not extraction of an existing body. |
| [MetaOCaml](https://www.okmij.org/ftp/ML/MetaOCaml.html) | Typed quotation and splicing support runtime code generation, but purely generative code values do not provide arbitrary body inspection. |
| [Python source inspection](https://docs.python.org/3/library/inspect.html#inspect.getsource) | Source availability is a real condition, not something reflection can universally assume. |
