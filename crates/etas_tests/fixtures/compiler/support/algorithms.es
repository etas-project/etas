module tests.compiler.support.algorithms;

import std.collections.Array;
import std.collections.{len as list_len};
import std.option.unwrap;
import std.text.{join, len as text_len, lines, parse_i32 as std_parse_i32, split, to_string_i32, to_string_usize, trim as text_trim};

public flow abs(value: i32) -> i32 {
    if value < 0 {
        return 0 - value;
    }

    return value;
}

public flow join_i32_list(values: Array<i32>, separator: string) -> string {
    var parts: Array<string> = [];
    var index: usize = 0;

    while index < list_len(values) limit Iterations(4096) {
        parts = parts.push(to_string_i32(values[index]));
        index = index + 1;
    }

    return join(parts, separator);
}

public flow len_i32_list(values: Array<i32>) -> i32 {
    var count: i32 = 0;

    for _value in values limit Iterations(4096) {
        count = count + 1;
    }

    return count;
}

public flow len_i32_grid(values: Array<Array<i32>>) -> i32 {
    var count: i32 = 0;

    for _value in values limit Iterations(4096) {
        count = count + 1;
    }

    return count;
}

public flow matrix_i32(rows: i32, cols: i32, fill: i32) -> Array<Array<i32>> {
    var out: Array<Array<i32>> = [];
    var row: i32 = 0;

    while row < rows limit Iterations(4096) {
        out = out.push(repeat_i32(fill, cols));
        row = row + 1;
    }

    return out;
}

public flow parse_i32(input: string) -> i32 {
    return match std_parse_i32(text_trim(input)) {
        Ok(value) => value,
        Err(_) => abort("invalid integer")
    };
}

public flow parse_i32_list(input: string) -> Array<i32> {
    let parts = split(text_trim(input), " ");
    var out: Array<i32> = [];
    var index: usize = 0;

    while index < list_len(parts) limit Iterations(4096) {
        if parts[index] != "" {
            out = out.push(parse_i32(parts[index]));
        }
        index = index + 1;
    }

    return out;
}

public flow repeat_bool(value: bool, count: i32) -> Array<bool> {
    var out: Array<bool> = [];
    var index: i32 = 0;

    while index < count limit Iterations(4096) {
        out = out.push(value);
        index = index + 1;
    }

    return out;
}

public flow repeat_i32(value: i32, count: i32) -> Array<i32> {
    var out: Array<i32> = [];
    var index: i32 = 0;

    while index < count limit Iterations(4096) {
        out = out.push(value);
        index = index + 1;
    }

    return out;
}

public flow slice(values: Array<i32>, start: usize, end: usize) -> Array<i32> {
    var out: Array<i32> = [];
    var index = start;

    while index < end limit Iterations(4096) {
        out = out.push(values[index]);
        index = index + 1;
    }

    return out;
}

public flow split_lines(input: string) -> Array<string> {
    return lines(input);
}

public flow string_len(value: string) -> i32 {
    return parse_i32(to_string_usize(text_len(value)));
}

public flow to_string(value: i32) -> string {
    return to_string_i32(value);
}

public flow trim(input: string) -> string {
    return text_trim(input);
}
