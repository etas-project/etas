module app.effects.service;

import std.io.{println};
import app.effects.model.{EffectInput, EffectOutput};

@model(model = "gpt-5.5")
agent ProjectEffectAgent(input: EffectInput) -> EffectOutput
{
    return Prompt.new()
        .system(Trusted("Summarize negative project effect."))
        .data(input);
}

public flow run_effect_project(input: EffectInput) -> i32 ![]
{
    let result = ProjectEffectAgent.run(input);
    println(result.summary);
    return result.score;
}
