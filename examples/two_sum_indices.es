module examples.two_sum_indices;

import std.effects.Console;
import std.io.println;
import std.text.to_string_i32;

flow encode_pair(left: i32, right: i32) -> i32 {
    return left * 100 + right;
}

flow two_sum_indices(values: Array<i32>, target: i32) -> i32 {
    var left: i32 = 0;

    while left < 4 limit Iterations(32) {
        var right = left + 1;
        while right < 4 limit Iterations(32) {
            if values[left] + values[right] == target {
                return encode_pair(left, right);
            }
            right = right + 1;
        }
        left = left + 1;
    }

    return -1;
}

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>, Error<IndexError>]
{
    let values: Array<i32> = [2, 7, 11, 15];
    println(to_string_i32(two_sum_indices(values, 9)));
    return 0;
}
