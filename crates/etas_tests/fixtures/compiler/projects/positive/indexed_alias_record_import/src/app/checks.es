module app.checks;

import app.model.{MatchResult, PriorityItemQueue, PriorityQueue};

public flow score(result: MatchResult, queue: PriorityItemQueue, generic_queue: PriorityQueue<string>, index: i32) -> i32 ![Error<IndexError>]
{
    let left: i32 = result.pairs[0].left;
    let right: i32 = result.pairs[0].right;
    let priority: i32 = queue.items[index].priority;
    let value: i32 = queue.items[index].value;
    let generic_value: string = generic_queue.items[index].value;
    return left + right + priority + value;
}
