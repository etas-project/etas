module tests.compiler.algorithms.gcd_euclid;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow gcd(a: i32, b: i32) -> i32 {
    var x = abs(a);
    var y = abs(b);

    while y != 0 limit Iterations(128) {
        let next = x % y;
        x = y;
        y = next;
    }

    return x;
}

flow solve(input: string) -> string {
    let values = parse_i32_list(input);
    return to_string(gcd(values[0], values[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
