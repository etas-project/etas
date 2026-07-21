// support: Array, Slice, IndexError, Error.raise
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.checked_index_captured;

flow lookup(xs: Array<i32>, index: i32) -> i32 ![Error<IndexError>] {
    return xs[index];
}

flow fallback(xs: Array<i32>, index: i32) -> i32 {
    return handle {
        xs[index]
    } with {
        Error<IndexError>.raise(err) => {
            finish 0;
        }
    };
}

flow main(args: Array<string>) -> i32 {
    let value = fallback([1, 2, 3], 99);
    return 0;
}
