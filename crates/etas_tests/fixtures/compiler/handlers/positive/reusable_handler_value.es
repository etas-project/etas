module tests.compiler.handlers.positive.reusable_handler_value;

effect Gate {
    action request(message: string) -> bool;
}

let AutoGate: ![Gate => []] = handler {
    Gate.request(req) => {
        resume true;
    }
};

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("ship");
}

flow main(args: Array<string>) -> i32 ![]
{
    let accepted = ask_once() with AutoGate;
    if accepted {
        return 0;
    }
    return 1;
}
