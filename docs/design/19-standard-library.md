# Standard Library

This chapter specifies public standard-library APIs. It follows the
[five-layer model](01-overview.md#53-feature-map-by-layer): ordinary data
algorithms are Etas flows, selected standard contracts have explicit compiler
integration, and platform operations use the declared runtime substrate.
An API specified here is a design target, not a claim that every backend or
the current interpreter already implements it.

Console I/O and runtime substrate contracts remain in
[Standard IO](03-agents-tools-prompts-memory.md#24-standard-io) and
[Standard Substrate API Shape](03-agents-tools-prompts-memory.md#25-standard-substrate-api-shape).

**API Index**

Tables specify required public contracts, not implementation-status claims.
Receiver methods omit the implicit `self` argument in API rows. Generic owner
parameters come from the receiver or expected result type; method parameters
such as `<U, effect E>` are shown explicitly. A row grouping several names
assigns its contract to each name unless the behavior column distinguishes them.

| Area | Owning Module | API Reference |
|---|---|---|
| Text and Unicode | `std.text`, `std.text.unicode`, `std.char`, `std.codec.text` | [Section 1](#1-strings-and-text-apis) |
| Tasks and scheduling | `std.task`, `std.task.scheduler` | [Section 2](#2-task-and-scheduling-apis) |
| JSON | `std.json` | [Section 3](#3-json-values) |
| Model steps and native tool dispatch | `std.agent`, `std.agent.conversation`, `std.tool` | [Section 4](#4-single-step-agent-interaction) |
| Numeric helpers | `std.math` | [Section 5](#5-numeric-helpers) |
| Agent groups | `std.agent.group` | [Section 6](#6-round-robin-groups) |
| Logical trace stages | `std.runtime.trace` | [Section 7](#7-virtual-trace-stages) |
| Sanitization and release | `std.security.declassify` | [Section 8](#8-sanitization-and-declassification) |
| Containers, equality, hashing and ordering | `std.collections` | [Section 10](#10-collections-and-key-contracts) |
| Iteration and callback consumers | `std.collections` | [Section 11](#11-iteration-and-effectful-consumption) |
| Sequence algorithms | `std.collections` | [Section 12](#12-generic-sequence-algorithms) |
| Optional values and results | `std.option`, `std.result` | [Section 13](#13-option-and-result-apis) |

## 1. Strings And Text APIs

### 1.1 Modules And Value Semantics

| Module | Responsibility |
|---|---|
| `std.text` | String queries, traversal, search, splitting, slicing, conversion, editing, and assembly |
| `std.text.unicode` | Grapheme segmentation, normalization, and Unicode case folding |
| `std.char` | Unicode scalar properties and scalar/integer conversion |
| `std.codec.text` | Encoding and decoding between text and bytes |

`string` is an immutable UTF-8 value. `char` is a Unicode scalar value, not a
byte or necessarily a complete user-perceived character. Updating text returns
a new string and cannot mutate an existing value observed elsewhere.
Implementations may share immutable storage; passing a string does not require
eager copying. These APIs introduce no new keywords, implicit mutable receiver,
or external authority actions.

Except where an error effect is explicitly shown, the APIs below have an empty
escaping effect row. Text iteration and transformation do not perform console,
filesystem, network, or model operations. This describes their checked library
contracts; it does not change the source rule that omitted effect annotations
are inferred from a flow body.

### 1.2 Positions And Ranges

`len` and all search results use UTF-8 byte positions. `char_get` and
`char_slice` take scalar ordinals; `grapheme_slice` takes grapheme ordinals.
The positions emitted by `char_indices` and `grapheme_indices` are byte
positions, so they can be passed to byte-slicing APIs. A position from one
coordinate system must not be silently reinterpreted as another. Generic
`s[i]` and direct range slicing on `string` are not defined.

Position-taking APIs accept `I ~ Index`, following the existing
[indexing rules](02-general-programming-constructs.md#21-numeric-conversion-and-indexing).
This does not add general implicit integer conversion. Negative positions and
positions that cannot fit the supported size range are invalid inputs.
Optional accessors return `None`; checked accessors raise `Error<TextError>`.

Range parameters have type `Range<I>` with `I ~ Index`. They use the same
boundary normalization as collection slicing: `[i, j)` selects positions from
`i` through `j - 1`, while `(i, j]` selects positions from `i + 1` through `j`.
The normalized interval must be ordered and within bounds. Empty intervals at
valid positions are allowed. Byte slicing additionally requires both normalized
endpoints to be UTF-8 scalar boundaries. A start endpoint greater than the end
endpoint is invalid, not a request to reverse the string.

For example, the scalar U+00E9 occupies two UTF-8 bytes: its byte boundaries are
`0` and `2`, but its scalar interval is `[0, 1)`. A byte position of `1` is not
a valid string-slice boundary.

### 1.3 Length And Traversal

Signatures in the tables are API signatures; names without a module prefix
belong to the module named by the subsection. A row listing several names
assigns the displayed signature to each name.

| `std.text` API | Return Type | Behavior |
|---|---|---|
| `len(s: string)` | `usize` | UTF-8 byte length; constant-time |
| `char_count(s: string)` | `usize` | Number of Unicode scalars; scans the string |
| `is_empty(s: string)` | `bool` | Whether byte length is zero; constant-time |
| `is_ascii(s: string)` | `bool` | Whether all scalars are ASCII; true for an empty string |
| `chars(s: string)` | `Iter<char>` | Scalars in source order |
| `char_indices(s: string)` | `Iter<(usize, char)>` | Each scalar paired with its starting byte position |
| `lines(s: string)` | `Iter<string>` | Lines split at LF or CRLF, without the line terminator |

`Iter<T>` is a standard-library lazy sequence contract, not a source keyword
or a continuation type. The iterators specified here are finite, retain the
source as needed, and produce elements on demand when consumed by `for`.
They must not first materialize an `Array<T>`. Reusing the same iterator value
must not expose a shared mutable cursor to another observer; loop-local cursor
management is an implementation detail. Its integration with `for` belongs to
the standard iteration contract, not to special syntax for each text function.

For `lines`, an empty input produces no lines. A trailing line terminator does
not produce an additional empty line, but internal empty lines are preserved.
A standalone CR is ordinary content, not a terminator. `split` has different
trailing-empty behavior, defined below.

### 1.4 Search And Matching

| `std.text` API | Return Type | Behavior |
|---|---|---|
| `contains(s: string, pattern: string)` | `bool` | Whether an exact match exists |
| `starts_with(s: string, pattern: string)` | `bool` | Exact prefix test |
| `ends_with(s: string, pattern: string)` | `bool` | Exact suffix test |
| `find(s: string, pattern: string)` | `Option<usize>` | Starting byte position of the first match |
| `rfind(s: string, pattern: string)` | `Option<usize>` | Starting byte position of the last match |
| `find_from<I ~ Index>(s: string, pattern: string, start: I)` | `Option<usize> ![Error<TextError>]` | First match at or after `start`; validates the starting byte boundary |
| `find_all(s: string, pattern: string)` | `Iter<(usize, usize)>` | Non-overlapping half-open byte intervals, in ascending order |
| `count(s: string, pattern: string)` | `usize` | Number of matches using the `find_all` rule |

Matching is exact and case-sensitive. It does not implicitly normalize text,
case-fold it, interpret regular expressions, or use the platform locale.
`find_from` returns positions in the original string, not relative to `start`.
A valid search with no match returns `None`, not an error.

An empty pattern matches every scalar boundary, including `0` and `len(s)`.
Thus `find(s, "")` is `Some(0)`, `rfind(s, "")` is `Some(len(s))`, and
`find_all(s, "")` advances between successive scalar boundaries rather than
repeating the same zero-length match. On empty input there is one such match.
These rules also govern empty-pattern splitting and replacement.

### 1.5 Splitting

| `std.text` API | Return Type | Behavior |
|---|---|---|
| `split(s: string, separator: string)` | `Array<string>` | Eager exact splitting; preserves empty segments |
| `split_iter(s: string, separator: string)` | `Iter<string>` | Lazy form of the same operation |
| `split_once(s: string, separator: string)` | `Option<(string, string)>` | Split around the first match; excludes the separator |
| `rsplit_once(s: string, separator: string)` | `Option<(string, string)>` | Split around the last match; excludes the separator |
| `splitn(s: string, separator: string, n: usize)` | `Array<string>` | At most `n` segments; the last contains the unsplit remainder |
| `split_whitespace(s: string)` | `Iter<string>` | Split at runs of Unicode White_Space; omits empty segments |

Exact splitting uses non-overlapping matches and preserves leading, trailing,
and adjacent-separator empty segments. `split("", ",")` returns `[""]`.
`split("ab", "")` returns `["", "a", "b", ""]`.
`splitn(..., 0)` returns an empty array and `splitn(s, separator, 1)` returns
`[s]`. When the separator is absent, the once variants return `None`.

### 1.6 Access And Slicing

| `std.text` API | Return Type | Behavior |
|---|---|---|
| `char_get<I ~ Index>(s: string, index: I)` | `Option<char>` | Access by scalar ordinal; invalid index returns `None` |
| `byte_slice<I ~ Index>(s: string, range: Range<I>)` | `string ![Error<TextError>]` | Checked UTF-8 byte interval |
| `char_slice<I ~ Index>(s: string, range: Range<I>)` | `string ![Error<TextError>]` | Checked scalar interval |
| `get_byte_slice<I ~ Index>(s: string, range: Range<I>)` | `Option<string>` | Optional form of `byte_slice` |
| `get_char_slice<I ~ Index>(s: string, range: Range<I>)` | `Option<string>` | Optional form of `char_slice` |
| `is_char_boundary<I ~ Index>(s: string, index: I)` | `bool` | Whether a byte position is a scalar boundary; invalid positions return false |

Byte positions `0` and `len(s)` are valid boundaries. Scalar access at
`char_count(s)` is out of bounds, but an empty scalar slice at that position is
valid. Optional slice APIs return `None` on invalid ranges or byte boundaries;
they do not silently clamp, round, or replace input.

### 1.7 Conversion, Editing, And Assembly

| `std.text` API | Return Type | Behavior |
|---|---|---|
| `trim(s: string)`, `trim_start(s: string)`, `trim_end(s: string)` | `string` | Remove Unicode White_Space from the indicated ends |
| `strip_prefix(s: string, pattern: string)`, `strip_suffix(s: string, pattern: string)` | `Option<string>` | Remove one matching prefix/suffix; `None` if absent |
| `lowercase(s: string)`, `uppercase(s: string)` | `string` | Unicode default case conversion, independent of locale |
| `ascii_lowercase(s: string)`, `ascii_uppercase(s: string)` | `string` | Convert ASCII letters only; preserve other scalars |
| `eq_ignore_ascii_case(s: string, other: string)` | `bool` | Equality ignoring only ASCII letter case |
| `replace(s: string, pattern: string, replacement: string)` | `string` | Replace all non-overlapping matches |
| `replace_n(s: string, pattern: string, replacement: string, n: usize)` | `string` | Replace the first `n` non-overlapping matches; zero leaves input unchanged |
| `replace_range<I ~ Index>(s: string, range: Range<I>, replacement: string)` | `string ![Error<TextError>]` | Replace a checked byte interval |
| `concat(parts: Array<string>)` | `string` | Concatenate all parts in order |
| `join(parts: Array<string>, separator: string)` | `string` | Insert the separator between adjacent parts |
| `repeat(s: string, n: usize)` | `string ![Error<TextError>]` | Repeat the string; check output-length arithmetic overflow |
| `pad_start(s: string, width: usize, fill: char)`, `pad_end(s: string, width: usize, fill: char)` | `string` | Pad to a minimum scalar count; never truncate |
| `reverse_chars(s: string)` | `string` | Reverse scalar order, not grapheme order |

An empty replacement interval inserts text. Replacing an interval with `""`
deletes it. All editing APIs leave the original string unchanged.
`replace("ab", "", "-")` returns `"-a-b-"`, using the finite-boundary rule.
`concat([])` and `join([], separator)` return `""`; `repeat(s, 0)` also returns
`""`. An empty string can be repeated without iterating `n` times.

Unicode case conversion can change scalar count and byte length. Padding width
is neither grapheme count nor terminal display width. Reversing scalars may
separate combining marks and emoji sequences; use `reverse_graphemes` when
grapheme-preserving units are required.

### 1.8 Parsing And Formatting

The following families are concrete exported flows, not new generic-name
syntax. For each numeric type `N` listed below, std exports both `parse_N` and
`from_N`, for example `parse_i32` and `from_i32`.

| `std.text` API Family | Return Type | Behavior |
|---|---|---|
| `parse_N(s: string)`, for `N = i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64` | `Result<N, TextError>` | Parse a complete locale-independent numeric string; invalid input and overflow are explicit values |
| `from_N(value: N)`, for the same numeric types | `string` | Locale-independent numeric representation |
| `parse_bool(s: string)` | `Result<bool, TextError>` | Accept exactly `"true"` or `"false"` |
| `from_bool(value: bool)` | `string` | Produce `"true"` or `"false"` |
| `from_char(value: char)` | `string` | String containing exactly that scalar |
| `format(template: string, args: Array<string>)` | `Result<string, TextError>` | Positional substitution using already-rendered string arguments |

Parsing does not implicitly trim input, accept locale-specific separators, or
ignore a trailing suffix. Integer parsers use decimal digits and an optional
sign; unsigned parsers reject a minus sign. Floating-point parsers additionally
accept decimal fractions and exponents. Numeric rendering must be stable for
the selected toolchain and round-trip finite floating-point values through the
corresponding parser. This API does not introduce a generic `ToString` spec.

Formatting supports zero-based placeholders such as `{0}` and `{1}`, with
`{{` and `}}` for literal braces. Placeholders may repeat. Unmatched braces,
unsupported placeholder syntax, and missing argument indices return
`Err(InvalidFormat)`. Unused supplied arguments are allowed. Formatting does
not evaluate source expressions, inspect object fields, or invoke implicit
user callbacks. It is a library function, not string-interpolation syntax.

### 1.9 Unicode Operations

| `std.text.unicode` API | Return Type | Behavior |
|---|---|---|
| `graphemes(s: string)` | `Iter<string>` | Extended grapheme clusters in source order |
| `grapheme_indices(s: string)` | `Iter<(usize, string)>` | Each cluster paired with its starting byte position |
| `grapheme_count(s: string)` | `usize` | Number of extended grapheme clusters |
| `grapheme_slice<I ~ Index>(s: string, range: Range<I>)` | `string ![Error<TextError>]` | Checked grapheme interval |
| `reverse_graphemes(s: string)` | `string` | Concatenate the input clusters in reverse order |
| `normalize(s: string, form: Normalization)` | `string` | Apply NFC, NFD, NFKC, or NFKD |
| `is_normalized(s: string, form: Normalization)` | `bool` | Test the selected normalization form |
| `casefold(s: string)` | `string` | Unicode default full case folding, without implicit normalization |

`Normalization` is a standard enum with variants `NFC`, `NFD`, `NFKC`, and
`NFKD`. Segmentation follows the default extended grapheme cluster rules of
[Unicode UAX #29](https://www.unicode.org/reports/tr29/); normalization follows
[Unicode UAX #15](https://www.unicode.org/reports/tr15/). Unicode data and
algorithm versions must be fixed by the toolchain/library release and recorded
in its compatibility metadata, rather than selected from the host environment.

String equality and ordinary matching remain exact UTF-8 content comparison.
Canonical equivalence and case-insensitive Unicode matching require explicit
normalization or folding. Offsets produced from transformed text belong to that
transformed value, not automatically to the original string.

### 1.10 Character Properties And Encoding

| Module | API | Return Type / Behavior |
|---|---|---|
| `std.char` | `c.is_whitespace()` | `bool`; Unicode White_Space property |
| `std.char` | `c.is_alphabetic()` | `bool`; Unicode Alphabetic property |
| `std.char` | `c.is_alphanumeric()` | `bool`; Alphabetic or Unicode number category |
| `std.char` | `c.is_control()` | `bool`; Unicode General_Category Cc |
| `std.char` | `c.is_ascii_digit()` | `bool`; ASCII `0` through `9` only |
| `std.char` | `c.to_u32()` | `u32`; scalar numeric value |
| `std.char` | `char.from_u32(value)` | `Option<char>`; reject surrogates and values beyond U+10FFFF |
| `std.codec.text` | `utf8_encode(s: string)` | `bytes`; UTF-8 encoding with no external action |
| `std.codec.text` | `utf8_decode(body: bytes, malformed: MalformedInput)` | `Result<string, TextCodecError>`; explicit rejection or U+FFFD replacement for malformed input |

Character properties use the same pinned Unicode data as the Unicode module.
Encoding and decoding reuse the existing text-codec contracts rather than
inventing text-specific network or file effects. Lossy decoding must be
explicitly selected; a malformed sequence must never create an invalid
`string`. See [pure substrate helpers](06-effect-system-and-inference.md#11-minimal-standard-actions).

### 1.11 Errors And Performance

`TextError` is an ordinary standard-library ADT exported by `std.text`:

```etas
public enum TextError {
    InvalidIndex,
    InvalidRange,
    NotCharBoundary,
    ParseFailure,
    NumericOverflow,
    LengthOverflow,
    InvalidFormat,
}
```

An invalid position is `InvalidIndex`; invalid or out-of-bounds normalized
intervals are `InvalidRange`; an in-bounds byte endpoint inside a scalar is
`NotCharBoundary`. Parsing failures and unrepresentable numeric results use
`ParseFailure` and `NumericOverflow`. A checked output-size arithmetic overflow
in `repeat` uses `LengthOverflow`.

Checked APIs expose `Error<TextError>`; optional accessors expose `Option`, and
parsing/formatting expose `Result`. An expected failed search is not an error.
These are distinct, deliberate API contracts. Postfix `?` converts a checked
text error effect to a value-level result; it neither propagates nor unwraps an
existing `Result`:

```etas
let piece: Result<string, TextError> = std.text.byte_slice("Etas", [1, 3))?;
let number: Result<i32, TextError> = std.text.parse_i32("42");
```

This chapter does not redefine the language-wide behavior of allocation failure
or resource-limit exhaustion as a recoverable `TextError`. All allocating APIs
must use checked size arithmetic and obey the applicable execution limits;
unrepresentable allocations must not wrap lengths or corrupt storage.

Performance contracts and implementation freedom are:

- Byte length and emptiness are constant-time. Scalar/grapheme counts and
  ordinal-to-byte conversion may scan the input; no cached index is assumed.
- Sequential scalar traversal is linear in input bytes. Repeated independent
  `char_get` calls can be quadratic and should not replace a single iterator.
- `concat` and `join` should take time proportional to the input part count
  plus output bytes, using checked total-size computation and bulk assembly
  rather than repeatedly copying an accumulated prefix.
- `split_iter`, `chars`, `char_indices`, and grapheme iterators must not allocate
  a complete result array. Segment values may still require allocation.
- String slices, trimming, and split segments may share immutable backing
  storage. Zero-copy behavior is not guaranteed, and a retained small result
  may keep a larger source allocation alive. Observable value semantics remain
  unchanged whichever representation is selected.

No mutable string builder, receiver-update sugar, regular-expression syntax,
or general interpolation syntax is added by this API chapter.

### 1.12 Example

```etas
import std.text as text;
import std.text.TextError;

flow NormalizeTags(input: string) -> string {
    var tags: Array<string> = [];

    for part in text.split_iter(input, ",") {
        let tag = text.lowercase(text.trim(part));
        if !text.is_empty(tag) {
            tags = tags.push(tag);
        }
    }

    return text.join(tags, " | ");
}

flow Prefix(input: string) -> Result<string, TextError> {
    return text.char_slice(input, [0, 3))?;
}
```

## 2. Task And Scheduling APIs

These APIs specify structured task execution and handler-based scheduling.
Their scope, cancellation, and continuation semantics are defined in
[Concurrency](16-concurrency.md#121-handler-based-scheduling).

### 2.1 Standard Types And Actions

`std.task` exports `Task<T, E>`, where `T` is the result type and `E` is an
effect-row parameter declared with `effect E`. Ordinary calls infer both.
A task also retains its owning scope, requested-action/trace contract, and
determinism information as checked metadata. Its result type alone cannot
erase those facts. Package interfaces and helper calls must preserve them.
Task handles may be shared inside their valid lifetime; repeated awaits observe
the same completed result and do not execute the job again.

The callback to `std.task.run` receives an opaque, scope-bound `SchedulerHandle`
from `std.task.scheduler`. This running handle is distinct from the reusable
implementation value selected by the caller. The handle's owner is inferred,
not chosen by a runtime string. Neither tasks
nor scheduler handles may escape their owning execution scope, including via
returned closures or aggregate values. They are not model-schema values or
ordinary serializable checkpoint data. Managed execution is supplied by the
program entry and concurrency combinators; `run` creates an explicit nested
scope for code that needs its scheduler handle.

The standard control effect is owned by `std.task.Async`:

```etas
module std.task;

effect Async {
    action yield() -> unit;
    action await<T, effect E>(task: Task<T, E>) -> T;
}
```

`Async` is a standard scheduling effect, not a new broad external-authority
root. Its operations require a valid managed scope, not permission to perform
network, file, model, or secret operations. An import alias does not create a
different action identity. Omitting explicit type arguments in a handler arm
matches the generic action; the arm must work for its matched type parameters.

### 2.2 Public Operations

| API | Result | Required Contract |
|---|---|---|
| `std.task.run(body, implementation = DefaultScheduler)` | The callback result `R` | Create fresh scope-local policy/queue state for the selected implementation, install the standard async handlers, and call `body(scheduler)`. Wait for owned children and cleanup before returning. No owned task, continuation, running scheduler, or handler capturing it may escape in `R`. |
| `std.task.start<T, effect E>(job: () -> T ![E])` | `Task<T, E>` | Schedule a child in the active scope and return its handle without waiting for completion. Preserve the job's effects, requested actions, trace behavior, and inherited context. |
| `std.task.await<T, effect E>(task: Task<T, E>)` | `T` | Request `Async.await(task)`. Deliver its result or propagate failure through the task protocol; waiting must not block the whole scheduler thread. |
| `scheduler.requeue(k, value)` | `never` | Consume a live continuation owned by this scheduler, attach a value of its action's result type, queue it as runnable, and transfer control out of the current arm. |
| `scheduler.park(task, k)` | `never` | Consume the continuation from the matching `Async.await(task)` request and register a completion dependency. Deliver success, failure, or cancellation without losing a concurrent completion event. |
| `std.task.scheduler.handlers(scheduler: SchedulerHandle)` | `![Async => []]` | Construct the standard scope-bound `yield`/`await` handler adapter. Construction is pure; applying it uses checked transfer primitives and the policy bound to this handle. |

`run` and `start` are compiler-recognized std contracts over task-execution
primitives. They need no new source keyword or exposed `spawn` syntax. A
same-named user flow cannot acquire their lifetime or scheduling obligations
by spelling alone. The scheduler is an ordinary bound value with restricted
operations; `requeue` and `park` are registered control-transfer primitives,
not arbitrary callbacks made special by their method names. Scheduling policy
is Etas code implementing the `Scheduler` spec below; the running handle is
created by `run`, not by importing a live singleton.

`requeue` normally handles `Async.yield` with the unit value `()`. It can only
resume the original suspended computation with a correctly typed value; it
cannot move that computation into the calling task's identity or authority.
`park` handles both already-completed and pending tasks, using atomic
completion/registration semantics. A pending task is not runnable just because
its waiter yielded. Waiting and runnable queues have different purposes.

The `never` return describes the current arm's control path. Successful
transfer returns control to the owning scheduler, not to the next statement
in the arm; it does not abort the process. The scheduler later resumes or
cancels the original task. A plain queue insertion returning `unit` cannot
replace either operation. Operations must validate ownership and input types
before accepting the transfer. A failed scheduler invariant cannot silently
drop a live continuation; it fails and unwinds the affected managed scope.

### 2.3 Effects, Errors, And Default Handling

Starting work contributes its `E` and full behavioral summary to the enclosing
task scope immediately, even if its handle is never awaited. `await` preserves
the declared failure obligations in `E`; it cannot forge a value of `T` for a
failed task. Handling `Async.await` or `Async.yield` removes covered control
actions from escaping rows, not the effects or requested actions of child work.
The default handling of these control actions remains recorded in metadata.
No task API provides an implicit catch-all for `Error<_>` or an ad-hoc error
union. The postfix `?` retains its existing single-Error effect-to-Result rule.

The default `run` scope is fail-fast. The first unhandled failure initiates
sibling cancellation and propagates its original typed error after cleanup;
it is not converted into a successful task result or a default `T`. A task's
inherited handlers may recover before failure becomes unhandled. Asynchronous
error delivery restores the appropriate task/handler context rather than
resuming a continuation with a value of the wrong type. Scope cancellation
cannot be cleared by catching an incidental I/O cancellation error.

Default handlers cover ordinary task use, so business code need not handle
`Async` manually. A custom scoped handler may replace scheduling policy without
changing child lifetime, failure propagation, authority, budgets, or trace
requirements. Runtime adapters for models, streams, and other waiting operations
must use non-blocking completion or managed blocking offload. Calling `start`
or performing an action does not automatically make a blocking host function
asynchronous. Runtime-owned waits remain tied to the same task scope; custom
handlers do not gain permission to forge completion of protected host work.

See the [complete scheduling example](16-concurrency.md#126-complete-example)
for `as k`, `start`, `await`, and handler transfer used together.

### 2.4 Scheduler Spec And Default Value

`std.task.scheduler.Scheduler` is a behavioral type spec, not the type of a
running scheduler. An implementation is a reusable value whose ordinary local
state can evolve through `var self` during one run:

```etas
module std.task.scheduler;

public type SchedulerHandle;

public type ReadyTask = {
    id: u64,
    enqueue_order: u64,
}

public spec Scheduler {
    flow select(var self, ready: Array<ReadyTask>) -> usize ![];
}

public type FifoScheduler = {};

impl FifoScheduler ~ Scheduler {
    flow select(var self, ready: Array<ReadyTask>) -> usize ![] {
        return 0;
    }
}

public let DefaultScheduler = FifoScheduler {};
```

`ReadyTask` is a copyable scheduling snapshot, not `Task<T, E>`, a continuation,
or authority to execute work. Its id is scoped to the run. Every runnable-queue
entry gets a new enqueue order; snapshots are sorted oldest first. `select`
receives a non-empty snapshot and returns its chosen index. A policy may retain
ordinary value-semantic counters/configuration through `var self`; it never
owns a continuation through this record or stores one in an ordinary array.

Implementations must have empty escaping/requested-action summaries and be
deterministic and proven non-suspending. These are checked obligations of the
resolved standard spec, not facts inferred from `![]` alone or assertions a user
annotation can grant. Unknown imports/callbacks cannot satisfy the proof.
Selection runs under the existing receiver exclusivity and budget rules.

The `std.task` implementation imports `DefaultScheduler`, `Scheduler`, and
`SchedulerHandle` from `std.task.scheduler`. Its public call shape is:

```text
run<R, effect E>(
    body: SchedulerHandle -> R ![E],
    implementation: ? ~ Scheduler = DefaultScheduler
) -> R
```

This uses an existing object-safe existential bound, not general subtyping or
an assumed default generic type parameter. Defaults follow the ordinary
[default-parameter rules](02-general-programming-constructs.md#41-default-parameters).
Thus `task.run(body)` needs no caller import of `DefaultScheduler`;
`task.run(body, CustomScheduler)` or
`task.run(body, implementation = CustomScheduler)` explicitly selects another
value satisfying `Scheduler`. Callers import their custom implementation, not
a live running handle. The result's escaping row is inferred after async
handling, while all child-work summaries remain preserved.

Each `run` creates a fresh local copy of the implementation value and fresh
owned queue/wait state. Updating that copy never changes the supplied value or
another run's copy. Implementations cannot carry live task/continuation/scheduler
state across runs or smuggle it into a global constant. A factory flow may
construct a configuration value before passing it to `run`; no separate
factory protocol is required. Runtime worker pools and I/O drivers may be
shared physically without sharing logical scope ownership or queue state.
Combinators use the active run's selected policy for their owned child groups.
An explicit nested `run` creates a new policy instance; its omitted argument
uses `DefaultScheduler` according to normal default-parameter binding.

### 2.5 Standard Handlers And Runtime Responsibilities

The standard handler adapter is expressible in Etas:

```etas
import std.task.Async;

public flow handlers(scope: SchedulerHandle) -> ![Async => []] {
    return handler {
        Async.yield() as k => {
            scope.requeue(k, ());
        }

        Async.await(job) as k => {
            scope.park(job, k);
        }
    };
}
```

The adapter handles the task protocol; the selected implementation supplies the
ready-selection policy. A custom policy therefore need not reimplement task
completion, cancellation, or lost-wakeup prevention. An explicit inner handler
may still customize `Async` handling through the same checked transfer APIs.
It cannot detach work, forge results, or bypass the selected scope's ownership.
The handler value captures the handle's lifetime and cannot escape that owner.

| Layer | Responsibility |
|---|---|
| `Scheduler` implementation | Select a ready entry and maintain private value-semantic policy state. It may not suspend, request external authority, or operate on raw continuations. |
| `std.task` / `std.task.scheduler` | Own the default value and common handlers; establish the selected policy's run, install handlers, coordinate structured completion/failure, and invoke checked execution primitives. |
| Runtime primitives | Own live frames/continuations and protected queue/wait registrations; validate owner, readiness, result types, one-shot transfer, wakeup/cancellation races, and cleanup. Supply non-blocking I/O completion or managed offload. |
| Application | Choose an implementation when needed; write ordinary jobs and optional scoped handlers. It does not import active global queue state. |

`run` resolves the implementation and binds its scope-local copy before
executing the callback. New tasks, explicit yields, and completed waits enqueue
runnable work. For each dispatch, the coordinator obtains a non-empty snapshot,
calls the local policy's `select`, and dispatches the chosen still-live entry
through Runtime primitives. A cancellation/completion race may invalidate an
entry after the snapshot; refresh readiness rather than resume a dead task.
An out-of-range index is a scheduler contract violation: fail and unwind the
affected scope rather than execute an arbitrary task or silently choose another.
No-ready-work with outstanding waits parks the coordinator for completion;
it is not a busy-yield loop. Scope closure drains or cancels owned work before
releasing live handlers and continuations.

The default FIFO policy selects index zero. New, yielded, and woken tasks enter
the tail, so older runnable work is chosen first. This is cooperative fairness,
not a guarantee of CPU preemption or bounded model/provider latency. Existing
limits and cancellation still apply. Custom implementations can choose another
ready order or maintain local policy state; this contract does not claim to
replace OS scheduling, I/O drivers, or protected task-execution primitives.

## 3. JSON Values

`std.json` owns the shared JSON value type and deterministic codecs. EDK JSON
helpers use or re-export this type; they must not introduce a second incompatible
JSON tree solely for HTTP or model-tool payloads.

The public data model is a transparent recursive ADT:

```etas
module std.json;

public enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(string),
    Array(Array<JsonValue>),
    Object(Map<string, JsonValue>),
}

public enum JsonError {
    InvalidJson(string),
}
```

The payload types `Array<T>` and `Map<K, V>` above resolve to the standard
collection types, not to the same-named JSON variants. Applications may construct
and match these variants directly. JSON values retain ordinary immutable value
semantics. Object keys are strings; array order is significant, while object
member order is not part of this data model. Parsing duplicate object keys keeps
the last value. Source spelling, whitespace, and duplicate occurrences are not
preserved as a lossless syntax tree.

| API | Return Type | Contract |
|---|---|---|
| `std.json.parse(text: string)` | `Result<JsonValue, JsonError>` | Parse one complete JSON value, allowing surrounding JSON whitespace but no trailing non-whitespace content. Reject malformed input and numbers outside the supported finite numeric range. |
| `std.json.stringify(value: JsonValue)` | `Result<string, JsonError>` | Encode JSON without performing I/O. Reject non-finite numeric values. Object output order is not a canonical-signing contract. |

Both APIs have empty escaping effect rows. Codec failures are explicit `Err`
values, not `Error<JsonError>` actions; `?` is neither needed nor redefined for
them. Constructors cover all JSON value categories, but `Number(f64)` does not
promise arbitrary-precision decimals or exact large-integer round trips. Parsing
may round a representable numeric value. Neither encoding nor decoding preserves
its original numeric spelling.

## 4. Single-Step Agent Interaction

### 4.1 Purpose And Ownership

`Agentic.step<C, O>` performs one model generation and returns control to Etas
code. It does not execute requested tools, issue a second model request to
repair output, retry, or continue an agent loop. `Agentic.infer<C, O>` remains
the complete-loop convenience action; its standard controller can be expressed
using step, checked tool invocation, and conversation assembly.

The public wrapper is `std.agent.step`. `std.agent` exports the transparent
protocol types below; `std.tool.reference` and `std.tool.invoke` operate on those
same types. These are std types and APIs, not new language keywords. They do not
require opaque checked-call wrappers, execution receipts, or a second JSON type.
The existing `Conversation`, `Prompt`, and compiler-derived `Schema<O>` contracts
are reused rather than replaced by a new session or schema abstraction.

### 4.2 Transparent Types

```etas
module std.agent;

import std.json.JsonValue;

public type StepResult<C, O> = {
    turn: AssistantTurn,
    decision: StepDecision<C, O>,
    usage: Option<Usage>,
}

public enum StepDecision<C, O> {
    Final(O),
    CallTools(Array<ToolCall<C>>),
    Stopped(StopReason),
}

public type AssistantTurn = {
    request_id: Option<string>,
    content: Array<MessageContent>,
    provider_data: Map<string, JsonValue>,
}

public enum MessageContent {
    Text(string),
    Value(JsonValue),
}

public type ToolRef = {
    id: string,
    revision: string,
}

public type ToolCall<C> = {
    id: string,
    tool: ToolRef,
    args: JsonValue,
}

public type ToolResult<C> = {
    call_id: string,
    result: Result<JsonValue, StepDiagnostic>,
}

public enum StopReason {
    Truncated,
    Refused(string),
    InvalidOutput(StepDiagnostic),
    InvalidToolCall(StepDiagnostic),
}

public type StepDiagnostic = {
    code: string,
    message: string,
    path: Option<string>,
}

public type Usage = {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cost: Option<{ micros: u128, currency: string }>,
}
```

`C` is the resolved agent method identity, including its applicable revision and
declared tool surface, not a user-supplied runtime string. It is a checked
contract parameter, as in `Agentic.infer<C, O>`, rather than an ordinary data
type parameter chosen to acquire authority. It allows a call/result to retain
the relevant upper-bound summary; it is not evidence that a transparent record
has already been validated or executed.

`ToolRef.id` encodes a resolved package/module/item identity, not just a display
name. `revision` identifies the registered implementation snapshot. The same
`ToolRef` is used by reflection metadata. `std.tool.reference` constructs its
normal value from a resolved native tool declaration. Manually constructing or
editing the record is allowed, but never registers a tool or grants authority.
`step` resolves provider tool names against `C` to create these references;
an unresolvable name produces `Stopped(InvalidToolCall(...))`. This does not
replace argument or authorization checks at `invoke`, which must also work
safely on caller-constructed records.

`AssistantTurn` represents the assistant side of one protocol turn, not an
application `Message<T>` with sender/session provenance. Tool requests occur
only in `StepDecision.CallTools`, avoiding a second independently editable copy
in `AssistantTurn`. `provider_data` retains provider continuation metadata that
must survive the next request; it cannot override the model profile, tool
surface, policy, or budget. Adapters must preserve required continuation data
through the public protocol instead of relying on an inaccessible host object.
The initial content alternatives are text and structured JSON; this does not
claim a complete multimodal media API.

When produced by `step`, `Final(value)` guarantees successful decoding and
validation against `Schema<O>`, and `CallTools` is non-empty and contains
requests, not executed operations. A step does not both return a final answer
and request tools. These are API guarantees, not restrictions on constructing
transparent values: a caller-built `Final` is not proof of a model response,
and assembly must reject an empty `CallTools` or another invalid transition.
Invalid model output or tool-call syntax produces the corresponding `Stopped`
case without hidden repair. `StepDiagnostic.code` is a stable machine-readable
category, `message` is explanatory text, and `path` optionally identifies the
invalid schema field or tool argument; applications must not branch on message
wording.

An absent `usage`, token field, or cost means unknown, not zero. A provider that
does report usage must not have its cost discarded by the standard projection.
Unknown usage still follows conservative admission/accounting rules and cannot
disable parent limits. Fabricated protocol usage is not authoritative runtime
resource-accounting evidence.

### 4.3 Public APIs

The result column below describes returned values, not an implicit `![]`
annotation. Effectful entries preserve their inferred errors and behavioral
summaries as specified in section 4.4.

| API | Return Type | Contract |
|---|---|---|
| `std.agent.step<C, O>(conversation: Conversation, schema: Schema<O>)` | `StepResult<C, O>` | Request one `Agentic.step<C, O>` under the current agent execution context. Use the supplied output schema for this request without invoking tools or adding a repair request. |
| `std.tool.reference(tool)` | `ToolRef` | Describe a resolved Etas `tool` or imported native tool symbol, including its revision. Pure; a flow or raw name is not a substitute for a tool declaration. |
| `std.tool.invoke<C>(call: ToolCall<C>)` | `ToolResult<C>` | Resolve and validate the requested native tool, then dispatch its Etas body or declared primitive binding under the current checks. Validate and encode its result; return feedback correlated by `call_id`. |
| `std.agent.conversation.from_prompt(prompt: Prompt)` | `Conversation` | Construct an in-memory conversation while preserving prompt channels and trust information. Pure; do not implicitly load or write persistent session history. |
| `std.agent.conversation.append_step<C, O>(conversation: Conversation, step: StepResult<C, O>)` | `Result<Conversation, StepDiagnostic>` | Return a new conversation containing the complete assistant turn, including the decision's tool requests and required provider metadata. Pure; validate the protocol transition without executing anything. |
| `std.agent.conversation.append_results<C>(conversation: Conversation, results: Array<ToolResult<C>>)` | `Result<Conversation, StepDiagnostic>` | Return a new conversation with one correlated result for each pending call. Reject missing, duplicate, or foreign call ids and invalid protocol order. Pure; do not certify that tools actually ran. |

`O` is normally inferred from the schema/expected result; `C` is supplied by the
active agent method or the original inference request being handled. A handler
for `Agentic.infer<C, O>` can call `step` using that request's context even when
the handler itself was defined in an ordinary flow. Merely naming another `C`
does not switch identity or obtain its tools. With no applicable agent request
context, these APIs cannot fabricate one from protocol data.

Before contacting a provider, `step` checks that the conversation is a valid
next request, including that pending tool calls have results. Caller-built
history and provider metadata remain input data: they cannot upgrade trust,
prove an approval happened, or supply protected execution/usage evidence.

After `CallTools`, append the assistant step before appending its tool results.
Results can be supplied in a different array order when call ids identify them
unambiguously; assembly follows the pending-call order required by the provider.
The pending set is completed once, not partially replaced on each append.
`Final` and `Stopped` have no pending tool-result slots. Supplying a previously
constructed `ToolResult` is useful for mocks or simulations but records only
caller-provided feedback, never a genuine tool-execution event.

### 4.4 Effects, Execution Checks, And Failures

The default step handler consumes `Agentic.step<C, O>`, so it remains a requested
and default action rather than an ordinary escaping effect. The step remains
`NonDeterministic`; handling an action does not make real model execution pure.
It inherits the active model configuration, tool surface, trace specs, runtime
policy, limits, cancellation, and trace/replay mode. None is granted by the
conversation, schema, or transparent output records.

Before `std.tool.invoke` executes anything, it must:

1. resolve the tool identity and revision against the registered native tools
   exposed by `C`, rejecting stale or out-of-surface references;
2. decode arguments against the actual declaration's input schema, not schema
   claims supplied in the record;
3. enforce the tool boundary, caller boundary, current trace specs/policy,
   approval, sandbox, limits, and cancellation across transitive calls;
4. dispatch the registered Etas body or declared binding, and validate its
   output before encoding model-facing feedback.

Boundary diagnostics such as malformed arguments or a rejected tool request
may be returned as `Err(StepDiagnostic)` without executing the rejected tool.
This is not an implicit handler for every `Error<E>` raised inside a tool.
Ordinary typed failures remain escaping obligations unless the tool/controller
explicitly handles and translates them. Likewise, `Stopped` is a model-turn
outcome, not a catch-all for transport failures, cancellation, budget exhaustion,
or policy termination. Those retain the normal failure/unwind behavior. The
postfix `?` still captures exactly one residual `Error<E>` into `Result<T, E>`;
it is not needed to inspect the already value-level `ToolResult.result`.

Static analysis of `invoke<C>` includes the conservative union of the exposed
tools' effect, requested-action, determinism, and temporal summaries. Runtime
checking narrows the actual selected call. A default handler must not erase tool
authority behind the handled `Agentic.step`/`infer` action. Receiving a request
does not itself count as execution of the requested tool for approval ordering
or trace-spec satisfaction; the checked dispatch and actual actions supply
those events.

Invocation and replay records correlate request id, call id, tool revision, and
current execution context. A repeated record must not silently become permission
to execute a non-idempotent tool again; use the existing replay/deduplication
contract or report an uncertain outcome. This is not a promise of exactly-once
external effects. Multiple calls in one model turn are not automatically safe
to run in parallel; normal dependency, policy-order, and resource checks apply.

### 4.5 Complete Controller Example

This example replaces the full-loop handler with an explicit, bounded controller.
Its only exposed tool is a pure Etas tool, so the example introduces no external
authority actions or application-specific error types.

```etas
import std.agent.{StepDecision, ToolResult};
import std.agent.conversation;
import std.runtime.effects.Agentic;
import std.tool;

tool uppercase(text: string) -> string ![] {
    return std.text.uppercase(text);
}

@model("configured-model")
@tools([uppercase])
agent Writer(input: string) -> string {
    let prompt = Prompt.new()
        .system(Trusted("Use uppercase when needed, then return the final text."))
        .data(input);
    return perform infer<string>(prompt);
}

flow RunControlled(input: string) -> string {
    handle Writer.run(input) with {
        Agentic.infer<Writer.run, string>(prompt, schema) => {
            var history = conversation.from_prompt(prompt);

            for _ in [0, 8) {
                let step = std.agent.step(history, schema);
                history = match conversation.append_step(history, step) {
                    Ok(next) => next,
                    Err(error) => abort(error.message),
                };

                match step.decision {
                    Final(answer) => {
                        resume answer;
                    },
                    CallTools(calls) => {
                        var results: Array<ToolResult<Writer.run>> = [];
                        for call in calls {
                            results = results.push(tool.invoke(call));
                        }
                        history = match conversation.append_results(history, results) {
                            Ok(next) => next,
                            Err(error) => abort(error.message),
                        };
                    },
                    Stopped(_) => {
                        abort("Model step stopped without a final answer");
                    },
                };
            }

            abort("Model step limit reached");
        }
    }
}
```

`Final`, `CallTools`, and `Stopped` are the variants of the imported
`StepDecision`. The handler resumes its original inference request at most once;
the loop performs separate step requests, not multi-shot resumption. The bound
of eight turns is controller logic, not a reset of inherited token, cost, or
wall-time limits. This deliberately fail-fast example aborts on a stopped turn;
an application may instead explicitly repair context, delegate, or return a
typed failure. Such choices remain ordinary Etas code, not hidden behavior of
`step`.

## 5. Numeric Helpers

### 5.1 Signatures And Numeric Evidence

The following contracts replace the signature-only legacy declarations.
Section 5.2 specifies their numeric edge cases.

```etas
module std.math;

@std.role("std.math.Number")
public spec Number;
```

| `std.math` API | Return Type | Contract |
|---|---|---|
| `abs<T ~ Number>(value: T)` | `T ![]` | Magnitude at the original numeric width; signed-minimum overflow is a checked arithmetic fault. |
| `min<T ~ Number>(left: T, right: T)` | `T ![]` | Smaller operand under the numeric/NaN/signed-zero rules in section 5.2. |
| `max<T ~ Number>(left: T, right: T)` | `T ![]` | Larger operand under the same rules. |

`Number` is a compiler-known primitive numeric spec. Its declaration is
owned by Etas source. Compiler evidence admits the primitive signed and
unsigned integer families, `isize`, `usize`, `f32` and `f64`. User declarations
cannot extend that primitive family. Generic bodies bounded by `Number` may
use same-type comparisons and supported binary arithmetic. Operations with a
narrower domain require their checked primitive contract. In particular,
`Number` alone does not establish that unsigned unary negation is valid.
The magnitude primitive used by `abs` covers unsigned, signed and floating
inputs according to section 5.2. This evidence must survive
package imports and checked-code restoration, following the existing compiler
contract pattern for `Index`.

The operands of `min` and `max` have the same numeric type. Existing literal
inference applies, and conversions between concrete types remain explicit.
Each operation returns the same type and width. Nominal numeric wrappers must
use their declared conversion or their own ordinary helper. Strings, booleans,
collections and unconstrained generic values do not satisfy `Number`.

The single-argument legacy `min<T>(T) -> T` and `max<T>(T) -> T` were incomplete
declarations. Their replacements are binary numeric operations. Collection
reduction requires a separate API with its own empty-input contract.

### 5.2 Results And Edge Cases

`abs` returns the magnitude. Unsigned input is returned unchanged. For a signed
integer minimum, its positive magnitude does not fit the result type, so the
operation raises the existing checked arithmetic-overflow execution fault.
It follows the same failure mechanism as overflowing integer negation.
It neither wraps nor saturates. The empty Effect row describes the absence of
observable authority actions, following ordinary checked arithmetic.

Floating-point `abs` clears the sign bit at the input width. Both signed zeros
produce positive zero, negative infinity produces positive infinity, and a NaN
retains its payload and remaining encoding bits.

Integer `min` and `max` use numeric order and return one of their inputs.
Floating-point results follow these deterministic selection rules.

| Inputs | `min` | `max` |
| --- | --- | --- |
| Distinct ordered values, including infinities | Smaller operand | Larger operand |
| Equal values other than opposite signed zeros | Left operand | Left operand |
| Negative zero and positive zero, in either order | Negative zero | Positive zero |
| Exactly one NaN | The other operand | The other operand |
| Two NaNs | Left operand, preserving its encoding | Left operand, preserving its encoding |

The selected operand is returned at its original width. Selection does not
round through `f64` or canonicalize a selected NaN payload. Implementations
share the numeric rules across constant evaluation and runtime execution.
Ordinary Etas bodies own selection and branching. Checked primitive arithmetic
and float bit classification use declared primitive bindings where required.

## 6. Round-Robin Groups

### 6.1 Public Contract

`std.agent.group.round_robin` repeatedly invokes a homogeneous array of
state-transforming callbacks in array order. The state can contain an
application conversation or transcript. Agents with different entry signatures
are adapted with ordinary closures to this shared state contract.

```etas
module std.agent.group;

public enum GroupError { EmptyParticipants, }
public enum GroupStop { PredicateSatisfied, TurnLimitReached, }
public type RoundRobinResult<S> = {
    public state: S,
    public turns: usize,
    public reason: GroupStop,
};
```

| `std.agent.group` API | Return Type | Contract |
|---|---|---|
| `round_robin<S, effect E>(participants: Array<(S) -> S ![E]>, initial: S, max_turns: usize, stop_when: (S) -> bool ![])` | `RoundRobinResult<S> ![E, Error<GroupError>]` | Sequential state-transforming callbacks; reject an empty roster and preserve callback effects and inherited constraints. |

The source implementation performs these steps in order.

1. Reject an empty participant array with `Error<GroupError>.raise(EmptyParticipants)`.
2. Set `state = initial` and `turns = 0`. Evaluate `stop_when(state)` once.
   If true, return `PredicateSatisfied` with zero turns.
3. While `turns < max_turns`, invoke participant `turns % participants.len()`
   with the current state. Replace the state only after the callback returns
   successfully, then increment `turns` once.
4. Evaluate `stop_when` after each completed turn. Return immediately with
   `PredicateSatisfied` when true, including on the final permitted turn.
5. Otherwise return the current state and `TurnLimitReached` after the budget
   of turns is consumed. With a nonempty roster and `max_turns = 0`, this
   returns the initial state unless the initial predicate already succeeded.

One turn means one callback invocation. `max_turns` counts across all
participants. A three-participant group with five allowed turns invokes
indices `0, 1, 2, 0, 1`. Each call to `round_robin` starts at index zero.
The function returns the final state and completion reason.

### 6.2 Effects, Failure And Ownership

Calls are sequential. The next participant observes the state returned by its
predecessor. `E` includes the checked effects of the supplied callbacks, with
the existing row-inclusion rules determining a common row. A callback that
invokes an agent therefore exposes its Agentic actions. Pure callbacks are
also valid. The predicate is pure under its declared row and is evaluated once
at each specified boundary.

The combinator inherits handlers, policies, budgets, cancellation, trace and
session context. `max_turns` is an algorithm bound. Inherited execution limits
can terminate the call earlier and keep their existing failure behavior.
Callback errors and execution faults propagate immediately. Completed external
actions retain their effects. The combinator performs no implicit retries or
state rollback. Its implementation is an ordinary checked Etas loop.

## 7. Virtual Trace Stages

The public runtime signature remains
`VirtualStages(stages: Array<TraceLabel>) -> TraceLabel ![]` in
`std.runtime.trace`.

The flow constructs an immutable composite label containing its child labels
in order. Empty input constructs an empty composite. Single-element input
retains a composite node, duplicates are retained and nested composites retain
their hierarchy. It creates metadata without executing the stages. `Logical`
continues to denote the existing logical trace label.

`TraceLabel` retains its source-declared opaque nominal type for this migration.
The source flow calls a private, pure binding that allocates the structured
label payload. That binding validates the label inputs and preserves their
checked identities. It performs no host request, trace emission or provider
call. The opaque storage codec preserves this tree through checkpointing.

An annotation such as `@trace(VirtualStages([Researcher, Critic]))` uses
annotation-specific elaboration. The frontend resolves agent, flow or tool
declaration references to checked stage labels with their callable identity,
revision and source provenance. Label expressions such as `Logical` are also
accepted. Import aliases preserve the resolved identity. Unresolved or
non-callable declaration references produce a diagnostic. Ordinary runtime
calls to `VirtualStages` require actual `TraceLabel` values.

Trace metadata records a plan of logical stages. Physical inference, tool,
permission and usage events come from actual execution. A virtual stage label
alone cannot certify a model call, an approval or a tool invocation. Fusion
records the source-stage mapping and the actual physical invocation separately.
Display strings may accompany the structured plan, while runtime checks and
replay use its checked identities.

## 8. Sanitization And Declassification

### 8.1 Typed Strategies And Signatures

`std.security.declassify` defines these source-owned declarations.

```etas
@std.role("std.security.declassify.SanitizationStrategy")
public type SanitizationStrategy<T, U>;
@std.role("std.security.declassify.DeclassificationStrategy")
public type DeclassificationStrategy<T, U>;
public enum SecurityError {
    Rejected,
    Denied,
    InvalidReason,
    Unavailable,
    StrategyMismatch,
}
```

| `std.security.declassify` API | Return Type | Contract |
|---|---|---|
| `sanitize<T, U>(value: Untrusted<T>, strategy: SanitizationStrategy<T, U>)` | `Sanitized<U> ![Error<SecurityError>]` | Execute the validated typed strategy and retain its processing provenance. |
| `declassify<T, U>(value: Secret<T>, strategy: DeclassificationStrategy<T, U>, reason: string)` | `Sanitized<U> ![Secret, Error<SecurityError>]` | Authorize and audit release, execute the validated strategy, and retain release provenance. |

Both strategy types are immutable opaque references issued by trusted bindings.
Each identifies a validated strategy implementation, its revision and exact
input/output types. An installed strategy may execute a checked Etas body with
signature `(T) -> Result<U, SecurityError> ![]`. Binding installation supplies
the evidence associating that body with the strategy contract. The public APIs
validate this evidence and execute that body under the caller's limits.
Issuing a strategy handle and granting secret release use the existing trusted
binding and runtime authority mechanisms.

These signatures replace the two unrestricted legacy `T -> T` declarations.
Every call supplies a strategy explicitly. A strategy has a concrete contract,
such as removing designated HTML constructs or projecting approved incident
fields. The `Sanitized` wrapper records processing under that contract. Prompt
encoding retains data-channel placement and nested trust labels. A subsequent
conversion to a trusted instruction channel requires its own existing evidence.

These APIs and their strategy types are imported explicitly from
`std.security.declassify`. The illustrative `sanitize.html` and `sanitize.text`
names used in earlier examples denote application or EDK helpers. Those helpers
provide their own source definitions, choose a concrete strategy and handle
its errors.

`Secret<T>` is the trust wrapper containing a typed secret value.
`std.secret.SecretValue<K>` is an opaque host-secret reference indexed by a key
type. This `declassify` signature accepts the former. Handling an opaque host
secret requires a separately declared host operation and its authority checks.

### 8.2 Sanitization

`sanitize` validates the strategy and input/output types, passes the untrusted
payload only to that strategy's protected execution frame, and returns
`Sanitized<U>` after successful output validation. An explicit strategy may be
a validator that preserves the input when its checks pass, or a transformer
that produces a different type. Rejection raises `Error<SecurityError>`.
Missing or incompatible strategy evidence raises `Unavailable` or
`StrategyMismatch`.

The operation performs the strategy's declared deterministic computation.
It obtains neither secret access nor an implicit model/host request. Boundary
primitives provide validated payload access and wrapper construction. The
strategy's actual validation and transformation can be implemented in Etas.

### 8.3 Secret Release And Audit

`declassify` rejects a reason that is empty after whitespace trimming. It then
validates strategy identity, revision and type correspondence and checks the
current authority for the exact strategy before exposing the secret payload
to its protected strategy frame. Denial produces `Denied`, while an absent
implementation produces `Unavailable`. Strategy rejection produces `Rejected`.
Successful output validation constructs `Sanitized<U>` with release provenance.

The operation requests a new checked action `Secret.declassify` on the existing
Secret owner. Its inferred signature binders carry `T` and `U`, with zero static
selectors. Its payload carries the secret wrapper, typed strategy and reason.

```etas
impl std.effects.Secret {
    action<T, U> declassify(
        value: std.security.trust.Secret<T>,
        strategy: DeclassificationStrategy<T, U>,
        reason: string,
    ) -> Sanitized<U>;
}
```

The default handler enforces strategy-scoped grants, current trace specs and
approvals, then performs the protected transformation. The public row is
`![Secret, Error<SecurityError>]`. Custom handler support must preserve the
protected release check before any secret access or result-wrapper creation.

The audit record includes strategy identity and revision, input/output type
identities, caller trace, authorization outcome, the supplied reason and result
status. The reason is caller-supplied audit data and follows the existing
redaction policy. Secret payloads and strategy intermediate values are excluded
from ordinary errors, logs and trace payloads. Permission failures and strategy
failures produce trace-visible outcomes through the existing runtime mechanism.

Raw access to the secret and construction of release provenance belong to
explicit protected bindings. Ordinary casts or trust-wrapper constructors must
retain nested secrecy and cannot manufacture a successful release. A checked
strategy executes under inherited budgets and cancellation. An identity
release requires an explicitly configured strategy and the same authorization
as any other release.

## 9. Implementation And Migration Acceptance

The seven legacy declarations are historical API inventory entries with
incomplete execution contracts. The corrected signatures above are intentional
API changes. Preserve the recorded legacy baseline and classify each change
explicitly when comparing providers. Retain the Rust registry during migration.
Apply the new contracts to supported execution paths before advertising their
availability. Unsupported paths must report an actionable diagnostic.

Source flows, ordinary enums, result records and policy bodies live in the
existing `etas_std/library/std` modules. Their compilation reuses the existing
pass manager, pipelines and installed std artifact mechanism. Native bindings
own primitive numeric operations, opaque payload storage, secret authority and
protected wrapper/provenance operations.

The implementation tasks, code locations and acceptance cases are recorded in
[Legacy Flow Implementation](../migrations/legacy-flow-implementation.md).

### 9.1 Cross-Provider Acceptance

The API tables are the shared source/legacy contract, not permission to silently
fall back from one provider to another. Keep implementation coverage in the
engineering test matrix; mark a row available only for the provider actually
checked. Intentional changes, including unordered logical map/set equality and
new hash bounds, must be recorded as contract changes rather than concealed as
representation-only migration.

| Area | Required Acceptance Evidence |
|---|---|
| Container invariants | Reject external metadata writes and forged views; `len`, emptiness, lookup and iteration agree after every update. |
| Equality/hash/order | Primitive and nominal keys, collisions, equal records with different construction order, differing map/set insertion orders, signed zeros/NaNs, lawful custom comparators and stable duplicate behavior. |
| Snapshot preservation | Retain old values, slices and iterators while updating/removing/clearing current values; verify old observations and nested payload sharing remain correct. |
| Cost and storage | Measure comparisons, node/header copies, allocations and retained slots separately at growing sizes, for both reusable backing and shared snapshots. Include array draining, map construction, ordered deletion and long-lived queue churn. |
| Iterator consumption | Empty/exhausted iterators, repeated use, adapter composition, zero-sized chunks, short circuiting and bounded lazy allocation. |
| Effects and capture | Pure lazy adapters reject state-dependent/effectful callbacks; consumers propagate callback effects/actions/capture dependencies across source imports and package metadata. |
| Failure behavior | Invalid indexes use the specified error effect; callback failure stops later work without rolling back completed external actions; `unwrap` terminates rather than silently supplying a value. |
| Installed delivery | Build and run against the installed source SDK, including separately compiled imports. Legacy native-container tests alone do not certify source SDK behavior or performance. |

## 10. Collections And Key Contracts

### 10.1 Shared Rules And Cost Model

These are standard-library types and flows, not additional language keywords.
The [literal, indexing and value-semantics rules](02-general-programming-constructs.md#7-composite-types)
remain the source-language contract. Operations return updated values; they do
not mutate another observer's copy or implicitly add a `var self` receiver.
Discarding an update result leaves the original value unchanged.

| Shared Contract | Required Behavior |
|---|---|
| Internal metadata | Cached lengths, capacity, backing storage, tree heights, hash seeds and heap indexes are private. Users cannot assign them or construct an inconsistent container. `len()` and `is_empty()` are observations, not writable fields. |
| Snapshots | Updating one value preserves all previously retained values, slices and iterators. Physical sharing and copy-on-write are permitted. |
| Checked access | `I ~ Index` follows the existing conversion rules. `get` returns `None` for an invalid index; `at`, indexed update and checked slicing raise `Error<IndexError>`. |
| Allocation | Check size arithmetic and obey inherited allocation/execution limits. Overflow and resource exhaustion retain their existing execution-fault behavior; do not wrap lengths or invent successful truncated results. |
| Value contents | Element types retain their effects, capture dependencies, secrecy and scope/affine restrictions. A container cannot hide an invalid copy or escape. |
| Equality | Arrays, lists, slices, stacks, queues and deques compare their logical element sequences. Map equality compares key/value bindings; set equality compares membership, independently of insertion order. Private storage and hash seeds are not part of equality. |
| Equality bounds | Sequence equality requires element equality; map equality additionally requires value equality. Ordered containers with user comparators and priority queues have no implicit equality/hash contract. |
| Stable iteration | Map and set iteration follows first insertion order. Replacing an existing entry retains its position; removing and reinserting appends it. Literal fields/entries are evaluated in source order, including overwritten duplicates. |

Costs below count elements `n`, appended elements `m`, or emitted elements `k`.
Hashing, equality, comparison, callback and element-copy costs are additional;
large keys are not constant-cost merely because they occupy one slot. Expected
hash-table bounds assume a suitable hash distribution; collisions can degrade
lookup to `O(n)`. They are not worst-case security guarantees.

Update costs distinguish a value whose backing can be reused from a retained
snapshot. An array/hash-container update may copy `O(n)` element headers when
the old backing is shared. This is not a requirement to deep-copy payloads.
Tree updates should copy only an `O(log n)` path when retaining an old version.
Amortized queue/deque bounds apply to a linear update history; repeatedly
branching from the same old version must be measured separately. Read-only
queries must not copy a complete container.

### 10.2 Equality, Hashing And Order

| `std.collections` Contract / API | Result | Required Behavior |
|---|---|---|
| `Eq` | Type spec | Evidence that the existing value equality is reflexive, symmetric and transitive. It does not introduce an overload of `==`. |
| `Hash: Eq` | Behavioral type spec | Requires `hash(self, state: Hasher) -> Hasher ![]`. Feed the value into a value-semantic hash state; equal values must produce the same final hash from the same initial state. |
| `Order` | Behavioral type spec | Requires `compare(self, other: Self) -> i32 ![]`; negative, zero and positive mean less, equivalent and greater. Comparison defines a stable total order and equivalence relation. |
| `Hasher.new(seed: u64 = 0)` | `Hasher ![]` | Create an empty deterministic non-cryptographic state. The default constructor performs no implicit entropy or host action. |
| `Hasher.write(data: bytes)` | `Hasher ![]` | Return a state extended by these bytes, preserving the input state. Processing is linear in input bytes. |
| `Hasher.finish()` | `u64 ![]` | Observe the current digest without changing the state. Hash values are not a persistent encoding, signature, or cryptographic digest. |

Recognition of `Eq` attaches to the resolved standard spec, not its spelling.
Standard `Eq`/`Hash` implementations cover integers, booleans, chars, strings and
bytes. Structural equality evidence extends to finite tuples/records/enums and
supported value collections when their components satisfy the relevant contract.
Explicit `Eq` conformance must pass the same standard evidence checks; it is
not an unchecked assertion that an arbitrary type's `==` is reflexive.
`Hash` implementations use the corresponding logical value and nominal identity,
not addresses or storage layout; structural hashing must frame fields/variants
unambiguously. Ordinary nominal data with hashable components receive this
checked structural implementation unless they provide an explicit conforming
implementation. No new derive keyword is required. Functions, handlers, secrets
and resource handles do not acquire equality or hashing merely because their
implementation contains a comparable address.

Raw `f32`/`f64` retain ordinary floating equality and do not receive `Eq`/`Hash`
evidence: NaN breaks reflexivity. Applications needing such hash keys use a
nominal wrapper over an integer/byte encoding, with a documented normalization
and compatible hash contract; merely wrapping an unchanged float field does
not establish reflexive equality. Their standard `Order` instead orders raw
encodings: negative encodings in reverse bit order, followed by nonnegative
encodings in bit order. It distinguishes
signed zeros and NaN encodings; identical encodings compare as zero. `Order`
therefore does not imply `Eq` or redefine floating `==`.

Hashing and key comparators must be deterministic, non-suspending and independent
of mutable captured/resource state while a container retains them, with no
observable requested actions. `![]` alone does not prove those properties.
Algebraic laws remain implementation obligations; type conformance alone does
not prove a user's comparator transitive. Violating these obligations must not
compromise memory safety or grant host authority.
Hash equality is required only within the same algorithm/version and initial
state. Hash seeds do not change iteration or logical equality. Hashing a map or
set must be independent of its insertion order and private hasher.

### 10.3 Arrays, Lists And Slices

Unless a row shows an effect, its escaping row is `![]`. `from_iter` consumes
the pure iterator in order without first making a second complete input copy.

| Owner / API | Return Type | Behavior And Cost |
|---|---|---|
| `Array<T>.new()` | `Array<T>` | Empty array; `O(1)`. |
| `Array<T>.from_iter(items: Iter<T>)` | `Array<T>` | Bulk construction; `O(n)` excluding iterator work. |
| `Array<T>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `Array<T>.get<I ~ Index>(index: I)` | `Option<T>` | Optional access; `O(1)`. |
| `Array<T>.at<I ~ Index>(index: I)` | `T ![Error<IndexError>]` | Checked access; `O(1)`. |
| `Array<T>.first()`, `last()` | `Option<T>` | First/last element; `None` on empty; `O(1)`. |
| `Array<T>.push(value: T)` | `Array<T>` | Append; amortized `O(1)` with reusable backing. |
| `Array<T>.pop()` | `(Array<T>, Option<T>)` | Remove the last element; `O(1)` with reusable backing. Empty returns an empty array and `None`; do not rebuild the prefix by traversal. |
| `Array<T>.extend(values: Array<T>)` | `Array<T>` | Append in argument order; amortized `O(m)` with reusable backing. |
| `Array<T>.set<I ~ Index>(index: I, value: T)` | `Array<T> ![Error<IndexError>]` | Replace an existing element; `O(1)` with reusable backing. |
| `Array<T>.insert<I ~ Index>(index: I, value: T)` | `Array<T> ![Error<IndexError>]` | Insert before the index; `len()` is allowed; `O(n)`. |
| `Array<T>.remove<I ~ Index>(index: I)` | `(Array<T>, T) ![Error<IndexError>]` | Remove an existing index and return its old value; `O(n)`. |
| `Array<T>.reverse()` | `Array<T>` | Reversed value; `O(n)`. |
| `Array<T>.clear()` | `Array<T>` | Empty value; old snapshots remain valid. |
| `Array<T>.iter()` | `Iter<T>` | Lazy element-order snapshot; `O(1)` construction. |
| `List<T>.new()`, `from_iter(items: Iter<T>)` | `List<T>` | Empty / ordered bulk construction; `O(1)` / `O(n)`. |
| `List<T>.len()`, `is_empty()` | `usize`, `bool` respectively | `len` is `O(n)`; emptiness is `O(1)`. |
| `List<T>.head()`, `tail()` | `Option<T>`, `Option<List<T>>` respectively | `None` on empty; otherwise observe head/tail; `O(1)`. |
| `List<T>.get<I ~ Index>(index: I)` | `Option<T>` | Traverse to the requested position; `O(n)` worst case. |
| `List<T>.at<I ~ Index>(index: I)` | `T ![Error<IndexError>]` | Checked positional access; `O(n)` worst case. |
| `List<T>.push(value: T)` | `List<T>` | Prepend, equivalent to `value :: self`; `O(1)`. |
| `List<T>.pop()` | `(List<T>, Option<T>)` | Return tail and head; empty returns itself and `None`; `O(1)`. |
| `List<T>.extend(prepended: List<T>)` | `List<T>` | Preserve the existing cons-oriented contract: `prepended` followed by `self`; `O(m)`, sharing the old tail. Unlike array `extend`, this prepends. |
| `List<T>.reverse()` | `List<T>` | Reversed list; `O(n)`. |
| `List<T>.iter()` | `Iter<T>` | Lazy head-to-tail snapshot; `O(1)` construction. |
| `Slice<T>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `Slice<T>.get<I ~ Index>(index: I)` | `Option<T>` | Index relative to the view; `O(1)`. |
| `Slice<T>.at<I ~ Index>(index: I)` | `T ![Error<IndexError>]` | Checked view-relative access; `O(1)`. |
| `Slice<T>.first()`, `last()` | `Option<T>` | First/last view element; `O(1)`. |
| `Slice<T>.to_array()` | `Array<T>` | Materialize this view, not its full source; `O(n)`. |
| `Slice<T>.iter()` | `Iter<T>` | Lazy view-order snapshot; `O(1)` construction. |

Range-based slicing retains the established syntax and boundary rules. Creating
an array/slice view is `O(1)` and keeps its source alive; changing the source
binding afterwards cannot change the view. Neither `.length` assignment nor a
public constructor may forge a view beyond its source bounds. Costs of releasing
an unshared value, including `clear`, may be linear in the released storage.

### 10.4 Maps And Sets

`Map<K, V>` keys and `Set<T>` elements require the standard `Hash` contract,
which includes `Eq`. Literal inference retains these requirements; unconstrained
generic keys do not bypass them. Values need not be hashable merely to be stored.
`new()` uses `Hasher.new()`; `with_hasher` stores an independent base state and
starts each key hash from that state. Tables give expected hash-container costs,
subject to section 10.1's collision and retained-snapshot qualifications.

| Owner / API | Return Type | Behavior And Cost |
|---|---|---|
| `Map<K, V>.new()`, `with_hasher(base: Hasher)` | `Map<K, V>` | Empty map; `O(1)`. |
| `Map<K, V>.from_entries(items: Iter<(K, V)>)` | `Map<K, V>` | Bulk build in iterator order; duplicate keys keep the last value and first position; expected `O(n)`. |
| `Map<K, V>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `Map<K, V>.get(key: K)` | `Option<V>` | Missing is `None`; expected `O(1)`. |
| `Map<K, V>.contains_key(key: K)` | `bool` | Membership; expected `O(1)`. |
| `Map<K, V>.insert(key: K, value: V)` | `Map<K, V>` | Insert/replace, retaining an existing key's position; expected amortized `O(1)` with reusable backing. |
| `Map<K, V>.remove(key: K)` | `Map<K, V>` | Remove if present; missing is unchanged; expected amortized `O(1)` with reusable backing. |
| `Map<K, V>.take(key: K)` | `(Map<K, V>, Option<V>)` | Remove and return the old value; same bounds as `remove`. |
| `Map<K, V>.update<effect E>(key: K, f: Option<V> -> Option<V> ![E])` | `Map<K, V> ![E]` | Call `f` once; `Some` inserts/replaces and `None` removes; no mutable entry reference escapes. |
| `Map<K, V>.merge<effect E>(other: Map<K, V>, resolve: (K, V, V) -> V ![E])` | `Map<K, V> ![E]` | Visit `other` in iteration order; resolve only conflicts using key/left/right values. Retain left positions and append new right keys; expected `O(n + m)` plus callbacks. |
| `Map<K, V>.iter()`, `keys()`, `values()` | `Iter<(K, V)>`, `Iter<K>`, `Iter<V>` respectively | Lazy insertion-order snapshots; traversal linear in live entries with amortized bounded tombstone work. |
| `Map<K, V>.clear()` | `Map<K, V>` | Empty map retaining its hashing policy. |
| `Set<T>.new()`, `with_hasher(base: Hasher)` | `Set<T>` | Empty set; `O(1)`. |
| `Set<T>.from_iter(items: Iter<T>)` | `Set<T>` | Keep first occurrences in input order; expected `O(n)`. |
| `Set<T>.from_list(items: List<T>)` | `Set<T>` | Compatibility convenience with the same semantics as `from_iter(items.iter())`. |
| `Set<T>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `Set<T>.contains(value: T)` | `bool` | Membership; expected `O(1)`. |
| `Set<T>.insert(value: T)`, `remove(value: T)` | `Set<T>` | Existing insert/missing remove leaves the logical value unchanged; expected amortized `O(1)` with reusable backing. |
| `Set<T>.union(other: Set<T>)` | `Set<T>` | Left order, then previously absent right elements; expected `O(n + m)`. |
| `Set<T>.intersection(other: Set<T>)`, `difference(other: Set<T>)` | `Set<T>` | Retain qualifying left elements in left order; expected `O(n + m)`. |
| `Set<T>.symmetric_difference(other: Set<T>)` | `Set<T>` | Left-only elements, then right-only elements; expected `O(n + m)`. |
| `Set<T>.is_subset(other: Set<T>)`, `is_disjoint(other: Set<T>)` | `bool` | Membership tests with short circuiting; expected `O(n + m)`. |
| `Set<T>.iter()` | `Iter<T>` | Lazy insertion-order snapshot. |
| `Set<T>.clear()` | `Set<T>` | Empty set retaining its hashing policy. |

The prior `Map.from_entries(List<(K, V)>)` form remains a compatibility overload
and delegates to the iterator form. Rehashing, removal and slot compaction must
not reorder surviving entries. Replacing an equal key preserves the stored key
representative. A failed update/merge callback returns no updated container;
external actions already completed by the callback are not rolled back.

### 10.5 Stacks, Queues And Deques

| Owner / API | Return Type | Behavior And Cost |
|---|---|---|
| `Stack<T>.new()`, `Queue<T>.new()`, `Deque<T>.new()` | Respective container | Empty value; `O(1)`. |
| Each container's `from_iter(items: Iter<T>)` | Respective container | `O(n)` bulk build; iterator order is the subsequent pop order for stack/queue and front-to-back order for deque. |
| Each container's `len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `Stack<T>.push(value: T)`, `Queue<T>.push(value: T)` | Respective container | Add to stack top / queue back; amortized `O(1)` with reusable backing. |
| `Stack<T>.pop()`, `Queue<T>.pop()` | `(Container<T>, Option<T>)` | Remove stack top / queue front; `None` on empty; amortized `O(1)`. `Container` here denotes the respective owner, not a new type. |
| `Stack<T>.peek()`, `Queue<T>.peek()` | `Option<T>` | Observe the next pop result without removing it; `O(1)`. |
| `Deque<T>.push_front(value: T)`, `push_back(value: T)` | `Deque<T>` | Add at the selected end; amortized `O(1)`. |
| `Deque<T>.pop_front()`, `pop_back()` | `(Deque<T>, Option<T>)` | Remove at the selected end; `None` on empty; amortized `O(1)`. |
| `Deque<T>.peek_front()`, `peek_back()` | `Option<T>` | Observe an end without removal; `O(1)`. |
| Each container's `iter()` | `Iter<T>` | Stack pop order / FIFO order / front-to-back order; linear traversal. |
| Each container's `clear()` | Respective container | Empty value; retained snapshots remain unchanged. |

Queue/deque rebalancing may take `O(n)` for an individual operation. A sequence
of operations must meet its amortized bound; peeking must not repeatedly rebuild
the same entire queue/deque on every read. These APIs do not create synchronized
shared queues; task-safe shared resources require their explicit ownership and
synchronization contract.

### 10.6 Ordered Maps And Sets

Default construction requires `K ~ Order` / `T ~ Order`. The existing
`new(compare: (K, K) -> i32 ![])` or element-equivalent overload selects a
custom total order. Comparator equivalence, not an unrelated `==`, identifies
keys. Replacing an equivalent key retains its original representative.

| Owner / API | Return Type | Behavior And Cost |
|---|---|---|
| `OrderedMap<K, V>.new()`, `new(compare: (K, K) -> i32 ![])` | `OrderedMap<K, V>` | Empty map; `O(1)`. |
| `OrderedMap<K, V>.from_entries(items: Iter<(K, V)>)`, `from_entries(items: Iter<(K, V)>, compare: (K, K) -> i32 ![])` | `OrderedMap<K, V>` | Default/custom-order bulk build; last equivalent value wins while the first key representative is retained; `O(n log n)` comparisons or better. |
| `OrderedMap<K, V>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `OrderedMap<K, V>.get(key: K)`, `contains_key(key: K)` | `Option<V>`, `bool` respectively | Ordered lookup; `O(log n)` comparisons. |
| `OrderedMap<K, V>.insert(key: K, value: V)`, `remove(key: K)` | `OrderedMap<K, V>` | Balanced update; `O(log n)` comparisons/path work. Missing removal preserves the value. |
| `OrderedMap<K, V>.take(key: K)` | `(OrderedMap<K, V>, Option<V>)` | Remove and return the old value; `O(log n)`. |
| `OrderedMap<K, V>.first()`, `last()` | `Option<(K, V)>` | Least/greatest entry; `O(log n)` or better. |
| `OrderedMap<K, V>.lower_bound(key: K)`, `upper_bound(key: K)` | `Option<(K, V)>` | First key greater than or equal to / strictly greater than the argument; `O(log n)`. |
| `OrderedMap<K, V>.iter()`, `keys()`, `values()` | `Iter<(K, V)>`, `Iter<K>`, `Iter<V>` respectively | Ascending key order; `O(n)` traversal and `O(log n)` cursor state. |
| `OrderedMap<K, V>.range(lower: Option<K>, upper: Option<K>, include_lower: bool = true, include_upper: bool = false)` | `Iter<(K, V)>` | Lazy bounded traversal; `None` is unbounded; `O(log n + k)` consumption. |
| `OrderedSet<T>.new()`, `new(compare: (T, T) -> i32 ![])` | `OrderedSet<T>` | Empty set; `O(1)`. |
| `OrderedSet<T>.from_iter(items: Iter<T>)`, `from_iter(items: Iter<T>, compare: (T, T) -> i32 ![])` | `OrderedSet<T>` | Default/custom-order bulk build; retain the first equivalent representative; `O(n log n)` comparisons or better. |
| `OrderedSet<T>.len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `OrderedSet<T>.contains(value: T)` | `bool` | `O(log n)`. |
| `OrderedSet<T>.insert(value: T)`, `remove(value: T)` | `OrderedSet<T>` | Balanced update; `O(log n)`. |
| `OrderedSet<T>.first()`, `last()`, `lower_bound(value: T)`, `upper_bound(value: T)` | `Option<T>` | Corresponding ordered-map rule; `O(log n)` or better. |
| `OrderedSet<T>.iter()` | `Iter<T>` | Ascending order; `O(n)` traversal. |
| `OrderedSet<T>.range(lower: Option<T>, upper: Option<T>, include_lower: bool = true, include_upper: bool = false)` | `Iter<T>` | The same bound rules and `O(log n + k)` consumption. |
| Either container's `clear()` | Respective container | Empty value retaining the comparator. |

All rows in this subsection have `![]`. Inverted or mutually exclusive bounds
produce an empty iterator. These key bounds deliberately do not use `Range<K>`:
the language's `Range<I>` requires `Index`, whereas ordered keys may be strings
or other ordered data. Shape balancing and deletion must retain height/index
invariants in both current and previously shared versions.

### 10.7 Priority Queues

Default construction requires `P ~ Order`; a comparator overload accepts
`(P, P) -> i32 ![]`. Greater priority is popped first. Equal priorities are
FIFO by insertion time, independently of physical slot reuse or compaction.

| `PriorityQueue<T, P>` API | Return Type | Behavior And Cost |
|---|---|---|
| `new()`, `new(compare: (P, P) -> i32 ![])` | `PriorityQueue<T, P>` | Empty queue; `O(1)`. |
| `from_entries(items: Iter<(P, T)>)`, `from_entries(items: Iter<(P, T)>, compare: (P, P) -> i32 ![])` | `PriorityQueue<T, P>` | Default/custom-order bulk heap construction; `O(n)` comparisons. Input order breaks ties. |
| `len()`, `is_empty()` | `usize`, `bool` respectively | `O(1)`. |
| `push(value: T, priority: P)` | `PriorityQueue<T, P>` | `O(log n)` comparisons; amortized logarithmic update work with reusable backing. |
| `peek()`, `peek_entry()` | `Option<T>`, `Option<(P, T)>` respectively | Observe the next pop value/entry; `O(1)` after heap initialization. |
| `pop()` | `(PriorityQueue<T, P>, Option<T>)` | Remove the highest-priority value; empty yields `None`; amortized `O(log n)` comparisons/work. |
| `pop_entry()` | `(PriorityQueue<T, P>, Option<(P, T)>)` | Same operation retaining the removed priority. |
| `iter()` | `Iter<(P, T)>` | Live entries in insertion order, not priority order; amortized linear traversal in live storage. |
| `clear()` | `PriorityQueue<T, P>` | Empty queue retaining its comparator. |

All rows have `![]`. Logical backing storage must remain `O(n + 1)` in live
entries, not cumulative pushes. Popping replaces/removes payloads and eventually
reclaims empty slots; an empty queue retains no historical slot sequence.
Compaction may occasionally take linear work but must be amortized across
updates, preserve tie order and snapshots, and use checked ticket/index arithmetic.
Retained old snapshots account for their own reachable storage, not a leak in
the current queue. Native heap comparison counts alone do not establish these
storage, copying or allocation contracts for the source SDK.

### 10.8 Bytes

`bytes` is the existing immutable byte-sequence type, not a new collection
keyword. These free flows belong to `std.bytes`; existing receiver `len`,
`get`, `at`, checked slicing and `iter` conveniences follow the same rules.
All rows have `![]` unless a checked error is shown. Text encoding/decoding
remains in `std.codec.text` as specified in section 1.10; byte operations do not
silently decode text or normalize request bodies.

| `std.bytes` API | Return Type | Behavior And Cost |
|---|---|---|
| `empty()` | `bytes` | Empty sequence; `O(1)`. |
| `from_array(values: Array<u8>)` | `bytes` | Preserve every byte in order; `O(n)` or backing reuse. |
| `to_array(value: bytes)` | `Array<u8>` | Value-preserving conversion; `O(n)` or backing reuse. |
| `len(value: bytes)`, `is_empty(value: bytes)` | `usize`, `bool` respectively | `O(1)`. |
| `get<I ~ Index>(value: bytes, index: I)` | `Option<u8>` | `None` for an invalid index; `O(1)`. |
| `at<I ~ Index>(value: bytes, index: I)` | `u8 ![Error<IndexError>]` | Checked access; `O(1)`. |
| `slice<I ~ Index>(value: bytes, range: Range<I>)` | `bytes ![Error<IndexError>]` | The established checked range rules; may share immutable backing. |
| `iter(value: bytes)` | `Iter<u8>` | Lazy byte-order snapshot with `O(1)` construction and linear traversal. |
| `concat(parts: Array<bytes>)` | `bytes` | Ordered bulk assembly with checked total size; `O(part count + output bytes)`. |
| `join(parts: Array<bytes>, separator: bytes)` | `bytes` | Separator between parts only; empty input is empty; the same checked bulk-assembly bound. |

## 11. Iteration And Effectful Consumption

### 11.1 Pure Lazy Iterator Contract

`Iterable<T>.iter(self) -> Iter<T> ![]` produces a pure lazy snapshot.
`Iter<T>`'s step has type `() -> Option<(T, Iter<T>)> ![]`: `None` is exhausted;
`Some` carries an item and the remaining iterator. Reusing the same iterator
observes the same sequence, not a shared mutable cursor. Captured mutable
binding dependencies must not violate this property. This is a semantic
requirement beyond an empty escaping effect row.

The table preserves `Iter<T>`'s single type parameter. Pure adapters delay
their callbacks until consumption; they do not allocate a complete result on
construction. Effectful callbacks instead use the consumers in section 11.2,
so no callback effect is silently erased or attributed to adapter construction.
Standard producers preserve finiteness when their inputs are finite; inherited
limits still apply during iteration and callback execution.

| `Iter<T>` API | Return Type | Contract |
|---|---|---|
| `empty()` | `Iter<T> ![]` | Empty iterator. |
| `next()` | `Option<(T, Iter<T>)> ![]` | One step with no update to another observer's iterator. |
| `map<U>(f: T -> U ![])` | `Iter<U> ![]` | Apply a pure callback once per requested element. |
| `filter(predicate: T -> bool ![])` | `Iter<T> ![]` | Yield matching elements in input order. |
| `filter_map<U>(f: T -> Option<U> ![])` | `Iter<U> ![]` | Yield each `Some` payload and skip `None`. |
| `flat_map<U>(f: T -> Iter<U> ![])` | `Iter<U> ![]` | Concatenate the produced iterators in outer/inner order. |
| `chain(other: Iter<T>)` | `Iter<T> ![]` | All left elements, then all right elements. |
| `zip<U>(other: Iter<U>)` | `Iter<(T, U)> ![]` | Stop when either input is exhausted. |
| `enumerate()` | `Iter<(usize, T)> ![]` | Zero-based ordinal/item pairs. |
| `take(count: usize)`, `skip(count: usize)` | `Iter<T> ![]` | First at most `count` elements / remainder after at most that many. Zero is valid. |
| `take_while(predicate: T -> bool ![])`, `skip_while(predicate: T -> bool ![])` | `Iter<T> ![]` | Prefix selected/rejected by the predicate; preserve order. |
| `chunks(size: usize)` | `Iter<Array<T>> ![Error<CollectionError>]` | Non-overlapping chunks, with a possibly shorter final chunk. Reject zero at adapter construction. |

`CollectionError` is the ordinary std ADT `InvalidChunkSize(usize)`. A pure
lazy callback must not observe mutable captured state, be nondeterministic,
or request observable actions, including actions hidden by a default handler;
stateful/effectful work belongs in consumers, where the compiler retains its
capture, effect and requested-action summaries. Adapters retain their input
snapshots and constant additional cursor state; `flat_map` also retains its
active inner iterator and `chunks` allocates its current chunk. They must not
accumulate already consumed items in extra buffers. An input's documented
snapshot/view may still retain its underlying source allocation.

### 11.2 Consumers And Callback Effects

`effect E` uses existing effect-row polymorphism, not a new annotation or
generic-default syntax. Consumers execute callbacks sequentially in iterator
order, at most once per visited element. They preserve requested actions,
capture identities, determinism, inherited handlers/specs, limits and cancellation.
Callback failure stops consumption; already completed external actions are not
rolled back. No implicit retry, resampling or parallel execution is introduced.

| `Iter<T>` API | Return Type | Contract |
|---|---|---|
| `collect()`, `to_array()` | `Array<T> ![]` | Ordered bulk collection; `collect` is the array convenience form, not an unconstrained generic conversion. |
| `to_list()` | `List<T> ![]` | Ordered bulk collection. |
| `count()` | `usize ![]` | Number of elements. |
| `for_each<effect E>(f: T -> unit ![E])` | `unit ![E]` | Visit every element without allocating an output sequence. |
| `map_collect<U, effect E>(f: T -> U ![E])` | `Array<U> ![E]` | Effectful ordered map with eager output. |
| `filter_collect<effect E>(predicate: T -> bool ![E])` | `Array<T> ![E]` | Effectful ordered selection with eager output. |
| `fold<A, effect E>(initial: A, f: (A, T) -> A ![E])` | `A ![E]` | Left fold; empty returns `initial`. |
| `reduce<effect E>(f: (T, T) -> T ![E])` | `Option<T> ![E]` | First element initializes the accumulator; empty is `None`. |
| `any<effect E>(predicate: T -> bool ![E])`, `all<effect E>(predicate: T -> bool ![E])` | `bool ![E]` | Short circuit; empty is `false` / `true` respectively. |
| `find<effect E>(predicate: T -> bool ![E])` | `Option<T> ![E]` | First matching element; stop before later callbacks. |
| `position<effect E>(predicate: T -> bool ![E])` | `Option<usize> ![E]` | First matching ordinal. |
| `min()`, `max()` | `Option<T> ![]` | Require `T ~ Order`; first equivalent minimum/maximum; empty is `None`. |
| `min_by(compare: (T, T) -> i32 ![])`, `max_by(compare: (T, T) -> i32 ![])` | `Option<T> ![]` | The same rule using a supplied total order. |
| `sum()`, `product()` | `T ![]` | Require `T ~ std.math.Number`; left-to-right checked arithmetic; empty identities are zero/one. |

Full traversal is `O(n)` plus iterator/callback work; output allocation is
linear in emitted elements. Short-circuiting consumers must not visit later
items or execute their callbacks. A `Result` returned as ordinary callback data
is not automatically propagated; callers select explicit matching or an
`Error<E>` effect according to the existing error model.
Floating `min`/`max` consumers use `Order`, not `std.math`'s numeric NaN-selection
rule; callers choose the appropriate contract rather than silently mixing them.

For example, this contract permits I/O during mapping without making lazy
`Iter.map` implicitly effectful:

```etas
flow Visit(values: Array<i32>) -> Array<i32> {
    return values.iter().map_collect((value: i32) => {
        std.io.println("visited");
        return value + 1;
    });
}
```

The returned value is an array. Its construction executes the callbacks; the
flow infers the callback's residual `Error<IOError>` and retains the requested
`Console.stdout_write` actions even though the standard console handler is used.

## 12. Generic Sequence Algorithms

The following free flows belong to `std.collections`. They operate on arrays;
other containers use `iter().collect()` or a slice's `to_array()` explicitly.
They preserve the original input values and do not require new operators or
mutable-reference APIs.

| API | Return Type | Contract And Cost |
|---|---|---|
| `sort<T ~ Order>(values: Array<T>)` | `Array<T> ![]` | Ascending order; `O(n log n)` worst-case comparisons; equivalent items need not retain input order. |
| `sort_by<T>(values: Array<T>, compare: (T, T) -> i32 ![])` | `Array<T> ![]` | The same bounds with a supplied total order. |
| `stable_sort<T ~ Order>(values: Array<T>)` | `Array<T> ![]` | Ascending, preserving the order of equivalent items; `O(n log n)` comparisons. |
| `stable_sort_by<T>(values: Array<T>, compare: (T, T) -> i32 ![])` | `Array<T> ![]` | Stable sort under the supplied order. |
| `sort_by_key<T, K ~ Order>(values: Array<T>, key: T -> K ![])`, `stable_sort_by_key<T, K ~ Order>(values: Array<T>, key: T -> K ![])` | `Array<T> ![]` | Compute one pure key per input element in input order, then sort cached keys; the stable form preserves equivalent-key order. `O(n log n)` key comparisons plus `n` key calls. |
| `binary_search<T ~ Order>(values: Array<T>, key: T)` | `Result<usize, usize> ![]` | Sorted input; `Ok` is a matching index, `Err` the insertion position; `O(log n)` comparisons. Any equivalent matching index is allowed. |
| `binary_search_by<T>(values: Array<T>, compare_to_key: T -> i32 ![])` | `Result<usize, usize> ![]` | Sorted input and a monotone less/equal/greater classification relative to the sought key; same result rule. |
| `lower_bound<T ~ Order>(values: Array<T>, key: T)`, `upper_bound<T ~ Order>(values: Array<T>, key: T)` | `usize ![]` | First index not less than / strictly greater than the key; `len()` if absent; `O(log n)`. |
| `lower_bound_by<T>(values: Array<T>, compare_to_key: T -> i32 ![])`, `upper_bound_by<T>(values: Array<T>, compare_to_key: T -> i32 ![])` | `usize ![]` | The same boundary positions with monotone less/equal/greater classification; `O(log n)` comparisons. |
| `merge<T ~ Order>(left: Array<T>, right: Array<T>)` | `Array<T> ![]` | Merge sorted inputs; left precedes right on ties; `O(n + m)`. |
| `merge_by<T>(left: Array<T>, right: Array<T>, compare: (T, T) -> i32 ![])` | `Array<T> ![]` | The same stable merge under a supplied order. |
| `dedup<T ~ Eq>(values: Array<T>)` | `Array<T> ![]` | Remove adjacent equal runs, retaining their first element; `O(n)` equality tests. |
| `unique<T ~ Hash>(values: Array<T>)` | `Array<T> ![]` | Remove all later duplicates, retaining first-occurrence order; expected `O(n)`. |
| `partition<T, effect E>(values: Array<T>, predicate: T -> bool ![E])` | `(Array<T>, Array<T>) ![E]` | Matching/nonmatching arrays, each retaining input order; invoke the predicate once per item; `O(n)` plus callback work. |

Sort and search comparators obey section 10.2, including stability under repeated
calls. Their call counts/order are algorithm-dependent: a comparator must not
be used to perform logging, approvals, model calls or mutable-counter updates.
`partition`, unlike comparison-based algorithms, explicitly supports callback
effects. Sorting may use `O(n)` auxiliary storage. Search results use comparator
equivalence, not a separate equality test. Inputs to merge/search must use the
same order; the APIs do not silently sort them. Violating a sorted-input or
comparison-law precondition gives no meaningful search-order guarantee, but
must not cause unsafe access or bypass checked execution.
The `Err` insertion position from binary search is ordinary result data, not
an `Error<usize>` action; it does not need postfix `?`.

## 13. Option And Result APIs

These are ordinary value combinators in `std.option` / `std.result`. They do
not redefine `?`, whose sole error-capture rule remains `Error<E>` to
`Result<T, E>`. Callback effects are inferred/preserved through `effect E`;
only the selected branch executes a callback. Unless shown otherwise the row
is `![]`. Each operation is `O(1)` plus callback/value-copy work.

| Owner / API | Return Type | Contract |
|---|---|---|
| `Option<T>.is_some()`, `is_none()` | `bool` | Inspect the variant. |
| `Option<T>.unwrap()` | `T` | `Some` payload; `None` calls non-returning `abort`. It is not an error handler or Result conversion. |
| `Option<T>.unwrap_or(default: T)` | `T` | Payload or supplied value; ordinary eager argument evaluation applies. |
| `Option<T>.unwrap_or_else<effect E>(fallback: () -> T ![E])` | `T ![E]` | Evaluate fallback only on `None`. |
| `Option<T>.map<U, effect E>(f: T -> U ![E])` | `Option<U> ![E]` | Transform only `Some`. |
| `Option<T>.and_then<U, effect E>(f: T -> Option<U> ![E])` | `Option<U> ![E]` | Bind only `Some`; do not introduce a nested Option. |
| `Option<T>.filter<effect E>(predicate: T -> bool ![E])` | `Option<T> ![E]` | Keep `Some` only if its predicate succeeds. |
| `Option<T>.or_else<effect E>(fallback: () -> Option<T> ![E])` | `Option<T> ![E]` | Call fallback only on `None`. |
| `Option<T>.ok_or<F>(error: F)` | `Result<T, F>` | `Some` becomes `Ok`; `None` becomes `Err`. Error data is an eagerly evaluated argument. |
| `Option<T>.ok_or_else<F, effect E>(error: () -> F ![E])` | `Result<T, F> ![E]` | Construct error data only on `None`. |
| `Result<T, F>.is_ok()`, `is_err()` | `bool` | Inspect the variant. |
| `Result<T, F>.unwrap()` | `T` | `Ok` payload; `Err` calls `abort`; it does not re-perform `Error<F>`. |
| `Result<T, F>.unwrap_or(default: T)` | `T` | Payload or supplied value; argument evaluation is eager. |
| `Result<T, F>.unwrap_or_else<effect E>(fallback: F -> T ![E])` | `T ![E]` | Call fallback with error data only on `Err`. |
| `Result<T, F>.map<U, effect E>(f: T -> U ![E])` | `Result<U, F> ![E]` | Transform only `Ok`; preserve `Err`. |
| `Result<T, F>.map_err<G, effect E>(f: F -> G ![E])` | `Result<T, G> ![E]` | Transform only `Err`; preserve `Ok`. |
| `Result<T, F>.and_then<U, effect E>(f: T -> Result<U, F> ![E])` | `Result<U, F> ![E]` | Bind only `Ok`. |
| `Result<T, F>.or_else<G, effect E>(f: F -> Result<T, G> ![E])` | `Result<T, G> ![E]` | Recover only `Err`. |
| `Result<T, F>.ok()`, `err()` | `Option<T>`, `Option<F>` respectively | Keep the selected payload or return `None`. |

`Some`/`None` and `Ok`/`Err` remain the existing enum constructors. Empty
escaping rows do not imply that `unwrap` returns on every input or that a
callback is non-suspending. Lazy fallback APIs must not evaluate the unselected
branch, and must retain its checked may-effect upper bound in the callable's
contract without pretending that it executed.
