module app.support.agent;

import app.support.case.{CaseRequest, CaseSummary};

public flow BuildTriagePrompt(input: CaseRequest) -> Prompt ![] {
    return Prompt.new()
        .system(Trusted("Route the support case to the best queue."))
        .user(Public(input.body));
}

public agent TriageAgent(input: CaseRequest) -> CaseSummary {
    return BuildTriagePrompt(input);
}
