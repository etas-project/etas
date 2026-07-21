module tests.compiler.handlers.positive.inline_handler_block;

effect Gate {
    action request(message: string) -> bool;
}

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("ship");
}

flow main(args: Array<string>) -> i32 ![]
{
    let accepted = handle {
        ask_once()
    } with {
        Gate.request(req) => {
            resume true;
        }
    };
    if accepted {
        return 0;
    }
    return 1;
}
