module app.main;

import std.collections.Array;
import std.io.{println};
import app.agent.model.{request};
import app.agent.service.{run_multi_agent};
import app.agent.runtime_variants.{runtime_task, run_retry_checkpoint_version, run_memory_conflict_version, run_cached_review_version, run_abort_guard_version};

spec MultiAgentRuntimeGate: trace = +Agentic & +Memory & +Console;

flow smoke_main(args: Array<string>) -> i32 ![Error<IOError>] {
    let task = request("multi-agent smoke", "draft body");
    let result = run_multi_agent(task);
    if result == "" {
        return 1;
    }
    return 0;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Memory] ~ MultiAgentRuntimeGate
{
    let task = runtime_task("runtime", "draft");
    let conflict_task = runtime_task("runtime-conflict", "draft");
    let cached_task = runtime_task("runtime-cache", "draft");
    let guard_task = runtime_task("runtime-guard", "draft");
    let checkpointed = run_retry_checkpoint_version(task);
    let conflicted = run_memory_conflict_version(conflict_task);
    let cached = run_cached_review_version(cached_task);
    let guarded = run_abort_guard_version(guard_task);
    println(conflicted.summary);
    println(cached.summary);
    println(guarded.summary);
    if checkpointed.summary == "" {
        return 1;
    }
    if conflicted.summary == "" {
        return 1;
    }
    if cached.summary == "" {
        return 1;
    }
    if guarded.summary == "" {
        return 1;
    }
    return 0;
}

flow runtime_main(args: Array<string>) -> i32 ![Error<IOError>, Memory] ~ MultiAgentRuntimeGate
{
    return main(args);
}
