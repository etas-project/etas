module tests.compiler.algorithms.container_with_most_water;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow min_i32(left: i32, right: i32) -> i32 {
    if left < right {
        return left;
    }

    return right;
}

flow max_i32(left: i32, right: i32) -> i32 {
    if left > right {
        return left;
    }

    return right;
}

flow max_area(heights: Array<i32>) -> i32 {
    var left = 0;
    var right = len_i32_list(heights) - 1;
    var best = 0;

    while left < right limit Iterations(1024) {
        let area = (right - left) * min_i32(heights[left], heights[right]);
        best = max_i32(best, area);
        if heights[left] < heights[right] {
            left = left + 1;
        } else {
            right = right - 1;
        }
    }

    return best;
}

flow solve(input: string) -> string {
    return to_string(max_area(parse_i32_list(input)));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
