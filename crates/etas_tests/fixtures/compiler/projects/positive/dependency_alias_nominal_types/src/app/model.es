module app.model;

public flow project_path() -> string
{
    return "/workspace/project";
}

public flow normalize_path(value: string) -> string
{
    return value;
}
