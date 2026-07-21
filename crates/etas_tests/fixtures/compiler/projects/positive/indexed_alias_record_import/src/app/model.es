module app.model;

public alias MatchPair = {
    left: i32,
    right: i32,
};

public alias MatchResult = {
    pairs: Array<MatchPair>,
    count: i32,
};

public alias PriorityItem = {
    value: i32,
    priority: i32,
    sequence: i32,
};

public alias PriorityItemQueue = {
    items: Array<PriorityItem>,
};

public alias QueueItem<T> = {
    value: T,
    priority: i32,
    sequence: i32,
};

public alias PriorityQueue<T> = {
    items: Array<QueueItem<T>>,
    next_sequence: i32,
};

public flow sample_match_result() -> MatchResult
{
    let pairs: Array<MatchPair> = [
        { left = 1, right = 2 },
    ];
    return { pairs = pairs, count = 1 };
}

public flow sample_queue() -> PriorityItemQueue
{
    let items: Array<PriorityItem> = [
        { value = 10, priority = 3, sequence = 0 },
    ];
    return { items = items };
}

public flow sample_generic_queue() -> PriorityQueue<string>
{
    let items: Array<QueueItem<string>> = [
        { value = "generic", priority = 4, sequence = 1 },
    ];
    return { items = items, next_sequence = 2 };
}
