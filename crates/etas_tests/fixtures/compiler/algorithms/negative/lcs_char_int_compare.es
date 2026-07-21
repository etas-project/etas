module tests.compiler.algorithms.negative.lcs_char_int_compare;

import std.collections.{List, len};

flow lcs(a: string, b: string) -> i32 {
    if a[0] == 1 {
        return 1;
    }

    return 0;
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
