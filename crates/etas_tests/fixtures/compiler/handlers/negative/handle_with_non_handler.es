module tests.compiler.handlers.negative.handle_with_non_handler;

flow main(args: Array<string>) -> i32 ![]
{
    let value = handle {
        1
    } with 2;
    return value;
}
