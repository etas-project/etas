module app;

import std.text.join;
import std.collections.{Map, PriorityQueue};

flow main(args: Array<string>) -> i32 ![] {
    let text = join(["source", "std"], ":");
    if text != "source:std" { return 0; }
    let present: Option<string> = Some(text);
    let checked: Result<string, string> = Ok(present.unwrap());
    if checked.unwrap() != "source:std" { return 0; }

    let empty_values: Map<string, i32> = Map.new();
    let values = empty_values.insert("answer", 42);
    let answer = match values.get("answer") {
        Some(value) => value,
        None => 0,
    };

    let empty_queue: PriorityQueue<i32, i32> = PriorityQueue.new((left: i32, right: i32) => left - right);
    let queue = empty_queue.push(answer, 2).push(7, 1);
    let (_, first) = queue.pop();
    return match first { Some(value) => value, None => 0 };
}
