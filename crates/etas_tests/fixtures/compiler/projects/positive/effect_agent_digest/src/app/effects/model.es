module app.effects.model;

public type AgentRequest = {
    body: string,
};

public type AgentResult = {
    summary: string,
    score: i32,
};

public flow sample_request() -> AgentRequest ![]
{
    return AgentRequest { body = "review package effect project" };
}
