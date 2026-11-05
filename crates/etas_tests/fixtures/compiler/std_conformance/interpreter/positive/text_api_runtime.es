// support: Iter, TextError, Normalization
// layer: interpreter
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.interpreter.positive.text_api_runtime;

import std.text.{len, char_count, lines, find_all, split, get_byte_slice, parse_i32, from_i32, TextError, ParseFailure};
import std.text.unicode.{normalize, casefold, grapheme_count, NFC, NFD};
import std.char.{from_u32 as char_from_u32, is_alphabetic};

flow main(args: Array<string>) -> i32 {
    var line_values = [];
    for line in lines("a\r\n\nb\n") limit Iterations(4) {
        line_values = line_values.push(line);
    }

    var matches = [];
    for range in find_all("ab", "") limit Iterations(4) {
        matches = matches.push(range);
    }

    let decomposed = "e" + "́";
    let valid = len("é") == 2
        && char_count("é") == 1
        && line_values == ["a", "", "b"]
        && matches == [(0, 0), (1, 1), (2, 2)]
        && split("ab", "") == ["", "a", "b", ""]
        && get_byte_slice("é", Range.closed(1, 1)) == None
        && parse_i32("42") == Ok(42)
        && parse_i32(" 42") == Err(ParseFailure)
        && from_i32(-2147483648) == "-2147483648"
        && normalize(decomposed, NFC) == "é"
        && normalize("é", NFD) == decomposed
        && casefold("Straße") == "strasse"
        && grapheme_count("A👩‍🚀é") == 3
        && char_from_u32(55296) == None
        && is_alphabetic('界');
    if valid {
        return 0;
    }
    return 1;
}
