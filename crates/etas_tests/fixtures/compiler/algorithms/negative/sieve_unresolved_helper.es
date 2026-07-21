module tests.compiler.algorithms.negative.sieve_unresolved_helper;

import std.collections.{List, len};

flow sieve(limit: i32) -> List<i32> {
    return make_prime_table(limit);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
