module app.agent.reviewer;

import app.agent.model.{ReviewInput};

@limits([Tokens(128)])
public agent Reviewer(input: ReviewInput) -> string
{
    return perform infer(Prompt.new()
        .system(Trusted("Review the proposed plan in one concise sentence."))
        .data(input));
}
