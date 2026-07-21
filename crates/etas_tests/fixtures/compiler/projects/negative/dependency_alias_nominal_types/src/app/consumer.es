module app.consumer;

import app.model.{project_path};
import identity.types.{ExternalPath, ExternalProjectId, ExternalReview, ExternalUserId, accept_path, accept_project, publish_review};

public flow run() -> i32
{
    let raw: string = project_path();
    let path: ExternalPath = raw;
    let echoed: string = accept_path(path);
    let user: ExternalUserId = echoed;
    let project: ExternalProjectId = user;
    let review = {
        accepted = true,
        notes = "looks good"
    };
    let review_status: i32 = publish_review(review);
    return accept_project(project) + review_status;
}
