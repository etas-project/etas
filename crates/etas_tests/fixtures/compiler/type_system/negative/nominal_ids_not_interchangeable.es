module tests.compiler.type_system.negative.nominal_ids_not_interchangeable;

type UserId = string;
type ProjectId = string;

flow needs_project(id: ProjectId) -> i32
{
    return 1;
}

flow main(args: Array<string>) -> i32
{
    let user: UserId = UserId("user-1");
    return needs_project(user);
}
