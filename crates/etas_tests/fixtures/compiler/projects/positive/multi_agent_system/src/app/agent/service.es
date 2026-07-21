module app.agent.service;

import std.io.{println};
import app.agent.model.{AgentTask, review_input};
import app.agent.planner.{Planner};
import app.agent.reviewer.{Reviewer};

public flow run_multi_agent(task: AgentTask) -> string ![Error<IOError>] {
    let plan = Planner.run(task);
    let review = Reviewer.run(review_input(task.topic, plan));
    println(plan);
    println(review);
    return review;
}

