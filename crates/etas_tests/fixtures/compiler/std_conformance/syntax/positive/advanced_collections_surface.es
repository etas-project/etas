// support: Deque, Queue, Stack, PriorityQueue, OrderedMap, OrderedSet
// layer: syntax
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.syntax.advanced_collections_surface;

flow main(args: Array<string>) -> i32 {
    let deque = Deque.new<i32>().push_front(1).push_back(2);
    let (without_front, front) = deque.pop_front();
    let (trimmed, back) = without_front.pop_back();
    let queue = Queue.new<string>().push("a").push("b");
    let (queued_tail, queued) = queue.pop();
    let stack = Stack.new<i32>().push(1).push(2);
    let (stacked_tail, stacked) = stack.pop();
    let priorities = PriorityQueue.new<string, i32>().push("low", 5).push("high", 1);
    let (remaining_priorities, next) = priorities.pop();
    let ordered = OrderedMap.new<string, i32>().insert("b", 2).insert("a", 1);
    let ordered_value = ordered.get("a");
    let set = OrderedSet.new<string>().insert("b").insert("a");
    let has_a = set.contains("a");
    return 0;
}
