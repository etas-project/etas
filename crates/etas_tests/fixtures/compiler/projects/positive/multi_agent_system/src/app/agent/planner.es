module app.agent.planner;

import app.agent.model.{AgentTask};

@limits([Tokens(128)])
public agent Planner(input: AgentTask) -> string
{
    return Prompt.new()
        .system(Trusted("Return one concise implementation plan sentence for the task."))
        .data(input);
}
