module app.main;

import std.collections.Array;
import std.memory.StorageError;
import app.runtime.model.{request};
import app.runtime.service.{run_runtime};

spec RuntimeGate: trace = +Agentic & +Memory & +Console;

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<StorageError>, Memory.read, Memory.write] ~ RuntimeGate
{
    let input = request("phase1-runtime", "draft body");
    run_runtime(input);
    return 0;
}
