module examples.linear_search;

import std.effects.Console;
import std.io.println;
import std.text.to_string_i32;

flow linear_search(values: Array<i32>, target: i32) -> i32 {
    var index: i32 = 0;

    while index < 5 limit Iterations(16) {
        if values[index] == target {
            return index;
        }
        index = index + 1;
    }

    return -1;
}

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>, Error<IndexError>]
{
    let values: Array<i32> = [4, 8, 15, 16, 23];
    println(to_string_i32(linear_search(values, 16)));
    return 0;
}
