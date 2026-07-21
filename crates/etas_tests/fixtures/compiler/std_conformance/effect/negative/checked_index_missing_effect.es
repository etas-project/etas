// support: Array, Slice, IndexError
// layer: effect
// polarity: negative
// status: blocked-by-impl
// expect: checked indexing must infer Error<IndexError>
module tests.compiler.std_conformance.effect.checked_index_missing_effect;

flow unchecked(xs: Array<i32>, index: i32) -> i32 {
    return xs.at(index);
}

flow unchecked_slice(xs: Array<i32>, index: i32) -> i32 {
    let window = xs[0, 2);
    return window[index];
}

flow main(args: Array<string>) -> i32 {
    return unchecked([1, 2, 3], -1);
}
