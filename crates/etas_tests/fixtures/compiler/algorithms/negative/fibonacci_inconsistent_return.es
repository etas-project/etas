module tests.compiler.algorithms.negative.fibonacci_inconsistent_return;

import std.collections.{List, len};

flow fibonacci(n: i32) {
    if n <= 1 {
        return n;
    }

    return "large";
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
