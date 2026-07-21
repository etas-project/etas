module tests.compiler.algorithms.climbing_stairs;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow climb_stairs(n: i32) -> i32 {
    if n <= 2 {
        return n;
    }

    var first = 1;
    var second = 2;
    var step = 3;

    while step <= n limit Iterations(256) {
        let next = first + second;
        first = second;
        second = next;
        step = step + 1;
    }

    return second;
}

flow solve(input: string) -> string {
    return to_string(climb_stairs(parse_i32(trim(input))));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
