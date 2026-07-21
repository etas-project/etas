module tests.compiler.functional.closure_adder;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, parse_i32_list, to_string, trim};

flow make_adder(delta: i32) -> i32 -> i32 {
    return value => value + delta;
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let add = make_adder(parts[0]);
    return to_string(add(parts[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
