module app.runtime.service;

import std.io.{println};
import std.memory.{version};
import std.runtime.{checkpoint};
import app.runtime.agent.{RuntimePlanner, RuntimeReviewer};
import app.runtime.model.{RuntimeRequest, RuntimeReview, fallback_review, review_request};

alias RuntimeMemorySchema = MemoryRegion<{
    Reviews: Store<string, RuntimeReview>,
    Drafts: Store<string, string>,
}>;

let RuntimeMemory =
    std.memory.region<RuntimeMemorySchema>(
        stable_id = "full_phase1_runtime_memory",
        store = "full-phase1-runtime"
    );

public flow run_runtime(request: RuntimeRequest) -> RuntimeReview ![Error<IOError>, Memory]
{
    checkpoint("before-runtime-agent");
    RuntimeMemory.Drafts.put(request.topic, request.draft);

    let plan = RuntimePlanner.run(request);
    var review = fallback_review(request.topic);
    retry limit Attempts(2) {
        review = RuntimeReviewer.run(review_request(request, plan));
    }
    println(review.summary);

    RuntimeMemory.Reviews.upsert(request.topic, review);
    let handled = handle {
        let stale = version("999");
        RuntimeMemory.Reviews.put_versioned(request.topic, review, stale);
        review
    } with {
        Error<MemoryConflict>.raise(conflict) => {
            finish fallback_review(request.topic);
        }
    };
    if handled.score < 0 {
        return handled;
    }

    RuntimeMemory.Reviews.upsert(request.topic, handled);
    let cached = RuntimeMemory.Reviews.get(request.topic);
    checkpoint("after-runtime-memory");

    return handled;
}
