module identity.types;

public alias ExternalPath = string;

public type ExternalUserId = string;
public type ExternalProjectId = string;
public type ExternalReview = {
    accepted: bool,
    notes: string
};

public flow make_user_id(raw: string) -> ExternalUserId
{
    return ExternalUserId(raw);
}

public flow accept_user(id: ExternalUserId) -> ExternalUserId
{
    return id;
}

public flow accept_path(path: ExternalPath) -> string
{
    return path;
}
