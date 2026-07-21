module tests.compiler.algorithms.negative.dijkstra_cost_type_mismatch;

import std.collections.{List, len};

type Edge = {
    from: i32,
    to: i32,
    cost: i32,
};

flow edge() -> Edge {
    return Edge { from = 0, to = 1, cost = "-3" };
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
