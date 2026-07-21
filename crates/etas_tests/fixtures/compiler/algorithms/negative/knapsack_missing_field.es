module tests.compiler.algorithms.negative.knapsack_missing_field;

import std.collections.{List, len};

type Item = {
    weight: i32,
    value: i32,
};

flow score(item: Item) -> i32 {
    return item.cost;
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
