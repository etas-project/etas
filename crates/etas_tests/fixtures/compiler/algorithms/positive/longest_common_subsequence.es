module tests.compiler.algorithms.longest_common_subsequence;

import std.collections.{Array, len};
import std.io.{read_all, println};
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow lcs(a: string, b: string) -> i32 {
    let rows = string_len(a) + 1;
    let cols = string_len(b) + 1;
    var dp = matrix_i32(rows, cols, 0);
    var i = 1;

    while i <= string_len(a) limit Iterations(1024) {
        var j = 1;
        while j <= string_len(b) limit Iterations(1024) {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                dp[i][j] = dp[i - 1][j];
            } else {
                dp[i][j] = dp[i][j - 1];
            }
            j = j + 1;
        }
        i = i + 1;
    }

    return dp[string_len(a)][string_len(b)];
}

flow solve(input: string) -> string {
    let lines = split_lines(trim(input));
    return to_string(lcs(lines[0], lines[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_all();
    println(solve(input));
    return 0;
}
