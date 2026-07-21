// support: Array, Map, Queue, Stack
// layer: type
// polarity: negative
// status: covered-negative
// expect: Array.push returns Array<T>, not unit
// expect: Array.pop returns (Array<T>, Option<T>), not in-place mutation
// expect: Queue and Stack do not support positional slicing
module tests.compiler.std_conformance.type.collections_reject_old_mutation_style;

flow bad_push_unit(xs: Array<i32>) -> unit {
    return xs.push(4);
}

flow bad_pop_mutation(xs: Array<i32>) -> Option<i32> {
    return xs.pop();
}

flow bad_map_index(scores: Map<string, i32>) -> i32 {
    return scores[0];
}

flow bad_queue_slice(queue: Queue<i32>) -> Slice<i32> {
    return queue[0, 2);
}

flow bad_stack_slice(stack: Stack<i32>) -> Slice<i32> {
    return stack[0, 2);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
