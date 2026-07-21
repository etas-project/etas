// support: Deque, Queue, Stack, PriorityQueue, OrderedMap, OrderedSet
// layer: type
// polarity: negative
// status: covered-negative
// expect: collection element, key, and priority types must match
module tests.compiler.std_conformance.type.advanced_collections_bad_types;

flow bad_deque_element() -> Deque<i32> {
    return Deque.new<i32>().push_back("wrong");
}

flow bad_queue_element() -> Queue<string> {
    return Queue.new<string>().push(42);
}

flow bad_stack_element() -> Stack<i32> {
    return Stack.new<i32>().push("wrong");
}

flow bad_priority_type() -> PriorityQueue<string, i32> {
    return PriorityQueue.new<string, i32>().push("task", "high");
}

flow bad_ordered_map_key() -> OrderedMap<string, i32> {
    return OrderedMap.new<string, i32>().insert(1, 2);
}

flow bad_ordered_set_value() -> OrderedSet<string> {
    return OrderedSet.new<string>().insert(99);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
