module tests.compiler.algorithms.valid_parentheses;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow matches(open: char, close: char) -> bool {
    return (open == '(' && close == ')')
        || (open == '[' && close == ']')
        || (open == '{' && close == '}');
}

flow is_valid(text: string) -> bool {
    var stack: Array<char> = [];
    var i = 0;

    while i < string_len(text) limit Iterations(4096) {
        let ch = text[i];
        if ch == '(' || ch == '[' || ch == '{' {
            stack = stack.push(ch);
        } else {
            if len(stack) == 0 {
                return false;
            }

            let top = stack[len(stack) - 1];
            if !matches(top, ch) {
                return false;
            }
            let (next_stack, _top) = stack.pop();
            stack = next_stack;
        }
        i = i + 1;
    }

    return len(stack) == 0;
}

flow solve(input: string) -> string {
    if is_valid(trim(input)) {
        return "true";
    }

    return "false";
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
