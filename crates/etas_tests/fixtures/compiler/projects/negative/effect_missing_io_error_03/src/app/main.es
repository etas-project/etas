module app.main;

import std.collections.Array;
import app.effects.model.{sample_input};
import app.effects.service.{run_effect_project};

flow main(args: Array<string>) -> i32 ![]
{
    let input = sample_input();
    let result = run_effect_project(input);
    if result > 0 {
        return 0;
    }
    return 1;
}
