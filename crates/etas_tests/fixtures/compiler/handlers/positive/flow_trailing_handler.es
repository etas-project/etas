module tests.compiler.handlers.positive.flow_trailing_handler;

effect Gate {
    action request(message: string) -> bool;
}

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("deploy");
}

flow approved() -> bool
{
    return ask_once();
} with {
    Gate.request(req) => {
        resume true;
    }
}

flow main(args: Array<string>) -> i32 ![]
{
    if approved() {
        return 0;
    }
    return 1;
}
