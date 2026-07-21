module tests.compiler.algorithms.two_sum;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow two_sum(values: Array<i32>, target: i32) -> Array<i32> {
    var i = 0;

    while i < len_i32_list(values) limit Iterations(1024) {
        var j = i + 1;
        while j < len_i32_list(values) limit Iterations(1024) {
            if values[i] + values[j] == target {
                return [i, j];
            }
            j = j + 1;
        }
        i = i + 1;
    }

    return [-1, -1];
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let target = parts[0];
    let values = slice(parts, 1, len(parts));
    return join_i32_list(two_sum(values, target), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
