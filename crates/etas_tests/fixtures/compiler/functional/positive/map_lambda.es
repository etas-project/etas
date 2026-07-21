module tests.compiler.functional.map_lambda;

import std.collections.Array;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{join_i32_list, parse_i32_list};

flow map_i32(values: Array<i32>, f: i32 -> i32) -> Array<i32> {
    var out: Array<i32> = [];

    for value in values limit Iterations(1024) {
        out = out.push(f(value));
    }

    return out;
}

flow solve(input: string) -> string {
    let square = (value: i32) => value * value;
    return join_i32_list(map_i32(parse_i32_list(input), square), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
