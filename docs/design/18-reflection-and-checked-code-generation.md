# Reflection and Checked Code Generation

## 1. Scope And Status

This proposed extension supports inspecting, revising and checking retained code;
it is not a claim of implementation. Compilation checks compatibility, not whether
a revision improves an agent's behavior.

| Layer | Contract |
|---|---|
| Language | Prefix `^` reflects code without executing or constructing it. |
| Compiler-recognized std | Typed descriptors preserve the original static contract; ordinary field access cannot forge it. |
| Standard API | `std.compiler` exposes parsing, checking and compilation, including the `Code.compile` action. |

The checked paths are:

```text
agent A -> reflect -> AgentDef<A>
ordinary flow f: F -> reflect -> CallableDef<F>
generated string -> parse_body -> SyntaxBlock
AgentDef<A> + SyntaxBlock -> with_body -> AgentDef<A> -> compile -> agent A
CallableDef<F> + SyntaxBlock -> with_body -> CallableDef<F> -> compile -> flow F
```

Agent compilation produces a fresh scoped, empty-state, non-Autonomous instance;
flow compilation returns `F`, not a compilation-stage wrapper. Revision metadata
identifies the implementation. Existing annotation reflection remains valid.
Private state, secrets, live frames and runtime-policy internals are not exposed.

## 2. Reflection Syntax

```etas
let definition: AgentDef<Generator> = ^Generator;
let body = definition.body;
let flow_definition = ^normalize;
```

For `normalize: (string) -> string ![]`, reflection infers
`CallableDef<(string) -> string ![]>` from static typing, not from a runtime
`signature` field.

| Form | Meaning |
|---|---|
| `^A` | Agent declaration; no instance construction. |
| `^current` | Agent/flow revision referenced by the binding, not necessarily the original source. |
| `^A.run` | Selected method; member selection binds before reflection. |
| `(^A).signature`, `(^A).body` | Explicit `run` signature/body, absent when no such method exists; never the state-field block. |
| `^f` | Ordinary flow's typed definition. |
| `^A.run(input)` | Rejected: neither execution nor quotation of an arbitrary call. |

Accessible methods and values referring to definitions or compiled revisions
are reflectable. Snapshots are immutable; inspecting or editing one neither
generates methods nor replaces active invocations or existing references.
An instance descriptor exposes its code revision, not state, access rights,
call contexts or child references. No new reflection keyword is required.

## 3. Definition Types

### 3.1 Agent Definition

An immutable `AgentDef<A>` is a revision snapshot or edited draft for nominal
agent `A`, not an executable value or proof of validity. `A` identifies the type;
revision metadata distinguishes implementations preserving its public contract.
Callable fields describe the explicit `run` and its transitive calls; auxiliary
methods retain separate summaries. Editing invalidates old inferred analysis
(§5).

| Field | Type | Meaning |
|---|---|---|
| `signature` | `Option<SignatureInfo>` | The default entry's generic binders, named inputs, and output type; `None` when there is no default entry. |
| `body` | `Option<SyntaxBlock>` | Explicit `run` method body, when retained and accessible; never the state-field block. |
| `effects` | `EffectSummary` | Preserved declared bound and optional current inferred behavior of the default invocation. |
| `determinism` | `Option<Determinism>` | Current inferred determinism of that invocation, or `None` when unavailable. |
| `trace` | `Option<TraceSummary>` | Current static behavior summary of that invocation, or `None` when unavailable. |
| `identity` | `AgentId` | Stable identity of the nominal agent. |
| `revision` | `RevisionInfo` | Content fingerprint and available parent-revision metadata. |
| `annotations` | `Array<AnnotationInfo>` | Static declaration metadata, such as model and default limits. |
| `tools` | `Array<ToolRef>` | Declared model-callable tool surface, not granted execution authority. |
| `specs` | `Array<SpecInfo>` | Descriptions of attached constraints, not runtime spec evidence. |
| `dependencies` | `DependencyInfo` | Stable symbol bindings, lexical-scope information, and locked package dependencies. |
| `methods` | `Array<CallableDef>` | Read-only, type-erased descriptors of accessible additional methods introduced through `impl A`, not a second editable copy of the default invocation. |

`AgentId`, `CallableId`, and `ToolRef` identify resolved declarations, rather than
unqualified display names. Callable identities use the existing resolved-item
identity model and distinguish methods selected through different specs.
Dependency information does not embed secret values, live connections, or
arbitrary captured execution state.

When `run` exists, `(^A).signature = Some((^A.run).signature)`; the other
callable fields also refer to the same implementation, not independent copies.

| Agent Definition | `signature` | `body` |
|---|---|---|
| Default entry with accessible retained source | `Some(...)` | `Some(...)` |
| Default entry without accessible retained source | `Some(...)` | `None` |
| No default entry | `None` | `None` |

An agent without `run` still exposes identity and accessible methods.
`with_body` cannot invent its missing signature: checking/compiling such a draft
reports a missing entry contract. Analysis availability is independent of source
availability.

### 3.2 Shared Callable Fields And Additional Methods

Shared callable-field conventions do not imply inheritance or value subtyping.
Agents additionally expose their identity, configuration and inference context.

`CallableDef<F>` preserves an ordinary flow's static callable type, including
escaping effects; editing preserves `F`, and successful compilation returns
`F`. The descriptor is neither callable nor proof that its draft is valid.
`CallableId` identifies the definition, not a nominal agent type.

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

Typed descriptors can be projected to the erased read-only view, but not back.
`compile(info)` is rejected for `info: CallableDef`; reading its signature
cannot recover a static type or authorize a cast. Descriptor construction cannot
forge a typed contract. Generic reflection follows ordinary flow typing and
does not introduce first-class polymorphic schemes.

`SignatureInfo` contains type and effect-parameter binder descriptions,
`Array<ParameterInfo>`, and a return-type descriptor. `ParameterInfo` records the
parameter name, its `TypeInfo`, and any method receiver mode (`self` or
`var self`), plus whether the declaration supplies a default. Default
descriptions preserve declaration/revision identity and available descriptive
content; they are not caller-executable expressions or permission to alter a
frozen default contract. A descriptive mode does not grant access to a caller's binding or
make a bound updating method escapable. Resolved types must remain
distinguishable from any original annotation, including when the return type
was inferred. Checked method/package contracts retain their suspension facts;
reading reflection data cannot manufacture a non-suspension proof.

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

An inference request remains in `requested` and, when applicable,
`default_handled`, even when its escaping row is empty. Agent declaration
alone records no inference request. `TraceSummary` describes ordering and
branching separately from may-action sets; copying it is not proof for edited code.

## 5. Editing And Compilation Results

`AgentDef<A>` and `CallableDef<F>` describe code; neither is executable. Editing
does not execute the code or establish that it is well typed. Successful
compilation returns an agent of type `A` or an ordinary callable of type `F`,
invoked in the same way as the original. Agent results are fresh scoped
instances, not replacements of an existing instance's code or state.
Revision metadata distinguishes the implementation. No extra public type is needed solely to say that compilation
has succeeded.

The API signatures are in section 6.1.

| Operation | Contract |
|---|---|
| `parse_body(source)` | Pure; accepts body contents without enclosing braces and returns `Result<SyntaxBlock, ParseError>`. Parsing does not establish names, types or spec conformance. |
| `definition.with_body(syntax)` | Pure; returns a new descriptor of the same family and target type, with `body = Some(syntax)`. Retains identity, signature, frozen contract and dependency environment. Accepts `SyntaxBlock`, not `string`. |
| `compile(AgentDef<A>)` | Returns a fresh scoped, empty-state, non-Autonomous `A`; records `Agentic.create<A>`. No method execution or live replacement. Fielded agents, including injected fields, are rejected by this overload. |
| `compile(CallableDef<F>)` | Returns the original ordinary callable type `F`, inferred from the input descriptor. Missing required source is a rejection, not an executable placeholder. |

Fielded and Autonomous agents remain reflectable/checkable. This API supplies
no state initialization, activation or migration contract for compiling them.

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
| Callable interface | Preserve the input/output interface, generic constraints, receiver/dispatch mode, state-field schema, method visibility and promised role/spec conformance. |
| Escaping effects | Newly inferred effects must fit the frozen interface's escaping-effect upper bound. |
| Requested actions | New requests must fit its caller-visible action upper bound, including handled requests and relevant parameter scopes. An escaping row is not that bound. |
| Temporal behavior | Preserve the trace-state assumptions and guarantees used in caller-side composition, including behavior before and after the call. Merely checking the candidate's own attached specs is insufficient. |
| Determinism | Preserve a promised `Deterministic` classification; such a computation cannot be replaced by a `NonDeterministic` one. |

Compatibility freezes the allowed behavior, not the exact old algorithm.
Inferred caller-visible summaries remain the reference; candidate inference
cannot widen them. For every permitted entry state, candidate behavior must
preserve outcomes and intermediate obligations: `send; approval` cannot
replace `approval; send` merely because their action sets or final states match.

Unproved compatibility is rejection. Supported residual checks may discharge
dynamic obligations only under the [safety rules](14-safety-analysis-static-dynamic.md).
A changed public contract requires explicit caller revalidation outside this API.
Revision-specific assumptions must be invalidated or revalidated before reuse.

### 5.3 Adopting A Revision

Adopting a compiled implementation is separate from editing and compilation:
`current = candidate` replaces a local `var` binding of type `A` or `F`; subsequent
calls through `current` execute the new implementation. Other bindings, the
original declaration, and in-progress calls do not change. Old instances keep
their owners and are not destroyed by rebinding one reference. Local adoption
does not transfer state, children, receiver access or running continuations.

## 6. Effectful Compilation

`Code.check` and `Code.compile` accept the typed descriptor families below,
never erased `CallableDef`. Their default handlers consume the request and
retain requested/default-handled metadata. Operational failure raises
`Error<CompileError>`; invalid code is a rejected report from `check` and a
rejection error from `compile`.

| Boundary | Rule |
|---|---|
| Inspection and editing | Retained metadata, parsing and draft construction may be pure; source I/O uses ordinary authorized actions. |
| Candidate execution | Candidate effects remain latent until invocation; checking/compilation never executes its model/tool actions. |
| Custom handler | May reject, cache or supply a compatible validated revision; cannot forge its contract or grant authority. |
| `?` | Captures the one residual `Error<CompileError>` as `Result`; unrelated errors, cancellation and denial retain their own semantics. |
| Checking first | Optional; `compile` independently validates the draft. A user-constructed report is not evidence. |
| Persistence | Section 6.5; there is no binary load/export API. |

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
| `std.compiler.compile<A>(definition: AgentDef<A>)` | `A` | Check and compile a fresh empty-state, non-Autonomous instance of `A` in the managed caller scope. Preserve frozen contract; wrap `Code.compile` and record creation. Failures use `Error<CompileError>`; infer `A`. |
| `std.compiler.check<F>(definition: CallableDef<F>)` | `CheckReport` | Check an edited ordinary flow against its frozen contract without executing it. Wraps `Code.check`; rejection/report and service-error rules are the same as for agents. |
| `std.compiler.compile<F>(definition: CallableDef<F>)` | `F` | Check and compile an ordinary flow under its frozen contract. Wraps `Code.compile`; checking/compilation failures use `Error<CompileError>`, and `F` is inferred. |
| `std.compiler.diff(old, new)` | `ChangeReport` | Compare available source and analysis metadata of immutable snapshots. Pure; it is not a proof of behavioral equivalence or improvement. |



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



The existing support types have distinct responsibilities:

| Result Or Error | Required Public Contract |
|---|---|
| `ParseError` | Has `diagnostics`, using the shared diagnostic records; returned by `parse_body` in `Err`. |
| `CheckReport` | Has the fields above. Invalid source or unproved compatibility produces `status = "rejected"`; it is not a service failure. |
| `CompileError` | Has `kind: string` (`rejected` or `service`) and `diagnostics`. `compile` uses `rejected` for candidate rejection; both `check` and `compile` use `service` for compiler-service failure. |

Unavailable diagnostic data must be absent, not fabricated. Runtime denial,
cancellation and budget exhaustion retain their own semantics, not source-repair
errors. Unsupported or unguardable obligations remain rejections.

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

| Resource | Contract |
|---|---|
| Parent scope | Checking, compilation, retries and revisions share its limits and cancellation; no budget reset. |
| Time and memory | Elapsed `WallTime` includes waiting; CPU and memory ceilings use explicit time/byte units. Pure parsing/formatting is not exempt. |
| Model work | Ordinary compilation does not consume model tokens. `Tokens`/`ContextTokens` count actual model use; `Cost` counts applicable paid work. |
| Failure | Applicable limit exhaustion uses `Error<BudgetExceeded>`; service failure uses `Error<CompileError>`; cancellation and authority denial propagate normally. |

Multiple residual error families still require explicit handling before `?`.

### 6.5 Source Persistence

Save source with its resolved target, frozen-contract/dependency fingerprints
and locked package/compiler configuration through authorized storage APIs.
The original target and surrounding definitions must remain available.

Restore through the statically known target's typed reflection, validate the
environment, parse the body, apply `with_body`, and compile again. Mismatch
cannot silently select a new target or widen its contract. Stored summaries and
erased signature metadata cannot replace checking.

Source persistence does not serialize executable objects, permissions, secrets,
connections, session state or live continuations. Restored code runs under
current authority and limits.

## 7. Example: Revising An Ordinary Flow

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
