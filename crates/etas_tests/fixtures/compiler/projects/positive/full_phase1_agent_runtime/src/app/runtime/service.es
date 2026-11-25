module app.runtime.service;

import std.io.{println};
import std.memory.{StorageError, version};
import std.runtime.{checkpoint};
import std.task.{run, Scheduler};
import app.runtime.agent.{RuntimePlanner, RuntimeReviewer};
import app.runtime.model.{RuntimeRequest, RuntimeReview, fallback_review, review_request};

type RuntimeMemorySchema = {
    Reviews: Store<string, RuntimeReview>,
    Drafts: Store<string, string>,
};

let RuntimeMemory =
    std.memory.region<RuntimeMemorySchema>(
        stable_id = "full_phase1_runtime_memory",
        store = "full-phase1-runtime"
    );

public flow run_runtime(request: RuntimeRequest) -> RuntimeReview ![Error<IOError>, Error<StorageError>, Memory]
{
    checkpoint("before-runtime-agent");
    RuntimeMemory.Drafts.put(request.topic, request.draft);

    let review = run((scheduler: Scheduler) => {
        let plan = RuntimePlanner {}.run(request);
        var reply = fallback_review(request.topic);
        retry {
            reply = RuntimeReviewer {}.run(review_request(request, plan));
        }
        return reply;
    });
    println(review.summary);

    RuntimeMemory.Reviews.upsert(request.topic, review);
    let handled = handle {
        let stale = version("mv1:0000000000000000000000000000000000000000000000000000000000000000:00000000000000000000000000000000:0000000000000001");
        RuntimeMemory.Reviews.put_versioned(request.topic, review, stale);
        review
    } with {
        Error<StorageError>.raise(error) => {
            if error.code == "condition_conflict" {
                finish fallback_review(request.topic);
            }
            perform Error.raise<StorageError>(error);
        }
    };
    if handled.score < 0 {
        return handled;
    }

    RuntimeMemory.Reviews.upsert(request.topic, handled);
    let cached = RuntimeMemory.Reviews.get(request.topic);
    checkpoint("after-runtime-memory");

    return cached.unwrap();
}
