# Type System and Errors

## 1. Type System

Etas separates the type system from the effect system.

```text
Type system   = classifies values
Effect system = classifies computations
```

The type checker and effect checker are implemented in the same compiler pipeline and share the typed AST/AIR, symbol table, flow signatures, agent signatures, and tool signatures. Conceptually, however, they answer different questions:

```text
type(flow input/output) = what values are consumed and produced
effects(flow body)     = what observable actions may occur during execution
```

Etas's value-level type system is layered:

```text
Ordinary types
+ Structured output schemas
+ Trust types
+ Provenance types
+ Protocol/session types
```

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
arguments, spec implementations, effect indices, and policy predicates, the
compiler does not allocate runtime values for them. `TcpStream` is also
bodyless, but it represents an opaque runtime handle because values of that type
are returned and passed at runtime.

The MVP does not need an `opaque` keyword. If future module privacy needs a
stronger distinction between "nominal but representation-visible inside a
module" and "representation hidden across module boundaries", `opaque` can be
added as a visibility modifier later. For now, `type` means identity by default,
and `alias` is the explicit escape hatch for transparent abbreviations.

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

Etas does not include refinement types in the MVP language. Value constraints
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

This keeps the type system small while still allowing agent outputs and tool
arguments to be checked after structured decoding. Future versions may add
declarative schema annotations, but the MVP should not expose predicate-refined
types as a type form.

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

Specs have three MVP kinds:

| Kind | Constrains | Typical subjects |
|---|---|---|
| `type` | Types, resource markers, static relations, and effect-row/static entities | `TlsStream ~ ByteStream`, `R ~ Within<ReportsRoot>`, `effect E ~ RetrySafeEffects` |
| `callable` | Flow/tool/agent-method call shape | `Writer.run ~ Stage<Brief, Draft, E>` |
| `trace` | Requested-action traces, authority, and temporal constraints | `+Email.send`, `-Secret.read`, `Approval.request >> Email.send` |

`EffectSpec` is not a separate kind in the MVP. Effect rows are static entities,
so constraints over them are `type` specs. `ProtocolSpec` is also not part of the
MVP spec system; protocol declarations remain a separate advanced construct.

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

Generic APIs use `~` bounds:

```etas
flow read_until_limit<S ~ ByteStream>(
    stream: S,
    limit: ByteLimit,
    timeout: Timeout?
) -> bytes;
```

Parameterized marker specs can express relationships between types. This is
the preferred way to model resource inclusion, stream capabilities, prompt
encoding support, path regions, and other compile-time facts without adding
general subtyping:

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
strings. If code needs to widen a region-indexed value, it should use an
ordinary flow whose signature states the required relation:

```etas
flow widen_path<Child ~ Region + Within<Parent>, Parent ~ Region>(
    path: WorkspacePath<Child>
) -> WorkspacePath<Parent> {
    return path;
}
```

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

flow read_one(stream: ? ~ ByteStream) -> bytes {
    return std.stream.read_until_limit(stream, ByteLimit(4096), Timeout.ms(1000));
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
downcast to the implementation type in the MVP.

MVP existential bounds are limited to object-safe type specs:

- marker specs are object-safe;
- behavioral specs are object-safe only if their required flows have no generic
  type parameters and their public signatures do not expose the hidden concrete
  type except as the receiver;
- callable specs are not existential object bounds in the MVP. Flow values already
  have arrow types with effect rows, and heterogeneous stage registries should
  use explicit wrapper records or enums until callable-spec existentials are
  designed.

Existential use may require dynamic evidence dispatch or an evidence package at
runtime. The compiler still knows the allowed spec surface and effect row of each
operation; it simply does not know the concrete implementation type after the
value has been packed.

#### 1.5.3 Callable Specs

Callable specs target computations: flows, agents, tools, and composed flows.
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
restrict it. To require an effect-free computation, write `![]` explicitly, as
in `Pure<I, O> I => O ![]`.

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
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & -Secret.read<_>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

Trace-spec operators are kinded:

| Operator | Kind | Meaning |
|---|---|---|
| `+A` | `ActionPattern -> trace` | Allow matching action pattern `A` |
| `-A` | `ActionPattern -> trace` | Deny matching action pattern `A` |
| `A >> B` | `ActionPattern x ActionPattern -> trace` | Every `B` must be preceded by matching `A` |
| `A << B` | `ActionPattern x ActionPattern -> trace` | Every `A` must be preceded by matching `B` |
| `S & T` | `Spec<K> x Spec<K> -> Spec<K>` | Both specs must hold |
| `S | T` | `TraceSpec x TraceSpec -> TraceSpec` | Either trace spec may accept; MVP reserves this for trace specs only |

Applying `+`, `-`, `>>`, or `<<` to type specs or callable specs is rejected.
Combining specs of different kinds with `&` is also rejected:

```etas
spec Bad =
    ByteStream & +CompanyEmail.send<WorkAccount>; // rejected
```

Trace specs replace the old policy-block surface syntax. The language may still
use "policy" informally for the runtime/enforcement concept, but the source
constraint is a `trace` spec consumed through `~`.

#### 1.5.5 Spec Targets And Kind Checking

Type specs, callable specs, and trace specs are kind-checked and cannot be mixed:

| Target | May satisfy |
|---|---|
| Nominal type | Type spec |
| Generic applied type | Type spec |
| Effect row / effect parameter | Type spec |
| Flow declaration/value | Callable spec and trace spec |
| Agent declaration / generated agent method | Callable spec and trace spec |
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
| Same signature but different semantic law | MVP does not check laws |

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

Effects are defined in detail in [Effect System and Inference](06-effect-system-and-inference.md). They are not ordinary value types, but flow type annotations can include an effect row. Most local declarations omit effects and let the compiler infer them. When a flow type needs an explicit effect row, it is written after the output type with `![...]`:

```etas
Url -> Paper ![Network, ReadPDF]
```

Here `ReadPDF` is a library-defined effect that extends the core `FileIO` effect.

This example is non-deterministic under the MVP two-class model because it
performs network/file effects. Agent calls are also non-deterministic when the
called agent method performs model inference, but `Agentic.infer<C, O>` is
recorded as requested-action metadata rather than as a source-level escaping
effect row. Determinism is inferred from the body instead of from a separate
callable keyword.

```etas
flow read_paper(url: Url) -> Paper
{
    ...
}
```

The compiler infers `![Network, ReadPDF]` from the body as a computation
summary. Agent declarations do not by themselves record an
`Agentic.infer<C, O>` requested action. A requested action is recorded where an
agent method executes agent-scoped `perform infer<T>(prompt)`; callers such as
`A.run(input)` see that action through the callee summary, not as an escaping
effect they must handle. Public agent metadata may still expose ordinary
escaping effects, tool surface, runtime support requirements, and schema
information.

```etas
@model("gpt-5.5-coder")
@tools([repo.read, test.run])
agent Coder(input: Issue) -> Patch {
    let prompt = Prompt.new()
        .system(Trusted("Generate a minimal patch and test plan."))
        .data(input);
    return perform infer<Patch>(prompt);
}
```

### 1.7 Effect Action Authority

Concrete runtime authority is represented by effect actions, not by a separate
source-level permission type. A tool may expose a fine-grained effect boundary:

```etas
tool write_report(path: Path, content: string) -> unit ![ProjectWorkspace.write<"reports/**">]
{
    perform ProjectWorkspace.write(path, content);
}
```

An agent may list only model-callable tools. The tools themselves carry their
effect/action boundaries:

```etas
@model("gpt-5.5-coder")
@tools([repo.read, patch.write])
agent Coder(input: Issue) -> Patch {
    let prompt = Prompt.new()
        .system(Trusted("Generate a patch within the checked workspace boundary."))
        .data(input);
    return perform infer<Patch>(prompt);
}
```

Deployment/runtime configuration grants or denies concrete effect/action
instances such as `ProjectWorkspace.read<"src/**">`,
`ProjectWorkspace.write<"src/**">`, or `Command.run<DefaultCommandSandbox>`.
Source code should refer to typed effect actions and policy rules, not raw
permission strings. `ProjectWorkspace` is a host/project package effect, not a
core standard-library effect.

### 1.7 Bottom Type

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

### 1.8 Provenance Types

For factual claims:

```etas
type Claim = {
    text: string,
    evidence: Array<Citation>,
    confidence: Confidence
}
```

A policy may require:

```etas
policy CitationPolicy {
    require evidence for Claim;
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
| `BudgetExceeded` | Token, cost, or time budget exceeded |
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
LoadReport : Path -> Report
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
e : T ![Error<E>, ...]
-----------------------
e? : Result<T, E> ![...]
```

The captured `Error<E>` effect is removed from the resulting expression. Other
effects remain:

```text
std.io.read_line()  : string ![Error<IOError>]
std.io.read_line()? : Result<string, IOError>
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

In the MVP, `?` is accepted only when the expression may raise exactly one
distinct `Error<E>` effect. If an expression may raise multiple error effects,
the program must use `handle` and map the error cases explicitly before applying
`?`. The MVP does not infer ad-hoc error unions or implicit error conversions.

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

As an expression form, the lowering is equivalent to:

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

Handlers can recover from selected effect actions without introducing unrestricted continuations:

```etas
flow LoadOrDefault(path: Path) -> Report {
    handle {
        LoadReport(path)
    } with {
        Error<AppError>.raise(err) => {
            finish Report {
                title = "Untitled",
                body = err.message
            };
        }
    }
}
```

If the handler fully handles `Error<AppError>`, that effect does not escape `LoadOrDefault`. Handler arms may perform their own effects, and those effects are included in the surrounding signature.

An action can be resumed only if its return type is not `never`:

```etas
effect Approval {
    action request(req: ApprovalRequest) -> ApprovalDecision;
}
```

Operations returning `never`, such as `Error.raise`, cannot use a `resume`
statement in a handler arm. They can use `finish value;` to complete the handled
expression with a fallback value, or they can terminate with another
non-returning expression such as `abort(...)`. Because `never` is the bottom
type, such a non-returning arm is accepted for any handled expression answer
type. `resume` and `finish` are control-flow keywords, not standard-library
functions.

An ordinary expression arm such as `Error<E>.raise(err) => Err(err)` is not
valid handler syntax. Use `finish Err(err);` so the control-flow boundary is
explicit.

### 2.4 Declarative Retry Policy

```etas
flow ReviewWithRetry(draft: Draft) -> Review {
    retry
        limit Attempts(2)
    {
        return Reviewer.run(draft);
    }
}
```

Retry behavior is explicit and traceable.

---
