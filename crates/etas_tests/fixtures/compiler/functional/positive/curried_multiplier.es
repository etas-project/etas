module tests.compiler.functional.curried_multiplier;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32_list, to_string};

flow multiply_by(factor: i32) -> i32 -> i32 {
    return value => value * factor;
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let times = multiply_by(parts[0]);
    return to_string(times(parts[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
