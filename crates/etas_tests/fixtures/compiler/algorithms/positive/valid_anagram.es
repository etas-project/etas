module tests.compiler.algorithms.valid_anagram;

import std.collections.{Array, len};
import std.io.{read_all, println};
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow is_anagram(left: string, right: string) -> bool {
    if string_len(left) != string_len(right) {
        return false;
    }

    var used = repeat_bool(false, string_len(right));
    var i = 0;
    while i < string_len(left) limit Iterations(1024) {
        var matched = false;
        var j = 0;
        while j < string_len(right) limit Iterations(1024) {
            if !used[j] && left[i] == right[j] {
                used[j] = true;
                matched = true;
                break;
            }
            j = j + 1;
        }

        if !matched {
            return false;
        }
        i = i + 1;
    }

    return true;
}

flow solve(input: string) -> string {
    let lines = split_lines(trim(input));
    if is_anagram(lines[0], lines[1]) {
        return "true";
    }

    return "false";
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_all();
    println(solve(input));
    return 0;
}
