module tests.compiler.algorithms.negative.quick_sort_missing_limit;

import std.collections.{List, len};

flow quick_sort_range(values: List<i32>, low: i32, high: i32) -> unit {
    while low < high {
        let pivot = values[high];
        values[low] = pivot;
    }
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
