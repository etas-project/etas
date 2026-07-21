module tests.compiler.functional.fold_sum;

import std.collections.Array;
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{parse_i32_list, to_string};

flow fold_i32(values: Array<i32>, seed: i32, folder: (i32, i32) -> i32) -> i32 {
    var acc = seed;

    for value in values limit Iterations(1024) {
        acc = folder(acc, value);
    }

    return acc;
}

flow solve(input: string) -> string {
    let add = (acc: i32, value: i32) => acc + value;
    return to_string(fold_i32(parse_i32_list(input), 0, add));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
