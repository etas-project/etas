module app.effects.service;

import std.io.{println};
import app.effects.model.{AgentRequest, AgentResult};

@model(model = "gpt-5.5")
agent ProjectEffectAgent(input: AgentRequest) -> AgentResult
{
    return Prompt.new()
        .system(Trusted("Summarize the project effect boundary."))
        .data(input);
}

public flow run_effect_project(input: AgentRequest) -> i32 ![Error<IOError>]
{
    let result = ProjectEffectAgent.run(input);
    println(result.summary);
    return result.score;
}
