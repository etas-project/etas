module tests.compiler.algorithms.sieve_primes;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow sieve(limit: i32) -> Array<i32> {
    var prime = repeat_bool(true, limit + 1);
    var p = 2;

    while p * p <= limit limit Iterations(1024) {
        if prime[p] {
            var multiple = p * p;
            while multiple <= limit limit Iterations(4096) {
                prime[multiple] = false;
                multiple = multiple + p;
            }
        }
        p = p + 1;
    }

    var out: Array<i32> = [];
    var n = 2;
    while n <= limit limit Iterations(4096) {
        if prime[n] {
            out = out.push(n);
        }
        n = n + 1;
    }

    return out;
}

flow solve(input: string) -> string {
    return join_i32_list(sieve(parse_i32(trim(input))), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
