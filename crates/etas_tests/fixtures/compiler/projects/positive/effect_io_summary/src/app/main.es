module app.main;

import std.collections.Array;
import app.effects.service.{run_effect_project};

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var seed = 1;
    if args.len() > 0 {
        seed = seed + 1;
    }
    let result = run_effect_project(seed);
    if result > 0 {
        return 0;
    }
    return 1;
}
