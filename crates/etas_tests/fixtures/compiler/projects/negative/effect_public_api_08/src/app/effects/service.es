module app.effects.service;

import std.io.{println};
import app.effects.model.{EffectInput};

public flow run_effect_project(input: EffectInput) -> i32 ![]
{
    println(input.body);
    return 3;
}
