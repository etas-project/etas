module app.runtime.agent;

import std.agent.prompt.Prompt;
import std.json.JsonValue;
import std.runtime.limits.Tokens;
import app.runtime.model.{RuntimeRequest, RuntimeReview, RuntimeReviewRequest};
import app.runtime.tools.{EvidenceLookup};

@limits([Tokens(96)])
public agent RuntimePlanner = {};

impl RuntimePlanner {
    public flow run(self, input: RuntimeRequest) -> string {
        return perform infer(Prompt.new()
            .system(Trusted("Return one concise implementation plan sentence for the requested runtime task."))
            .data(JsonValue.Object({
                "topic" => JsonValue.String(input.topic),
                "draft" => JsonValue.String(input.draft),
            })));
    }
}

@tools([EvidenceLookup], choice = "required")
@limits([Tokens(256)])
public agent RuntimeReviewer = {};

impl RuntimeReviewer {
    public flow run(self, input: RuntimeReviewRequest) -> RuntimeReview {
        return perform infer(Prompt.new()
            .system(Trusted("Call EvidenceLookup exactly once before the final answer. EvidenceLookup arguments must be a JSON object with topic:string and draft:string copied from input.request. After the tool result, return a JSON object with fields summary:string and score:i32."))
            .data(JsonValue.Object({
                "request" => JsonValue.Object({
                    "topic" => JsonValue.String(input.request.topic),
                    "draft" => JsonValue.String(input.request.draft),
                }),
                "plan" => JsonValue.String(input.plan),
            })));
    }
}
