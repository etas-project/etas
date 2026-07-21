module tests.compiler.algorithms.move_zeroes;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow move_zeroes(values: Array<i32>) -> Array<i32> {
    var write: usize = 0;
    var read: usize = 0;

    while read < len(values) limit Iterations(1024) {
        if values[read] != 0 {
            values[write] = values[read];
            write = write + 1;
        }
        read = read + 1;
    }

    while write < len(values) limit Iterations(1024) {
        values[write] = 0;
        write = write + 1;
    }

    return values;
}

flow solve(input: string) -> string {
    return join_i32_list(move_zeroes(parse_i32_list(input)), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
