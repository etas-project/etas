module app.main;

import std.collections.Array;
import app.consumer.{run_consumer};

flow main(args: Array<string>) -> i32 ![]
{
    let value: i32 = run_consumer(3);
    return value;
}
