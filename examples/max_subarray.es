module examples.max_subarray;

import std.effects.Console;
import std.io.println;
import std.text.to_string_i32;

flow max_subarray_sum(values: Array<i32>) -> i32 {
    var index: i32 = 1;
    var best = values[0];
    var current = values[0];

    while index < 6 limit Iterations(16) {
        let value = values[index];
        if current + value > value {
            current = current + value;
        } else {
            current = value;
        }

        if current > best {
            best = current;
        }
        index = index + 1;
    }

    return best;
}

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>, Error<IndexError>]
{
    let values: Array<i32> = [-2, 1, -3, 4, -1, 2];
    println(to_string_i32(max_subarray_sum(values)));
    return 0;
}
