module tests.compiler.algorithms.roman_to_integer;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow roman_value(ch: char) -> i32 {
    if ch == 'I' {
        return 1;
    }
    if ch == 'V' {
        return 5;
    }
    if ch == 'X' {
        return 10;
    }
    if ch == 'L' {
        return 50;
    }
    if ch == 'C' {
        return 100;
    }
    if ch == 'D' {
        return 500;
    }
    return 1000;
}

flow roman_to_integer(text: string) -> i32 {
    var total = 0;
    var i = 0;

    while i < string_len(text) limit Iterations(128) {
        let current = roman_value(text[i]);
        if i + 1 < string_len(text) && current < roman_value(text[i + 1]) {
            total = total - current;
        } else {
            total = total + current;
        }
        i = i + 1;
    }

    return total;
}

flow solve(input: string) -> string {
    return to_string(roman_to_integer(trim(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
