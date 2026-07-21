module app.main;

import std.collections.Array;
import app.effects.model.{sample_request};
import app.effects.service.{run_effect_project};

flow main(args: Array<string>) -> i32 ![]
{
    let request = sample_request();
    let result = run_effect_project(request);
    if result > 0 {
        return 0;
    }
    return 1;
}
