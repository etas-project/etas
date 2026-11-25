module app.agent.planner;

import std.agent.prompt.Prompt;
import std.json.JsonValue;
import std.runtime.limits.Tokens;
import app.agent.model.{AgentTask};

@limits([Tokens(128)])
public agent Planner = {};

impl Planner {
    public flow run(self, input: AgentTask) -> string {
        return perform infer(Prompt.new()
            .system(Trusted("Return one concise implementation plan sentence for the task."))
            .data(JsonValue.Object({
                "topic" => JsonValue.String(input.topic),
                "draft" => JsonValue.String(input.draft),
            })));
    }
}
