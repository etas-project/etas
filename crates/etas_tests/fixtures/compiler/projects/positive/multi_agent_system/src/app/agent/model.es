module app.agent.model;

public type AgentTask = {
    topic: string,
    draft: string,
};

public type ReviewInput = {
    topic: string,
    plan: string,
};

public flow request(topic: string, draft: string) -> AgentTask {
    return AgentTask {
        topic = topic,
        draft = draft,
    };
}

public flow review_input(topic: string, plan: string) -> ReviewInput {
    return ReviewInput {
        topic = topic,
        plan = plan,
    };
}

