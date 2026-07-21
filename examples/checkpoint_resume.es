module examples.checkpoint_resume;

import std.runtime.{checkpoint};

flow main(args: Array<string>) -> i32 {
    checkpoint("after-start");
    return 0;
}
