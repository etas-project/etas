module tests.compiler.algorithms.knapsack_01;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type Item = {
    weight: i32,
    value: i32,
};

flow knapsack(items: Array<Item>, capacity: i32) -> i32 {
    var dp = repeat_i32(0, capacity + 1);

    for item in items limit Iterations(256) {
        var w = capacity;
        while w >= item.weight limit Iterations(4096) {
            let candidate = dp[w - item.weight] + item.value;
            if candidate > dp[w] {
                dp[w] = candidate;
            }
            w = w - 1;
        }
    }

    return dp[capacity];
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let capacity = parts[0];
    let items = [
        Item { weight = parts[1], value = parts[2] },
        Item { weight = parts[3], value = parts[4] },
        Item { weight = parts[5], value = parts[6] },
        Item { weight = parts[7], value = parts[8] },
    ];
    return to_string(knapsack(items, capacity));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
