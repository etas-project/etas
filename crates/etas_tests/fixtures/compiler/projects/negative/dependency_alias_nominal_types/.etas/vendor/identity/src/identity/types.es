module identity.types;

public alias ExternalPath = string;

public type ExternalUserId = string;
public type ExternalProjectId = string;
public type ExternalReview = {
    accepted: bool,
    notes: string
};

public flow accept_path(path: ExternalPath) -> string
{
    return path;
}

public flow accept_project(id: ExternalProjectId) -> i32
{
    return 1;
}

public flow publish_review(review: ExternalReview) -> i32
{
    return 1;
}
