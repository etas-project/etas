# General Programming Constructs

This chapter defines ordinary source constructs. Type/spec details are in
[Types](05-type-system-and-errors.md); public collection APIs are in
[Standard Library](19-standard-library.md).

## 1. Lexical Style

Etas source uses four spaces per indentation level. It supports `//` line comments and `/*... */` block comments. Comments are ignored by the parser and cannot express trace specs, runtime policy, effect boundaries, limits, or other semantic constraints.

```etas
// Good: semantic constraints are written as source constructs.
@limits([Iterations(3)])
while needs_revision(draft) {
    draft = (Rewriter {}).run(draft);
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

### 2.1 Numeric Conversion And Indexing

Etas has no general implicit numeric conversion between integer types. Assignments, flow calls, method calls, and ordinary comparisons require matching numeric types except for integer literals, which may be typed from context.

```etas
let i: i32 = 3;
let n: usize = i;        // rejected
take_size(i);            // rejected if the parameter expects usize

let z: usize = 0;        // allowed: integer literal is contextually typed
```

Conversions between concrete integer types must be explicit. Indexing has a checked numeric contract. Etas defines `Index` as a compiler-known prelude spec, not as an ordinary runtime type. `Index` is satisfied by integer literals and by all concrete integer types:

```text
Index:=
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

Generic numeric helpers use the source-declared, compiler-known
`std.math.Number` spec. It admits the primitive integer and float families,
preserves concrete widths and keeps ordinary conversion rules. Its evidence
and the `abs/min/max` contracts are defined in
[Numeric Helpers](19-standard-library.md#5-numeric-helpers).

Parameterized types, specs, effects, and actions use `<...>` in source, for
example `Result<T, E>`, `Within<ReportsRoot>`, `Error<E>`, and `Fs.read<R>`.
Effect rows and handler types keep the distinct `![...]` notation. Plain
`[...]` remains expression syntax for arrays, indexing, slicing, and value lists
passed to annotations such as `@tools([repo.read])`.

### 2.2 Annotations

Annotations use `@` and do not introduce a new keyword. Item annotations attach
metadata or compiler-recognized contracts to the following declaration.
`@inject` / `@inject(Action)` marks stored record/agent fields or supported
named-callable parameters; see [Scoped Injection](#72-scoped-injection).

```etas
@model("gpt-5.5-thinking")
@tools([repo.read, latex.check])
@limits([ContextTokens(32_000), Attempts(2)])
agent Reviewer = {};
impl Reviewer {
    public flow run(self, input: Draft) -> Review {
        return perform infer<Review>(build_review_prompt(input));
    }
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

Item annotation arguments must be statically evaluable. Literals, paths, arrays,
records, named arguments, and compile-time constants are allowed. Runtime
effects are rejected:

```etas
@model(read_model_from_file())    // rejected: runtime effect
agent Bad = {};
impl Bad {
    public flow run(self, input: Draft) -> Review {
        return perform infer<Review>(build_review_prompt(input));
    }
}
```

`@limits(bounds)` is also permitted immediately before a `for`, `while`, `retry`,
or block. It establishes a checked resource scope, not an optional optimization
hint. Local bounds may depend on current parameters or local values:

```etas
flow Revise(draft: Draft, token_budget: u64) -> Draft {
    var current = draft;
    @limits([Iterations(3), Tokens(token_budget)])
    while needs_revision(current) {
        current = (Rewriter {}).run(current);
    }
    return current;
}
```

| Placement | Argument and Scope Rules |
|---|---|
| Executable `flow`, `agent`, or `tool` declaration, including methods | Static arguments; each invocation's child budget covers omitted-parameter resolution, body and trailing handler, not supplied-argument evaluation. |
| `for`, `while`, `retry` | Local arguments are evaluated once before entry, not once per iteration or attempt; the budget covers the whole construct. |
| Block | Local arguments are evaluated once; the budget covers all computation and owned child tasks in the block. The block retains its result type and ordinary `return` target. |
| Local argument expressions | Must be deterministic, non-suspending and action-free; an empty escaping effect row alone is insufficient. Evaluation remains subject to enclosing budgets. |
| Invalid annotation | Wrong argument types, unsupported targets, or invalid constant bounds are compile errors. Dynamic bounds must be validated before entering the annotated computation; invalid values cannot disable enforcement. |

`bounds` has type `Array<Limit>`; the dimension constructors are std support
values. Only `@limits` is admitted on these local constructs; this does not
enable arbitrary expression annotations or runtime evaluation of other item
metadata. Use a local annotated block for a parameter-dependent declaration
budget. Nested bounds only restrict inherited budgets, and checkpoint/resume
retains remaining counters. See [Resource Contracts](04-flows-human-gates-and-protocols.md#14-resource-contracts-and-budgeted-loops)
for composition, exhaustion, and error semantics.

Agent `@model` and `@tools` apply as method defaults. A method annotation
replaces its model/tool default, not inherited authority; `@tools([])` selects
no model tools. Member names such as `A.ask_coder` bind the current instance
outside the model argument schema. Agent/method resource bounds conjoin.
A declaration bound is per request; an enclosing annotated block supplies a
shared lifetime budget around construction and use.

Standard annotations have only their specified checks and behavior.
User-defined annotations are metadata by default. Annotation inspection does
not expose private state or grant authority; code reflection is specified
[separately](18-reflection-and-checked-code-generation.md).

Annotations do not grant authority. In particular, `@tools([email.send])`
exposes a model-callable surface, but execution still depends on the tool's
effect summary, the caller's effect boundary, active policy, handlers, limits,
sandboxing, and runtime checks.

### 2.3 Specs And Spec Polymorphism

`~` expresses spec satisfaction, not value subtyping. Spec kind is inferred
from its declaration; optional kind annotations check that inference.

| Kind | Target |
|---|---|
| Type | Types/effect types; marker, relation or behavioral requirements. |
| Callable | Flows, tools or agent methods; input/output and optional effect bounds. |
| Trace | Computation behavior; allowed/denied actions and ordering constraints. |

Generic evidence, existential `? ~ S`, method selection `value::Spec.method`,
diamonds and kind checking are specified in [Spec Polymorphism](05-type-system-and-errors.md#15-spec-polymorphism).

## 3. Bindings and Mutation

`let` creates an immutable binding. `var` creates a mutable local binding.
In a local declaration, `var` is a binding modifier, not a reference type. It
permits the declared name to be rebound to another value of the same type.
The separate method-receiver form `var self` is specified in section 3.2; it
does not introduce general mutable parameters or first-class references.

```etas
let title: string = "Trace-aware agents";
var retries: u32 = 0;

retries = retries + 1;
```

Ordinary flow, agent, tool, and method parameters are immutable bindings. The
only updating parameter form is the restricted receiver `var self`. Passing a
local `var` as an ordinary argument still passes its current value, without
transferring mutability of the caller's binding:

```etas
flow append(values: Array<i32>, value: i32) -> Array<i32> {
    return values.push(value);
}

flow Example() {
    var values = [1, 2];
    let updated = append(values, 3);

    // values is still [1, 2].
    // updated is [1, 2, 3].
}
```

Reassigning an ordinary parameter is rejected. Arbitrary `var` parameters
remain unsupported:

```etas
flow invalid(values: Array<i32>) {
    values = []; // rejected: parameters are immutable bindings
}

flow unsupported(var values: Array<i32>) -> unit; // rejected syntax
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

type ProjectMemorySchema = {
    Papers: Store<PaperId, PaperRecord>,
    Drafts: Store<Topic, Draft>,
};

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

Top-level initializers must not perform ordinary runtime effects, including
omitted-field injection even when a local handler consumes its escaping row.
A fully explicit passive record can still be an action-free initializer:

```etas
let now = Time.now();             // rejected: runtime observation at module load
let papers = web.search("LLM");   // rejected: tool call at module load
var cache = Map<string, Draft>{}; // rejected: mutable global state
```

Type names and value names share one module namespace , so prefer a
schema type name such as `ProjectMemorySchema` and a resource handle name such
as `ProjectMemory`.

Top-level resource handles are ordinary module items for visibility and import.
If exported, they can be imported by other files just like flows, agents, tools,
or types:

```etas
// company/memory.es
module company.memory;

public type ProjectMemorySchema = {
    Papers: Store<PaperId, PaperRecord>,
};

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

`import... as...` is a name-resolution alias only. It does not create a new
resource handle or a second persistent store identity. Static analyses should
resolve both `ProjectMemory.Papers.get(...)` and `PM.Papers.get(...)` to the
same resource symbol.

Agent references are the explicit exception to passive value copying: copying
one shares the same isolated instance. Records/containers may contain them but
must preserve their scope bounds; copies do not clone state or transfer ownership.

### 3.2 Restricted Updating Receivers

An ordinary type method may declare `var self` as its first parameter. It
updates the caller's local binding while preserving value semantics; it does
not expose a shared mutable object or a reference value. Plain `self` remains
an immutable value parameter.

```etas
type Switch = { enabled: bool }

impl Switch {
    flow enable(var self) -> unit {
        self = Switch { enabled = true };
    }
}

flow Example() {
    var a = Switch { enabled = false };
    let b = a;
    a.enable();
    // a.enabled is true; b.enabled is still false.
}
```

The restrictions are:

- These rules govern passive type methods and their behavioral spec signatures.
  Agent receivers instead follow [instance-access semantics](03-agents-tools-prompts-memory.md#13-operational-intuition).
  Free flows, tools and other parameters do not gain an updating mode.
- The receiver must be a direct local `var` binding, or the current method's
  `var self` binding for a nested update. Reject `let`, ordinary parameters,
  temporaries, fields, indexes, and arbitrary receiver expressions. A receiver
  type is inferred from its enclosing `impl`/spec; `var self: T` is not a
  second receiver syntax.
- Resolve the target binding/method, evaluate ordinary arguments left to
  right, then acquire exclusive access and read the receiver's current value.
  Argument evaluation may take value snapshots; it is not part of the update
  interval.
- While the call holds that access, another task, callback, or reentrant path
  cannot read or write the same caller binding. Independent value copies remain
  usable. The method reads and updates its own local `self`; passing that value
  to an ordinary parameter does not transfer update access.
- Update access cannot escape, be stored, or be captured by an escaping flow.
  Extracting a bound updating method, such as `let f = a.enable`, is rejected.
  Returning/capturing an independent value snapshot is not a reference escape.
- Receiver mode is part of method/spec signature checking, but not a basis for
  overload selection: declarations with the same name/signature cannot differ
  only by `self` versus `var self`. Existing `Array.push`/`List.push` still
  return new values; an updating alternative needs a different name.

#### 3.2.1 Commit And Failure

The observable model is local computation followed by writeback. On normal
return, commit the final `self` to the caller binding before exposing the
method result. On error escape, cancellation, or another control transfer
(including `finish`) that unwinds the call, discard that pending update and
release exclusive access. A non-resumable error escaping the call releases its
update access before an outer error handler can suspend. Ordinary arguments'
earlier side effects are not undone.

Completion and cancellation choose one outcome: a completed normal return
commits once; cancellation that unwinds the call first discards the update.
Cancellation after a committed return does not undo that earlier call.

Returning `Err(...)` is a normal return and therefore commits. An error handled
inside the method also commits if execution later returns normally. Nested
updating calls commit to the enclosing method's local `self`; they do not
commit the outer caller binding early. This rollback guarantee covers only
the receiver value, not changes already made through files, stores, resource
handles, or other external actions.

Implementations may share immutable storage or use copy-on-write. In-place
optimizations must preserve other value copies and the no-writeback-on-unwind
guarantee; exclusive binding access alone is not permission to destroy the
old value before a potentially failing call completes.

#### 3.2.2 No Suspension While Updating

A passive updating call may not hold receiver access across a suspension. The compiler
tracks a transitive may-suspend summary separately from escaping effects and
determinism. Model interaction, task waits/yields, delayed handler resumption,
and unknown callback/primitive behavior prevent a non-suspension proof. Neither
`![]`, default handling, nor `Deterministic` alone proves non-suspension.
Handler behavior and nested calls must be included; an unhandled terminating
error that unwinds the update is not a suspended update.
An override that can suspend cannot reuse a proof based only on a synchronous
default handler; reject the affected updating call unless its context is proved
safe.

Reject the updating method/call when non-suspension or exclusivity cannot be
established. Imported methods must preserve these checked facts in package
metadata; user annotations cannot assert them unchecked. Use a value-returning
method for asynchronous or uncertain work. Ordinary helper flows can still
update their own local copy and return it for higher-order use:

```etas
flow Enabled(value: Switch) -> Switch {
    var next = value;
    next.enable();
    return next;
}
```

### 3.3 Lambda And Handler Capture

Anonymous flows and Handler values use the same lexical capture rules.
Capturing a local `var` within its owning task preserves the original binding,
and assignments are immediately visible to other accesses to that binding.
Captured `let` bindings retain their immutability. Copying a callable preserves
its captured bindings. Captured local `var` bindings are scope-bound and task-local.

The value, lifetime, concurrency, analysis and checkpoint requirements are
defined in [Lambda And Handler Capture](20-lexical-capture.md).

## 4. Flows As The Ordinary Callable Unit

A `flow` is an ordinary first-class callable. Use `tool` only for a
model-callable boundary and an agent method for checked isolated-state access.
Function types use `I -> O ![E]`, or `(A, B) -> O ![E]`; arrows associate
right. Omitting a local effect row requests inference, not purity.

```etas
flow normalize(text: string) {
    std.text.trim(text)
}

let normalize_again = (text: string) => std.text.trim(text);
```

The `flow` keyword declares named callables; anonymous flows use `=>`.
Values can be passed, stored and returned without executing their bodies.
Invocation retains latent effects, captures and scope bounds.

| Return inference | Rule |
|---|---|
| Explicit `-> T` | Every returning path must fit T. |
| Omitted return type | Unify explicit `return expr` values and the final expression. |
| `return;` | Contributes `unit`. |
| `never` expression | Does not constrain a returned value type. |
| No value-returning expression | Infer `unit`. |
| Incompatible branches | Reject; use an explicit shared ADT such as `Result`. |

Stage composition/application use `|` and `~>`; see
[Stage Composition](04-flows-human-gates-and-protocols.md#12-stage-composition).

### 4.1 Default And Injected Parameters

```etas
flow decorate(text: string, prefix: string = "[", suffix: string = "]") -> string {
    std.text.concat([prefix, text, suffix])
}
let a = decorate("ready");
let b = decorate("ready", suffix = "!");
```

| Parameter contract | Eligible declarations |
|---|---|
| `name: T = expr` | Named flows, methods, tools, effect actions and behavioral spec methods. |
| `@inject name: T` / `@inject(Action) name: T` | Named flows, methods, tools and behavioral spec methods; not effect actions. |
| Restrictions | No receivers/lambdas or combined default/injection. Required parameters precede optional ones; bind T, not Option. |

| Binding phase | Contract |
|---|---|
| Match | Positional order, then names; names may skip optional parameters. Reject duplicates, unknowns, positional-after-named and missing required inputs. |
| Supplied arguments | Once in source order; explicit values, including None, suppress their default/provider. |
| Omitted arguments | Once per call in declaration order, interleaving defaults/injection under callee restrictions/budgets and caller handlers. |
| Default expression | Declaration-site lexical bindings; may use preceding ordinary parameters, generics and permitted immutable values, not later parameters or self. Must fit T and be deterministic, action-free, non-suspending under declared bounds; unknown behavior is rejected. `![]` alone is insufficient. |
| Injection | Perform the [provider request](#72-scoped-injection); it may act, suspend or fail. No implicit cache or guarantee of freshest data. |
| Body entry | Bind/validate all parameters before receiver acquisition/body execution. Trailing callable `with` handles only body; call-site `f(...) with h` covers binding too. |
| Failure | Error, cancellation or finish during binding prevents body entry. Completed external actions are not rolled back. |
| Scope | Preserve owner/capture bounds; no suspension through an enclosing non-suspending exclusive access. |

Full-argument arrows/flow effect bounds describe the body and trailing handler.
Omitted-provider summaries apply only at omitted-argument calls; tool model
boundaries cover them too. First-class values require full inputs; wrap with
`() => f(x)` for omission. No currying, overloads, variadics or new stage arities.

Spec impls inherit defaults, injection labels/providers and parameter names;
reject additions/replacements or conflicting inherited contracts. Resolve before
impl entry. Actions retain pure defaults and complete payloads. Imports,
re-exports and frozen revisions preserve these contracts.
Tool schema projection and staged checks follow [Tool Rules](03-agents-tools-prompts-memory.md#2-tool).

## 5. Determinism Inference

Etas statically infers a determinism class for every flow. This is separate from effect inference:

| Class | Meaning |
|---|---|
| `Deterministic` | No reachable non-deterministic operation or callee; captured/local-state dependencies are checked separately. |
| `NonDeterministic` | May observe/change external state, infer, read time/randomness, access persistent memory, request approval, or call a non-deterministic computation. |

Examples:

```text
normalize_title       => Deterministic, effects = []
read_config           => NonDeterministic, requested actions include [Fs.read<ProjectRoot>], escaping effects = [Error<IOError>]
Draft                 => NonDeterministic, requested actions include [Agentic.infer<Writer.run, Draft>]
```

Agent/tool calls use their callee summaries; the declaration kind does not
itself force model inference or non-determinism. A pure runtime primitive may
also remain deterministic. Determinism does not replace effects, requested
actions, capture dependencies, or suspension/lifetime checks; optimization and
replay decisions must preserve all of them.

## 6. Program Entry Point

The executable entry is `main(args: Array<string>) -> i32`. It executes
directly and returns a process status; it does not return a deferred flow.
Return/effect inference uses the ordinary rules. Libraries need no `main`.

```etas
flow main(args: Array<string>) -> i32 {
    return 0;
}
```

Module loading evaluates only permitted read-only initializers, never arbitrary
top-level calls. Project entry selection is defined in
[Packages](15-package-management.md#132-entry-points).

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

Resolve a record head by name/kind, not capitalization:

| Form | Meaning |
|---|---|
| `{ fields }` / `Paper { fields }` | Construct; supply required fields except omitted nominal `@inject` fields. |
| `paper { title, year }` | Same record type; replace listed fields, retain others. Shorthand means `title = title, year = year`. |
| Agent head | A type constructs an instance; an instance cannot be updated this way. |

```etas
flow revise(paper: Paper, title: string, year: u32) -> Paper {
    return paper { title, year };
}
```

| Update Rule | Contract |
|---|---|
| Type | A value path must have a known ordinary record type; preserve its nominal identity/generic arguments or structural shape. Reject unknown/duplicate fields and type mismatches. |
| Evaluation | Snapshot the base once, then replacements once in source order. Names resolve lexically, not to old/new fields. |
| Retention | No implicit base mutation or initializer/provider reruns; `base {}` preserves its value. Failure yields no partial result; external effects are not rolled back. |
| Encapsulation | Full-reconstruction visibility (including retained fields), copy/affine and lifetime checks apply. No resource cloning; immutable backing may be shared. |
| Syntax | Parenthesize constructions/updates in control heads, e.g. `for author in (paper { title }).authors { ... }`; see [Grammar](09-syntax-principles.md#27-statements). |

Expand to hygienic temporaries and ordinary reconstruction, not runtime reflection.

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

Variants can also carry inline named fields, constructed and matched with
braces:

```etas
enum Response {
    Success(string),
    Failure { code: i32, message: string },
}

let response: Response = Response.Failure { code = 503, message = "unavailable" };
```

ETAS supports recursive and mutually recursive ADTs, including generic trees
and nested agent plans:

```etas
enum Plan {
    Task(string),
    Sequence(Array<Plan>),
    Parallel(Array<Plan>),
    Retry(Plan, u32),
}
```

These are ordinary immutable data values. `Parallel` and `Retry` here are enum
constructors, not execution primitives; a plan executor must explicitly
interpret them under normal effect, trace-spec, and limit checks. See
[Algebraic Data Types And Recursion](05-type-system-and-errors.md#112-algebraic-data-types-and-recursion)
for construction, exhaustive matching, recursive storage, and schema rules.

Common generic container types:

| Type | Meaning | Literal |
|---|---|---|
| `Array<T>` | Default immutable sequence, supports checked indexing and slicing | `[a, b, c]` |
| `List<T>` | Functional linked list, useful for `head`, `tail`, and `cons` | `[a; b; c]`, `a:: xs`, `[]` |
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
let ys = 0:: xs;        // List<i32>
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

Sequence-like containers use the compiler-known `Index` spec for checked
indexing APIs. The authoritative method tables, equality/hash/order bounds,
iteration order and complexity contracts are in
[Collections And Key Contracts](19-standard-library.md#9-collections-and-key-contracts).
Map keys and set elements require `Hash`, which includes the standard `Eq`
contract. Their literals do not bypass those bounds. Cached lengths and storage
metadata are private observations; users cannot assign them to forge an
inconsistent container or slice.

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

Slicing is supported for `Array<T>`, `Slice<T>`, `bytes`, and `Range<I>`. It is not positional on `Map`, `Set`, `Queue`, `Stack`, or `PriorityQueue`. Generic `string` slicing is intentionally not defined here because Unicode character offsets and byte offsets have different safety and performance properties; string slicing uses explicit APIs such as `std.text.char_slice(...)` or `std.text.byte_slice(...)`, specified in [Strings And Text APIs](19-standard-library.md#16-access-and-slicing).

Negative indexes and negative slice bounds are not supported:

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

Copies have independent observable value semantics; physical deep copying is
not required, and storage sharing cannot expose accidental shared mutation.

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

Persistent mutation uses explicit `Store<K, V>` APIs. Local updating receivers
follow their separate writeback rules; neither changes collection value semantics.

Bulk construction uses the specified `from_iter`, `from_entries` and iterator
consumer APIs rather than an unspecified mutable builder protocol. Their
implementations may reuse private buffers or immutable structure while
preserving the same value semantics. See
[Iteration And Effectful Consumption](19-standard-library.md#10-iteration-and-effectful-consumption)
for pure lazy adapters and callbacks whose effects occur during consumption.

### 7.2 Scoped Injection

For an omitted field or parameter `name: T`, `@inject` requests `Inject<T>.get("name")`.
`@inject(Action)` instead uses a declaration-site-resolved, zero-argument action
returning an assignable value. Neither executes at declaration time. Infer T
from ordinary type arguments/context, never by searching installed providers.

| Consumer | Timing |
|---|---|
| Stored record/agent field | Once per construction when omitted; save the value, not its provider. A saved agent reference is not a snapshot of that agent's state. |
| Named-callable parameter | Once per omitted argument per call under section 4.1; no instance-field update. |
| Explicit `perform` | Only when execution reaches it; branches/loops may request again. |

| Dependency arm | Expansion |
|---|---|
| `name => expr;` | `Inject<T>.get("name") => { resume expr; }`; infer T from expr. |
| `name: T => expr;` | Same request, with explicit expected T, including existential packing. |
| `name,` or final `name` | `name => name;`; the right name is a lexical value, not a new binding. |
| `T => expr;` | `Inject<T>.get(_) => { resume expr; }`; type-wide fallback. |

Both `handler { ... }` and anonymous `with { ... }` accept these arms, separated
by comma/semicolon; only final shorthand may omit one. A type-name head selects
the type form; other identifiers label fields, with `name: T` forcing that form.
Reject other non-type heads. Provider expressions run per request and implicitly
resume like ordinary action tails, never implicitly finish. No arm-level `return`.

| Rule | Contract |
|---|---|
| Target | One annotation per nominal record/agent field or eligible parameter. Reject enum payloads, anonymous-record fields, receivers, lambdas, action parameters and non-action annotation arguments. |
| Matching | Match names/normalized types; aliases agree, nominal types differ. Nearest handler wins, then exact name before type-wide arm. Reject duplicate keys, including expanded sugar; no implementation search. |
| Labels | Names identify demands, not owners. Same-name/same-type fields and parameters use the same matching rules; use nested handlers or named actions to distinguish them. Renaming changes the contract. |
| Construction | Explicit fields/shorthand suppress their providers. Validate missing/duplicate/unknown fields, evaluate supplied values in source order, then omitted providers in declaration order. No implicit cache or cycle resolution. |
| Effects | Retain demand names/types and provider behavior. Inject has no default provider; uncovered requests escape. Unknown labels need type-wide coverage. Handling never grants authority or certifies an injected identity. |
| Completion | Publish fully initialized values only. Error, cancellation or `finish` abandons construction and cleans up new agent scopes, not external effects. |
| Lifetime | Capture/owner/alias bounds remain; no adoption, extended lifetime or new permission. Later reads/copies/patterns/equality/serialization never rerun providers. |
| Decoding | Required schema fields stay required; derived decoding never invokes providers. Custom constructors/decoders retain checked effects. |

## 8. Encapsulation and Methods

Private fields and methods protect record invariants.

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

`impl T` defines ordinary `flow` methods. Plain `self` reads a value; updates
return a new value or use restricted [`var self`](#32-restricted-updating-receivers).
Private access is module-scoped. Target kind restricts members: ordinary types
accept flows, effects accept actions, agents accept checked methods/member tools.
These kinds cannot be mixed; active receiver rules remain in
[Agents](03-agents-tools-prompts-memory.md#13-operational-intuition).

```etas
impl Session {
    flow pause(self, interrupt: Interrupt) -> Result<Session, SessionError> {
        if !valid_transition(self.state, Waiting) {
            return Err(InvalidStateTransition { from = self.state, to = Waiting });
        }

        return Ok(self {
            state = Waiting,
            current_interrupt = Some(interrupt),
            events = self.events.push(InterruptRaised { id = self.id }),
        });
    }
}
```

Validation stays close to the data it protects. `pause` returns a new session;
ignoring that return value does not update the caller. Persisting a transition
requires an explicit typed-store operation under the usual version checks.

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

@limits([Iterations(20)])
for paper in papers {
    process(paper);
}

@limits([Iterations(3)])
while needs_revision(draft) {
    draft = revise(draft);
}
```

In pattern position, `true`, `false`, integer literals, string literals, and char literals are constant matches rather than variable bindings. A non-deterministic loop whose termination cannot be statically proven requires an enforceable `@limits(...)` resource contract; an annotation inside its body does not bound the whole loop.

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

`etas.toml` describes package identity, dependencies, import roots and targets;
`etas.lock` fixes resolutions. Neither file grants runtime authority.
See [Package Manifest](15-package-management.md#4-package-manifest).

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

Resolved item identity includes its package, logical module and item, not merely
its physical source path. Dependency versions, lockfiles and exported contracts
are defined in [Packages](15-package-management.md).

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
An ordinary record with private fields cannot be constructed or record-updated
outside its defining module; use public factories/methods. Injection is no bypass.

Methods inside `impl` blocks default to `private`. Public methods form the
callable API of the type:

```etas
impl Session {
    public flow start(var self) -> Result<unit, SessionError> {
        return self.transit_to(SessionState.Started);
    }

    private flow transit_to(var self, to: SessionState) -> Result<unit, SessionError> {
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
