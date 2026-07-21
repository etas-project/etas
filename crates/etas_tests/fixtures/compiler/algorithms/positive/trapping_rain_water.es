module tests.compiler.algorithms.trapping_rain_water;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow trap(heights: Array<i32>) -> i32 {
    var left = 0;
    var right = len_i32_list(heights) - 1;
    var left_max = 0;
    var right_max = 0;
    var water = 0;

    while left < right limit Iterations(1024) {
        if heights[left] < heights[right] {
            if heights[left] >= left_max {
                left_max = heights[left];
            } else {
                water = water + left_max - heights[left];
            }
            left = left + 1;
        } else {
            if heights[right] >= right_max {
                right_max = heights[right];
            } else {
                water = water + right_max - heights[right];
            }
            right = right - 1;
        }
    }

    return water;
}

flow solve(input: string) -> string {
    return to_string(trap(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
