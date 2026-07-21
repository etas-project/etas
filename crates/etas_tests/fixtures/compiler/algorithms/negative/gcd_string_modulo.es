module tests.compiler.algorithms.negative.gcd_string_modulo;

import std.collections.{List, len};

flow gcd(a: i32, b: string) -> i32 {
    return a % b;
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
