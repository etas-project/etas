module app.effects.service;

import std.io.{println};
import app.effects.model.{EffectInput};

public flow run_effect_project(input: EffectInput) -> i32 ![Error<IOError>]
{
    let labels = [input.body, "fallback"];
    let selected = labels[0];
    println(selected);
    return 5;
}
