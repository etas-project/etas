module tests.compiler.functional.thunk_lazy_value;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, to_string, trim};

flow force(thunk: unit -> i32) -> i32 {
    return thunk();
}

flow solve(input: string) -> string {
    let base = parse_i32(trim(input));
    let delayed = () => base * base;
    return to_string(force(delayed));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
