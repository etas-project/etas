module tests.compiler.algorithms.house_robber;

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

flow rob(values: Array<i32>) -> i32 {
    var skip = 0;
    var take = 0;

    for value in values limit Iterations(1024) {
        let next_take = skip + value;
        skip = max_i32(skip, take);
        take = next_take;
    }

    return max_i32(skip, take);
}

flow solve(input: string) -> string {
    return to_string(rob(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
