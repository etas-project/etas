
module app.main;
import std.task.{Async, Scheduler, try_join};
flow main(args: Array<string>) -> i32 {
    return verify_fifo([]);
}
type StatefulScheduler = { dispatches: usize };
impl StatefulScheduler ~ std.task.scheduler.Scheduler {
    public flow select(var self, ready: Array<std.task.scheduler.ReadyTask>) -> usize ![] {
        self = StatefulScheduler { dispatches = self.dispatches + 1 };
        return ready.length - 1;
    }
}
flow verify_fifo(args: Array<string>) -> i32 {
    return std.task.run((scope: Scheduler) => {
        let adapter = std.task.scheduler.handlers(scope);
        return handle {
            let first = std.task.start(() => {

                std.runtime.checkpoint("20");
                return 20;
            });
            let second = std.task.start(() => {

                std.runtime.checkpoint("22");
                return 22;
            });
            let left = perform Async.await(first);
            let right = perform Async.await(second);
            left + right
        } with adapter;
 }, std.task.scheduler.DefaultScheduler);
}
flow verify_stateful(args: Array<string>) -> i32 {
    let supplied = StatefulScheduler { dispatches = 0 };
    let result = std.task.run((scope: Scheduler) => {
        let adapter = std.task.scheduler.handlers(scope);
        return handle {
            let first = std.task.start(() => {

                std.runtime.checkpoint("20");
                return 20;
            });
            let second = std.task.start(() => {

                std.runtime.checkpoint("22");
                return 22;
            });
            let left = perform Async.await(first);
            let right = perform Async.await(second);
            left + right
        } with adapter;
 }, supplied);
    if supplied.dispatches != 0 { std.core.abort("run changed its supplied policy value"); }
    return result;
}
flow verify_yielding(args: Array<string>) -> i32 {
    return std.task.run((scope: Scheduler) => {
        let adapter = std.task.scheduler.handlers(scope);
        return handle {
            let first = std.task.start(() => {
                perform Async.yield();
                std.runtime.checkpoint("20");
                return 20;
            });
            let second = std.task.start(() => {
                perform Async.yield();
                std.runtime.checkpoint("22");
                return 22;
            });
            let left = perform Async.await(first);
            let right = perform Async.await(second);
            left + right
        } with adapter;
 }, std.task.scheduler.DefaultScheduler);
}
flow verify_try_join(args: Array<string>) -> i32 {
    let jobs: Array<() -> i32 ![Error<string>, Async]> = [
        () => { perform Async.yield(); perform Error<string>.raise("lower"); },
        () => perform Error<string>.raise("higher"),
        () => {
            while true { perform Async.yield(); }
            std.runtime.checkpoint("loser-finished");
            return 99;
        },
    ];
    return match try_join<i32, string, ![Async]>(jobs) {
        Err(error) => if error == "lower" { 42 } else { 0 },
        Ok(_) => 0,
    };
}

effect ScopeProbe {
    action step() -> i32;
}

flow verify_lexical(args: Array<string>) -> i32 {
    return std.task.run((scope: Scheduler) => {
        let adapter = std.task.scheduler.handlers(scope);
        var ticks = 0;
        let resumed = handle {
            perform ScopeProbe.step()
        } with {
            ScopeProbe.step() => {
                ticks = ticks + 1;
                handle { resume 20; } with adapter;
            }
        };
        let finished = handle {
            perform ScopeProbe.step()
        } with {
            ScopeProbe.step() => {
                ticks = ticks + 1;
                handle { finish 22; } with adapter;
            }
        };
        var iteration = 0;
        var total = 0;
        while iteration < 3 {
            iteration = iteration + 1;
            handle {
                if iteration == 1 { continue; }
                if iteration == 3 { break; }
                total = total + 2;
            } with adapter;
        }
        let answer = () => {
            total = total + 1;
            handle {
                return resumed + finished + total - 3;
            } with adapter;
            return 0;
        };
        let computed = answer();
        return computed + total + ticks - 5;
    }, std.task.scheduler.DefaultScheduler);
}
