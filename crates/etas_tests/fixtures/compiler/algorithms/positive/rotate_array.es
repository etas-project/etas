module tests.compiler.algorithms.rotate_array;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow reverse(values: Array<i32>, start: i32, end: i32) -> Array<i32> {
    var out = values;
    var left = start;
    var right = end;

    while left < right limit Iterations(1024) {
        let temp = out[left];
        out[left] = out[right];
        out[right] = temp;
        left = left + 1;
        right = right - 1;
    }

    return out;
}

flow rotate(values: Array<i32>, k: i32) -> Array<i32> {
    let n = len_i32_list(values);
    let shift = k % n;
    var out = values;
    out = reverse(out, 0, n - 1);
    out = reverse(out, 0, shift - 1);
    out = reverse(out, shift, n - 1);
    return out;
}

flow solve(input: string) -> string {
    let parts = parse_i32_list(input);
    let k = parts[0];
    let values = slice(parts, 1, len(parts));
    return join_i32_list(rotate(values, k), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
