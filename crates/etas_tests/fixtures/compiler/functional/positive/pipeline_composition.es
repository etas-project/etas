module tests.compiler.functional.pipeline_composition;

import std.collections.List;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32, to_string, trim};

flow increment(value: i32) -> i32 {
    return value + 1;
}

flow double(value: i32) -> i32 {
    return value * 2;
}

flow solve(input: string) -> string {
    let pipeline = increment | double;
    return to_string(pipeline(parse_i32(trim(input))));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
