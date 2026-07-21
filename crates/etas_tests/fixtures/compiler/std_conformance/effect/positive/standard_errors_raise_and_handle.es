// support: IndexError, Error.raise
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.standard_errors_raise_and_handle;

flow first_or_fallback(values: Array<i32>, index: i32) -> i32 ![] {
    return handle {
        values[index]
    } with {
        Error<IndexError>.raise(err) => {
            finish 0;
        }
    };
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
