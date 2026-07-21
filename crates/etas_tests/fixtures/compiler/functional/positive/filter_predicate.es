module tests.compiler.functional.filter_predicate;

import std.collections.Array;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{join_i32_list, parse_i32_list};

flow filter_i32(values: Array<i32>, predicate: i32 -> bool) -> Array<i32> {
    var out: Array<i32> = [];

    for value in values limit Iterations(1024) {
        if predicate(value) {
            out = out.push(value);
        }
    }

    return out;
}

flow solve(input: string) -> string {
    let is_even = (value: i32) => value % 2 == 0;
    return join_i32_list(filter_i32(parse_i32_list(input), is_even), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
