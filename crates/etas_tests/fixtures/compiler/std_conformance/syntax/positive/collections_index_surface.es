// support: Array, Slice, Map, Range
// layer: syntax
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.syntax.collections_index_surface;

flow main(args: Array<string>) -> i32 {
    let xs = [1, 2, 3, 4];
    let first = xs[0];
    let second = xs.at(1);
    let maybe = xs.get(2);
    let grown = xs.push(5);
    let restored = grown.pop();
    let extended = xs.extend([5, 6]);
    let prefix = extended[0, 3);
    let middle = extended(1, 4];
    let copy = prefix.to_array();
    let scores = { "alice" => 10, "bob" => 8 };
    let alice = scores.get("alice");
    let closed = Range.closed(0, 3);
    let open = Range.open(0, 3);
    return 0;
}
