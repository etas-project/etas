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
