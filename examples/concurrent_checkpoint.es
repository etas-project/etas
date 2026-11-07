module examples.concurrent_checkpoint;

import std.io.println;
import std.runtime.checkpoint;
import std.task.{run, start, await, Scheduler};
import std.task.scheduler.park;

flow burn() -> i32 {
    var step = 0;
    while step < 100000 limit Iterations(100000) { step = step + 1; }
    return 1;
}

flow main(args: Array<string>) -> i32 {
    println("starting tasks");
    let result = run((scheduler: Scheduler) => {
        let first = start(burn);
        let second = start(burn);
        let marker = start(() => { checkpoint("active-tasks"); return 1; });
        let a = handle { await(first) } with {
            Async.await(waited) as k => { park(scheduler, waited, k); }
        };
        return a + await(second) + await(marker);
    });
    println("tasks completed");
    if result == 3 { return 0; }
    return 1;
}
