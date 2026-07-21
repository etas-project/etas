module examples.binary_search;

import std.effects.Console;
import std.io.println;
import std.text.to_string_i32;

flow binary_search(values: Array<i32>, target: i32) -> i32 {
    var low: i32 = 0;
    var high: i32 = 4;

    while low <= high limit Iterations(16) {
        let mid = low + ((high - low) / 2);
        let probe = values[mid];

        if probe == target {
            return mid;
        }

        if probe < target {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }

    return -1;
}

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>, Error<IndexError>]
{
    let values: Array<i32> = [3, 6, 9, 12, 15];
    println(to_string_i32(binary_search(values, 15)));
    return 0;
}
