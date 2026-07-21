module tests.compiler.algorithms.maximum_subarray;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow max_i32(left: i32, right: i32) -> i32 {
    if left > right {
        return left;
    }

    return right;
}

flow max_subarray(values: Array<i32>) -> i32 {
    var best = values[0];
    var current = values[0];
    var i: usize = 1;

    while i < len(values) limit Iterations(4096) {
        current = max_i32(values[i], current + values[i]);
        best = max_i32(best, current);
        i = i + 1;
    }

    return best;
}

flow solve(input: string) -> string {
    return to_string(max_subarray(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
