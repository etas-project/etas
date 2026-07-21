module app.main;

import std.collections.Array;
import app.pipeline.{run_pipeline};
import app.dependency_bridge.{dependency_ready};

flow main(args: Array<string>) -> i32 ![]
{
    let value: i32 = run_pipeline(5);
    let ready: bool = dependency_ready();
    return value;
}
