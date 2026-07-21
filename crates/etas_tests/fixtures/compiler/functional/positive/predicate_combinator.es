module tests.compiler.functional.predicate_combinator;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, to_string, trim};

flow negate(predicate: i32 -> bool) -> i32 -> bool {
    return value => !predicate(value);
}

flow solve(input: string) -> string {
    let is_positive = (value: i32) => value > 0;
    let is_not_positive = negate(is_positive);
    if is_not_positive(parse_i32(trim(input))) {
        return "true";
    }

    return "false";
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
