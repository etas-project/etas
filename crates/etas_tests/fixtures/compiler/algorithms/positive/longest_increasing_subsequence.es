module tests.compiler.algorithms.longest_increasing_subsequence;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow max_i32(left: i32, right: i32) -> i32 {
    if left > right {
        return left;
    }

    return right;
}

flow lis(values: Array<i32>) -> i32 {
    var dp = repeat_i32(1, len_i32_list(values));
    var best = 1;
    var i: i32 = 0;

    while i < len_i32_list(values) limit Iterations(1024) {
        var j = 0;
        while j < i limit Iterations(1024) {
            if values[j] < values[i] {
                dp[i] = max_i32(dp[i], dp[j] + 1);
            }
            j = j + 1;
        }
        best = max_i32(best, dp[i]);
        i = i + 1;
    }

    return best;
}

flow solve(input: string) -> string {
    return to_string(lis(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
