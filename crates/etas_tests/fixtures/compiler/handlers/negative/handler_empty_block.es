module tests.compiler.handlers.negative.handler_empty_block;

effect Gate {
    action request(message: string) -> bool;
}

flow main(args: Array<string>) -> i32 ![]
{
    let invalid: ![Gate => []] = handler {
    };
    return 0;
}
