module tests.compiler.algorithms.merge_sort;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow merge(left: Array<i32>, right: Array<i32>) -> Array<i32> {
    var out: Array<i32> = [];
    var i: usize = 0;
    var j: usize = 0;

    while i < len(left) && j < len(right) limit Iterations(2048) {
        if left[i] <= right[j] {
            out = out.push(left[i]);
            i = i + 1;
        } else {
            out = out.push(right[j]);
            j = j + 1;
        }
    }

    while i < len(left) limit Iterations(2048) {
        out = out.push(left[i]);
        i = i + 1;
    }

    while j < len(right) limit Iterations(2048) {
        out = out.push(right[j]);
        j = j + 1;
    }

    return out;
}

flow merge_sort(values: Array<i32>) -> Array<i32> {
    if len(values) <= 1 {
        return values;
    }

    let mid = len(values) / 2;
    let left = merge_sort(slice(values, 0, mid));
    let right = merge_sort(slice(values, mid, len(values)));
    return merge(left, right);
}

flow solve(input: string) -> string {
    let values = parse_i32_list(input);
    return join_i32_list(merge_sort(values), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
