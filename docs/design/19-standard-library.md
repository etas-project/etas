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
| Console I/O | `std.io` | [Console API table](03-agents-tools-prompts-memory.md#24-standard-io) |
| TCP, TLS and byte streams | `std.net.tcp`, `std.tls`, `std.stream` | [Substrate API tables](06-effect-system-and-inference.md#11-minimal-standard-actions) |
| Files, wire HTTP, codecs, secrets and browser transport | `std.fs`, `std.http.codec`, `std.codec.text`, `std.secret`, `std.crypto`, `std.browser.protocol` | [Substrate API tables](06-effect-system-and-inference.md#11-minimal-standard-actions) |
| Prompt construction and encoding | `std.agent.prompt`, `std.agent.schema` | [Prompt values](03-agents-tools-prompts-memory.md#4-prompt-values) |
| Session history and context publication | `std.agent.session` | [Section 16.4](#164-session-history-and-context-publication) |
| Persistent stores | `std.memory` | [Section 16](#16-persistent-stores-and-session-publication) |
| Concurrent combinators | `std.task` | [Full signatures and boundary cases](16-concurrency.md#4-standard-combinators) |
| Checked code generation | `std.compiler` | [Compiler API table](18-reflection-and-checked-code-generation.md#61-standard-library-surface) |
| Commands and sandbox profiles | `std.command`, `std.host.command`, `std.host.sandbox` | [Command sandboxing](06-effect-system-and-inference.md#3-command-sandboxing); lifecycle gaps in section 17 |
| Time and randomness | `std.runtime.time`, `std.random` | [Section 14](#14-time-and-randomness) |
| Resource ownership and cleanup | `std.resource`, stream/transport/filesystem APIs | [Section 15](#15-managed-resource-lifecycles) |
| Incremental builders | `std.text.TextBuilder`, `std.bytes.BytesBuilder` | [Section 1.13](#113-incremental-text-and-byte-builders) |
| Bit sets and bounded deques | `std.collections` | [Section 10.9](#109-bit-sets), [section 10.10](#1010-bounded-deques) |
| Grouping, counting and Top-K | `std.collections` | [Section 12.2](#122-grouping-counting-and-selection) |
| Effectful pull sources | `std.stream` | [Section 15.4](#154-effectful-pull-sources) |
| Storage/query encoding | `std.memory` | [Section 16.5](#165-storage-encoding-and-query-contracts) |

An index entry is not evidence of API completeness. [Section 17](#17-api-coverage-and-open-contracts)
separates specified interfaces from contracts that still need design decisions.

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

| Position / Range | Contract |
|---|---|
| `len`, search results, `char_indices`, `grapheme_indices` | UTF-8 byte positions, including the positions paired with scalars/graphemes. |
| `char_get`, `char_slice` / `grapheme_slice` | Scalar ordinals / grapheme ordinals, not byte offsets. |
| Input index | `I ~ Index`; negative or unrepresentable positions are invalid. No implicit numeric conversion. |
| `Range<I>` | Normalize `[i, j)` to `i..j` and `(i, j]` to `(i + 1)..(j + 1)` with checked arithmetic. Ordered, in-bounds empty intervals are valid. |
| Byte slice | Both normalized endpoints must be UTF-8 scalar boundaries. Never clamp, round or reverse an invalid interval. |
| Invalid input | Optional accessor: `None`; checked accessor: `Error<TextError>`. |
| Source syntax | Generic `s[i]` and direct range slicing on `string` are not defined; select the coordinate-specific API. |

Index conversion follows the existing
[indexing rules](02-general-programming-constructs.md#21-numeric-conversion-and-indexing).

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

| Condition | Outcome |
|---|---|
| Invalid position / normalized interval | `InvalidIndex` / `InvalidRange`. |
| In-bounds byte endpoint inside a scalar | `NotCharBoundary`. |
| Invalid numeric input / unrepresentable numeric result | `ParseFailure` / `NumericOverflow`. |
| `repeat` output-length arithmetic overflow | `LengthOverflow`. |
| Invalid format template | `InvalidFormat`. |
| Failed search / optional access | `None`, not an error action. |
| Checked access | `Error<TextError>`; postfix `?` captures it into `Result`. |
| Parsing / formatting | Already returns `Result`; `?` does not unwrap it. |
| Allocation failure / resource exhaustion | Existing execution-limit/fault rules; checked size arithmetic, no length wrap or truncated success. Not a recoverable `TextError` by default. |

```etas
let piece: Result<string, TextError> = std.text.byte_slice("Etas", [1, 3))?;
let number: Result<i32, TextError> = std.text.parse_i32("42");
```

| Operation | Performance Contract |
|---|---|
| Byte length / emptiness | `O(1)`. |
| Scalar/grapheme count or ordinal lookup | May scan input; no cached index is required. Repeated `char_get` can be quadratic. |
| Sequential scalar traversal | Linear in input bytes. |
| `concat`, `join` | Linear in part count plus output bytes; checked bulk assembly, not repeated prefix copying. |
| Lazy text iterators | No complete result array; producing individual segments may allocate. |
| Slices / trims / split segments | Sharing is permitted, not guaranteed. A small retained result may keep the source backing alive. |

Incremental builders use the existing restricted `var self` rule in section
1.13. This chapter adds no receiver-update operator, regular-expression syntax
or general interpolation syntax.

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

### 1.13 Incremental Text And Byte Builders

`std.text.TextBuilder` and `std.bytes.BytesBuilder` are ordinary opaque
value types, not shared mutable buffers. The table shows updating receivers
explicitly; they require a direct local `var` and cannot suspend.

| Owner / API | Result | Contract |
|---|---|---|
| `TextBuilder.new()` / `BytesBuilder.new()` | Respective builder `![]` | Empty value; no host action. |
| `TextBuilder.append(var self, part: string)` | `unit ![]` | Append exact UTF-8 text; no normalization or implicit formatting. |
| `TextBuilder.append_char(var self, value: char)` | `unit ![]` | Append the scalar's UTF-8 encoding. |
| `BytesBuilder.append(var self, part: bytes)` | `unit ![]` | Append exact bytes. |
| `BytesBuilder.push(var self, value: u8)` | `unit ![]` | Append one byte. |
| Each builder's `len_bytes()`, `is_empty()` | `usize`, `bool`, both `![]` | Constant-time logical observations. |
| Each builder's `clear(var self)` | `unit ![]` | Clear this binding; previously copied builders/results are unchanged. |
| `TextBuilder.build()` / `BytesBuilder.build()` | `string ![]` / `bytes ![]` | Materialize an immutable value without consuming or clearing the builder. |

| Builder Boundary | Rule |
|---|---|
| Snapshot / failure | Copying preserves a value snapshot. Updates use normal commit-on-return semantics; unwind does not expose a partial receiver update. |
| Cost | A linear append history has total work O(part count + appended bytes), excluding final builds and retained snapshots. No repeated copying of the entire prefix. |
| Retained versions | Chunk sharing/COW is allowed; retained snapshots account for their own storage. Repeated build may copy the full output each time. |
| Limits | Check length arithmetic before commit; allocation faults and inherited execution-limit handling apply. No silent truncation or byte-to-text conversion. |

```etas
var output = std.text.TextBuilder.new();
output.append("Result: ");
output.append("accepted");
let text = output.build();
```

## 2. Task And Scheduling APIs

These APIs specify structured task execution and handler-based scheduling.
Their scope, cancellation, and continuation semantics are defined in
[Concurrency](16-concurrency.md#121-handler-based-scheduling).

### 2.1 Standard Types And Actions

| Type / Context | Contract |
|---|---|
| `std.task.Task<T, E>` | Inferred result type and `effect E`; retain owner scope, requested/trace summary and determinism across imports and helpers. |
| Repeated `await` | Observe the same completed result; do not execute the task again. Handles may be shared within their valid lifetime. |
| `std.task.scheduler.SchedulerHandle` | Opaque live run handle; distinct from the reusable scheduler implementation value. The owner is inferred, not selected by a string. |
| Lifetime | Tasks/handles cannot escape their scope, including inside aggregates or closures. No model-schema or ordinary checkpoint serialization. |
| Managed scope | Supplied by entry/combinators; `run` creates a nested scope and passes its handle to the callback. |

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

| Transfer Rule | Requirement |
|---|---|
| Recognition | `run`/`start` are resolved compiler-known std contracts; `requeue`/`park` are registered transfer primitives, not magic names user flows can copy. |
| Identity | Transfer keeps the suspended task's identity, authority and owner. `yield` normally requeues with `()`. |
| Parking | Handle completed and pending tasks with atomic completion/registration. Waiting is not runnable readiness. |
| `never` | Leave the current arm for its scheduler; do not continue after transfer or abort the process. A plain `unit` queue insertion is not equivalent. |
| Validation | Check owner, continuation and result type before transfer; invariant failure unwinds the scope rather than dropping a live continuation. |
| Implementation | Etas `Scheduler` code selects ready work; `run` creates the live handle. No new keyword, `spawn` syntax or imported live singleton. |

### 2.3 Effects, Errors, And Default Handling

Default handlers cover ordinary task use; business code need not handle
`Async` manually.

| Case | Effect / Failure Rule |
|---|---|
| `start(job)` | Contribute `E` and the full job summary immediately, even if never awaited. |
| `await(task)` | Preserve typed failures; never forge a `T` for a failed task. |
| Handling `Async` | Remove covered scheduling actions, not child-work effects/actions. Record default handling in metadata. |
| Typed errors | No implicit `Error<_>` catch-all or generated error union; `?` still captures exactly one residual Error type. |
| Unhandled failure | Fail-fast: cancel siblings, finish cleanup, propagate the original typed error. Inherited handlers may recover before it becomes unhandled. |
| Asynchronous delivery | Restore the correct task/handler context; do not resume with a wrong-type value. Catching an I/O error cannot clear scope cancellation. |
| Custom handler | May change scheduling policy, not lifetime, authority, budgets, tracing or cleanup obligations. |
| Waiting substrate | Non-blocking completion or managed blocking offload. `start` alone cannot turn a blocking host call into asynchronous I/O or authorize forged completion. |

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

| Scheduler Contract | Requirement |
|---|---|
| `ReadyTask` | Copyable metadata, not a task/continuation or execution grant; id is run-local. |
| `select` input | Nonempty snapshot sorted by enqueue order, oldest first; each enqueue gets a new order. |
| `select` result | Index into that snapshot. `var self` may retain value-semantic counters/configuration, not live continuations. |
| Checked behavior | Deterministic, non-suspending, empty escaping/requested summaries; `![]` alone or an unchecked annotation is insufficient. |
| Generic/imported calls | Preserve evidence; unknown callback/import behavior cannot satisfy the proof. |
| Update discipline | Receiver exclusivity and execution budgets still apply. |

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

| Run Isolation | Rule |
|---|---|
| Implementation state | Fresh local value copy and owned queue/wait state; updates do not change the supplied value or another run. |
| Live resources | Never carry live task/continuation/scheduler state across runs or into globals. |
| Construction | An ordinary flow can build configuration; no separate factory protocol. |
| Physical sharing | Worker pools/I/O drivers may be shared, not logical scope ownership or queues. |
| Combinators / nested `run` | Combinators inherit the selected policy; nested `run` creates a new instance, using `DefaultScheduler` when omitted. |

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

| Dispatch Event | Required Behavior |
|---|---|
| Run entry | Resolve and copy the implementation before the callback. |
| New / yielded / woken task | Enqueue runnable work; take a nonempty snapshot, call `select`, dispatch the still-live entry through checked primitives. |
| Completion/cancellation race | Refresh readiness; never resume a dead task. |
| Out-of-range selection | Contract violation: fail/unwind the scope, not arbitrary fallback selection. |
| No ready task, outstanding waits | Park for completion; no busy-yield loop. |
| Scope closure | Drain/cancel owned work before releasing handlers and continuations. |
| Default FIFO | Select index zero; enqueue at the tail. Cooperative fairness, not CPU preemption or a provider-latency bound. |
| Custom scheduling | May change ready order/state, not OS scheduling, I/O drivers or protected execution rules. |

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

| Protocol Value | Required Meaning / Validation |
|---|---|
| `C` | Checked agent-method identity, revision and tool surface, not a runtime string or authority-granting type argument. Retain the upper-bound summary; it does not certify a transparent record. |
| `ToolRef` | Resolved package/module/item id and implementation revision, shared with reflection metadata. Construction/editing never registers a tool or grants authority. |
| Tool-name resolution | `step` resolves against `C`; an unknown name yields `Stopped(InvalidToolCall(...))`. `invoke` still validates arguments/authority, including caller-built records. |
| `AssistantTurn` | One protocol assistant turn, not application `Message<T>` provenance. Requests live only in `StepDecision.CallTools`, not a second editable copy. |
| `provider_data` | Preserve required continuation data through the public protocol. It cannot override profile, tools, policy or budget; no inaccessible host-only continuation object. |
| Content | Text/JSON alternatives do not imply a complete multimodal API. |
| Produced `Final` | Successfully decoded/validated against `Schema<O>`; a caller-constructed value is not evidence of a model response. |
| Produced `CallTools` | Nonempty requests, not executed tools. A step cannot simultaneously return a final answer; assembly rejects invalid transitions and empty request batches. |
| Invalid output | Corresponding `Stopped` case; no hidden repair call. |
| `StepDiagnostic` | Stable machine-readable `code`, explanatory `message`, optional schema/argument `path`; never branch on message wording. |
| Usage | Missing fields mean unknown, not zero. Preserve reported cost; conservative accounting still enforces parent limits. Caller-built usage is not protected execution evidence. |

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

Section 5.2 specifies the numeric edge cases for these signatures.

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

| Numeric Evidence | Rule |
|---|---|
| Ownership | Source-declared compiler-known spec; preserve evidence across imports and checked-code restoration as for `Index`. |
| Admitted types | Primitive signed/unsigned integer widths, `isize`, `usize`, `f32`, `f64`. User declarations cannot extend this primitive family. |
| Generic operations | Same-type comparisons and supported binary arithmetic. Narrower operations need their own primitive evidence; `Number` does not make unsigned unary negation valid. |
| Operands/results | Same concrete type and width; normal literal inference, explicit numeric conversions. |
| Nominal wrappers | Use declared conversion or their own helper; strings, bools, collections and unconstrained generics do not satisfy `Number`. |
| Arity | `min`/`max` are binary. Iterator reductions in section 11 define empty-input behavior separately. |

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

### 5.3 Integer Arithmetic

`std.math.Integer ~ Number` and `std.math.Float ~ Number` are resolved
standard numeric specs. `Integer` admits the primitive integer families;
`Float` admits only `f32`/`f64`. They do not authorize user-defined primitive
representations. All APIs below have `![]`; checked failures are Result data.

```etas
public enum ArithmeticError {
    Overflow,
    DivisionByZero,
    InvalidShift,
}

public enum MathError {
    InvalidBounds,
}
```

| `std.math` API | Result | Contract |
|---|---|---|
| `min_value<I ~ Integer>()`, `max_value<I ~ Integer>()` | `I` | Representable bounds at the selected width; unsigned minimum is zero. |
| `bit_width<I ~ Integer>()` | `u32` | Width of `I`; pointer-sized widths follow the checked target profile. |
| `checked_add<I ~ Integer>(a: I, b: I)`, `checked_sub<I ~ Integer>(a: I, b: I)`, `checked_mul<I ~ Integer>(a: I, b: I)` | `Result<I, ArithmeticError>` | Exact integer result or `Err(Overflow)`; no narrowing or wrap. |
| `checked_div<I ~ Integer>(a: I, b: I)`, `checked_rem<I ~ Integer>(a: I, b: I)` | `Result<I, ArithmeticError>` | Quotient truncates toward zero; remainder has the dividend's sign. Zero divisor fails. Signed minimum divided by -1 overflows; its remainder is zero. |
| `checked_abs<I ~ Integer>(value: I)` | `Result<I, ArithmeticError>` | Unsigned identity; signed-minimum magnitude fails. |
| `checked_pow<I ~ Integer>(base: I, exponent: u32)` | `Result<I, ArithmeticError>` | Nonnegative integer power; exponent zero returns one, including `0^0`; overflow fails. |
| `div_euclid<I ~ Integer>(a: I, b: I)`, `rem_euclid<I ~ Integer>(a: I, b: I)` | `Result<I, ArithmeticError>` | `a = q*b + r`, `0 <= r < abs(b)` mathematically; reject zero divisor or unrepresentable result. |
| `wrapping_add<I ~ Integer>(a: I, b: I)`, `wrapping_sub<I ~ Integer>(a: I, b: I)`, `wrapping_mul<I ~ Integer>(a: I, b: I)` | `I` | Arithmetic modulo `2^width`; interpret signed results as two's complement. |
| `saturating_add<I ~ Integer>(a: I, b: I)`, `saturating_sub<I ~ Integer>(a: I, b: I)`, `saturating_mul<I ~ Integer>(a: I, b: I)` | `I` | Clamp the exact mathematical result to `I`'s bounds. |
| `checked_shl<I ~ Integer>(value: I, count: u32)`, `checked_shr<I ~ Integer>(value: I, count: u32)` | `Result<I, ArithmeticError>` | Reject `count >= width`. Left shift drops high bits; signed right shift extends the sign, unsigned right shift inserts zeros. This checks shift count, not multiplication overflow. |
| `rotate_left<I ~ Integer>(value: I, count: u32)`, `rotate_right<I ~ Integer>(value: I, count: u32)` | `I` | Rotate the fixed-width bit representation by `count % width`. |
| `count_ones<I ~ Integer>(value: I)`, `leading_zeros<I ~ Integer>(value: I)`, `trailing_zeros<I ~ Integer>(value: I)` | `u32` | Count in the fixed-width representation; zero has `width` leading/trailing zeros. |
| `clamp<T ~ Number>(value: T, lower: T, upper: T)` | `Result<T, MathError>` | Reject reversed bounds or any NaN input; otherwise return the value or the appropriate bound. |

### 5.4 Floating-Point Operations

These `std.math` APIs preserve `F ~ Float` and have `![]`. They do not
convert through a different public width or read the host's ambient rounding mode.

| API | Result | Contract |
|---|---|---|
| `is_nan<F ~ Float>(x: F)`, `is_finite<F ~ Float>(x: F)`, `is_infinite<F ~ Float>(x: F)`, `is_normal<F ~ Float>(x: F)`, `is_sign_negative<F ~ Float>(x: F)` | `bool` | Classify the encoding; normal excludes zero, subnormal, infinity and NaN. Sign testing includes signed zero/NaN. |
| `floor<F ~ Float>(x: F)`, `ceil<F ~ Float>(x: F)`, `trunc<F ~ Float>(x: F)`, `round<F ~ Float>(x: F)` | `F` | Toward -infinity / +infinity / zero / nearest with ties to even. Remain floating values, not implicit integer conversions. |
| `sqrt<F ~ Float>(x: F)` | `F` | Square root; negative nonzero input produces NaN; preserve signed zero. |
| `exp<F ~ Float>(x: F)`, `exp2<F ~ Float>(x: F)`, `ln<F ~ Float>(x: F)`, `log2<F ~ Float>(x: F)`, `log10<F ~ Float>(x: F)` | `F` | Natural/base-2 exponentials and natural/base-2/base-10 logarithms. Log zero is -infinity; negative input is NaN. |
| `sin<F ~ Float>(x: F)`, `cos<F ~ Float>(x: F)`, `tan<F ~ Float>(x: F)`, `asin<F ~ Float>(x: F)`, `acos<F ~ Float>(x: F)`, `atan<F ~ Float>(x: F)` | `F` | Radians; asin/acos outside [-1, 1] and trig of infinity produce NaN. |
| `atan2<F ~ Float>(y: F, x: F)` | `F` | Quadrant-aware angle in [-pi, pi], including signed-zero quadrants. |
| `pow<F ~ Float>(base: F, exponent: F)` | `F` | Real power: negative finite base with noninteger finite exponent is NaN; integer exponent parity controls its sign. |
| `hypot<F ~ Float>(x: F, y: F)` | `F` | Nonnegative Euclidean magnitude; avoid intermediate square overflow when the result is representable. Infinity dominates NaN. |
| `copysign<F ~ Float>(magnitude: F, sign: F)` | `F` | Copy only the sign bit, preserving magnitude/payload bits. |
| `total_cmp<F ~ Float>(a: F, b: F)` | `i32` | The bit-total `Order` rule in section 10.2, not numeric `min`/`max` selection. |
| `pi<F ~ Float>()`, `tau<F ~ Float>()`, `e<F ~ Float>()`, `epsilon<F ~ Float>()` | `F` | Width-specific constants; epsilon is the gap from one to the next larger representable value. |

| Floating Boundary | Rule |
|---|---|
| Ordinary domain failure | NaN, not an `Error` action; overflow yields signed infinity and underflow retains subnormals/signed zero. |
| Nonfinite rounding | floor/ceil/trunc/round preserve infinity, NaN and signed zero; no integer-range fault. |
| Power exceptions | `x^0 = 1`, `1^y = 1`, including NaN operands in those positions. For signed zero/infinity with finite nonzero exponent, odd integer exponents retain the negative sign; exponent sign selects zero versus infinity. |
| Infinite exponent | For magnitude below/above one, positive infinity exponent gives zero/infinity and negative infinity reverses them; magnitude one gives one. Other NaN cases produce NaN. |
| Reproducibility | The numeric compatibility profile pins algorithm/version, rounding and documented error bounds. Constant evaluation and runtime use the same profile; changing it invalidates numerical replay/cache assumptions. |
| Accuracy | sqrt, rounding and constants are correctly rounded at the selected width. Transcendentals must publish per-operation accuracy bounds and pass profile conformance tests; no unspecified host-libm substitution. |
| Scope | Big integers, decimal financial arithmetic, complex numbers and locale formatting remain separate library designs. |


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

| Boundary | Required Step |
|---|---|
| Entry | Reject an empty roster with `Error<GroupError>.raise(EmptyParticipants)`. |
| Initial state | Set `state = initial`, `turns = 0`; test `stop_when` once. Success returns `PredicateSatisfied` with zero turns. |
| Each allowed turn | Call index `turns % participants.len()`; after success replace state and increment turns once. |
| After each turn | Test `stop_when`; success wins even on the final allowed turn. |
| Exhausted turn allowance | Return state and `TurnLimitReached`. With a nonempty roster and zero allowance, return the initial state unless the initial predicate succeeded. |
| Counting | One turn is one callback, across all participants; three participants and five turns invoke `0, 1, 2, 0, 1`. Every call starts at zero. |

### 6.2 Effects, Failure And Ownership

Calls are sequential. The next participant observes the state returned by its
predecessor. `E` joins callback escaping rows; requested actions and capture
dependencies propagate separately, including Agentic actions when inference
is reachable. `stop_when` must be deterministic, non-suspending, independent of
mutable captures and free of requested actions; `![]` alone is insufficient.
It is evaluated once at each specified boundary.

The combinator inherits handlers, policies, budgets, cancellation, trace and
session context. `max_turns` is an algorithm bound. Inherited execution limits
can terminate the call earlier and keep their existing failure behavior.
Callback errors and execution faults propagate immediately. Completed external
actions retain their effects. The combinator performs no implicit retries or
state rollback. Its implementation is an ordinary checked Etas loop.

## 7. Virtual Trace Stages

| `std.runtime.trace` API / Value | Type | Contract |
|---|---|---|
| `VirtualStages(stages: Array<TraceLabel>)` | `TraceLabel ![]` | Construct an immutable composite label; execute no stage or host request. |
| `Logical` | `TraceLabel` | Existing logical-stage label. |

| Trace-Label Rule | Requirement |
|---|---|
| Tree shape | Preserve order, duplicates and nesting. Empty/single-element inputs still construct composite nodes. |
| Storage | Opaque nominal type; a private pure binding validates identities and allocates the tree. Checkpoint encoding preserves it. |
| Annotation references | `@trace(VirtualStages([Researcher, Critic]))` resolves callable references to identity/revision/provenance labels. Import aliases preserve identity; unresolved/non-callable references are errors. |
| Ordinary calls | Require actual `TraceLabel` values; annotation elaboration is not general implicit conversion. |
| Evidence | Labels describe a plan, not proof of inference, approval or tool execution. Actual physical actions/usage are recorded separately. |
| Fusion / replay | Preserve source-stage mapping and physical invocation identities. Display strings cannot replace checked identities. |

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
| `declassify<T, U>(value: Secret<T>, strategy: DeclassificationStrategy<T, U>, reason: string)` | `Sanitized<U> ![Error<SecurityError>]` | Standard handling consumes `Secret.declassify`; retain it in requested/default metadata, authorize release and preserve provenance. |

| Strategy / Value | Contract |
|---|---|
| Strategy handle | Immutable opaque reference issued through a trusted binding; carries implementation, revision and exact input/output types. |
| Strategy body | May be checked Etas `T -> Result<U, SecurityError> ![]`; binding evidence associates it with the protected contract. |
| Selection | Explicit argument on every call. Application/EDK helpers such as `sanitize.html` must define and select their own strategy. |
| Result | `Sanitized<U>` records the processing contract, not universal safety. Preserve nested trust labels and data-channel placement; instruction-channel promotion needs separate evidence. |
| Secret input | `Secret<T>` is a typed trust wrapper. It is not `std.secret.SecretValue<K>`, an opaque host-secret handle requiring its own authorized operations. |
| Ownership | Import strategies/APIs from `std.security.declassify`; granting release uses the existing runtime authority mechanism. |

### 8.2 Sanitization

| Sanitization Step | Outcome |
|---|---|
| Validate strategy and input/output evidence | Missing: `Unavailable`; incompatible: `StrategyMismatch`. |
| Execute deterministic validation/transformation | Only the strategy's protected frame receives the untrusted payload; rejection raises `Error<SecurityError>`. |
| Validate output | Construct `Sanitized<U>` with processing provenance. |
| Authority | No secret access, implicit model call or host request. Etas may implement transformation; protected bindings own wrapper access/construction. |

### 8.3 Secret Release And Audit

| Release Step | Outcome |
|---|---|
| Trim/check reason | Empty reason: `InvalidReason`. |
| Validate strategy identity/revision/types | Missing implementation: `Unavailable`; mismatched evidence: `StrategyMismatch`. |
| Authorize exact strategy before payload access | Denied: `Denied`; inherited trace specs and approvals still apply. |
| Execute and validate | Rejection: `Rejected`; success: `Sanitized<U>` with release provenance. |

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
approvals, then performs the protected transformation. The residual public row
is `![Error<SecurityError>]`; authorization follows requested
`Secret.declassify`, not a fabricated escaping `Secret` tag. A raw perform with
no covering handler also carries that action in its escaping row.
Custom handler support must preserve the
protected release check before any secret access or result-wrapper creation.

| Audit / Protection Rule | Requirement |
|---|---|
| Record | Strategy identity/revision, input/output type identities, caller trace, authorization outcome, reason and result status. |
| Redaction | The reason is caller-supplied audit data; redact normally. Exclude secret payloads/intermediates from ordinary errors, logs and traces. |
| Failure | Record authorization and strategy failures through the existing trace mechanism. |
| Protected operations | Raw access and release-provenance construction require protected bindings; casts/wrapper constructors cannot discard nested secrecy or forge release. |
| Limits | Inherit budgets/cancellation. Identity release still requires an explicit strategy and ordinary authorization. |

## 9. Implementation Conformance

| Ownership | Rule |
|---|---|
| API contract | All supported execution paths implement the same specified behavior. Unsupported paths report a diagnostic rather than silently selecting another implementation. |
| Etas source | Ordinary flows, enums, records and policy bodies belong in `etas_std/library/std`. |
| Primitive bindings | Own primitive arithmetic, opaque storage, secret authority and protected provenance operations. |

### 9.1 Acceptance Evidence

API availability belongs in the implementation's test matrix. Representation
changes must preserve the specified equality, error, ownership and cost
contracts; they do not justify changing public behavior.

| Area | Required Acceptance Evidence |
|---|---|
| Container invariants | Reject external metadata writes and forged views; `len`, emptiness, lookup and iteration agree after every update. |
| Equality/hash/order | Primitive and nominal keys, collisions, equal records with different construction order, differing map/set insertion orders, signed zeros/NaNs, lawful custom comparators and stable duplicate behavior. |
| Snapshot preservation | Retain old values, slices and iterators while updating/removing/clearing current values; verify old observations and nested payload sharing remain correct. |
| Cost and storage | Measure comparisons, node/header copies, allocations and retained slots separately at growing sizes, for both reusable backing and shared snapshots. Include array draining, map construction, ordered deletion and long-lived queue churn. |
| Iterator consumption | Empty/exhausted iterators, repeated use, adapter composition, zero-sized chunks, short circuiting and bounded lazy allocation. |
| Effects and capture | Pure lazy adapters reject state-dependent/effectful callbacks; consumers propagate callback effects/actions/capture dependencies across source imports and package metadata. |
| Failure behavior | Invalid indexes use the specified error effect; callback failure stops later work without rolling back completed external actions; `unwrap` terminates rather than silently supplying a value. |
| Installed delivery | Build and run against the installed source SDK, including separately compiled imports. Testing a different backend alone does not certify source SDK behavior or performance. |

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
| `Hash ~ Eq` | Behavioral type spec | Requires `hash(self, state: Hasher) -> Hasher ![]`. Feed the value into a value-semantic hash state; equal values must produce the same final hash from the same initial state. |
| `Order` | Behavioral type spec | Requires `compare(self, other: Self) -> i32 ![]`; negative, zero and positive mean less, equivalent and greater. Comparison defines a stable total order and equivalence relation. |
| `Hasher.new(seed: u64 = 0)` | `Hasher ![]` | Create an empty deterministic non-cryptographic state. The default constructor performs no implicit entropy or host action. |
| `Hasher.write(data: bytes)` | `Hasher ![]` | Return a state extended by these bytes, preserving the input state. Processing is linear in input bytes. |
| `Hasher.finish()` | `u64 ![]` | Observe the current digest without changing the state. Hash values are not a persistent encoding, signature, or cryptographic digest. |

| Equality / Hash / Order Rule | Requirement |
|---|---|
| Standard identity | Recognize the resolved `Eq` spec, not its spelling. Explicit conformance must pass the same evidence checks; it cannot assert arbitrary non-reflexive equality. |
| Primitive evidence | Integers, booleans, chars, strings and bytes receive `Eq`/`Hash`. |
| Structural evidence | Finite tuples/records/enums and supported collections require corresponding component evidence. Ordinary nominal data gets checked structural hashing unless an explicit conforming implementation is supplied; no derive keyword. |
| Hash representation | Hash logical values and nominal identity, not addresses/layout; frame fields and variants unambiguously. |
| Non-data values | Functions, handlers, secrets and resource handles do not acquire equality/hash from a comparable implementation address. |
| Raw floats | `f32`/`f64` have no `Eq`/`Hash`: NaN breaks reflexivity. A nominal wrapper over normalized integer/byte encodings can define a key; wrapping an unchanged float field is insufficient. |
| Float `Order` | Negative encodings in reverse bit order, then nonnegative encodings in bit order. Distinguish signed zeros/NaN encodings; identical encodings compare as zero. This neither implies `Eq` nor changes float `==`. |
| Callback behavior | Hashing/comparators are deterministic, non-suspending, independent of mutable captured/resource state, and have no observable requested actions. `![]` alone is insufficient. |
| Algebraic laws | User lawfulness is an implementation obligation, not proved by conformance. Violations must not cause memory unsafety or authority bypass. |
| Reproducibility | Equal hashes are required within the same algorithm/version and initial state, not across releases. |
| Container hashing | Seeds cannot change iteration or logical equality; map/set hashing is independent of insertion order and private hasher. |

### 10.3 Arrays, Lists And Slices

Unless a row shows an effect, its escaping row is `![]`. `from_iter` consumes
the pure iterator in order without first making a second complete input copy.

| Owner / API | Return Type | Behavior And Cost |
|---|---|---|
| `Array<T>.new()` | `Array<T>` | Empty array; `O(1)`. |
| `Array<T>.from_iter(items: Iter<T>)` | `Array<T>` | Bulk construction; `O(n)` excluding iterator work. |
| `Array<T>.init<effect E>(count: usize, f: usize -> T ![E])` | `Array<T> ![E]` | Call f once per index from 0 to count-1 in order; O(count) plus callback work. Zero calls none. Failure stops later callbacks; no partial array returned. |
| `Array<T>.repeat(count: usize, value: T)` | `Array<T>` | Value copies in every position; O(count). T must permit repetition; this cannot clone an affine handle or introduce a shared mutable object. |
| `Array<T>.split_at<I ~ Index>(index: I)` | `(Slice<T>, Slice<T>) ![Error<IndexError>]` | Views before/from index; 0 and len are valid; O(1) construction. |
| `Array<T>.retain<effect E>(predicate: T -> bool ![E])` | `Array<T> ![E]` | Return a filtered value, not receiver mutation; input-order callbacks once per item, O(n) plus callback work. |
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
| `List<T>.clear()` | `List<T>` | Empty value; old snapshots remain valid. Releasing unshared nodes may take `O(n)`. |
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
| `OrderedSet<T>.union(other: OrderedSet<T>)`, `intersection(other: OrderedSet<T>)`, `difference(other: OrderedSet<T>)`, `symmetric_difference(other: OrderedSet<T>)` | `OrderedSet<T> ![Error<CollectionError>]` | Ascending merge, O(n + m) comparisons/output work; equivalent representatives prefer the left input. |
| `OrderedSet<T>.is_subset(other: OrderedSet<T>)`, `is_disjoint(other: OrderedSet<T>)` | `bool ![Error<CollectionError>]` | Short-circuit ordered merge; O(n + m) comparisons or better. |
| `OrderedMap<K, V>.split_at_key(key: K)` | `(OrderedMap<K, V>, OrderedMap<K, V>)` | Keys below / at-or-above key; same comparator; O(log n) tree-path work with sharing. |
| `OrderedSet<T>.split_at_value(value: T)` | `(OrderedSet<T>, OrderedSet<T>)` | Same split rule for values. |
| Either container's `clear()` | Respective container | Empty value retaining the comparator. |

Rows without an explicit effect have `![]`. Binary ordered-set operations
require the same comparator identity/profile; copied containers preserve it.
Different custom comparator instances raise `IncompatibleComparator`, even if
users believe them equivalent; do not attempt extensional flow equality.
Inverted or mutually exclusive bounds
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
| `push_pop(value: T, priority: P)` | `(PriorityQueue<T, P>, T)` | Equivalent to push then pop, including FIFO ties; O(log(n + 1)) comparisons, and empty input returns the new value with an empty queue. |
| `replace_top(value: T, priority: P)` | `(PriorityQueue<T, P>, Option<T>)` | Pop old top then push new value; on empty insert and return None; O(log(n + 1)). |
| `to_sorted_array()` | `Array<T>` | Descending priority, FIFO ties; O(n log(n + 1)) comparisons; original queue unchanged. |
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
| `starts_with(value: bytes, prefix: bytes)`, `ends_with(value: bytes, suffix: bytes)` | `bool` | Exact byte comparison; empty prefix/suffix matches. |
| `find(value: bytes, pattern: bytes)`, `rfind(value: bytes, pattern: bytes)` | `Option<usize>` | First/last starting byte offset; absent is `None`. Empty pattern matches at `0` / `len(value)` respectively. |
| `contains(value: bytes, pattern: bytes)` | `bool` | Equivalent to `find(...).is_some()`; no text decoding. |

### 10.9 Bit Sets

`std.collections.BitSet` represents a fixed universe [0, size), with immutable
value semantics. Let w be its number of storage words and k its set-bit count.

| API | Result | Contract / Cost |
|---|---|---|
| `BitSet.new(size: usize)` | `BitSet ![]` | All bits clear; zero-size universe is valid; O(w). |
| `BitSet.from_indices(size: usize, indices: Iter<usize>)` | `BitSet ![Error<IndexError>]` | Insert checked indices; duplicates harmless; O(w + input count). |
| `universe_len()`, `count()`, `is_empty()` | `usize`, `usize`, `bool`, all `![]` | Universe size / set-bit count / no set bits; O(1) / O(w) or cached / O(w) or cached. |
| `contains<I ~ Index>(index: I)` | `bool ![]` | False for invalid/out-of-universe indices; O(1). |
| `insert<I ~ Index>(index: I)`, `remove<I ~ Index>(index: I)` | `BitSet ![Error<IndexError>]` | Checked value update; O(1) with reusable storage, up to O(w) with shared backing. |
| `union(other: BitSet)`, `intersection(other: BitSet)`, `difference(other: BitSet)`, `symmetric_difference(other: BitSet)` | `BitSet ![Error<CollectionError>]` | Word-wise operation; equal universe lengths required, otherwise IncompatibleBounds; O(w). |
| `is_subset(other: BitSet)`, `is_disjoint(other: BitSet)` | `bool ![Error<CollectionError>]` | Same universe check; short-circuit word scan, O(w). |
| `complement()`, `clear()` | `BitSet ![]` | Flip within the universe / clear all bits; preserve size, mask unused tail bits; O(w). |
| `iter()` | `Iter<usize> ![]` | Ascending set indices; O(w + k) traversal, no array materialization. |

Eq/Hash include both universe length and membership, never padding or storage
capacity. This type is not implicitly interchangeable with Set<usize>.

### 10.10 Bounded Deques

`std.collections.BoundedDeque<T>` is a value container, not a channel.
`OverflowPolicy` is the enum `Reject, DropFront, DropBack`.

| API | Result | Contract |
|---|---|---|
| `BoundedDeque<T>.new(capacity: usize, overflow: OverflowPolicy = Reject)` | `BoundedDeque<T> ![Error<CollectionError>]` | Positive capacity; zero is InvalidCapacity. |
| `push_front(value: T)`, `push_back(value: T)` | `(BoundedDeque<T>, Option<T>) ![Error<CollectionError>]` | If full, Reject raises Full; otherwise evict the selected old end before inserting, and return the evicted value. No eviction returns None. |
| `pop_front()`, `pop_back()` | `(BoundedDeque<T>, Option<T>) ![]` | Empty returns unchanged value and None; normal deque order. |
| `peek_front()`, `peek_back()` | `Option<T> ![]` | Observe without removal. |
| `len()`, `capacity()`, `is_empty()`, `is_full()` | `usize`, `usize`, `bool`, `bool`, all `![]` | O(1) observations. |
| `iter()`, `clear()` | `Iter<T> ![]`, `BoundedDeque<T> ![]` | Snapshot traversal / empty value retaining capacity and overflow policy. |

Deque amortized/COW costs apply. Eq/Hash require corresponding T evidence and
include capacity, overflow policy and logical sequence. Overflow never blocks
a task or silently changes policy; backpressure belongs to stream/synchronization.

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
| `once(value: T)`, `repeat_n(value: T, count: usize)` | `Iter<T> ![]` | One item / a finite repeated value; preserve value-copy and affine restrictions; O(1) cursor state. |
| `init(count: usize, f: usize -> T ![])` | `Iter<T> ![]` | Lazy indexed generation; no callback at construction, increasing indices when consumed. |
| `unfold<S>(state: S, step: S -> Option<(T, S)> ![])` | `Iter<T> ![]` | Explicit state transition; None ends the sequence. Reusing an iterator repeats the same observation; no shared mutable cursor. |
| `peek()` | `Option<T> ![]` | Observe the next element without returning a different cursor. May recompute pure work; not an implicit memoization promise. |
| `scan<A, U>(initial: A, step: (A, T) -> (A, U) ![])` | `Iter<U> ![]` | Thread explicit accumulator state, emitting one U per input; does not emit initial itself. |
| `Iter<Iter<U>>.flatten()` | `Iter<U> ![]` | Concatenate in outer/inner order, equivalent to flat_map(identity). |
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
| `windows(size: usize)` | `Iter<Array<T>> ![Error<CollectionError>]` | Overlapping full windows, advancing one item each time; no output when the input is shorter than `size`. Reject zero at construction. |
| `step_by(step: usize)` | `Iter<T> ![Error<CollectionError>]` | Yield input ordinals `0, step, 2 * step, ...`; reject zero at construction. |

`CollectionError` is an ordinary standard ADT:

```etas
public enum CollectionError {
    InvalidChunkSize(usize),
    InvalidWindowSize(usize),
    InvalidStep(usize),
    InvalidCapacity(usize),
    Full,
    IncompatibleBounds,
    IncompatibleComparator,
}
```

| Lazy Adapter Rule | Requirement |
|---|---|
| Callbacks | Deterministic; no mutable captured-state observation or observable actions, including default-handled actions. Stateful/effectful callbacks use section 11.2 consumers. |
| Storage | Constant cursor state plus retained input; `flat_map` also keeps its active inner iterator. `chunks`/`windows` may buffer up to `size` items, not all consumed prefixes. |
| Output snapshots | Previously emitted arrays remain unchanged. Copying a window can cost `O(size)`; overlapping output does not promise linear total copying. |
| Failures | Validate zero sizes/steps at construction. Callback/iteration execution still obeys inherited limits. |

An input's documented snapshot/view may still retain its source allocation.

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
| `Iter<(A, B)>.unzip()` | `(Array<A>, Array<B>) ![]` | One traversal, preserving both component orders; O(n) work/storage. |
| `min_by_key<K ~ Order>(key: T -> K ![])`, `max_by_key<K ~ Order>(key: T -> K ![])` | `Option<T> ![]` | Compute each visited key once; first equivalent extremum, None on empty. |
| `count()` | `usize ![]` | Number of elements. |
| `nth<I ~ Index>(index: I)` | `Option<T> ![]` | Zero-based access by traversal; invalid/out-of-range is `None`; no shared cursor is advanced. |
| `last()` | `Option<T> ![]` | Traverse to the final item; empty is `None`. |
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

### 12.1 Sorting, Search And Basic Transforms

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

| Algorithm Boundary | Rule |
|---|---|
| Comparators | Section 10.2 applies; call order/count is algorithm-dependent. No logging, approvals, model calls or mutable-counter updates. |
| Effectful predicate | `partition` explicitly supports callback effects and preserves input order within each output. |
| Storage | Sorting may use `O(n)` auxiliary storage; original values remain unchanged. |
| Search / merge precondition | Inputs use the same comparator order; no implicit sorting. Search uses comparator equivalence, not separate `==`. |
| Invalid precondition/law | No meaningful search-order guarantee, but no unsafe access or authority bypass. |
| Binary-search `Err` | Insertion position is ordinary result data, not `Error<usize>`; no postfix `?` is needed. |

### 12.2 Grouping, Counting And Selection

These flows also belong to `std.collections`. They return existing containers;
no Counter/MultiMap language construct or implicit shared mutation is needed.

| API | Result | Contract / Cost |
|---|---|---|
| `group_by<T, K ~ Hash, effect E>(items: Iter<T>, key: T -> K ![E])` | `Map<K, Array<T>> ![E]` | Global grouping; keys in first-seen order and items in input order. Key called once per item; expected O(n), O(n) output. |
| `count_by<T, K ~ Hash, effect E>(items: Iter<T>, key: T -> K ![E])` | `Map<K, usize> ![E]` | First-seen key order; checked counts; expected O(n), O(distinct keys) storage. |
| `frequencies<T ~ Hash>(items: Iter<T>)` | `Map<T, usize> ![]` | count_by with identity. |
| `merge_counts<K ~ Hash>(left: Map<K, usize>, right: Map<K, usize>)` | `Map<K, usize> ![]` | Checked addition on conflicts; left order then new right keys; expected O(n + m). Zero is retained as explicit data. |
| `group_adjacent_by<T, K ~ Eq>(items: Iter<T>, key: T -> K ![])` | `Iter<(K, Array<T>)> ![]` | Only consecutive equal keys; no sorting. Cache boundary lookahead/key. O(n) traversal; buffer the current run, which may be large. |
| `top_k<T ~ Order>(items: Iter<T>, k: usize)`, `bottom_k<T ~ Order>(items: Iter<T>, k: usize)` | `Array<T> ![]` | Highest/lowest k in descending/ascending order; equal items retain input order. Zero consumes none; k > n returns all sorted. |
| `top_k_by<T>(items: Iter<T>, k: usize, compare: (T, T) -> i32 ![])`, `bottom_k_by<T>(items: Iter<T>, k: usize, compare: (T, T) -> i32 ![])` | `Array<T> ![]` | Same tie/output rules with a lawful total comparator. |
| `top_k_by_key<T, K ~ Order>(items: Iter<T>, k: usize, key: T -> K ![])`, `bottom_k_by_key<T, K ~ Order>(items: Iter<T>, k: usize, key: T -> K ![])` | `Array<T> ![]` | Cache exactly one key per visited item; no callback when k = 0. |
| `select_nth<T ~ Order, I ~ Index>(values: Array<T>, index: I)` | `(Array<T>, T) ![Error<IndexError>]` | Return a reordered value and its nth item; preceding/following items compare <=/>= it. Original unchanged; equivalent items need not be stable. O(n) comparisons. |
| `select_nth_by<T, I ~ Index>(values: Array<T>, index: I, compare: (T, T) -> i32 ![])` | `(Array<T>, T) ![Error<IndexError>]` | Same partition contract; invalid index, including any index on empty, raises IndexError. |
| `is_sorted<T ~ Order>(items: Iter<T>)`, `is_sorted_by<T>(items: Iter<T>, compare: (T, T) -> i32 ![])` | `bool ![]` | Nondecreasing; empty/singleton true; stop at first inversion; O(n) comparisons. |
| `partition_point<T>(values: Array<T>, predicate: T -> bool ![])` | `usize ![]` | True-prefix/false-suffix input; return first false or len; O(log n) predicate calls. Invalid monotonicity gives no meaningful boundary, never unsafe access. |
| `dedup_by<T>(values: Array<T>, equal: (T, T) -> bool ![])` | `Array<T> ![]` | Collapse adjacent equivalence runs, retaining first representatives; lawful equivalence required; O(n) comparisons. |
| `dedup_by_key<T, K ~ Eq>(values: Array<T>, key: T -> K ![])`, `unique_by_key<T, K ~ Hash>(values: Array<T>, key: T -> K ![])` | `Array<T> ![]` | Adjacent/global duplicate removal; first representatives retained, key computed once per item. O(n) / expected O(n). |
| `rotate_left<T>(values: Array<T>, count: usize)`, `rotate_right<T>(values: Array<T>, count: usize)` | `Array<T> ![]` | Empty unchanged; count modulo len otherwise; O(n) work, original unchanged. |

| Algorithm Boundary | Rule |
|---|---|
| Top-K cost | For k > 0, retain at most min(k, n) candidates; O(n log(k + 1) + k log(k + 1)) comparisons and O(k) working storage as an upper bound. Do not fully materialize/sort n items when k is small. |
| Pure callbacks | Same deterministic/non-suspending/no-requested-action checks as comparators and lazy Iter adapters. An empty escaping row alone is insufficient. |
| Effectful grouping | group_by/count_by preserve E and callback summaries, call once per input in order, and stop on failure; no rollback of completed actions. |
| Counts / sizes | Checked usize arithmetic and existing execution-fault rules; no wrap. Global grouping and one adjacent run can grow with input; inherited limits still apply. |
| Infinite input | Top-K/global grouping need exhaustion; lazy grouping may never finish an infinite equal-key run. Apply take or a bounded source consumer explicitly. |
| Value semantics | No in-place mutation visible through another value; reuse/COW is an implementation choice. Existing snapshot/copy costs still apply. |

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
| `Option<T>.zip<U>(other: Option<U>)` | `Option<(T, U)>` | `Some` only when both inputs are `Some`; argument evaluation is eager. |
| `Option<Option<T>>.flatten()` | `Option<T>` | Remove one Option layer; either `None` becomes `None`. |
| `Option<Result<T, F>>.transpose()` | `Result<Option<T>, F>` | `None` becomes `Ok(None)`; `Some(Ok(v))` becomes `Ok(Some(v))`; `Some(Err(e))` becomes `Err(e)`. |
| `Result<T, F>.is_ok()`, `is_err()` | `bool` | Inspect the variant. |
| `Result<T, F>.unwrap()` | `T` | `Ok` payload; `Err` calls `abort`; it does not re-perform `Error<F>`. |
| `Result<T, F>.unwrap_or(default: T)` | `T` | Payload or supplied value; argument evaluation is eager. |
| `Result<T, F>.unwrap_or_else<effect E>(fallback: F -> T ![E])` | `T ![E]` | Call fallback with error data only on `Err`. |
| `Result<T, F>.map<U, effect E>(f: T -> U ![E])` | `Result<U, F> ![E]` | Transform only `Ok`; preserve `Err`. |
| `Result<T, F>.map_err<G, effect E>(f: F -> G ![E])` | `Result<T, G> ![E]` | Transform only `Err`; preserve `Ok`. |
| `Result<T, F>.and_then<U, effect E>(f: T -> Result<U, F> ![E])` | `Result<U, F> ![E]` | Bind only `Ok`. |
| `Result<T, F>.or_else<G, effect E>(f: F -> Result<T, G> ![E])` | `Result<T, G> ![E]` | Recover only `Err`. |
| `Result<T, F>.ok()`, `err()` | `Option<T>`, `Option<F>` respectively | Keep the selected payload or return `None`. |
| `Result<Result<T, F>, F>.flatten()` | `Result<T, F>` | Remove one Result layer with the same error type; preserve the first `Err`. |
| `Result<Option<T>, F>.transpose()` | `Option<Result<T, F>>` | `Ok(None)` becomes `None`; `Ok(Some(v))` becomes `Some(Ok(v))`; `Err(e)` becomes `Some(Err(e))`. |

`Some`/`None` and `Ok`/`Err` remain the existing enum constructors. Empty
escaping rows do not imply that `unwrap` returns on every input or that a
callback is non-suspending. Lazy fallback APIs must not evaluate the unselected
branch, and must retain its checked may-effect upper bound in the callable's
contract without pretending that it executed.

## 14. Time And Randomness

### 14.1 Time Values And Arithmetic

`std.runtime.time` is the canonical module; clocks/deadlines are not calendar
or timezone APIs. Pure conversions return Result data; observed time and waits
use Clock actions. Existing `WallTime(Duration)` limits use the same Duration.

| Public Type | Definition / Invariant |
|---|---|
| `Duration` | Value record `{ millis: u128 }`; nonnegative elapsed milliseconds. Zero is valid. |
| `Time` | Value record `{ unix_millis: i128 }`; UTC milliseconds relative to the Unix epoch, including earlier dates. Not a monotonic deadline. |
| `Instant` | Opaque monotonic sample tied to one clock domain/run. Not ordinary serializable or cross-run comparable data. |
| `TimeError` | Enum: `Overflow`, `InvalidOrder`, `ClockMismatch`, `Unavailable`, `Host(string)`. |

| `std.runtime.time` API | Result | Contract |
|---|---|---|
| `Duration.from_millis(value: u128)` | `Duration ![]` | Exact construction. |
| `Duration.from_seconds(value: u128)` | `Result<Duration, TimeError> ![]` | Checked multiplication by 1000. |
| `Duration.as_millis()`, `as_seconds()` | `u128 ![]` | Milliseconds / whole seconds rounded down. |
| `Duration.checked_add(other: Duration)`, `checked_sub(other: Duration)`, `checked_mul(factor: u128)` | `Result<Duration, TimeError> ![]` | Overflow fails; negative subtraction is `InvalidOrder`. |
| `Time.checked_add(duration: Duration)`, `checked_sub(duration: Duration)` | `Result<Time, TimeError> ![]` | Exact timestamp arithmetic or `Overflow`. |
| `Time.since(earlier: Time)` | `Result<Duration, TimeError> ![]` | Nonnegative elapsed wall time; reversed order fails. Not suitable for timeout enforcement. |
| `Instant.since(earlier: Instant)` | `Result<Duration, TimeError> ![]` | Reject different clock domains or reversed samples. |
| `now()` | `Time ![Error<TimeError>]` | Request/default-handle `Clock.now`; wall time may move backwards. |
| `monotonic()` | `Instant ![Error<TimeError>]` | Request/default-handle `Clock.monotonic`; samples do not decrease within a domain. |
| `elapsed(start: Instant)` | `Duration ![Error<TimeError>]` | Read monotonic time and validate its relation to `start`. |
| `sleep(duration: Duration)` | `unit ![Error<TimeError>]` | Request/default-handle `Clock.sleep`; suspend the current task without blocking the scheduler. |

| Time Boundary | Rule |
|---|---|
| Sleep / deadlines | Timer uses monotonic time; wake no earlier than the requested duration unless cancelled. Actual wake may be later. Zero completes without a delay; it is not a guaranteed yield. |
| Inheritance | Local timeouts only shorten the parent deadline. Partial progress/retry cannot reset the enclosing deadline or budget. |
| Cancellation | Keeps structured unwind semantics; no successful timer completion is forged. Catching an incidental adapter error cannot uncancel the scope. |
| Replay | Record clock observations and wake decisions; replay uses them, not current wall time. A saved wall timestamp cannot recreate a live Instant. |
| Precision | Millisecond API precision does not promise millisecond scheduler latency. Calendar parsing, timezone and business-day arithmetic belong in separate packages. |

### 14.2 Seeded And Secure Randomness

`std.random` separates deterministic value computation from external entropy.
`Rng` is a value-semantic, non-cryptographic generator; copying it copies its
sequence position, not a shared mutable source.

| Public Type / Action | Definition / Invariant |
|---|---|
| `Rng` | Opaque ordinary value holding algorithm/version and state; no live host resource. |
| `RandomError` | Enum: `InvalidRange`, `Unavailable`, `Host(string)`. |
| `Random.entropy(count: usize) -> bytes` | Standard action for protected platform entropy. It is not a Time or Secret action, and adds no source keyword. |

| `std.random` API | Result | Contract |
|---|---|---|
| `Rng.new(seed: u64)` | `Rng ![]` | Deterministic seed initialization; no implicit clock/entropy request. |
| `Rng.next_u64()` | `(Rng, u64) ![]` | Return successor state and the next 64-bit pseudorandom word. |
| `Rng.next_f64()` | `(Rng, f64) ![]` | Uniform choice from the 53-bit grid in [0, 1); never one. |
| `Rng.int_range<I ~ std.math.Integer>(lower: I, upper: I)` | `Result<(Rng, I), RandomError> ![]` | Uniform half-open interval by rejection sampling; reject `lower >= upper`. Do not use biased modulo reduction. |
| `Rng.choose<T>(values: Array<T>)` | `(Rng, Option<T>) ![]` | Uniform index; empty input returns unchanged state and `None`. |
| `Rng.shuffle<T>(values: Array<T>)` | `(Rng, Array<T>) ![]` | Value-preserving unbiased permutation; preserve original input; linear swaps/draws in expectation. |
| `secure_bytes(count: usize)` | `bytes ![Error<RandomError>]` | Request/default-handle `Random.entropy`; exact byte count from the platform cryptographic source, or failure. Zero returns empty without requesting entropy. |

| Randomness Boundary | Rule |
|---|---|
| Seeded reproducibility | Algorithm/version and output test vectors are part of the std compatibility profile; no host-random implementation substitution. Preserve successor state explicitly. |
| Concurrency | Sharing the same Rng value repeats its stream. No implicit independent stream splitting or hidden mutable global generator. |
| Security | Rng is not for keys/tokens. Secure entropy must fail closed when unavailable, never fall back to Rng, timestamps or predictable data. |
| Data versus secrecy | Returned bytes are ordinary data, not an automatic `Secret` wrapper or permission to disclose a secret. Applications remain responsible for their use and exposure. |
| Trace / replay | Entropy is a nondeterministic requested action. Sensitive payload capture follows protected trace storage; replay needs retained authorized bytes or reports unavailable replay, never silently draws new bytes. |
| Bounds | All draws/allocations obey inherited limits; rejection sampling is not exempt from cancellation/budget checks. |

## 15. Managed Resource Lifecycles

### 15.1 Scope And Cleanup

`std.resource.scope` exposes the cleanup contract already required by managed
task frames. It is a compiler-recognized std flow, not a `using`/`finally`
keyword or a general shared-reference feature.

| Public Type / API | Result / Definition | Contract |
|---|---|---|
| `CleanupFailure` | Record: `resource_kind: string`, `code: string`, `message: string` | Redacted diagnostic, not a live resource or permission token. |
| `ResourceError` | Enum: `CleanupFailed(Array<CleanupFailure>)` | Failures encountered while releasing a normally completed scope. |
| `std.resource.scope<R, effect E>(body: () -> R ![E])` | `R ![E, Error<ResourceError>]` | Establish a fresh resource owner, run body, join owned child work and release owned resources before returning. |

| Lifecycle Boundary | Required Behavior |
|---|---|
| Registration | Successful standard resource creation registers with the nearest resource scope, or otherwise the managed task/root owner, before exposing the handle. Failed acquisition cleans up any partial allocation. Ordinary helper returns do not create an owner boundary. |
| Escape | Newly owned handles cannot escape through R, captured closures, handlers or outer mutable bindings. Compiler summaries preserve owner relationships; unknown external escape is rejected. |
| Existing handles | Borrowing an outer resource does not transfer ownership or close it at inner-scope exit. Ordinary data/results may escape. |
| Alias | Handle copies identify one resource; they do not copy a connection/file. Closing invalidates all aliases. Concurrent use obeys the resource's access contract. |
| Release order | Reverse successful acquisition order, respecting dependencies; children finish/cancel first. Explicit close marks a registration released; it is not closed twice. |
| Wrapping | Admitted TLS upgrade transfers the TCP registration into the TLS handle atomically. Every raw TCP alias becomes unusable; handshake failure closes the connection. Pre-admission validation/busy failure does not transfer ownership. |
| Normal return | Release everything; if cleanup fails, raise `ResourceError.CleanupFailed` rather than return a successful value. |
| Existing failure / finish / cancellation | Preserve the primary control outcome, continue cleanup, and attach cleanup failures to trace diagnostics. No replacement of an original typed error by an unrelated cleanup error. |
| Authority | Cleanup retains action/provenance/spec checks. Denied remote flushing or protocol operations are not performed; local disposal still prevents leaked handles and the denial is recorded. |
| Observability | Requested close/flush actions and their possible failures belong to the owning scope summary, including implicit cleanup. Cleanup is not an invisible purity escape. |
| Limits | Cleanup has a bounded runtime cleanup allowance, not a fresh application budget. Forced process termination cannot guarantee remote cleanup or durable completion. |
| Extension | Standard bindings register resource release primitives. This API does not allow an arbitrary library to forge ownership/evidence by naming a method `close`. |

### 15.2 Stream And Transport Values

| Type / Configuration | Public Contract |
|---|---|
| `std.stream.ByteLimit` | Record `{ bytes: usize }`; zero is valid; no signed count or token-to-byte reinterpretation. |
| `std.stream.Timeout` | Alias of `std.runtime.time.Duration`; APIs use `Option<Timeout>`, with `None` retaining parent deadlines. |
| `std.net.tcp.Host` | Opaque validated hostname/IP value. `Host.parse(text: string) -> Result<Host, NetworkError> ![]` rejects empty input, schemes, paths and embedded ports; does not resolve DNS. `to_string() -> string ![]` returns its canonical spelling. |
| `std.net.tcp.Port` | Opaque connect-port value. `Port.new(value: u16) -> Result<Port, NetworkError> ![]` accepts 1..65535; `value() -> u16 ![]` observes it. |
| `std.tls.Host` | Re-export/alias of `std.net.tcp.Host`, never a second incompatible nominal type. |
| `std.net.tcp.NetworkError` | Record `{ code: string, message: string }`. Stable codes: invalid_input, resolve_failed, refused, timed_out, unavailable, host. Messages are redacted diagnostics, not machine-readable codes. |
| `std.tls.TlsError` | Record `{ code: string, message: string }`. Stable codes: invalid_config, busy, certificate, hostname, protocol, timed_out, host. Validation failure never falls back to plaintext. |
| `TcpOptions` | Record `{ timeout: Option<Timeout>, nodelay: bool }`; `DefaultTcpOptions` is `{ timeout = None, nodelay = true }`. |
| `TlsVersion` / `TlsTrust` | Enums `V12, V13` / `Platform, Bundle(bytes)`; bundles contain public trust anchors, not secret client keys. |
| `TlsConfig` | Record `{ timeout: Option<Timeout>, minimum: TlsVersion, trust: TlsTrust, alpn: Array<string> }`; `DefaultTlsConfig` uses None, V12, Platform and an empty ALPN list. |
| TLS validation | Always check certificate chain and server identity; no implicit insecure fallback. Empty ALPN makes no application-protocol claim; requested unsupported negotiation fails with `TlsError`. |

The existing connect/handshake APIs gain ordinary default arguments
`options = DefaultTcpOptions` and `config = DefaultTlsConfig`. DNS/connect and
handshake each respect a single local timeout and the inherited parent deadline.

| Additional `std.stream` API | Result | Requested Action / Contract |
|---|---|---|
| `read_exact<S ~ ByteStream>(stream: S, count: usize, timeout: Option<Timeout> = None)` | `bytes ![Error<StreamError>]` | `Stream.read<S>`; exactly count bytes or `UnexpectedEof`. Consumed prefixes are not restored. Zero validates an open readable handle, then completes without a read. |
| `write_all<S ~ ByteStream>(stream: S, body: bytes, timeout: Option<Timeout> = None)` | `unit ![Error<StreamError>]` | `Stream.write<S>`; extends the existing API with an optional whole-write deadline. Partial failure is not rollback. |
| `flush<S ~ ByteStream>(stream: S, timeout: Option<Timeout> = None)` | `unit ![Error<StreamError>]` | `Stream.flush<S>`; bounded adapter flush, not remote acknowledgment or filesystem durability. |

`StreamError` additionally includes `UnexpectedEof`, `Busy` and
`UnsupportedOperation`. File mode violations use the latter. `ByteStream`
does not statically promise both read and write access; no general subtyping or
unchecked readable/writable cast is introduced.

| Same-Resource Operations | Required Admission Rule |
|---|---|
| Two reads, or two writes | Reject the later overlapping call with `StreamError.Busy`; do not silently split input or interleave output. Admission is atomic across aliases. |
| TCP/TLS read plus write | Permit one reader and one writer concurrently. Protocol-level request ordering remains the application's responsibility. |
| FileStream read plus write | Reject overlap: the stream has one shared file position. Separate opens have distinct cursors but still share file/region authority and conflict checks. |
| Compound operation | `read_exact`/`read_until_limit` hold the read slot for the whole call; `write_all`/`flush` hold the write slot. Internal chunks do not release it or reacquire as unrelated calls. |
| Close | Mark closed atomically, wake pending operations and release the resource. Close is idempotent, never Busy; no successful I/O may be fabricated after closure. |
| TLS upgrade | Require no active I/O before transfer; pre-admission busy/validation failure leaves TCP ownership unchanged. Once handshake begins, failure closes the underlying connection. |
| Cancellation | Release admission slots on every unwind path. A local typed error cannot clear cancellation of the enclosing scope. |
| Analysis / ordering | Known conflicting calls must not be parallelized automatically. Explicit overlapping calls retain the dynamic guard; the winning admission/order is trace-visible. Different resources are not serialized globally. |

### 15.3 Filesystem Lifecycle And Bounded Operations

`std.io.IOError` is the shared nominal error enum, re-exported by `std.fs`
and `std.runtime.error`. Independent same-named error types are not equivalent.

| Supporting Type | Contract |
|---|---|
| `IOError` | Existing Eof/Interrupted/PermissionDenied/InvalidUtf8/Host plus `NotFound`, `AlreadyExists`, `InvalidInput`, `NotDirectory`, `IsDirectory`, `DirectoryNotEmpty`, `Closed`, `TimedOut`, `LimitExceeded`, `Unsupported`. |
| `FileMode` | Enum: `Read`, `WriteTruncate`, `Append`, `ReadWrite`, `CreateNew`. |
| `FileStream<R>` | Opaque region-indexed handle satisfying `ByteStream`; registration/aliases follow section 15.1. |
| `FsStat` | Record `{ is_file: bool, is_dir: bool, len: u64 }`; flags describe the resolved target and len is file bytes. Directory length is zero, not a portable entry count. |

All following signatures use `R ~ std.fs.Region`, inherit path-escape guards,
and leave `Error<IOError>` after standard handling.

| `std.fs` API | Result | Requested Action / Contract |
|---|---|---|
| `open<R ~ Region>(path: WorkspacePath<R>, mode: FileMode)` | `FileStream<R> ![Error<IOError>]` | `Fs.read<R>` and/or `Fs.write<R>` according to mode; a dynamic mode conservatively requires both. Stream identity retains the mode and concrete region. |
| `read_bytes_limited<R ~ Region>(path: WorkspacePath<R>, max_bytes: usize)` | `bytes ![Error<IOError>]` | `Fs.read<R>`; inclusive size cap, `LimitExceeded` rather than truncation. Reading/stat races cannot bypass the bound. |
| `list_limited<R ~ Region>(path: WorkspacePath<R>, max_entries: usize, max_bytes: usize)` | `List<WorkspacePath<R>> ![Error<IOError>]` | `Fs.list<R>`; cap entry count and UTF-8 bytes of relative names; no successful partial listing. |
| `create_dir<R ~ Region>(path: WorkspacePath<R>, recursive: bool = false)` | `unit ![Error<IOError>]` | `Fs.create_dir<R>`; existing directory succeeds; other existing targets fail. Nonrecursive creation requires existing parents. |
| `rename<R ~ Region>(source: WorkspacePath<R>, destination: WorkspacePath<R>)` | `unit ![Error<IOError>]` | `Fs.rename<R>`; same region, no replacement of an existing destination, no cross-device copy fallback. |
| `remove_file<R ~ Region>(path: WorkspacePath<R>)` | `unit ![Error<IOError>]` | `Fs.remove_file<R>`; unlink the file/link, not the link's target; missing raises NotFound. |
| `remove_dir<R ~ Region>(path: WorkspacePath<R>)` | `unit ![Error<IOError>]` | `Fs.remove_dir<R>`; empty directories only. Recursive deletion remains an explicit checked traversal. |
| `sync<R ~ Region>(file: FileStream<R>)` | `unit ![Error<IOError>]` | `Fs.sync<R>`; request backend stable-storage synchronization or Unsupported, not merely flush user buffers. |

| Open / Mutation Boundary | Rule |
|---|---|
| Read | Existing file, read-only; no creation. |
| WriteTruncate / Append | Create if missing; respectively truncate on open / place each accepted write at current end. Append does not imply atomic multi-write records. |
| ReadWrite | Existing file, no truncation or creation. |
| CreateNew | Atomically create a missing file for writing; existing target is AlreadyExists. |
| Close / durability | Close releases ownership; it does not implicitly fsync. `atomic_replace` promises visibility atomicity, not crash durability. |
| Failure | Observe partial/uncertain effects honestly; no automatic retry of writes, append, rename or deletion. |
| Region provenance | Stream reads/writes over FileStream remain covered by the originating FileIO region; erasing its concrete type cannot broaden authority. |


### 15.4 Effectful Pull Sources

`std.stream.Source<T, E>` is a managed, single-cursor resource, not the
repeatable pure `Iter<T>`. E is its latent effect row, including release work.
The result schemes below infer G as the union of E and callback row F; G is
notation for an inferred row, not a new source operator or generic argument.

| Public Type | Definition / Contract |
|---|---|
| `Source<T, effect E>` | Opaque managed source with a checked owner, current state, pull/release callbacks and latent action/effect/capture summaries. No ordinary serialization or independent cursor copy. |
| `SourceError` | Enum: `Busy`, `Closed`, `InvalidChunkSize(usize)`, `InvalidLineLimit(usize)`, `LineTooLong`, `InvalidUtf8`. |
| Source aliases | Refer to the same cursor. One successful next advances it for all aliases; overlapping next fails Busy rather than fetching two values unpredictably. |

| `std.stream` API / Method | Result Scheme | Contract |
|---|---|---|
| `unfold<S, T, effect E>(state: S, step: S -> (S, Option<T>) ![E], release: S -> unit ![E])` | `Source<T, E> ![]` | Register state and callbacks in the active managed owner; do not invoke them on construction. E covers both callbacks. |
| `from_iter<T>(items: Iter<T>)` | `Source<T, E> ![]`, E empty | Adapt a pure iterator to one managed cursor; does not make the original iterator effectful or consume it yet. |
| `chunks<S ~ ByteStream>(stream: S, size: usize, timeout: Option<Timeout> = None)` | `Source<bytes, E> ![Error<SourceError>, Error<StreamError>]` | E is the residual `Error<StreamError>` row for stream read/close. Reject zero size; each item has 1..size bytes; EOF ends. |
| `lines<S ~ ByteStream>(stream: S, max_line_bytes: usize, timeout: Option<Timeout> = None)` | `Source<string, E> ![Error<SourceError>, Error<StreamError>]` | Same stream E; positive line bound, strict UTF-8. Strip LF or CRLF; yield a final unterminated nonempty line once. Empty lines are valid items. |
| `Source<T, E>.next()` | `Option<T> ![E, Error<SourceError>]` | None means exhausted, never a swallowed transport error. Successful step commits successor state before returning its item. |
| `Source<T, E>.close()` | `unit ![E, Error<SourceError>]` | Idempotent disposal, not traversal/draining. Release callback runs at most once. |
| `map<U, effect F>(f: T -> U ![F])` | `Source<U, G> ![Error<SourceError>]` | Validate a non-busy open input, transfer it into a lazy adapter; callback runs once per pulled item, not at construction. |
| `filter<effect F>(predicate: T -> bool ![F])` | `Source<T, G> ![Error<SourceError>]` | Same transfer checks; pull until one item passes or input ends, with no unbounded prefetch. |
| `take(count: usize)` | `Source<T, E> ![Error<SourceError>]` | Same transfer checks; bound delivered items and close input at the bound. Zero performs no pull, but disposal still occurs when consumed/closed. |
| `collect(max_items: usize)` | `Array<T> ![E, Error<SourceError>]` | Consume at most max_items, then close; bounded-prefix result, not proof of EOF. Zero closes without pulling. |
| `for_each<effect F>(f: T -> unit ![F], max_items: usize)` | `unit ![E, F, Error<SourceError>]` | Sequential callback consumption with the same prefix bound and cleanup. |
| `fold<A, effect F>(initial: A, f: (A, T) -> A ![F], max_items: usize)` | `A ![E, F, Error<SourceError>]` | Thread an ordinary accumulator in input order, then close; earlier external effects are not rolled back. |

| Source Boundary | Required Rule |
|---|---|
| Creation / latent summaries | Preserve owner, E, requested/default actions and callback dependencies through imports, containers and adapters. Creating an adapter performs no callback/I/O; consuming or closing exposes the relevant summaries. |
| Local state / optimization | Cursor allocation and view transfers retain resource/ownership dependencies even without external actions. An empty escaping row does not allow duplicating, memoizing or reordering these transitions as pure value computation. |
| Ownership transfer | chunks/lines take operational ownership of the supplied stream; reject an already-busy/closed stream before transfer. Old raw aliases cannot bypass the wrapper or its buffering. Closing the source closes the underlying stream. |
| Adapter transfer | map/filter/take invalidate prior input views while the returned adapter owns the cursor. Old aliases fail Closed. This is a checked resource contract, not shared mutable ordinary data. |
| EOF / explicit close | EOF runs release, records exhaustion and makes subsequent next return None. Explicit close makes next fail Closed. Neither path repeats release. |
| Admission failure | Busy/Closed before starting work neither advances the cursor nor disposes another active operation/owner. |
| Step failure | Once a step starts, preserve its original typed failure, dispose the source and retain cleanup failures as diagnostics. No implicit retry of a partial read or callback; later next fails Closed. |
| Early exit / cancellation | Terminal consumers dispose on success, error, bound exhaustion or unwind. Dropping a source relies on managed-owner cleanup; a collector cannot survive termination of its own scope. |
| Backpressure | Each next admits only the work needed for its next item; no detached producer. filter may inspect many inputs, but inherited budgets remain active. |
| Timeout | The optional adapter timeout bounds one complete logical pull, including all chunks of one line, not each underlying read afresh. Parent deadline never resets. |
| Line framing | max_line_bytes counts bytes excluding the final LF/CRLF; excess raises LineTooLong. Buffering is bounded by this cap plus a bounded transport chunk/lookahead; do not accumulate the entire stream. |
| Concurrent close | Mark disposed, cancel/wake an in-flight pull, then release its final owned state exactly once. A late completion cannot publish another item or resurrect the source. |
| Async / var self | Pull/release may suspend under standard handlers. These are resource operations, not updating value receivers; they do not hold a caller's var-self access over suspension. |
| Pure distinction | Source does not satisfy the pure Iterable contract. Use its explicit consumers; there is no implicit for-loop/Iter conversion hiding I/O. |

```etas
flow ReadPreview<S ~ ByteStream>(stream: S) -> Array<string> {
    let lines = std.stream.lines(stream, max_line_bytes = 4096);
    return lines.collect(max_items = 20);
}
```

This transfers and closes the stream, returns a bounded prefix, and infers
StreamError/SourceError plus requested read/close metadata. It does not turn
transport failure into EOF or use `?` to unwrap a Result.

## 16. Persistent Stores And Session Publication

### 16.1 Shared Storage Types

`std.memory` owns the types below. Public records are data; constructing a
version, cursor, operation reference or receipt never creates backend evidence.
The backend validates binding, generation, schema and retained operation state.

| Public Type | Shape / Contract |
|---|---|
| `MemoryRegion<S>`, `Store<K, V>` | Declarative schema/store handles with checked stable region provenance; not in-memory Map values. |
| `MemoryVersion`, `MemoryTombstone`, `MemoryCursor` | Opaque token values. Versions/tombstones belong to one key/generation; cursors to one store/query snapshot. |
| `MemoryEntry<K, V>` | Record `{ key: K, value: V, version: MemoryVersion }`. |
| `MemoryPage<K, V>` | Record `{ entries: List<MemoryEntry<K, V>>, cursor: Option<MemoryCursor> }`. |
| `StorageOperationRef` | Record `{ key: string, fingerprint: string }`; one operation identity bound to one exact request. |
| `StorageError` | Record `{ code: string, message: string, operation: Option<StorageOperationRef> }`; stable codes below, redacted explanatory message. |
| `WriteCondition` | Enum `Any`, `Missing`, `Exists`, `Match(MemoryVersion)`. |
| `MemoryWriteIntent<K, V>` | Immutable prepared mutation plus operation reference and receiver provenance; preparing does not publish a write. |
| `MemoryWriteTarget<K>` | Record `{ region: string, store: Array<string>, schema_fingerprint: Option<string>, key: K }`. |
| `MemoryWriteChange` | Enum `Written(MemoryVersion)`, `Deleted(MemoryTombstone)`. |
| `MemoryWriteReceipt<K>` | Record `{ operation: StorageOperationRef, target: MemoryWriteTarget<K>, change: MemoryWriteChange, durability: StorageDurability }`. |
| `StorageDurability` | Portable enum `Volatile`, `Durable`. Durable means stable storage under the backend's documented failure model, not immunity to all failures. Backend labels such as SQLite modes are metadata, not portable guarantees. |
| `MemoryWriteRejection` | Enum `ConditionConflict(Option<MemoryVersion>, Option<MemoryVersion>)`, `Rejected(StorageError)`, `Unchanged`. Conflict payloads are expected/actual versions. |
| `WriteOutcome<A, B>` | Enum `Committed(A)`, `NotCommitted(StorageOperationRef, B)`, `Unknown(StorageOperationRef)`. |
| `ConfirmedOutcome<A, B>` | Enum `Committed(A)`, `NotCommitted(StorageOperationRef, B)`; no uncertain variant. |
| `ReconcileResult<A, B>` | Enum `Found(ConfirmedOutcome<A, B>)`, `Unresolved`, `Expired`. Expired does not prove non-commit. |

| `StorageError.code` | Meaning |
|---|---|
| `invalid_request`, `unsupported` | Invalid option/token/query or an unsupported backend operation. |
| `schema_mismatch`, `invalid_cursor` | Stored schema incompatible with the checked binding, or stale/foreign cursor. |
| `condition_conflict`, `rejected` | Convenience mutation did not satisfy its declared condition or was otherwise confirmed rejected. |
| `outcome_unknown` | A convenience write may have committed; `operation` must be Some for reconciliation. |
| `unavailable`, `timed_out`, `host` | Backend failure; use an uncertain outcome instead if commitment cannot be ruled out. |
| `limit_exceeded`, `measure_unavailable` | Explicit API data bound exceeded or required selection measurement unavailable. Exhausting an enclosing execution budget instead ends that scope and delivers BudgetExceeded outside its boundary; it is not a StorageError. |

### 16.2 Store APIs

In this table `R` denotes the receiver's checked persistent place, not a new
type argument. Standard handlers leave the shown error rows while requested
Memory actions retain that place. K must satisfy `StorageKey`, V must satisfy
`StorageData`, and both need the codecs in section 16.5. Encoding failure is not
silently converted into missing data.

| API | Result | Requested Action / Contract |
|---|---|---|
| `std.memory.region<S>(stable_id: string, store: string)` | `MemoryRegion<S> ![]` | Declare a binding; no connection or mutation at module initialization. Schema/store identity must resolve in deployment metadata. |
| `std.memory.version(token: string)` | `MemoryVersion ![]` | Reconstruct token data only; backend validation remains mandatory. |
| `Store<K, V>.get(key: K)` | `Option<V> ![Error<StorageError>]` | `Memory.read<R>`; None only for absent key, never backend failure. |
| `get_entry(key: K)` | `Option<MemoryEntry<K, V>> ![Error<StorageError>]` | Value and version from the same read. |
| `contains(key: K)` | `bool ![Error<StorageError>]` | Same existence/failure semantics as get. |
| `page(cursor: Option<MemoryCursor>, page_size: u32)` | `MemoryPage<K, V> ![Error<StorageError>]` | `Memory.read<R>`; positive entry bound, None starts a scan. A changed snapshot invalidates the cursor rather than silently skipping/duplicating entries. |
| `keys()` | `List<K> ![Error<StorageError>]` | Bounded full scan under inherited storage limits; same snapshot rules as pages. Use page for large stores. |
| `prepare_put(key: K, value: V, condition: WriteCondition)` | `MemoryWriteIntent<K, V> ![Error<StorageError>]` | Validate encoding/target and bind a fresh operation identity; no backend mutation. Retain execution-context identity, not an assumed pure constant. |
| `prepare_delete(key: K, condition: WriteCondition)` | `MemoryWriteIntent<K, V> ![Error<StorageError>]` | Same preparation contract for deletion. |
| `MemoryWriteIntent<K, V>.operation_ref()` | `StorageOperationRef ![]` | Observe the reference before submission so it can be retained for recovery. |
| `MemoryWriteIntent<K, V>.commit()` | `WriteOutcome<MemoryWriteReceipt<K>, MemoryWriteRejection> ![Error<StorageError>]` | `Memory.write<R>`; atomically evaluate the condition and apply this one mutation. Reusing the same intent/reference reconciles/deduplicates, not a second write. |
| `Store<K, V>.reconcile(operation: StorageOperationRef)` | `ReconcileResult<MemoryWriteReceipt<K>, MemoryWriteRejection> ![Error<StorageError>]` | `Memory.read<R>`; query retained evidence without submitting the mutation again. |

| Convenience Mutation On `Store<K, V>` | Condition / Outcome |
|---|---|
| `put(key: K, value: V)`, `upsert(key: K, value: V)` | Any; create or replace. Upsert is an alias, not a read/modify/write callback. |
| `insert(key: K, value: V)` | Missing; existing key is a condition conflict. |
| `update(key: K, value: V)` | Exists; absent key is a condition conflict. |
| `put_versioned(key: K, value: V, version: MemoryVersion)` | Match the exact version. |
| `delete(key: K)` | Any; absent key is successful no-op. |
| `delete_versioned(key: K, version: MemoryVersion)` | Match the exact version; a mismatch is not successful deletion. |
| All rows above | Return `unit ![Error<StorageError>]`; prepare and commit once, preserving `Memory.write<R>` as requested metadata. Confirmed rejection becomes condition_conflict/rejected error; Unknown becomes outcome_unknown with its operation reference. |
| `clear()` | `unit ![Error<StorageError>]`; snapshot entries, then conditional per-key deletes. Records read and write actions; **not atomic**. Concurrent replacement causes a conflict instead of being silently deleted. Earlier deletes remain committed. |

| Storage Boundary | Required Rule |
|---|---|
| Identity across recompilation | Same deployment binding/stable region id/store path and compatible schema reach the same persistent data. A new source revision alone does not reset it. |
| Schema change | Reject incompatibility until an explicit migration runs; no automatic lossy coercion. |
| Atomicity | One prepared key mutation and its condition are atomic. Multiple commits or clear are not a transaction. |
| Idempotency | Same operation key with a different fingerprint is invalid. Deduplication is limited to the backend's retained evidence window. |
| Cancellation / timeout | If commitment may have occurred, preserve the operation ref and reconcile; cancellation is not rollback. Crash-recoverable operation mode must durably record the ref before dispatch and reject admission if that record cannot be saved. Volatile mode promises in-run reconciliation only, not recovery after process loss. |
| Durability | Receipt reports the achieved durability, never upgrades Volatile to Durable. |
| Query ordering | Backend-selected but stable for a valid scan snapshot; not Map insertion order. Unsupported snapshot retention returns a clear error, not fabricated consistency. |
| Multi-store transactions | No portable API is approved here. A `MemoryTransaction` implementation type alone does not establish atomicity, callback restrictions or retry semantics. |

### 16.3 Deferred Selections

`MemorySelection<V>` is a compiler-known deferred query value, not a pure
`Iter<V>`. It retains store identity, query encoding, limits and latent
requested/error summaries through captures, imports and prompt encoding.

| API | Result | Contract |
|---|---|---|
| `Store<K, V>.scan()` | `MemorySelection<V> ![]` | Describe a full scan without reading it yet. |
| `select(query: MemoryQuery<V>)`, `query(query: MemoryQuery<V>)` | `MemorySelection<V> ![Error<StorageError>]` | Validate the explicit query language/version against the bound store; query is an alias. Arbitrary Q is not accepted without query encoding. |
| `related_to(query: MemoryQuery<V>)` | `MemorySelection<V> ![Error<StorageError>]` | Same explicit encoding contract for a registered similarity query/index. Missing support is unsupported, not an implicit model/embedding call. |
| `MemorySelection<V>.take(count: usize)` | `MemorySelection<V> ![]` | At most count entries; zero selects none. Multiple entry bounds use the minimum. |
| `limit(bound: Limit)` | `MemorySelection<V> ![Error<StorageError>]` | Accept Tokens/ContextTokens as content budgets, not row counts. Other dimensions are invalid here; ordinary enclosing limits still apply. |
| `collect()` | `Array<V> ![Error<StorageError>]` | Execute bounded retrieval, recording `Memory.read<R>`. Each call is a new observation unless explicitly replayed; no hidden cross-call cache. |
| Prompt encoding of a selection | Inferred read/error summary | Materialize under the prompt's resolved measurement context. Missing tokenizer/measurement gives measure_unavailable; no implicit inference or summary generation. |

### 16.4 Session History And Context Publication

`std.agent.session` reuses the storage outcome/error types above.

| Public Type | Shape / Contract |
|---|---|
| `SessionId` | Nominal string value; constructing it does not create/authorize a session. |
| `SessionCursor`, `SessionHistoryFence` | Backend-validated tokens tied to session, generation and selected history/context state. |
| `SessionContextContent` | Record `{ text: string, provenance: Map<string, string> }`; caller-produced derived content, not proof of correctness. |
| `SessionPublishedContext` | Record `{ content: SessionContextContent, fence: SessionHistoryFence, version: u64 }`. |
| `SessionSummary` | Record `{ text: string, message_count: usize }`; compatibility view of existing published content, never a summarizer request. |
| `SessionHistoryPage` | Record `{ fence: SessionHistoryFence, messages: Array<Message<JsonValue>>, published_context: Option<SessionPublishedContext>, summary: Option<SessionSummary>, cursor: Option<SessionCursor> }`. |
| `SessionContextReceipt` | Record `{ operation: StorageOperationRef, session: SessionId, generation: string, context_version: u64, durability: StorageDurability }`. |
| `SessionContextRejection` | Enum `StaleHistory`, `Rejected(StorageError)`. |

| API | Result | Requested Action / Contract |
|---|---|---|
| `history_page(config: SessionConfig, cursor: Option<SessionCursor>, page_size: u32)` | `SessionHistoryPage ![Error<StorageError>]` | `Session.read`; positive bounded page, no implicit session creation or model call. |
| `Conversation.load(config: SessionConfig)` | `Conversation ![Error<StorageError>]` | Load selected history/published context; preserve fence and cursor. Do not catch storage failure and silently abort or create an empty conversation. |
| `SessionConfig.prepare_context(fence: SessionHistoryFence, content: SessionContextContent)` | `StorageOperationRef ![Error<StorageError>]` | Bind exact config/fence/content to an operation; no publication or generated summary. |
| `SessionConfig.publish_context(fence: SessionHistoryFence, content: SessionContextContent, operation: StorageOperationRef)` | `WriteOutcome<SessionContextReceipt, SessionContextRejection> ![Error<StorageError>]` | `Session.publish`; atomic generation/history/context-version validation and publication, or confirmed/unknown outcome. |
| `SessionConfig.reconcile_context(operation: StorageOperationRef)` | `ReconcileResult<SessionContextReceipt, SessionContextRejection> ![Error<StorageError>]` | `Session.reconcile`; resolve evidence without regenerating or republishing content. |

| Session Boundary | Rule |
|---|---|
| Effect coverage | `Session extends Memory`; read/reconcile require history-read authority, publication requires context-write authority. Concrete session ids are runtime payloads, not arbitrary static type selectors. |
| Cursor / fence | A changed generation or incompatible history invalidates it; stale publication never overwrites newer context. |
| Retry | Reuse the exact operation and request. Changed content/fence requires a new prepared operation; unresolved/expired evidence is not a reason to blindly resend. |
| Retention | Publication does not delete history; retention/deletion remains separate. SummaryPlusRecent only selects existing published content. |
| Trust / limits | Provenance, nested secrecy, admission limits and cancellation remain checked; a public receipt record cannot manufacture backend confirmation. |


### 16.5 Storage Encoding And Query Contracts

These are standard data/spec APIs, not a universal serialization keyword.
Custom codecs are Etas values; they cannot grant resource access or export
protected values. A store view keeps its original resource identity.

| Public Contract | Definition / Rule |
|---|---|
| `StorageData` | Compiler-recognized marker spec for finite, persistable data. Primitive values, tuples, ordinary records/enums and data collections qualify recursively. Nominal identity is retained. |
| Excluded data | Live handles, tasks, continuations, flows, handlers, Sources and host secret handles are not admitted by a custom codec. Secret payload export still requires its explicit protected contract. |
| Trust-labelled data | Standard codecs may persist admitted Trusted/Untrusted/Public/Sanitized values only with their labels and protected provenance intact. User-defined decoding cannot manufacture trusted provenance or discard nested secrecy. |
| `StorageKey ~ StorageData + Eq` | Stable key identity. Equal keys require identical canonical encoded bytes; distinct unequal keys must not collide in their encoding. Raw float keys lack Eq. |
| `StorageCodec<T>` | Record `{ format: string, version: u32, encode: T -> Result<bytes, StorageError> ![], decode: bytes -> Result<T, StorageError> ![] }`. Format is nonempty; version is positive. |
| Codec callbacks | Deterministic, non-suspending, no mutable-capture observations or requested actions, including default-handled actions. The compiler checks behavior; semantic round-trip laws remain the implementer's obligation. |
| `MemoryQuery<V>` | Transparent record `{ language: string, version: u32, payload: bytes }`. V names the expected stored value type, not evidence that the backend accepted the query. |

| `std.memory` API | Result | Contract |
|---|---|---|
| `codec<T ~ StorageData>()` | `StorageCodec<T> ![]` | Derive a value codec when structural representation is available; otherwise require an explicit codec. Missing derivation is a compile-time error, not a runtime abort. |
| `key_codec<K ~ StorageKey>()` | `StorageCodec<K> ![]` | Derive canonical key encoding consistent with Eq, including order-independent map/set keys. |
| `Store<K, V>.with_codecs(key: StorageCodec<K>, value: StorageCodec<V>)` | `Store<K, V> ![]` | Declarative view with checked callback evidence and the same persistent place; no connection, migration or authority change. It does not modify other views. |
| `query<Q ~ StorageData, V>(language: string, version: u32, value: Q, codec: StorageCodec<Q> = std.memory.codec<Q>())` | `MemoryQuery<V> ![Error<StorageError>]` | Validate nonempty language/positive version, encode the payload and raise encoding failure. No backend query executes here; V is inferred from the receiving store. |

| Encoding / Backend Boundary | Rule |
|---|---|
| Defaults | Store operations use key_codec for K and codec for V unless a view installs explicit ones. Declaration of a region is declarative; every executable store use must resolve its codec evidence. Imported summaries preserve that evidence. |
| Standard encoding | Versioned binary data, not JSON/f64 conversion. Preserve integer widths/ranges, float encodings, record/enum identity, field names and logical container order. Match fields independently of construction-expression order. The selected storage-format profile publishes byte layout/test vectors; backend-native serialization is not an interchangeable encoding. |
| Key canonicalization | Key encoding ignores construction order and map/set insertion order where equality ignores them. Stable nominal/schema identity participates in the encoding. No physical addresses or randomized hashes as keys. |
| Schema compatibility | Codec format/version, nominal schema and representation fingerprint are part of the persistent binding. A different view cannot reinterpret existing bytes without compatibility validation or explicit migration. |
| Round trip | Value codecs preserve representation, including float bits and sequence order. Key codecs preserve Eq identity and may normalize irrelevant map/set order. Invalid bytes return a typed storage error; size/depth validation precedes unbounded allocation. |
| Custom code | Ordinary data with private representation can publish a codec from its defining module. Supplying one bypasses neither StorageData restrictions nor validation of decoded values. |
| Query extension | An EDK/application adapter supplies a language/version encoder and matching backend decoder/executor for the declared store. Register this pair in the deployment binding; unknown language/version or wrong result schema is unsupported/schema_mismatch. |
| Query safety | Validate payloads and parameterize database access. A language string or payload never grants raw SQL, network access, secret access or cross-region authority. Retrieval remains Memory.read on the original place. |
| Backend variation | Query languages/index features are explicitly adapter-specific; CRUD and canonical storage contracts remain portable. No implicit embedding/model request on an unsupported similarity query. |

## 17. API Coverage And Open Contracts

This is a SPEC coverage audit, not an implementation checklist. A module is not
complete merely because it has exported names or a runtime binding. The API
index identifies the owning contract; do not duplicate it in multiple chapters.

| API Review Item | Required Before Calling A Contract Complete |
|---|---|
| Surface | Canonical module, exact parameter/result types, defaults, generic/spec bounds and visibility. |
| Behavior | Empty/invalid input, ordering, equality, units, encoding and allocation/complexity rules. |
| Effects | Separate residual effects from requested/default actions; preserve callback summaries. |
| Failure | Distinguish `None`, `Result`, `Error<E>`, execution faults and cancellation. No accidental redefinition of `?`. |
| Resources | Creation, use, close/cleanup, alias behavior, timeouts and partial/uncertain outcomes. |
| Verification | Examples and cross-provider tests for the stated contract, not only happy-path execution. |

| Area | Specified Surface | Remaining API Design Work |
|---|---|---|
| Text, collections, iterators, Option/Result | Tables in sections 1, 10-13, including builders, BitSet, bounded deques, generation/scan, grouping, Top-K and boundary/error/cost rules | Cross-provider implementation conformance remains separate from SPEC coverage. Specialized graph, tensor, cache and index libraries are package-level, not implicit standard types. |
| Numeric operations | Checked/wrapping/saturating integer operations, float helpers and profile rules in section 5 | Big integers, decimals and complex numbers are separate library designs. |
| Console | Six string APIs, UTF-8, EOF and residual I/O error behavior | Bounded/streaming input and explicit flushing are not yet specified. No generic `Display` requirement is implied. |
| TCP / TLS | Connect/handshake, canonical host/port/config values and upgrade ownership in section 15 | Server/listen/accept and client-certificate APIs are not specified. |
| Streams | Read/write/flush/close, checked concurrent admission, exact/bounded reads and effectful Source with bounded line/chunk adapters in section 15 | Seek, shared duplex adapters, multicast and statically refined access modes remain separate extensions. A Source is not a channel or broadcast stream. |
| Filesystem | Shared errors, modes, bounded read/list, create/rename/remove and sync in section 15 | Seek, link metadata/watch APIs and durable atomic-replace recipes need separate contracts. |
| HTTP codec | Wire types, fallible encoding, complete/incremental decoding | Request-aware HEAD/CONNECT and upgrade framing; bounded header/body/trailer options; a reusable incremental-state API and its cumulative cost. An EOF reader is not a general HTTP client. |
| Bytes / text codecs | Byte conversion/search/assembly and strict/lossy UTF-8 | Explicit additional charsets, hex/base64 and endian numeric codecs need separate contracts; no ambient-charset fallback. |
| JSON | Shared recursive value type and pure parse/stringify | Depth/size bounds, typed schema conversion and exact-number alternatives are not specified by the current `Number(f64)` representation. |
| Clock / randomness | Monotonic versus wall time, duration arithmetic, timers, seeded RNG and protected entropy in section 14 | Calendar/timezone APIs remain package-level; implementation profiles must supply accuracy/RNG conformance vectors. |
| Commands / environment | Sandboxed command execution and support types | Canonical wrapper ownership, streaming process lifecycle, exit versus launch errors, output bounds, environment/argument APIs and cancellation/cleanup. |
| Tasks / synchronization | Full combinator signatures, limits, empty/race/error rules in the concurrency chapter | Locks, channels, semaphores and atomics still require their own ownership/coordination design. |
| Persistent memory / sessions | CRUD, conditional commit/reconciliation, paging, explicit storage/query codecs, deferred selection and context publication in section 16 | Portable multi-store transactions and explicit session creation/append/retention administration remain open. Storage-format profiles and backend query registrations require conformance tests. |
| Secrets / crypto / browser | Protected secret read/use; hashing/comparison; browser protocol operations | Secret rotation/lifecycle, complete digest/encoding contracts, browser event bounds/deadlines and cancellation. No secret byte-export or high-level browser automation is implied. |
| Agent / compiler / security | Step/tool-dispatch, checked-code-generation and sanitization API tables | Keep frozen-contract, tool-context and trusted-strategy restrictions when adding overloads; an arbitrary callback is not a substitute for checked evidence. |
| Trace / checkpoint | Trace labels and runtime recovery principles | Public query/export and checkpoint result/error/durability APIs are not fully specified by `VirtualStages` or a `checkpoint(...)` example. |

HTTP clients, databases, vector search, email, browser automation and integration
retries remain EDK/package responsibilities over public std substrate. This table
does not approve new keywords, mutable-reference APIs or private host shortcuts.

### 17.1 Acceptance Cases For These Extensions

| Contract | Required Test Cases |
|---|---|
| Budget boundary | Handle a child-local BudgetExceeded in a live parent; reject attempts to continue an exhausted shared scope; no budget reset through ?/collect/retry. |
| Stream admission | Two reads, two writes, duplex TCP, shared file cursor, alias use after TLS/Source transfer, close racing a read, and slot release on failure. |
| Codec / query | Exact wide integers; float bits; canonical equal record/map keys; incompatible schema/version; unsupported query language; protected-label round trips; forged receipt/query does not grant access. |
| Builders | Retained snapshots, non-ASCII text, empty parts, update unwind, checked lengths and linear append-history allocation/copy measurements. |
| BitSet / bounded deque | Zero universe, word-boundary padding, mismatched universes, invalid indices, full/empty deque, every overflow policy and returned eviction value. |
| Algorithms | Empty/k=0/k>n, stable ties, lawful custom comparisons, large equal-key groups, callback-once/order, early failures, and partial selection without full sorting. |
| Source | No pull on construction, alias/adapter invalidation, early close, EOF versus error, release once, invalid UTF-8, line overflow, pull/close races and preserved latent effects. |

### 17.2 Library Design References And Scope

These are design precedents, not claims that Etas implements those libraries
or shares their mutation/error semantics.

| Official Reference | Adopted Direction / Boundary |
|---|---|
| [Rust Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html), [slice algorithms](https://doc.rust-lang.org/std/primitive.slice.html) | Generation, scan/unzip and selection/search APIs; Etas preserves value snapshots and checked callback behavior. |
| [Java BitSet](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/BitSet.html), [Collectors](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/stream/Collectors.html) | Compact bit membership and global grouping; Etas BitSet has an explicit finite universe and grouping returns existing Map/Array values. |
| [Python collections](https://docs.python.org/3/library/collections.html), [itertools](https://docs.python.org/3/library/itertools.html), [heapq](https://docs.python.org/3/library/heapq.html) | Bounded deques, counts, adjacent grouping and Top-K; no new Counter type or implicit shared cursor in pure Iter. |
| [OCaml Buffer](https://ocaml.org/manual/5.4/api/Buffer.html), [Seq](https://ocaml.org/manual/5.4/api/Seq.html) | Incremental assembly and explicit-state unfold; Etas separates updating value receivers from suspendable live sources. |
| [Rust BufRead](https://doc.rust-lang.org/std/io/trait.BufRead.html) | Bounded line/chunk consumption belongs with resource lifetime, errors and cancellation, not the pure Iterable contract. |
| [Python graphlib](https://docs.python.org/3/library/graphlib.html) | Topological ordering is useful for Agent DAGs; place graph representations, cycle diagnostics and ready-set algorithms in an EDK/package contract, not a new std/compiler graph construct. |
