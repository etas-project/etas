module examples.concurrent_generic;

import std.io.println;
import std.task.{run, start, await, map_concurrent, Scheduler};
import std.task.scheduler.park;

flow burn<T>(value: T) -> T {
    var step = 0;
    while step < 100000 limit Iterations(100000) { step = step + 1; }
    return value;
}

flow main(args: Array<string>) -> i32 {
    let results = run((scheduler: Scheduler) => {
        let mapped = start(() => { return map_concurrent([7, 3, 9, 1], burn); });
        return handle { await(mapped) } with {
            Async.await(waited) as k => { park(scheduler, waited, k); }
        };
    });
    if results == [7, 3, 9, 1] {
        println("generic CPU tasks completed in input order");
        return 0;
    }
    return 1;
}
