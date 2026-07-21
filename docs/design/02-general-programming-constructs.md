# General Programming Constructs

Etas is a general-purpose programming language first, with agentic constructs added on top. Deterministic computation should use ordinary language features instead of agents.

## 1. Lexical Style

Etas source uses four spaces per indentation level. It supports `//` line comments and `/* ... */` block comments. Comments are ignored by the parser and cannot express policies, effect boundaries, limits, or other semantic constraints.

```etas
// Good: semantic constraints are written as source constructs.
while needs_revision(draft)
    limit Iterations(3)
{
    draft = Rewriter.run(draft);
}
```

## 2. Primitive Types

Etas uses lowercase primitive type names.

| Type | Meaning |
|---|---|
| `bool` | Boolean value, either `true` or `false` |
| `i8` | 8-bit signed integer |
| `i16` | 16-bit signed integer |
| `i32` | 32-bit signed integer |
| `i64` | 64-bit signed integer |
| `i128` | 128-bit signed integer |
| `isize` | Pointer-sized signed integer |
| `u8` | 8-bit unsigned integer or byte value |
| `u16` | 16-bit unsigned integer |
| `u32` | 32-bit unsigned integer |
| `u64` | 64-bit unsigned integer |
| `u128` | 128-bit unsigned integer |
| `usize` | Unsigned size value used by length, capacity, and low-level size APIs |
| `f32` | 32-bit floating-point number |
| `f64` | 64-bit floating-point number |
| `char` | Unicode scalar value |
| `string` | UTF-8 string |
| `bytes` | Byte sequence |
| `unit` | Empty return value |
| `never` | Non-returning computation, such as `abort` |

`never` is the bottom type. An expression of type `never` does not produce a
value, so it is accepted wherever any other value type is expected. It does not
create an implicit conversion at runtime; it simply means control flow does not
reach the point that would need the value.

The numeric type names use the Rust style. Etas should support the full integer width family from the language design, even if the first implementation internally lowers some widths through a smaller runtime representation.

The first implementation can start with `bool`, `i32`, `u32`, `f64`, `string`, `bytes`, and `unit`, then add the remaining numeric widths as needed.

### 2.1 Numeric Conversion And Indexing

Etas has no general implicit numeric conversion between integer types. Assignments, flow calls, method calls, and ordinary comparisons require matching numeric types except for integer literals, which may be typed from context.

```etas
let i: i32 = 3;
let n: usize = i;        // rejected
take_size(i);            // rejected if the parameter expects usize

let z: usize = 0;        // allowed: integer literal is contextually typed
```

Conversions between concrete integer types must be explicit. Indexing is the only special numeric context in the MVP. Etas defines `Index` as a compiler-known prelude spec, not as an ordinary runtime type. `Index` is satisfied by integer literals and by all concrete integer types:

```text
Index :=
    integer literal
  | i8 | i16 | i32 | i64 | i128 | isize
  | u8 | u16 | u32 | u64 | u128 | usize
```

`Index` does not imply assignment coercion or subtyping:

```etas
let pos: i32 = find_position(items);
let value = items[pos];  // allowed: checked index context

let size: usize = pos;   // rejected: no general i32 -> usize coercion
```

For signed index values, checked indexing fails if the value is negative. For all index values, checked indexing fails if the value is outside the collection bounds. The original index type remains visible to the typed IR; the type checker should treat this as an index-specific spec rule rather than a general conversion rule.

Parameterized types, specs, effects, and actions use `<...>` in source, for
example `Result<T, E>`, `Within<ReportsRoot>`, `Error<E>`, and `Fs.read<R>`.
Effect rows and handler types keep the distinct `![...]` notation. Plain
`[...]` remains expression syntax for arrays, indexing, slicing, and value lists
passed to annotations such as `@tools([repo.read])`.

### 2.2 Item Annotations

Annotations attach static metadata to the following item. They use `@` and do
not introduce a new keyword:

```etas
@model("gpt-5.5-thinking")
@tools([repo.read, latex.check])
@limits([ContextTokens(32_000), Attempts(2)])
agent Reviewer(input: Draft) -> Review {
    return perform infer<Review>(build_review_prompt(input));
}

@derive([Schema, PromptEncode])
type Review = {
    summary: string,
    issues: Array<string>,
}

@test
flow parser_accepts_record() -> unit {
    assert(parse("{ x = 1 }").is_ok());
}
```

Annotation arguments must be statically evaluable. Literals, paths, arrays,
records, named arguments, and compile-time constants are allowed. Runtime
effects are rejected:

```etas
@model(read_model_from_file())    // rejected: runtime effect
agent Bad(input: Draft) -> Review {
    return perform infer<Review>(build_review_prompt(input));
}
```

Compiler-known annotations may affect checking or lowering. MVP examples are
`@model`, `@tools`, `@limits`, `@derive`, `@test`, `@deprecated`, `@trace`, and
`@optimization`. User-defined annotations are metadata by default and may be
consumed by package, documentation, test, IDE, or deployment tooling.

Reflection over annotations is static-first. Build tools, package tools, test
runners, documentation generators, and IDEs may inspect item annotations through
compiler metadata. Ordinary runtime code should not receive unrestricted
reflection over value layout, private fields, effect summaries, or policy
internals in the MVP.

Annotations do not grant authority. In particular, `@tools([email.send])`
exposes a model-callable surface, but execution still depends on the tool's
effect summary, the caller's effect boundary, active policy, handlers, limits,
sandboxing, and runtime checks.

### 2.3 Specs And Spec Polymorphism

Etas uses `spec` for static evidence. A spec is not a parent class, a runtime
object interface, or a general subtyping relation. The `~` relation means
"satisfies this spec" or "entails this spec":

```text
:   value/type annotation
~  spec satisfaction or spec entailment
```

Specs have three kinds:

| Kind | Target | Example | Meaning |
|---|---|---|---|
| Type spec | nominal types and generic applied types | `impl TlsStream ~ ByteStream;` | Type-level evidence about values of a type. |
| Callable spec | flows, agents, tools, and composed flows | `flow Normalize(...) ~ Pure { ... }` | Computation-shape evidence about input, output, and effects. |
| Trace spec | flows, agents, tools, and composed flows | `flow Send(...) ~ SafeEmail { ... }` | Temporal and authority evidence about requested actions. |

#### 2.3.1 Type Specs

Type specs can be marker specs, relation specs, or behavioral specs:

```etas
public spec ByteStream;
public spec Region;
public spec Within<Parent>;

public spec PromptEncode {
    flow encode(self) -> PromptPart;
}
```

Types satisfy type specs through explicit evidence:

```etas
public type TcpStream;
public type TlsStream;

impl TcpStream ~ ByteStream;
impl TlsStream ~ ByteStream;

impl Report ~ PromptEncode {
    flow encode(self) -> PromptPart {
        return PromptPart.data(self);
    }
}
```

Generic flows can require spec bounds. Multiple bounds use `+`:

```etas
flow read_all<S ~ ByteStream>(stream: S, limit: ByteLimit) -> bytes {
    return std.stream.read_until_limit(stream, limit, Timeout.none());
}

flow render<T ~ PromptEncode + Schema>(value: T) -> PromptPart {
    return value.encode();
}
```

Relation specs express facts between types. The subject appears on the left of
`~`, and spec type arguments are the other participants in the relation:

```etas
public type WorkspaceRoot;
public type ReportsRoot;
public type DraftsRoot;

impl WorkspaceRoot ~ Region;
impl ReportsRoot ~ Region;
impl DraftsRoot ~ Region;

impl ReportsRoot ~ Within<WorkspaceRoot>;
impl DraftsRoot ~ Within<ReportsRoot>;
```

`impl DraftsRoot ~ Within<ReportsRoot>;` means "the `DraftsRoot` region is
within the `ReportsRoot` region". It does not mean `DraftsRoot` is a subtype of
`ReportsRoot`; it is explicit type-level evidence that generic code can ask for.

Indexed support types can use the same relation:

```etas
public type WorkspacePath<R ~ Region> = string;

flow read_report_path<R ~ Region + Within<ReportsRoot>>(
    path: WorkspacePath<R>
) -> bytes {
    return std.fs.read_bytes(path);
}
```

This accepts `WorkspacePath<ReportsRoot>` and `WorkspacePath<DraftsRoot>`, but it
does not accept an unrelated `WorkspacePath<TempRoot>` unless there is imported
evidence `impl TempRoot ~ Within<ReportsRoot>;`. The compiler checks imported
spec evidence; it should not invent region relationships from path strings.

#### 2.3.2 Callable Specs

Callable specs describe computation shape. They are bodiless and use a function
signature form:

```etas
public spec Stage<I, O, effect E>: callable I => O ![E];
public spec Pure<I, O>: callable I => O ![];
public spec ReportWriter: callable Brief => Draft ![WriterEffects];
```

Generic names used by a callable spec must be declared in `<...>`. Names not
declared there, such as `Brief`, `Draft`, or `WriterEffects`, must resolve to
existing types or effect rows.

If a callable spec omits `![...]`, it imposes no effect constraint. The
implementation still has its own inferred effect row, but the spec does not
restrict it. To require an effect-free computation, write `![]` explicitly, as
in `Pure<I, O> I => O ![]`.

Flows, tools, composed flows, and generated agent entrypoints can satisfy callable
specs. The `agent` declaration itself is a nominal component; its generated
`Name.run` entrypoint is the flow-like callable value. If the spec arguments can
be inferred from the declaration signature, the implementation may write only
the spec name:

```etas
flow Normalize(text: string) -> string ~ Pure {
    return std.text.trim(text);
}

@model("gpt-5")
agent Writer(input: Brief) -> Draft {
    return perform infer<Draft>(Prompt.new().user(input));
}
```

`Normalize` is checked against `Pure<string, string>`; the user does not need to
write those arguments. The generated `Writer.run` entrypoint can be checked
against the callable spec `ReportWriter`; the `agent` declaration itself
remains a nominal component, not a flow value. If inference is ambiguous, the
compiler asks for an explicit instantiation such as
`~ Stage<Brief, Draft, WriterEffects>`.

Type specs and callable specs do not share targets:

```etas
impl Report ~ PromptEncode;       // ok: type spec
flow F(x: X) -> Y ~ Stage { ... } // ok: callable spec

impl Report ~ Stage;              // rejected
flow F(x: X) -> Y ~ PromptEncode  // rejected
```

#### 2.3.3 Trace Specs

Trace specs describe authority and temporal constraints over requested actions.
They are static constraints, not runtime values:

```etas
public spec ReportsOnly: trace =
    +Fs.read<R ~ Within<ReportsRoot>>;

public spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

Trace spec operators are compile-time spec operators:

| Operator | Meaning |
|---|---|
| `+A` | Allow requested action pattern `A`. |
| `-A` | Deny requested action pattern `A`. |
| `A >> B` | Require `A` before `B`. |
| `A << B` | Require `A` after `B`, equivalent to `B >> A`. |
| `S & T` | Require both trace specs. |
| `S | T` | Reserved for trace specs; either spec may accept in the MVP. |

The same type-level evidence used by ordinary values can refine action
patterns. `+Fs.read<R ~ Within<ReportsRoot>>` means reads are allowed only for
regions with imported evidence that they are within `ReportsRoot`.

#### 2.3.4 Spec Entailment And Diamonds

Specs can entail other specs:

```etas
spec ReadableStream ~ ByteStream;
spec WritableStream ~ ByteStream;
spec ReadWriteStream ~ ReadableStream + WritableStream;
```

This is constraint entailment, not subtyping. If `TcpStream ~
ReadWriteStream`, then the compiler can also use evidence that `TcpStream ~
ReadableStream`, `TcpStream ~ WritableStream`, and `TcpStream ~ ByteStream`.
But `ByteStream` is still not a value type:

```etas
let s: ByteStream = tcp; // rejected
```

Diamond-shaped spec hierarchies are allowed because they only create duplicate
evidence paths. The compiler deduplicates those paths. Two restrictions keep the
system simple:

1. the spec entailment graph must be acyclic;
2. inherited behavioral method requirements must merge without signature
   conflicts.

Spec satisfaction is a separate type-checking judgment:

```text
Type equality / unification      checks ordinary value compatibility
Spec satisfaction               checks bounds such as S ~ ByteStream
Effect row inclusion/inference   checks computation effects
```

Effect and trace-spec checks may reuse the same indexed types and spec evidence:

```etas
flow read_path<R ~ Region>(path: WorkspacePath<R>) -> bytes
    ![Fs.read<R>, Error<IOError>]
{
    return std.fs.read_bytes(path);
}

spec ReportsOnly: trace =
    +Fs.read<R ~ Within<ReportsRoot>>;
```

The important point is that resource constraints are expressed once, at the type
level, then reused by ordinary value typing, effect rows, and trace-spec
matching.

## 3. Bindings and Mutation

`let` creates an immutable binding. `var` creates a mutable local binding.

```etas
let title: string = "Trace-aware agents";
var retries: u32 = 0;

retries = retries + 1;
```

Mutable state should remain local by default. Persistent state should go through compiler-known standard memory types such as `MemoryRegion<S>` and `Store<K, V>` so reads and writes remain auditable without adding dedicated persistent-state syntax.

### 3.1 Top-Level Bindings

Etas has no mutable global variables. Top-level `var` is not allowed, and a
top-level `let` is not a hidden process-local cache.

Top-level immutable bindings are allowed only for:

- constants whose initializer is compile-time deterministic and effect-free;
- named runtime resources created by compiler-known standard resource
  constructors, such as `std.memory.region<...>`.

```etas
let DefaultDraftLimit = Tokens(40_000);

type ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, PaperRecord>,
    Drafts: Store<Topic, Draft>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );
```

The `ProjectMemory` binding is immutable. It names a runtime resource handle;
it is not ordinary mutable global state. Reads and writes still happen only
through typed store APIs such as `ProjectMemory.Papers.get(...)` and
`ProjectMemory.Drafts.put(...)`, and those calls contribute region-sensitive
effects.

Creating a top-level resource handle is declarative. It does not connect to the
backend, read data, write data, or perform deployment-time migration by itself.
The runtime binds and validates the resource when a program is deployed or run.

Top-level initializers must not perform ordinary runtime effects:

```etas
let now = Time.now();             // rejected: runtime observation at module load
let papers = web.search("LLM");   // rejected: tool call at module load
var cache = Map<string, Draft>{}; // rejected: mutable global state
```

Type names and value names share one module namespace in the MVP, so prefer a
schema type name such as `ProjectMemorySchema` and a resource handle name such
as `ProjectMemory`.

Top-level resource handles are ordinary module items for visibility and import.
If exported, they can be imported by other files just like flows, agents, tools,
or types:

```etas
// company/memory.es
module company.memory;

public type ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, PaperRecord>,
}>;

public let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );

// company/agents/writer.es
module company.agents.writer;

import company.memory.ProjectMemory;
import company.memory.ProjectMemory as PM;
```

`import ... as ...` is a name-resolution alias only. It does not create a new
resource handle or a second persistent store identity. Static analyses should
resolve both `ProjectMemory.Papers.get(...)` and `PM.Papers.get(...)` to the
same resource symbol.

## 4. Flows As The Ordinary Callable Unit

`flow` is the ordinary user-defined callable unit in Etas. A flow may be pure deterministic computation, ordinary effectful computation, or non-deterministic agent orchestration. The compiler infers which case applies. A `tool` with a body is also written in Etas, but it is a model-callable boundary with extra schema, policy, effect/action, and output-safety rules; ordinary helper logic should remain `flow`.

```etas
flow normalize_title(title: string) -> string {
    return trim(lowercase(title));
}

flow clamp_score(score: i32) -> i32 {
    if score < 1 {
        return 1;
    }

    if score > 10 {
        return 10;
    }

    return score;
}
```

Flows are first-class values. In type positions, Etas uses arrow types. An optional effect row can be written after the output type with `![...]`, but local code can usually omit it and let the compiler infer effects:

```text
Input -> Output ![Effects]
Input -> Output
```

For a single input type, parentheses may be omitted. For multiple inputs, use parentheses or a structured input record:

```text
(A, B) -> Output ![Effects]
```

The compiler normalizes flow types to the canonical internal representation used by the type checker and runtime:

```text
Flow<Input, Output, Effects>
```

Flow declarations may omit the return type. The compiler infers the return type from explicit `return` statements and from the final expression of the body, if present:

```etas
flow normalize_title(title: string) {
    trim(lowercase(title))
}
```

This is equivalent to:

```etas
flow normalize_title(title: string) -> string {
    return trim(lowercase(title));
}
```

This also works with explicit `return`:

```etas
flow clamp_score(score: i32) {
    if score < 1 {
        return 1;
    }

    if score > 10 {
        return 10;
    }

    return score;
}
```

The inference rule is:

```text
If a flow omits `-> ReturnType`:
1. collect every `return expr`;
2. collect the final expression if the body has one;
3. treat `return;` as `unit`;
4. ignore expressions of type `never` as constraints, because they do not return;
5. require all remaining return candidates to unify;
6. infer `unit` if there is no return expression and no final expression.
```

For multi-branch bodies, every returning branch and the final expression must unify to one return type:

```etas
flow clamp_score(score: i32) {
    if score < 1 {
        1
    } else if score > 10 {
        10
    } else {
        score
    }
}
```

If return candidates do not unify, the flow is rejected unless the user makes the result type explicit through a common type such as `Result<T, E>`:

```etas
flow bad(flag: bool) {
    if flag {
        return 1;
    }

    return "no";
}
```

The previous example is rejected because `i32` and `string` do not unify. Prefer:

```etas
flow parse(flag: bool) {
    if flag {
        return Ok(1);
    }

    return Err("no");
}
```

Explicit return types remain recommended for exported APIs, public examples, and flows whose contract is more important than local brevity.

Higher-order programming uses arrow types directly:

```etas
flow choose_drafter(fast: bool) -> string -> Document {
    if fast {
        FastDraft
    } else {
        CarefulDraft
    }
}
```

Anonymous flows use `=>`:

```etas
let normalize = (title: string) => trim(lowercase(title));

let branches = [
    () => StepA(input),
    () => {
        let value = StepB(input);
        return validate(value);
    },
];
```

The `flow` keyword is reserved for named declarations. Anonymous flow expressions use `=>`, and flow types use arrow notation such as `I -> O` or `I -> O ![E]`.

Agent, flow, and tool stages can also be composed with `|`. Composition returns a flow value:

```etas
let pipeline = Researcher | Writer | publish.write;
```

If the stage types line up as `I -> M` and `M -> O`, the composed value has type `I -> O`. The composed flow's effects are the union of the stage effects, and its determinism is inferred from the stages.

The implementation may lower deterministic flows to ordinary internal functions. That is an optimization target, not a separate source construct.

## 5. Determinism Inference

Etas statically infers a determinism class for every flow. This is separate from effect inference:

| Class | Meaning | Typical lowering |
|---|---|---|
| `Deterministic` | Depends only on explicit input values and deterministic computation | Direct internal function |
| `NonDeterministic` | May observe or change external state, call a runtime-provided primitive symbol, call an Etas tool whose body is non-deterministic, read time or randomness, use typed memory APIs, call agents, perform model inference, request human approval, call a non-deterministic subflow, or use runtime choice | Flow runtime mediation; durable orchestration when trace/replay/checkpoint behavior is needed |

Examples:

```text
normalize_title       => Deterministic, effects = []
read_config           => NonDeterministic, effects = [FileRead]
Draft                 => NonDeterministic, requested actions include [Agentic.infer<Writer.run, Draft>]
```

Determinism affects optimization, caching, replay, test behavior, and whether the runtime needs durable orchestration. It does not replace the effect system. A deterministic flow can still be rejected by type checking, and a non-deterministic flow can still require effect/action or policy checks.

## 6. Program Entry Point

An Etas program starts with a `main` flow. `main` is the program's entry flow and is executed directly by the interpreter or runtime.

The canonical entry shape is:

```text
flow main(args: Array<string>) -> i32
```

`main` returns an `i32` process status code. By convention, `0` means success and non-zero values indicate failure.
Like every other flow, `main` may omit its effect row and let the compiler infer
it, or write an explicit `![...]` row as an upper-bound contract checked against
the inferred body effects.

```etas
flow main(args: Array<string>) -> i32 {
    let draft = Writer.run("Draft the project update");

    if approve("Send this update?", draft, risk = Medium) {
        email.send("team@example.com", "Project update", draft.body);
        return 0;
    }

    return 1;
}
```

The runtime checks `main`'s inferred effects and actions, policies, limits, and handlers, then executes `main`.

`main` may call other flows normally:

```etas
flow FastResearch() -> i32 {
    return 0;
}

flow DeepResearch() -> i32 {
    return 0;
}

flow main(args: Array<string>) -> i32 {
    if contains(args, "--fast") {
        return FastResearch();
    }

    return DeepResearch();
}
```

Static analysis starts from `main` and conservatively summarizes all reachable effects and actions. For example, if `main` may call either `FastResearch` or `DeepResearch`, then `effects(main)` includes the union of effects from both reachable flows.

Later implementations may allow richer host-provided input and output types, but the MVP should keep `main(args: Array<string>) -> i32` as the portable command-line entry shape.

## 7. Composite Types

Records are the default structured data form:

```etas
type Paper = {
    title: string,
    authors: Array<string>,
    year: u32,
    venue: string,
    url: Url
}
```

Record expressions support explicit field initialization and shorthand initialization:

```etas
let input = {
    topic = topic,
    papers = papers
};

let same_input = { topic, papers };
```

The shorthand `{ topic, papers }` expands to `{ topic = topic, papers = papers }`. It is part of the source language because Etas agents should normally take one structured input value rather than multiple positional arguments:

```etas
agent Writer(input: { topic: string, papers: Array<Citation> }) -> Draft {
    return perform infer<Draft>(WriterPrompt(input));
}

let draft = Writer.run({ topic, papers });
```

Field shorthand keeps the call site concise while preserving field names for prompt encoding, schema generation, trace output, cache keys, and debugging.

Tuples are useful for small positional values:

```etas
type Span = (u32, u32);
```

Enums represent tagged alternatives:

```etas
enum ApprovalDecision {
    Approved,
    Rejected(reason: string),
}
```

Common generic container types:

| Type | Meaning | Literal |
|---|---|---|
| `Array<T>` | Default immutable sequence, supports checked indexing and slicing | `[a, b, c]` |
| `List<T>` | Functional linked list, useful for `head`, `tail`, and `cons` | `[a; b; c]`, `a :: xs`, `[]` |
| `Map<K, V>` | Key-value map with stable iteration order | `{k => v, k2 => v2}` |
| `Set<T>` | Unique-value set with stable iteration order | `#{a, b, c}` |
| `Range<I>` | Integer range, normally used by loops and slicing | `[i, j)`, `(i, j]` |
| `Slice<T>` | Immutable read-only view over a contiguous sequence range | `xs[i, j)`, `xs(i, j]` |
| `Deque<T>` | Double-ended queue | constructor or builder |
| `Queue<T>` | FIFO queue | constructor or builder |
| `Stack<T>` | LIFO stack | constructor or builder |
| `PriorityQueue<T, P>` | Priority queue ordered by priority `P` | constructor or builder |
| `OrderedMap<K, V>` | Key-sorted map | constructor |
| `OrderedSet<T>` | Key-sorted set | constructor |
| `Option<T>` | `Some(T)` or `None` | enum constructors |
| `Result<T, E>` | `Ok(T)` or `Err(E)` | enum constructors |

`Array<T>` is the default sequence type. A non-empty comma-separated bracket literal creates an array:

```etas
let xs = [1, 2, 3];      // Array<i32>
let ys = xs.push(4);     // returns a new Array<i32>
```

`List<T>` is a functional linked list. It is not the default sequence literal:

```etas
let xs = [1; 2; 3];      // List<i32>
let ys = 0 :: xs;        // List<i32>
```

The empty sequence literal `[]` needs type context because it can be either an empty `Array<T>` or an empty `List<T>`:

```etas
let a: Array<i32> = [];
let l: List<i32> = [];
```

Collections are homogeneous at the outer type level. Heterogeneous values should
be represented explicitly, either with tuples/enums for closed sets or with an
existential element type for open spec-based sets:

```etas
let streams: Array<? ~ ByteStream> = [tcp, tls, file];
```

Here the array element type is the single type `? ~ ByteStream`. Each element
packs a concrete stream handle together with evidence that its hidden type
satisfies `ByteStream`.

Map and set literals are separated to avoid conflict with record shorthand:

```etas
let user = { name, email };              // record shorthand
let scores = { "alice" => 10, "bob" => 8 }; // Map<string, i32>
let users = #{ alice, bob };             // Set<User>
```

Empty record, map, and set literals need type context:

```etas
let config: Config = {};
let counts: Map<string, i32> = {};
let seen: Set<string> = #{};
```

Sequence-like containers use the compiler-known `Index` spec for checked indexing APIs:

```etas
impl Array<T> {
    flow len(self) -> usize;
    flow get<I ~ Index>(self, index: I) -> Option<T>;
    flow at<I ~ Index>(self, index: I) -> T; // infers Error<IndexError>
    flow push(self, value: T) -> Array<T>;
    flow pop(self) -> (Array<T>, Option<T>);
    flow extend(self, values: Array<T>) -> Array<T>;
}

impl List<T> {
    flow is_empty(self) -> bool;
    flow head(self) -> Option<T>;
    flow tail(self) -> Option<List<T>>;
    flow push(self, value: T) -> List<T>; // cons-style push; returns value :: self
    flow pop(self) -> (List<T>, Option<T>); // returns tail and previous head
}

impl Slice<T> {
    flow len(self) -> usize;
    flow get<I ~ Index>(self, index: I) -> Option<T>;
    flow at<I ~ Index>(self, index: I) -> T; // infers Error<IndexError>
    flow to_array(self) -> Array<T>;
}
```

`at` and `[]` are checked operations. On an out-of-bounds index, they raise
`Error<IndexError>`. Code that wants an explicit value can capture that effect:

```etas
let item: Result<Task, IndexError> = tasks[i]?;
```

The `[]` operator uses the same `Index` constraint:

```etas
let pos: i32 = find_position(items);
let item = items[pos];
```

Conceptually, `items[pos]` lowers to a checked indexing operation. It does not require converting `pos` to `usize` at the source level. Map lookup is different: `Map<K, V>.get(key)` accepts a key of type `K`, not `Index`.

Ranges support two literal forms:

```etas
let left_closed = [0, n);   // contains x iff 0 <= x < n
let right_closed = (0, n];  // contains x iff 0 < x <= n
```

Other boundary modes should use named constructors rather than additional literal forms:

```etas
Range.closed(i, j);         // [i, j]
Range.open(i, j);           // (i, j)
```

Slices use range bounds directly after the sequence:

```etas
let prefix = xs[0, n);      // Slice<T>
let middle = xs(i, j];      // Slice<T>
```

Slicing is supported for `Array<T>`, `Slice<T>`, `bytes`, and `Range<I>`. It is not positional on `Map`, `Set`, `Queue`, `Stack`, or `PriorityQueue`. Generic `string` slicing is intentionally not defined here because Unicode character offsets and byte offsets have different safety and performance properties; string slicing should use explicit text APIs such as `text.char_slice(...)` or `text.byte_slice(...)`.

Negative indexes and negative slice bounds are not part of the MVP:

```etas
xs[-1];          // rejected
xs[-20, 0);      // rejected
```

### 7.1 Value Semantics

Records, tuples, `Array<T>`, `List<T>`, `Map<K, V>`, `Set<T>`, and other collection values have value semantics. Passing one of these values to a flow, agent, or tool does not give the callee authority to mutate the caller's value.

```etas
flow add_tag(tags: Array<string>) -> Array<string> {
    return tags.push("reviewed");
}

flow Example() {
    let xs = ["draft"];
    let ys = add_tag(xs);

    // xs is still ["draft"].
    // ys is ["draft", "reviewed"].
}
```

This is the observable language rule. It should feel like safe deep-copy semantics to the programmer, but an implementation must not be required to eagerly copy an entire collection or record at every call boundary. Implementations may use immutable structural sharing, reference counting, arenas, or copy-on-write as long as the caller cannot observe accidental shared mutation.

Collection update APIs on ordinary value collections return updated values. They
must not be typed as `unit`, because `unit` would imply hidden in-place mutation:

```etas
var xs = [1, 2, 3];
xs = xs.push(4);

let (shorter, last) = xs.pop();
```

This rule applies to `Array<T>`, `List<T>`, `Map<K, V>`, `Set<T>`, and related
value collections. Methods such as `push`, `pop`, `insert`, `remove`, and
`extend` return a new collection, or a tuple containing the new collection plus
the removed/found value. An implementation should warn when the result of a
known value-update method is discarded:

```etas
xs.push(4); // likely bug: result is ignored, xs is unchanged
```

`var` changes the binding, not the value-semantics rule:

```etas
var current = ["draft"];
let original = current;

current = current.push("reviewed");

// original is still ["draft"].
// current is ["draft", "reviewed"].
```

Persistent mutable state is represented by `Store<K, V>` APIs, not by hidden shared mutation inside ordinary collections. Local in-place mutation may be added later as an optimization for non-escaping values, but it must remain observationally equivalent to value semantics or be exposed through explicit mutable APIs that the analyzer can see.

For efficient local construction, the standard library may provide explicit
builder types:

```etas
var b = Array.builder<Paper>();

for paper in papers
    limit Iterations(papers.len())
{
    b.push(paper); // mutates the local builder handle, not an Array<T>
}

let built: Array<Paper> = b.freeze();
```

Builder mutation is a visible mutable API. A builder is an opaque local
construction handle, not an ordinary immutable collection. Builder values should
not be prompt-encodable, schema-encodable for model tool calls, stored in
persistent memory, or passed to agents unless a later design explicitly adds
linear/unique ownership for them. This keeps high-performance construction
available without making ordinary `Array<T>` or `List<T>` shared mutable
objects.

## 8. Encapsulation and Methods

Etas records should support private fields and methods so domain objects can protect their own invariants. This matters for long-running agent state such as sessions, interrupts, approvals, and resumable executions.

```etas
enum SessionState {
    Running,
    Waiting,
    Completed,
    Failed(reason: string),
    Cancelled(by: string),
}

type Session = {
    private id: SessionId,
    private tenant_id: TenantId,
    private user_id: UserId,
    private state: SessionState,
    private current_interrupt: Option<Interrupt>,
    private events: Array<DomainEvent>,
}
```

Methods are declared in an `impl` block whose target kind is an ordinary type. A method uses the same `flow` declaration form as a top-level callable; deterministic methods are lowered to direct internal methods by the compiler. Code outside the defining module cannot mutate private fields directly.

`impl` is also used by the effect system to attach `action` signatures to an effect tag, but the forms are kind-checked and cannot be mixed:

```text
impl TypeName   { flow ... }    // ordinary methods
impl EffectName { action ...; } // effect actions
```

An ordinary type `impl` cannot contain `action`, and an effect `impl` cannot contain ordinary `flow` methods.

```etas
impl Session {
    flow new(tenant_id: TenantId, user_id: UserId) -> Result<Session, SessionError> {
        if is_empty(tenant_id) || is_empty(user_id) {
            return Err(MissingIdentity);
        }

        var s = Session {
            id: new_session_id(),
            tenant_id,
            user_id,
            state: Running,
            current_interrupt: None,
            events: [],
        };

        s.emit(SessionStarted { id: s.id });
        return Ok(s);
    }

    flow reconstruct(
        id: SessionId,
        tenant_id: TenantId,
        user_id: UserId,
        state: SessionState,
    ) -> Session {
        assert(not is_empty(tenant_id));
        assert(not is_empty(user_id));

        return Session {
            id,
            tenant_id,
            user_id,
            state,
            current_interrupt: None,
            events: [],
        };
    }

    flow pause(self, interrupt: Interrupt) -> Result<unit, SessionError> {
        match self.transit_to(Waiting) {
            Ok(_) => {},
            Err(err) => return Err(err),
        }

        self.current_interrupt = Some(interrupt);
        self.emit(InterruptRaised { id: self.id });
        return Ok(unit);
    }

    flow is_terminal(self) -> bool {
        match self.state {
            Completed => true,
            Failed(_) => true,
            Cancelled(_) => true,
            _ => false,
        }
    }

    private flow transit_to(self, to: SessionState) -> Result<unit, SessionError> {
        if not valid_transition(self.state, to) {
            return Err(InvalidStateTransition { from: self.state, to });
        }

        self.state = to;
        return Ok(unit);
    }

    private flow emit(self, event: DomainEvent) -> unit {
        self.events = self.events.push(event);
    }
}
```

The important rule is that validation stays close to the data it protects. A session should not become `Completed`, `Waiting`, or `Cancelled` by direct field assignment from an HTTP handler, hook callback, timer, or flow body.

## 9. Control Flow

Etas supports normal control flow:

```etas
if condition {
    ...
} else {
    ...
}

match decision {
    Approved => send(),
    Rejected(reason) => log(reason),
}

match (is_valid, draft) {
    (true, ready) => publish(ready),
    (false, _) => request_revision(),
}

for paper in papers
    limit Iterations(20)
{
    process(paper);
}

while needs_revision(draft)
    limit Iterations(3)
{
    draft = revise(draft);
}
```

In pattern position, `true`, `false`, integer literals, string literals, and char literals are constant matches rather than variable bindings. The `limit` clause is required when a non-deterministic loop cannot be statically proven to terminate.

## 10. Modules, Packages, And Imports

Modules provide source-level namespacing. Packages provide distribution,
dependency resolution, versioning, and the boundary of a public API. Etas does
not need a `package` source keyword: package metadata belongs in the package
manifest, while source files declare their module path.

```etas
module research.survey;

import std.collections.List;
import agents.researcher.Researcher;
```

### 10.1 Package Manifest Boundary

An Etas package is described by an `etas.toml` manifest at the package root.
The source language does not have a `package` keyword and does not put registry
URLs, Git URLs, or local filesystem dependency paths inside `import`
declarations. Those concerns belong to package management tooling.

The manifest supplies package identity, source root, dependencies, and runnable
entry points:

```toml
[package]
name = "research-assistant"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[dependencies]
std = { version = "0.1" }
company_agents = { package = "company-agents", version = "1.2", import = "company.agents" }

[[bin]]
name = "research-assistant"
module = "app.main"
flow = "main"
```

The package resolver maps dependency import roots such as `std` or
`company.agents` to resolved packages. Source imports are resolved through this
package graph. The compiler records the resolved package, module, item, and
version in package metadata so public APIs remain reviewable and reproducible.

Detailed manifest, lockfile, registry, workspace, dependency metadata, and tool
binding rules are specified in [Package Management](15-package-management.md).

### 10.2 Module Path Resolution

Import paths are logical module paths. They are not raw filesystem paths. The
default package resolver maps logical module paths to source files by
convention, but the language semantics depend on resolved package/module/item
identity rather than on directory strings.

The default source root is the package's `[source].root`, normally `src`.
Within a source root, the default resolver maps:

```text
module foo.bar;  ->  src/foo/bar.es
module foo.bar;  ->  src/foo/bar/mod.es
```

For example:

```text
src/
  company/
    agents/
      writer.es      // module company.agents.writer;
    memory.es        // module company.memory;
```

This layout allows source code to stay logical:

```etas
module company.agents.writer;

import std.io.println;
import company.memory.ProjectMemory;
```

The resolver must enforce these rules:

- the `module` declaration inside a file must match the logical module path that
  resolved to that file;
- if both `src/foo/bar.es` and `src/foo/bar/mod.es` define `module foo.bar`,
  the package is ambiguous and must be rejected;
- Etas supports module merging: several source files may contribute declarations
  to the same logical module when the resolver maps them to that module;
- all module parts share one top-level module namespace, so duplicate top-level
  declaration names in the merged module are rejected;
- imports are local to the module part that declares them; `public import` from
  any part contributes to the merged module's public API;
- relative filesystem imports such as `import "./writer.es";` are not part of
  the source language;
- generated modules, virtual standard-library modules, and dependency modules
  are valid as long as the package resolver maps them to stable logical module
  identities.

The compiler should lower every resolved reference to a stable identity:

```text
PackageId + ModuleId + ItemId
```

This gives the type checker, effect checker, public API metadata, LSP, FIR, and
AIR a stable representation that does not depend on the physical layout chosen
by a build tool.

Package management is tooling, not source syntax. It is responsible for:

- dependency version resolution;
- a lockfile for reproducible builds;
- package-level public API metadata;
- generated effect/action summaries for exported flows, agents, tools, and
  re-exports;
- LSP/completion data for module paths and exported names.

### 10.3 Import Forms

Etas supports module imports, item imports, aliases, grouped imports, wildcard
imports, and public re-exports:

```etas
import std.io;                         // use std.io.println
import std.io as io;                   // use io.println
import std.io.println;                 // use println
import std.io.println as log;          // use log
import std.io.{print, println, eprintln};
import std.io.{println as log, read_line};
import std.io.*;                       // import all public names from std.io
public import std.io.{println, eprintln};
public import std.prelude.*;
```

An import without `public` is private to the current module. `public import`
re-exports the imported public names as part of the current module's public API.
Wildcard import only imports public names; it never exposes private declarations
from the imported module.

Wildcard imports are legal because they are useful for application modules and
prelude-style modules. The compiler should still keep them analyzable:

- local declarations and explicit imports have priority over wildcard imports;
- if a name is provided by more than one wildcard import, an unqualified use is
  ambiguous and must be rejected or fixed by an explicit import;
- `public import module.*` is allowed, but package metadata must record the exact
  exported name set for the resolved dependency version;
- library packages may lint wildcard imports in public APIs, but this is a lint,
  not a language restriction.

Importing a name does not perform effects. For example, importing
`std.io.println` only brings the stdlib flow into scope; calling it is what
contributes `Console.stdout_write` to effect inference.

### 10.4 Visibility

Top-level declarations default to `private`. A declaration must be marked
`public` to be imported from another module or exported from a package:

```etas
private flow normalize(text: string) -> string {
    return text.trim();
}

public flow summarize(input: string) -> Summary {
    let body = normalize(input);
    return Summary { body };
}
```

The explicit `private` modifier is optional at top level, but it is useful when
the author wants the boundary to be obvious in public modules.

Record fields default to `public` because ordinary data records should not force
boilerplate on every field. Sensitive or invariant-bearing fields should be
marked `private`:

```etas
public type Session = {
    id: SessionId,
    user_id: UserId,
    private state: SessionState,
    private token_budget: TokenBudget,
}
```

A public type does not make its private fields visible. Code outside the
defining module can name `Session`, pass it around, and call public methods, but
cannot directly read or write `state` or `token_budget`.

Methods inside `impl` blocks default to `private`. Public methods form the
callable API of the type:

```etas
impl Session {
    public flow start(self) -> Result<unit, SessionError> {
        return self.transit_to(SessionState.Started);
    }

    private flow transit_to(self, to: SessionState) -> Result<unit, SessionError> {
        ...
    }
}
```

The visibility model is therefore:

| Construct | Default visibility | Notes |
|---|---|---|
| Top-level `type`, `enum`, `spec`, `let`, `flow`, `agent`, `tool`, `effect`, `protocol` | `private` | Mark `public` to export |
| Import | `private` | `public import` re-exports |
| Record field | `public` | Mark sensitive fields `private` |
| `impl` method | `private` | Mark public API methods `public` |

This is intentionally closer to Rust's enforced visibility than Python's naming
conventions. Agent systems need compiler-visible boundaries for typed memory APIs,
effect boundaries, tool access, approval state, session state, and public effect
contracts.
