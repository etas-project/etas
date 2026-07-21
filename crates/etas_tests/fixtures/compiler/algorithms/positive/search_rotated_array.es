module tests.compiler.algorithms.search_rotated_array;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow search(values: Array<i32>, target: i32) -> i32 {
    var low = 0;
    var high = len_i32_list(values) - 1;

    while low <= high limit Iterations(1024) {
        let mid = low + ((high - low) / 2);
        if values[mid] == target {
            return mid;
        }

        if values[low] <= values[mid] {
            if values[low] <= target && target < values[mid] {
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        } else {
            if values[mid] < target && target <= values[high] {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
    }

    return -1;
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let target = parts[0];
    let values = slice(parts, 1, len(parts));
    return to_string(search(values, target));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
