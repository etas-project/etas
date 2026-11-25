module app.effects.service;

import std.io.{println};
import std.agent.prompt.Prompt;
import std.json.JsonValue;
import app.effects.model.{AgentRequest, AgentResult};

@model(model = "gpt-5.5")
agent ProjectEffectAgent = {};

impl ProjectEffectAgent {
    public flow run(self, input: AgentRequest) -> AgentResult {
        return perform infer(Prompt.new()
            .system(Trusted("Summarize the project effect boundary."))
            .data(JsonValue.Object({ "body" => JsonValue.String(input.body) })));
    }
}

public flow run_effect_project(input: AgentRequest) -> i32 ![Error<IOError>]
{
    let result = ProjectEffectAgent {}.run(input);
    println(result.summary);
    return result.score;
}
