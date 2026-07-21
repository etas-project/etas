module tests.compiler.algorithms.palindrome_number;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow is_palindrome_number(value: i32) -> bool {
    if value < 0 {
        return false;
    }

    var remaining = value;
    var reversed = 0;

    while remaining > 0 limit Iterations(64) {
        reversed = reversed * 10 + (remaining % 10);
        remaining = remaining / 10;
    }

    return reversed == value;
}

flow solve(input: string) -> string {
    if is_palindrome_number(parse_i32(trim(input))) {
        return "true";
    }

    return "false";
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
