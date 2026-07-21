module tests.compiler.type_system.negative.raw_string_to_user_id;

type UserId = string;

flow needs_user(id: UserId) -> i32
{
    return 1;
}

flow main(args: Array<string>) -> i32
{
    let raw: string = "user-1";
    return needs_user(raw);
}
