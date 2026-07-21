module tests.compiler.type_system.positive.nominal_user_id_explicit_constructor;

type UserId = string;

flow accept_user(id: UserId) -> UserId
{
    return id;
}

flow main(args: Array<string>) -> i32
{
    let raw: string = "user-1";
    let id: UserId = UserId(raw);
    let accepted: UserId = accept_user(id);
    return 0;
}
