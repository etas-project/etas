module tests.compiler.functional.compose_flows;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, to_string, trim};

flow compose(f: i32 -> i32, g: i32 -> i32) -> i32 -> i32 {
    return value => f(g(value));
}

flow solve(input: string) -> string {
    let double = (value: i32) => value * 2;
    let inc = (value: i32) => value + 1;
    let pipeline = compose(double, inc);
    return to_string(pipeline(parse_i32(trim(input))));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
