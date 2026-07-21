module tests.compiler.algorithms.negative.binary_search_bool_index;

import std.collections.{List, len};

flow binary_search(values: List<i32>, target: i32) -> i32 {
    let bad_index = true;
    return values[bad_index];
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
