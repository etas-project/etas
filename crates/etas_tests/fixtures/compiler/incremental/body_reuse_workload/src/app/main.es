module app.main;

import std.collections.Array;
import app.body.entry.{run_body_workload};

flow main(args: Array<string>) -> i32 ![]
{
    let value: i32 = run_body_workload(7);
    return value;
}
