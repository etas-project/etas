# Type System and Errors

## 1. Type System

Types classify values; effects classify computations. Their constraints are
checked together at callable and handler boundaries. The value system includes
ordinary data, structured-output schemas, trust labels and provenance.

### 1.1 Ordinary Types

```etas
type Paper = {
    title: string,
    authors: Array<string>,
    year: i32,
    venue: string,
    url: Url
}
```

#### 1.1.1 Alias, Nominal, And Bodyless Types

Etas separates transparent aliases from nominal types:

```etas
alias A = B;    // transparent alias
type A = B;     // nominal/newtype with representation B
type A;         // bodyless nominal type
```

`alias` introduces no new type identity. It is a source-level abbreviation used
to shorten long type expressions:

```etas
alias JsonText = string;

let raw: string = "hello";
let json: JsonText = raw;    // accepted: JsonText is string
```

`type A = B` introduces a distinct nominal type whose runtime representation may
still be `B`. Assignment between the nominal type and its representation requires
an explicit constructor, accessor, or conversion flow:

```etas
type UserId = string;
type ProjectId = string;

let user = UserId("u1");
let project: ProjectId = user;    // rejected
let raw: string = user;           // rejected without explicit conversion
```

This keeps domain IDs, paths, secrets, error categories, and structured agent
messages from being accidentally mixed just because they have the same runtime
shape.

`type A;` declares a nominal type with no public representation. It has two
intended uses:

1. zero-runtime marker types used only as type indices;
2. opaque runtime handles whose values are created by trusted standard-library,
   runtime, or package APIs.

```etas
type WorkspaceRoot;
type ReportsRoot;

spec Region;
impl WorkspaceRoot ~ Region;
impl ReportsRoot ~ Region;

type WorkspacePath<R ~ Region> = string;

type TcpStream;
```

`WorkspaceRoot` and `ReportsRoot` are marker types: if they only appear in type
arguments, spec implementations, effect indices, and trace-spec predicates, the
compiler does not allocate runtime values for them. `TcpStream` is also
bodyless, but it represents an opaque runtime handle because values of that type
are returned and passed at runtime.



#### 1.1.2 Algebraic Data Types And Recursion

ETAS supports product types (records and tuples), sum types (`enum`), generic
ADTs, and direct or mutual recursion through nominal type declarations. No
additional recursion keyword is required. Type names are registered before
their fields and variant payloads are checked, so mutually recursive
declarations in the same module may refer forward to each other. References
across modules remain subject to the ordinary import and visibility rules.

```etas
enum Tree<T> {
    Leaf(T),
    Branch(Tree<T>, Tree<T>),
}

type Directory = {
    name: string,
    entries: Array<Entry>,
}

enum Entry {
    File { name: string, content: bytes },
    Subdirectory(Directory),
}
```

An enum variant may be nullary, carry positional payloads, or carry inline
named fields. Inline fields belong to that variant; they do not declare a
separate record type. Constructing a variant produces a value of its enclosing
enum type. Variants cannot mix positional and named-field payloads in a single
declaration. Variant names within an enum and field names within a variant
must be unique. Named-field construction requires every declared field exactly
once, rejects unknown fields, and supports ordinary record shorthand. Enum
payload fields do not support `@inject`; constructor injection is confined to
named record types and agents.

```etas
enum Response {
    Success(string),
    Failure { code: i32, message: string },
}

let response: Response = Response.Failure { code = 503, message = "unavailable" };

flow describe(response: Response) -> string {
    match response {
        Response.Success(text) => text,
        Response.Failure { code: _, message } => message,
    }
}
```

Recursive nominal types are checked by declaration identity and type arguments,
not by repeatedly expanding their definitions. Constructor application introduces
the enum value and matching eliminates it; users do not write fold/unfold
operations or manage recursive storage. Transparent alias expansion must
terminate at a nominal type or a nonrecursive representation. A cycle consisting
only of aliases and structural type constructors is rejected:

```etas
alias Bad = Array<Bad>; // rejected: recursive transparent alias
alias IntTree = Tree<i32>; // accepted: names an existing nominal ADT
```

Recursion describes the shape of values, not a facility for creating cyclic
object references. Ordinary first-order ADT values are finite values; the
implementation may share immutable subtrees and insert heap indirection as
needed without exposing mutable aliases. Neither `Box` nor an `indirect`
modifier is required in ETAS source. Type identity, value semantics, and pattern
matching must not depend on that storage choice. This feature does not add
GADT-style constructor-specific result types or arbitrary structural recursive
type equivalence.

`match` must be exhaustive for its scrutinee type; missing cases are a compile
error. Coverage includes payload patterns, so a case matching only one literal
payload does not cover every value of that variant. A wildcard or variable
pattern can cover the remaining values. Unreachable arms should produce a
warning. Recursive coverage checking must terminate without infinitely unfolding
the ADT; an explicit catch-all is required if coverage cannot be established.
These rules apply to both statement and expression forms of `match`.

Recursive ADTs are valid independently of model schema support. Derivation,
encoding, decoding, and printing must track recursive type references rather
than infinitely expand definitions, and enforce documented depth/size limits
when traversing values. Using such a type as model input/output still requires
the relevant encoding/schema support; an unsupported provider schema must be
reported before dispatch, rather than silently flattening or truncating the
type. Validated decoding cannot create cyclic values.

#### 1.1.3 Record Equality

For two values of the same instantiated nominal record type, `==` compares all
fields by name using each field type's equality semantics. The result is `true`
exactly when every corresponding field comparison is `true`; construction field
order does not matter. `!=` is the negation of `==`. If any field type does not
support equality, whole-record comparison is a type error. Distinct nominal
record types cannot be compared directly merely because their fields match;
transparent aliases preserve the underlying type identity.

```etas
type Point = { x: i32, y: i32 }

let a = Point { x = 1, y = 2 };
let b = Point { y = 2, x = 1 };
let equal = a == b; // true
```

When hashing is supported, equal records must have equal hashes under the same
hashing scheme, independent of construction field order. Equality support alone
does not imply hashing support; unequal values may have the same hash.

Active agent references do not derive structural Eq/Hash. Whole-record
comparison therefore requires separate valid evidence for any agent-valued
field; matching private state does not make two instances equal.

Supplied record field expressions run in source order, followed by omitted
`@inject` actions in declaration order. Equality/hash compare the completed
stored fields, never providers or annotations. Canonical layout cannot reorder
observable initialization; see [Constructor Injection](02-general-programming-constructs.md#72-constructor-injection).

For a field `f: T` annotated with action `a`, checking requires a zero-argument
signature `a() -> U` whose result is assignable to `T` by the ordinary rules (no new coercion).
Declaration-site generic/spec bounds must justify this for every admitted
instantiation. At each nominal construction, fill only omitted annotated fields
with checked `perform a()` expressions before effect/trace analysis. This
changes the construction's behavior, not the resulting nominal type or the
effect of subsequent stored-field reads. Lifetime, visibility and schema rules
are unchanged.

Standard collection abstractions use their specified logical equality, not the
fields of a particular source/runtime representation. Sequence collections
compare elements in order; maps compare key/value bindings and sets compare
membership independently of insertion order. Hash seeds, cached lengths,
backing identity and tree/heap metadata do not participate. This exception
attaches to the resolved standard contracts; an arbitrary user record cannot
opt out of field comparison by hiding fields. Equality does not permit the
compiler to reorder observable iteration.

The standard `Eq` spec witnesses reflexive equality; not every type admitting
`==` satisfies it. In particular, raw floating-point NaNs prevent automatic
`Eq`/`Hash` evidence. `Hash` includes `Eq`, and equal values must hash equally
under the same initial hasher. The exact standard evidence, ordered-container
comparison semantics and API bounds are specified in
[Collections And Key Contracts](19-standard-library.md#92-equality-hashing-and-order).

### 1.2 Structured Output Types

Agent output should be schema-checked.

```etas
type Review = {
    summary: string,
    strengths: Array<string>,
    weaknesses: Array<string>,
    score: i32
}
```

### 1.3 Validators And Schemas

Etas does not include refinement types in the language. Value constraints
such as ranges, non-empty collections, valid paths, valid email addresses, or
domain-specific invariants should be expressed with ordinary validation flows,
schema metadata, and runtime errors:

```etas
type Review = {
    summary: string,
    score: i32,
    confidence: f64,
    major_issues: Array<string>
}

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

Validation does not introduce predicate-refined types.

### 1.4 Trust Types

```etas
Trusted<string>
Untrusted<string>
Secret<string>
Public<string>
Sanitized<string>
```

Example:

```etas
flow build_system_prompt(x: Trusted<string>) -> Prompt {
...
}

let page: Untrusted<string> = web.read(url);
build_system_prompt(page); // compile-time error
```

### 1.5 Spec Polymorphism

Etas supports specs as static constraints. A spec is not a concrete value type
and does not create a subtype relation. Spec satisfaction is checked as a
separate judgment from ordinary type unification:

```text
Γ ⊢ Subject ~ SpecRef
```

The source language keeps `:` for type annotation and optional spec-kind
annotations, and uses `~` for spec satisfaction and spec entailment:

```etas
let name: string = "Ada";
impl TcpStream ~ ByteStream;
flow read_all<S ~ ByteStream>(stream: S) -> bytes;
```

Specs have three kinds:

| Kind | Constrains | Typical subjects |
|---|---|---|
| `type` | Types, resource markers, static relations, and effect-row/static entities | `TlsStream ~ ByteStream`, `R ~ Within<ReportsRoot>`, `effect E ~ RetrySafeEffects` |
| `callable` | Flow/tool/agent-method call shape | `Writer.run ~ Stage<Brief, Draft, E>` |
| `trace` | Requested-action traces, authority, and temporal constraints | `+Email.send`, `-Secret.read`, `Approval.request >> Email.send` |

`EffectSpec` is not a separate kind. Effect rows are static entities,
so constraints over them are `type` specs. `ProtocolSpec` is also not part of the
spec system; protocol declarations remain a separate advanced construct.

#### 1.5.1 Type Specs

Type specs target nominal types and generic applied types. They can be marker
specs, relation specs, or behavioral specs:

```etas
public spec ByteStream;
public spec Region;
public spec Within<Parent>;

public spec PromptEncode {
    flow encode(self) -> PromptPart;
}
```

Explicit implementations provide evidence:

```etas
impl TcpStream ~ ByteStream;
impl TlsStream ~ ByteStream;

impl Report ~ PromptEncode {
    flow encode(self) -> PromptPart {
        return PromptPart.data(self);
    }
}
```

Generic implementations declare bounds in the owner pattern:
`impl List<T ~ PromptEncode> ~ PromptEncode {... }`. Here `T ~ PromptEncode`
binds an implementation parameter (or constrains an explicitly declared one);
the owner type is `List<T>`. Unbounded parameters are still inferred from the
owner pattern. Parameters and bounds scope over the entire impl; multiple
bounds use `+`. Ordinary type arguments do not introduce parameters.

Behavioral type specs may require the restricted updating receiver:

```etas
public spec Resettable {
    flow reset(var self) -> unit;
}
```

Receiver mode and dispatch kind are part of the required contract. Implementations
match `self`/`var self`, arguments, results and effects exactly.
Passive updating methods require non-suspension and direct-local-var writeback;
they cannot be extracted as ordinary bound flow values. Active methods instead
retain instance identity, scope and request-dispatch evidence in bound callables.
Their exclusive update may suspend without exposing pending state.

`std.agent.Active` is compiler-provided marker evidence for active types.
An updating role intended for mailbox dispatch entails it:

```etas
spec Reviewer ~ Active {
    flow review(var self, patch: string) -> string ![];
}
```

Ordinary user impls cannot grant `Active` to passive types. An active updating
implementation cannot masquerade as passive updating evidence; a role requiring
active updates must entail `Active`. Read-only role methods may use either
dispatch kind, preserved by the evidence. See
[Agent Semantics](03-agents-tools-prompts-memory.md#1-agent) and
[Passive Receivers](02-general-programming-constructs.md#32-restricted-updating-receivers).

Defaults on behavioral method signatures belong to the spec's call interface.
Implementations inherit them rather than providing competing defaults. An
optional argument still binds the declared parameter type; it does not add an
implicit `Option` or alter the method's arrow-type arity. See
[Default Parameters](02-general-programming-constructs.md#41-default-parameters).

Generic APIs use `~` bounds:

```etas
flow read_until_limit<S ~ ByteStream>(
    stream: S,
    limit: ByteLimit,
    timeout: Option<Timeout>
) -> bytes;
```

Parameterized marker specs can express relationships between types. This is
the preferred way to model resource inclusion, stream evidence, prompt encoding
support, path regions, and other compile-time facts without adding general
subtyping:

```etas
public type WorkspaceRoot;
public type ReportsRoot;
public type DraftsRoot;

impl WorkspaceRoot ~ Region;
impl ReportsRoot ~ Region;
impl DraftsRoot ~ Region;

impl ReportsRoot ~ Within<WorkspaceRoot>;
impl DraftsRoot ~ Within<ReportsRoot>;

public type WorkspacePath<R ~ Region> = string;

flow read_report<R ~ Region + Within<ReportsRoot>>(
    path: WorkspacePath<R>
) -> bytes;
```

The judgment for `R ~ Within<ReportsRoot>` is based on explicit imported spec
implementations. It does not make `WorkspacePath<R>` a subtype of
`WorkspacePath<ReportsRoot>`, and it does not derive facts from runtime path
strings. Changing a value's region index requires an explicitly specified
checked conversion; spec evidence alone does not coerce values.

#### 1.5.2 Existential Spec Objects

The existential form is:

```text
? ~ SpecA
? ~ SpecA + SpecB
```

It is a type, not a spec declaration and not a subtype cast. `?` cannot appear
alone; it must be followed by `~` and one or more type specs. The form is allowed
in normal type positions:

```etas
let s: ? ~ ByteStream = tcp;
let xs: Array<? ~ ByteStream> = [tcp, tls, file];
flow make_stream(kind: StreamKind) -> ? ~ ByteStream;
flow copy_any(from: ? ~ ByteStream, to: ? ~ ByteStream) -> unit;

flow read_one(stream: ? ~ ByteStream, limit: ByteLimit) -> bytes {
    return std.stream.read_until_limit(stream, limit, None);
}
```

`? ~ Spec` is different from a generic parameter:

```etas
flow copy_same<S ~ ByteStream>(from: S, to: S) -> unit;
flow copy_any(from: ? ~ ByteStream, to: ? ~ ByteStream) -> unit;
```

`copy_same` requires both arguments to have the same concrete type `S`.
`copy_any` accepts two independently hidden stream types; one argument may be a
`TcpStream` and the other may be a `TlsStream`.

Packing into an existential is implicit only when the expected type is known:

```etas
let s: ? ~ ByteStream = tls;                 // packs `tls`
let xs: Array<? ~ ByteStream> = [tcp, tls];  // packs each element
let bad = [tcp, tls];                        // rejected without context
```

Existential unpacking is lexical and type-safe. When a value `x: ? ~ ByteStream`
is used at a call site requiring `S ~ ByteStream`, the compiler introduces a
fresh hidden type and evidence for that expression. User code cannot name the
hidden type, compare two hidden types for equality, access concrete fields, or
downcast to the implementation type.

Existential bounds are limited to object-safe type specs:

- marker specs are object-safe;
- behavioral specs are object-safe only if their required flows have no generic
  type parameters and their public signatures do not expose the hidden concrete
  type except as the receiver;
- callable specs are not existential object bounds. Flow values already
  have arrow types with effect rows, and heterogeneous stage registries should
  use explicit wrapper records or enums until callable-spec existentials are
  designed.

Existential use may require dynamic evidence dispatch or an evidence package at
runtime. The compiler still knows the allowed spec surface and effect row of each
operation; it simply does not know the concrete implementation type after the
value has been packed.

Existential evidence preserves receiver mode, dispatch kind, scopes and method
summaries. Passive updating objects still require direct local `var` writeback
and non-suspension, as for `? ~ Scheduler`. Active roles instead dispatch
requests through scoped references:

```etas
let reviewers: Array<? ~ Reviewer> = [security, performance, mock];
let reviews = std.task.map_concurrent(reviewers, r => r.review(patch));
```

Each element satisfies the same active role; a deterministic mock is also an
agent. Packing does not grant ownership, erase tool/trace bounds, or permit a
reference to outlive its instance.

#### 1.5.3 Callable Specs

Callable specs target computations: flows, agent entrypoints/bound methods,
tools, and composed flows. A nominal agent type itself is not a callable spec implementation.
They are bodiless and declare a callable shape:

```etas
public spec Stage<I, O, effect E>: callable I => O ![E];
public spec Pure<I, O>: callable I => O ![];
public spec ReportWriter: callable Brief => Draft ![WriterEffects];
```

Names used by a callable spec body are not implicitly generic. Generic variables
must be declared in `<...>`; otherwise names such as `Brief`, `Draft`, and
`WriterEffects` must resolve to existing types or effect rows.

If a callable spec omits `![...]`, it imposes no effect constraint. The
implementation still has its own inferred effect row, but the spec does not
restrict it. To require no escaping effects, write `![]` explicitly, as in
`Pure<I, O>` above. Despite its illustrative name, this spec only constrains
the escaping row: handled actions, mutable captures and suspension require
separate behavior checks.

When a flow, agent, or tool writes `~ SpecName`, the compiler may infer the
spec's type and effect arguments from the declaration signature:

```etas
flow Normalize(text: string) -> string ~ Pure {
    return std.text.trim(text);
}
```

This is checked as `Pure<string, string>`; the implementation does not need to
write `~ Pure<string, string>`. If inference is ambiguous, the compiler
requires an explicit instantiation.

#### 1.5.4 Trace Specs

Trace specs constrain requested-action traces. They are compile-time spec
expressions, not runtime values and not ordinary types:

```etas
public spec SafeEmail: trace =
    (Approval.request >> CompanyEmail.send<WorkAccount>)
    & -Secret.read<_>;
```

Trace-spec operators are kinded:

| Operator | Kind | Meaning |
|---|---|---|
| `+A` | `ActionPattern -> trace` | Allow matching action pattern `A` |
| `-A` | `ActionPattern -> trace` | Deny matching action pattern `A` |
| `A >> B` | `ActionPattern x ActionPattern -> trace` | Allow the two action patterns within this spec, and require valid matching `A` evidence before every `B` |
| `A << B` | `ActionPattern x ActionPattern -> trace` | Allow the two action patterns within this spec, and require valid matching `B` evidence before every `A`; equivalent to `B >> A` |
| `S & T` | `Spec<K> x Spec<K> -> Spec<K>` | Both specs must hold |
| `S \| T` | `TraceSpec x TraceSpec -> TraceSpec` | Either trace spec may accept; Reserved this for trace specs only |

`A >> B` includes local permissions for both `A` and `B`. It normalizes to the
following internal components, not another
source-level `>>` expression:

```text
Allow(A), Allow(B), Before(guard = A, target = B)
```

The implicit permissions use exactly the written patterns, including resource
indices and spec constraints. They do not allow sibling actions or broader
resource scopes. `Before` remains a prerequisite on every matching `B`, even
if `+B` is also written explicitly. Neither action is required to occur; an
empty trace or an `A` without a subsequent `B` can satisfy this clause.

Explicit `+A` remains useful for independent actions with no ordering
requirement. Redundant explicit permissions are accepted and deduplicated.
Within one local conjunctive clause group, collect explicit and implicit allow
patterns into the same allow set; conjoin its guards and denials. This local
collection does not union the permissions of independently composed specs or
enclosing scopes. All of those constraints still apply, and a matching denial
wins. In particular, `-B & (A >> B)` cannot execute `B`.

For `S | T`, keep each alternative's permissions and temporal checks together:
at least one complete alternative must accept the trace. Do not union allow
sets across alternatives or choose a fresh alternative for each action. No
implicit permission overrides an outer restriction, effect/action boundary,
sandbox, or runtime grant. Mentioning an action in a spec does not cause the
program to request it or add it to inferred may-effects.

Guard evidence includes its outcome and relevant resource/request identity.
For `Approval.request >> B`, requesting approval is permitted, but only a
matching accepted approval satisfies the prerequisite for `B`. A pending,
denied, failed, or unrelated approval does not. The operator does not itself
invoke either action.

Applying `+`, `-`, `>>`, or `<<` to type specs or callable specs is rejected.
Combining specs of different kinds with `&` is also rejected:

```etas
spec Bad =
    ByteStream & +CompanyEmail.send<WorkAccount>; // rejected
```

The source constraint is a trace spec consumed through `~`; runtime policy is
the corresponding deployment/enforcement context, not a separate source block.

#### 1.5.5 Spec Targets And Kind Checking

Type specs, callable specs, and trace specs are kind-checked and cannot be mixed:

| Target | May satisfy |
|---|---|
| Passive nominal type | Type spec |
| Active agent type | Type specs/roles; trace conformance constrains methods in the annotated declaration/impl |
| Generic applied type | Type spec |
| Effect row / effect parameter | Type spec |
| Flow declaration/value | Callable spec and trace spec |
| Explicit agent method / bound method value | Callable spec and trace spec; retain active dispatch and instance scope |
| Tool declaration | Callable spec and trace spec |
| Composed flow | Callable spec and trace spec, inferred |
| Alias | Expanded first, then checked |
| Effect/action/module | Not a direct spec target |

#### 1.5.6 Spec Entailment And Diamonds

Specs can entail other specs:

```etas
spec ReadableStream ~ ByteStream;
spec WritableStream ~ ByteStream;
spec ReadWriteStream ~ ReadableStream + WritableStream;
```

This is constraint entailment, not subtyping. A value of type `TlsStream` is not
assignable to a variable whose type is `ByteStream`; `ByteStream` is not a value
type. Instead, `TlsStream` can be passed to generic APIs that require `S ~
ByteStream`.

Diamond-shaped spec hierarchies are allowed. They only create duplicate
evidence paths, which the compiler deduplicates. The spec entailment graph must
be acyclic, and inherited behavioral method requirements must merge without
signature conflicts:

| Inherited requirement | Result |
|---|---|
| Same method name and same signature | Merge into one requirement |
| Same method name and different signature | Reject the spec declaration |
| Same signature but different semantic law | The language does not check laws |

Effect and trace-spec checking may use the same type-level facts. For example, a
trace spec can match a region-indexed action through a spec relation:

```etas
spec ReportsOnly: trace =
    +Fs.read<R ~ Within<ReportsRoot>>;
```

This is the preferred mechanism for `Index`, `ByteStream`, `Region`,
`Within<Parent>`, `PromptEncode`, `Schema`, `ResponseDecode`, flow-shape
specs such as `Stage`, and similar support abstractions. It replaces ad-hoc
compiler-known support constraints with a language-level evidence system while
still avoiding general subtyping.

### 1.6 Flow Effect Annotations

`(I) -> O ![E]` combines a value-level callable type with an escaping-effect
row. Omission means inference; an explicit row is an upper bound, and `![]`
bounds escaping effects to empty. Handled requests remain separately visible.
Agent declarations alone produce no inference request; executing `perform infer`
does. Determinism depends on actual behavior, not the empty escaping row.

See [Effects And Inference](06-effect-system-and-inference.md) for row
polymorphism, declaration defaults and requested-action propagation.

### 1.7 Effect Action Authority

Effects classify behavior; explicit rows constrain escaping effects but do not
grant authority. Concrete requests remain subject to active trace constraints
and execution grants even when handled. Tools expose checked model-callable
boundaries, not permission tokens. See [Safety Contracts](14-safety-analysis-static-dynamic.md).

### 1.8 Bottom Type

`never` is the bottom type. An expression of type `never` never produces a
runtime value, so the type checker accepts it wherever a value of any type is
expected:

```text
never <: T
```

This is useful for `abort`, non-returning effect actions such as
`Error<E>.raise(...) -> never`, and branches that terminate control flow:

```etas
flow require_positive(n: i32) -> i32 {
    if n > 0 {
        n
    } else {
        abort("expected positive")
    }
}
```

The `else` branch has type `never`, so it does not force the `if` expression to
return a different type. This is a type rule only; no value of type `never` is
constructed at runtime.

### 1.9 Provenance Types

For factual claims:

```etas
type Claim = {
    text: string,
    evidence: Array<Citation>,
    confidence: Confidence
}
```

Evidence requirements use ordinary types, specs, and validators. For example, a
publishing flow can require an evidence-bearing value:

```etas
spec EvidenceBacked;

impl Claim ~ EvidenceBacked;

flow publish_claim<C ~ EvidenceBacked>(claim: C) -> PublishedClaim {
...
}
```

---

## 2. Error Handling

Agent systems have special error modes. Etas should make them explicit.

### 2.1 Error Types

| Error | Meaning |
|---|---|
| `SchemaError` | Model output does not match expected schema |
| `ValidationError` | Decoded value violates an application or schema validator |
| `ToolTimeout` | Tool call timed out |
| `ToolError` | Tool returned an error |
| `ToolDenied` | Runtime denied a tool call before execution |
| `PolicyViolation` | Agent attempted forbidden action |
| `PolicyDenied` | Active policy denied a requested operation before execution |
| `EffectBoundaryViolation` | Requested operation exceeded the active run's effect boundary |
| `SandboxViolation` | Tool or command attempted to escape its sandbox profile |
| `PromptInjectionRisk` | Taint analysis detected unsafe flow |
| `MissingCitation` | Claim lacks evidence |
| `BudgetExceeded` | Failure reported outside an exhausted limit scope after its work is stopped; handling it cannot revive that scope or reset budgets. See [limit outcomes](04-flows-human-gates-and-protocols.md#1-flow). |
| `ProtocolViolation` | Communication violated declared protocol |
| `HumanRejected` | Human approval was denied |

### 2.2 Error Effect And Result

```etas
flow LoadReport(path: Path) -> Report {
    let text = fs.read(path);
    return parse_report(text);
}

flow TryLoadReport(path: Path) -> Result<Report, AppError> {
    return LoadReport(path)?;
}
```

Recoverable failures are represented first as the typed error effect:

```etas
effect Error<E> {
    action raise(err: E) -> never;
}
```

For explicit failure, code may perform the action directly:

```etas
perform Error<AppError>.raise(AppError {
    code = "invalid_report",
    message = "Report is missing a title"
});
```

A flow that may raise `Error<E>` carries that effect in its inferred effect set:

```text
LoadReport: Path -> Report
inferred effects: [FileIO, Error<AppError>]
```

`Result<T, E>` is the value-level form of the same recoverable failure boundary:

```etas
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

It is useful at API boundaries, when storing results, returning errors to another
language, or making success/failure ordinary data.

Package and application error types should usually be modeled as an `enum` that
groups related cases, rather than as many repeated record types with the same
`message` field. This matches Etas's `Error<E>`, `Result<T, E>`, `match`, and
`impl` method model:

```etas
public enum HttpError {
    UrlParse { input: string, message: string },
    Header { name: string, message: string },
    Transport { host: string, message: string },
    Tls { message: string },
    Timeout { message: string },
}

impl HttpError {
    public flow message(self) -> string {
        match self {
            UrlParse { input, message } => message,
            Header { name, message } => message,
            Transport { host, message } => message,
            Tls { message } => message,
            Timeout { message } => message,
        }
    }
}
```

Avoid splitting every case into separate duplicate record types such as
`UrlParseError`, `HeaderError`, and `TransportError` unless those types are
reused independently outside the aggregate error. A single package-level error
enum keeps public signatures compact:

```etas
flow request(req: HttpRequest) -> HttpResponse ![Error<HttpError>]
```

The postfix `?` operator captures `Error<E>` actions raised by the expression on
its left and converts the result to `Result<T, E>`:

```text
e: T ![Error<E>,...]
-----------------------
e?: Result<T, E> ![...]
```

The captured `Error<E>` effect is removed from the resulting expression. Other
effects remain:

```text
std.io.read_line(): string ![Error<IOError>]
std.io.read_line()?: Result<string, IOError>
```

The underlying `Console.stdin_read_line` action remains in requested/default
action metadata for policy, trace, replay, and runtime mediation; it is not an
escaping effect of the `?` expression.

`?` is a postfix expression operator, not a statement terminator. A semicolon
belongs to the surrounding statement, not to `?` itself:

```etas
let result = std.io.read_line()?;  // semicolon ends the let statement
return LoadReport(path)?;          // semicolon ends the return statement
```

When `?` appears as the final expression of a block, no semicolon is required:

```etas
flow TryReadName() -> Result<string, IOError> {
    std.io.read_line()?
}
```

`?` is accepted only when the expression may raise exactly one
distinct `Error<E>` effect. If an expression may raise multiple error effects,
the program must use `handle` and map the error cases explicitly before applying
`?`. The language does not infer ad-hoc error unions or implicit error conversions.

For example, this is rejected if `LoadConfig(path)` may raise both
`Error<IOError>` and `Error<ParseError>`:

```etas
flow TryLoadConfig(path: Path) -> Result<Config, ConfigError> {
    LoadConfig(path)? // rejected: more than one Error<E> may be raised
}
```

The program must make the mapping explicit:

```etas
enum ConfigError {
    IO(IOError),
    Parse(ParseError),
}

flow TryLoadConfig(path: Path) -> Result<Config, ConfigError> {
    handle {
        Ok(LoadConfig(path))
    } with {
        Error<IOError>.raise(err) => {
            finish Err(ConfigError.IO(err));
        }

        Error<ParseError>.raise(err) => {
            finish Err(ConfigError.Parse(err));
        }
    }
}
```

The expression `e?` is semantically equivalent to:

```etas
handle {
    Ok(e)
} with {
    Error<E>.raise(err) => {
        finish Err(err);
    }
}
```

For a multi-step computation, apply `?` to a block expression:

```etas
flow TryLoadReport(path: Path) -> Result<Report, AppError> {
    return {
        let text = fs.read(path);
        parse_report(text)
    }?;
}
```

`?` is not a `Result` unwrapping or early-return operator. Given
`r: Result<T, E>`, `r?` is rejected unless `r` itself is produced by an
expression that may raise `Error<E>`. Propagating a value-level `Result` should
be written with `match` or a standard-library helper:

```etas
flow LoadThenValidate(path: Path) -> Result<Report, AppError> {
    match TryLoadReport(path) {
        Ok(report) => return validate_report(report),
        Err(err) => return Err(err),
    }
}
```

This keeps `?` tied to Etas's effect system instead of copying Rust's
`Result` propagation rule.

Non-returning unrecoverable failure uses the standard support flow:

```etas
flow abort(message: string) -> never
```

For example:

```etas
if unsafe {
    abort("Unsafe behavior");
}
```

`try`, `catch`, and `throw` are not part of the source syntax. Recoverable
runtime failures should use `Error<E>`, `perform`, `handle`, and `Result<T, E>`
when a value-level boundary is needed.

### 2.3 Runtime-Scoped Effect Handlers

Handlers eliminate handled escaping actions and add their own effects.
`resume value` supplies the action's result; `finish value` supplies the
whole handled expression's result. A `never` action cannot resume.
Implicit fallback expression arms and direct arm-level `return` are rejected.
Full typing, completion and dispatch rules are in
[Handler Scope](06-effect-system-and-inference.md#22-handler-scope).

#### 2.3.1 Scoped Continuation Handles

`Action(args) as k => {... }` binds a single-shot continuation for a resumable
action. Its type is inferred from the action result type, the handled answer
type, residual computation effects, and its owning task scope. These facts are
compiler-known; runtime signature data cannot forge a continuation. No public
continuation type annotation or new ownership keyword is required.

Unlike ordinary value-semantic data, `k` is affine: it cannot be copied, and a
transfer consumes the local binding. Aggregate or closure capture must preserve
this non-copyability and scope bound. It may enter only scope-preserving
scheduler storage, not an ordinary persistent/copying collection, global value,
model payload, or return value that outlives its owner. The original task and
handler frame must remain valid until resumption or cancellation. Merely
abandoning the handle is not successful handler completion.

Immediate `resume` uses the same handle even when `as k` was written. A path
cannot transfer and then resume it, nor transfer it twice. `finish` and
non-returning unwinding dispose of the affected suspended computation and run
the required scope cleanup. Actions returning `never` cannot bind a resumable
handle. The runtime also enforces one-use ownership and scope validity before
executing a deferred continuation.

`Task<T, E>` is a standard task-result handle; `E` is an effect-row parameter.
Its owning scope and requested-action/trace contract remain checked metadata,
not writable fields. Passing a task to another helper must preserve those
facts. Task handles can be shared within their lifetime and awaited more than
once without repeating the job; this does not make a continuation copyable.
Task and scheduler handles cannot escape their owning structured scope.

See [Task And Scheduling APIs](19-standard-library.md#2-task-and-scheduling-apis)
and [handler-based scheduling](16-concurrency.md#111-handler-based-scheduling).

### 2.4 Declarative Retry Policy

```etas
flow ReviewWithRetry(draft: Draft) -> Review {
    @limits([Attempts(2)])
    retry {
        return (Reviewer {}).run(draft);
    }
}
```

Retry behavior is explicit and traceable.

---
