module app.agent.runtime_variants;

import std.core.{abort};
import std.io.{println};
import std.memory.{version};
import std.runtime.{checkpoint};

public type RuntimeTask = {
    topic: string,
    draft: string,
};

public type RuntimeReviewInput = {
    topic: string,
    plan: string,
    evidence: string,
};

public type RuntimeEvidence = {
    evidence: string,
};

public type RuntimeReview = {
    summary: string,
    score: i32,
};

alias RuntimeMemorySchema = MemoryRegion<{
    Drafts: Store<string, string>,
    Evidence: Store<string, string>,
    Reviews: Store<string, RuntimeReview>,
}>;

let RuntimeMemory =
    std.memory.region<RuntimeMemorySchema>(
        stable_id = "multi_agent_system_runtime_variants",
        store = "multi-agent-system-runtime-variants"
    );

public tool EvidenceLookup(input: RuntimeTask) -> string {
    return "local evidence for " + input.topic;
}

@limits([Tokens(96)])
public agent Planner(input: RuntimeTask) -> string
{
    return Prompt.new()
        .system(Trusted("Return a concise plan for the task."))
        .data(input);
}

@tools([EvidenceLookup], choice = "required")
@limits([Tokens(256)])
public agent Researcher(input: RuntimeTask) -> RuntimeEvidence
{
    return Prompt.new()
        .system(Trusted("Your first assistant response must be an EvidenceLookup tool call, not text. EvidenceLookup arguments must be a JSON object with topic:string and draft:string copied from input. After the tool result is returned, answer with JSON containing evidence:string."))
        .data(input);
}

@limits([Tokens(128)])
public agent Reviewer(input: RuntimeReviewInput) -> RuntimeReview
{
    return Prompt.new()
        .system(Trusted("Return JSON with summary:string and score:i32."))
        .data(input);
}

public flow runtime_task(topic: string, draft: string) -> RuntimeTask {
    return RuntimeTask {
        topic = topic,
        draft = draft,
    };
}

public flow runtime_review_input(
    topic: string,
    plan: string,
    evidence: string,
) -> RuntimeReviewInput {
    return RuntimeReviewInput {
        topic = topic,
        plan = plan,
        evidence = evidence,
    };
}

public flow fallback_review(topic: string) -> RuntimeReview {
    return RuntimeReview {
        summary = "fallback review for " + topic,
        score = 1,
    };
}

public flow run_three_agent_version(task: RuntimeTask) -> RuntimeReview
    ![Error<IOError>, Memory]
{
    let plan = Planner.run(task);
    let evidence = Researcher.run(task);
    let review = Reviewer.run(runtime_review_input(task.topic, plan, evidence.evidence));
    println(review.summary);
    return review;
}

public flow run_retry_checkpoint_version(task: RuntimeTask) -> RuntimeReview
    ![Error<IOError>, Memory]
{
    checkpoint("multi-agent-retry-start");
    var plan = "";
    retry limit Attempts(2) {
        plan = Planner.run(task);
    }
    let evidence = Researcher.run(task);
    let review = Reviewer.run(runtime_review_input(task.topic, plan, evidence.evidence));
    checkpoint("multi-agent-retry-end");
    println(review.summary);
    return review;
}

public flow run_memory_conflict_version(task: RuntimeTask) -> RuntimeReview
    ![Error<IOError>, Memory]
{
    RuntimeMemory.Drafts.put(task.topic, task.draft);
    let review = run_three_agent_version(task);
    RuntimeMemory.Reviews.upsert(task.topic, review);
    let handled = handle {
        let stale = version("999");
        RuntimeMemory.Reviews.put_versioned(task.topic, review, stale);
        review
    } with {
        Error<MemoryConflict>.raise(conflict) => {
            finish fallback_review(task.topic);
        }
    };
    RuntimeMemory.Reviews.upsert(task.topic, handled);
    return handled;
}

public flow run_cached_review_version(task: RuntimeTask) -> RuntimeReview
    ![Error<IOError>, Memory]
{
    RuntimeMemory.Drafts.upsert(task.topic, task.draft);
    let review = run_retry_checkpoint_version(task);
    RuntimeMemory.Evidence.upsert(task.topic, "cached evidence");
    RuntimeMemory.Reviews.upsert(task.topic, review);
    let cached = RuntimeMemory.Reviews.get(task.topic);
    checkpoint("multi-agent-cache-read");
    return review;
}

public flow run_abort_guard_version(task: RuntimeTask) -> RuntimeReview
    ![Error<IOError>, Memory]
{
    if task.topic == "" {
        abort("runtime variant requires a topic");
    }
    let review = run_memory_conflict_version(task);
    if review.score < 0 {
        return fallback_review(task.topic);
    }
    return review;
}
