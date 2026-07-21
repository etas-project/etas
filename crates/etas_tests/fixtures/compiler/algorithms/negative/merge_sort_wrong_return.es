module tests.compiler.algorithms.negative.merge_sort_wrong_return;

import std.collections.{List, len};

flow merge_sort(values: List<i32>) -> List<i32> {
    if len(values) <= 1 {
        return values;
    }

    return "not a list";
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
