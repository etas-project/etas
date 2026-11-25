module app.agent.reviewer;

import std.agent.prompt.Prompt;
import std.json.JsonValue;
import std.runtime.limits.Tokens;
import app.agent.model.{ReviewInput};

@limits([Tokens(128)])
public agent Reviewer = {};

impl Reviewer {
    public flow run(self, input: ReviewInput) -> string {
        return perform infer(Prompt.new()
            .system(Trusted("Review the proposed plan in one concise sentence."))
            .data(JsonValue.Object({
                "topic" => JsonValue.String(input.topic),
                "plan" => JsonValue.String(input.plan),
            })));
    }
}
