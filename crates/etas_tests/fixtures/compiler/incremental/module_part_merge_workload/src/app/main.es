module app.main;

import std.collections.Array;
import app.pipeline.{run_pipeline};

flow main(args: Array<string>) -> i32 ![]
{
    let value: i32 = run_pipeline(5);
    return value;
}
