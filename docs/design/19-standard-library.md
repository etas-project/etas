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
| `std.char` | `is_whitespace(c: char)` | `bool`; Unicode White_Space property |
| `std.char` | `is_alphabetic(c: char)` | `bool`; Unicode Alphabetic property |
| `std.char` | `is_alphanumeric(c: char)` | `bool`; Alphabetic or Unicode number category |
| `std.char` | `is_control(c: char)` | `bool`; Unicode General_Category Cc |
| `std.char` | `is_ascii_digit(c: char)` | `bool`; ASCII `0` through `9` only |
| `std.char` | `to_u32(c: char)` | `u32`; scalar numeric value |
| `std.char` | `from_u32(value: u32)` | `Option<char>`; reject surrogates and values beyond U+10FFFF |
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
