module app.runtime.model;

public type RuntimeRequest = {
    topic: string,
    draft: string,
};

public type RuntimeReview = {
    summary: string,
    score: i32,
};

public type RuntimeReviewRequest = {
    request: RuntimeRequest,
    plan: string,
};

public flow request(topic: string, draft: string) -> RuntimeRequest {
    return RuntimeRequest {
        topic = topic,
        draft = draft,
    };
}

public flow review_request(request: RuntimeRequest, plan: string) -> RuntimeReviewRequest {
    return RuntimeReviewRequest {
        request = request,
        plan = plan,
    };
}

public flow fallback_review(topic: string) -> RuntimeReview {
    return RuntimeReview {
        summary = "fallback review for " + topic,
        score = 1,
    };
}
