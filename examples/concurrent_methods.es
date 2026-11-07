module examples.concurrent_methods;

import std.io.println;
import std.runtime.checkpoint;
import std.task.{run, start, await, map_concurrent, Scheduler};
import std.task.scheduler.park;

type Counter<T> = { fallback: T, valid: bool };

impl Counter<T> {
    flow keep(self, value: T) -> T {
        var step = 0;
        while step < 100000 limit Iterations(100000) { step = step + 1; }
        if self.valid { return value; }
        return self.fallback;
    }
}

flow main(args: Array<string>) -> i32 {
    let owner = Counter<i32> { fallback = 99, valid = true };
    let bound: (i32) -> i32 ![] = owner.keep;
    checkpoint("bound-method");
    let results = run((scheduler: Scheduler) => {
        let mapped = start(() => { return map_concurrent([7, 3, 9, 1], bound); });
        return handle { await(mapped) } with {
            Async.await(waited) as k => { park(scheduler, waited, k); }
        };
    });
    if results == [7, 3, 9, 1] {
        println("bound CPU methods completed in input order");
        return 0;
    }
    return 1;
}
