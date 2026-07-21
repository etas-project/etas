module tests.compiler.algorithms.best_time_stock;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow max_profit(prices: Array<i32>) -> i32 {
    var min_price = prices[0];
    var best = 0;
    var i: usize = 1;

    while i < len(prices) limit Iterations(1024) {
        if prices[i] < min_price {
            min_price = prices[i];
        }

        if prices[i] - min_price > best {
            best = prices[i] - min_price;
        }
        i = i + 1;
    }

    return best;
}

flow solve(input: string) -> string {
    return to_string(max_profit(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
