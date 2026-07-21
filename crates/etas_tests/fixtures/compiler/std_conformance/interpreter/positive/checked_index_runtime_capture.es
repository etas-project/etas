// support: Array, IndexError
// layer: interpreter
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.interpreter.checked_index_runtime_capture;

flow checked_lookup(xs: Array<i32>, index: i32) -> i32 ![Error<IndexError>] {
    return xs[index];
}

flow main(args: Array<string>) -> i32 {
    return handle {
        checked_lookup([1, 2, 3], 99)
    } with {
        Error<IndexError>.raise(err) => {
            finish 0;
        }
    };
}
