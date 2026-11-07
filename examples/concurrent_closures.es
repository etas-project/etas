module examples.concurrent_closures;

import std.io.println;
import std.runtime.checkpoint;
import std.task.{run, start, await, map_concurrent, Scheduler};
import std.task.scheduler.park;

flow make_worker(offset: i32) -> ((i32) -> i32 ![]) {
    return (value: i32) => {
        var step = 0;
        while step < 100000 limit Iterations(100000) { step = step + 1; }
        return value + offset;
    };
}

flow main(args: Array<string>) -> i32 {
    let worker = make_worker(1);
    checkpoint("frozen-closure");
    let results = run((scheduler: Scheduler) => {
        let mapped = start(() => { return map_concurrent([7, 3, 9, 1], worker); });
        return handle { await(mapped) } with {
            Async.await(waited) as k => { park(scheduler, waited, k); }
        };
    });
    if results == [8, 4, 10, 2] {
        println("frozen CPU closures completed in input order");
        return 0;
    }
    return 1;
}
