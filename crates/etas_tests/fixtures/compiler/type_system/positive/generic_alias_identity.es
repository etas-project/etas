module tests.compiler.type_system.positive.generic_alias_identity;

alias Id<T> = T;
alias Boxed<T> = { value: T };

flow unwrap_string(value: Id<string>) -> string
{
    let raw: string = value;
    return raw;
}

flow unwrap_box(input: Boxed<i32>) -> i32
{
    return input.value;
}

flow main(args: Array<string>) -> i32
{
    let raw: string = "alias";
    let same: Id<string> = raw;
    let value: string = unwrap_string(same);
    let boxed: Boxed<i32> = { value = 42 };
    let number: Id<i32> = unwrap_box(boxed);
    return 0;
}
