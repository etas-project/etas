module app.support.agent;

import std.agent.prompt.Prompt;
import app.support.case.{CaseRequest, CaseSummary};

public flow BuildTriagePrompt(input: CaseRequest) -> Prompt ![] {
    return Prompt.new()
        .system(Trusted("Route the support case to the best queue."))
        .user(Public { value = input.body });
}

public agent TriageAgent = {};

impl TriageAgent {
    public flow run(self, input: CaseRequest) -> CaseSummary {
        return perform infer(BuildTriagePrompt(input));
    }
}
