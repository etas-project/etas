module tests.compiler.functional.apply_twice;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, to_string, trim};

flow apply_twice(value: i32, f: i32 -> i32) -> i32 {
    return f(f(value));
}

flow solve(input: string) -> string {
    let start = parse_i32(trim(input));
    let inc = (value: i32) => value + 1;
    return to_string(apply_twice(start, inc));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
