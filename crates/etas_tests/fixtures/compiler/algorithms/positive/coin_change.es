module tests.compiler.algorithms.coin_change;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow min_i32(left: i32, right: i32) -> i32 {
    if left < right {
        return left;
    }

    return right;
}

flow coin_change(coins: Array<i32>, amount: i32) -> i32 {
    var dp = repeat_i32(1_000_000, amount + 1);
    dp[0] = 0;

    var value = 1;
    while value <= amount limit Iterations(4096) {
        for coin in coins limit Iterations(256) {
            if coin <= value {
                dp[value] = min_i32(dp[value], dp[value - coin] + 1);
            }
        }
        value = value + 1;
    }

    if dp[amount] >= 1_000_000 {
        return -1;
    }

    return dp[amount];
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let amount = parts[0];
    let coins = slice(parts, 1, len(parts));
    return to_string(coin_change(coins, amount));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
