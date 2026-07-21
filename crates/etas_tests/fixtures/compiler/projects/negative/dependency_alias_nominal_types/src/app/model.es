module app.model;

public flow project_path() -> string
{
    return "/workspace/project";
}

public flow fallback_status() -> i32
{
    return 0;
}
