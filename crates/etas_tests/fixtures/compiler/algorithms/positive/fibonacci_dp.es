module tests.compiler.algorithms.fibonacci_dp;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }

    var prev = 0;
    var curr = 1;
    var i = 2;

    while i <= n limit Iterations(256) {
        let next = prev + curr;
        prev = curr;
        curr = next;
        i = i + 1;
    }

    return curr;
}

flow solve(input: string) -> string {
    return to_string(fibonacci(parse_i32(trim(input))));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
