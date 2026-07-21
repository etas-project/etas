// support: Array, IndexError
// layer: interpreter
// polarity: negative
// status: blocked-by-impl
// expect: out-of-bounds checked index raises IndexError instead of returning unit, none, or default value
module tests.compiler.std_conformance.interpreter.checked_index_must_not_return_unit;

flow main(args: Array<string>) -> i32 {
    let xs = [1, 2, 3];
    let value: i32 = xs[99];
    return value;
}
