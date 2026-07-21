module app.runtime.agent;

import app.runtime.model.{RuntimeRequest, RuntimeReview, RuntimeReviewRequest};
import app.runtime.tools.{EvidenceLookup};

@limits([Tokens(96)])
public agent RuntimePlanner(input: RuntimeRequest) -> string
{
    return Prompt.new()
        .system(Trusted("Return one concise implementation plan sentence for the requested runtime task."))
        .data(input);
}

@tools([EvidenceLookup], choice = "required")
@limits([Tokens(256)])
public agent RuntimeReviewer(input: RuntimeReviewRequest) -> RuntimeReview
{
    return Prompt.new()
        .system(Trusted("Call EvidenceLookup exactly once before the final answer. EvidenceLookup arguments must be a JSON object with topic:string and draft:string copied from input.request. After the tool result, return a JSON object with fields summary:string and score:i32."))
        .data(input);
}
