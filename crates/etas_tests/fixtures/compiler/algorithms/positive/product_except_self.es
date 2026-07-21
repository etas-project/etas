module tests.compiler.algorithms.product_except_self;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow product_except_self(values: Array<i32>) -> Array<i32> {
    var out = repeat_i32(1, len_i32_list(values));
    var prefix = 1;
    var i: usize = 0;

    while i < len(values) limit Iterations(1024) {
        out[i] = prefix;
        prefix = prefix * values[i];
        i = i + 1;
    }

    var suffix = 1;
    var j = len(values);
    while j > 0 limit Iterations(1024) {
        let index = j - 1;
        out[index] = out[index] * suffix;
        suffix = suffix * values[index];
        j = index;
    }

    return out;
}

flow solve(input: string) -> string {
    return join_i32_list(product_except_self(parse_i32_list(input)), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
