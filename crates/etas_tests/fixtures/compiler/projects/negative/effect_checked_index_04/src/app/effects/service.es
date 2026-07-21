module app.effects.service;

import app.effects.model.{EffectInput};

public flow run_effect_project(input: EffectInput) -> i32 ![]
{
    let labels = [input.body, "fallback"];
    let selected = labels[0];
    return 5;
}
