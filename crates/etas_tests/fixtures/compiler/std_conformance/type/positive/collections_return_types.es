// support: Array, Slice, Map, Range
// layer: type
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.type.collections_return_types;

flow build_array(xs: Array<i32>) -> Array<i32> {
    let ys: Array<i32> = xs.push(4);
    let zs: Array<i32> = ys.extend([5, 6]);
    return zs;
}

flow check_slice(xs: Array<i32>) -> Array<i32> {
    let window: Slice<i32> = xs[0, 2);
    let copy: Array<i32> = window.to_array();
    return copy;
}

flow check_map(scores: Map<string, i32>) -> Option<i32> {
    return scores.get("alice");
}

flow check_ranges() -> Range<i32> {
    return Range.closed(0, 10);
}

flow main(args: Array<string>) -> i32 {
    let values = build_array([1, 2, 3]);
    let slice_values = check_slice(values);
    return 0;
}
