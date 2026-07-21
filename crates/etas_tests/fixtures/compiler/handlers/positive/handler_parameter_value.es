module tests.compiler.handlers.positive.handler_parameter_value;

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

flow run_with(gate_handler: ![Gate => []]) -> bool ![]
{
    return ask_once() with gate_handler;
}

flow main(args: Array<string>) -> i32 ![]
{
    if run_with(AutoGate) {
        return 0;
    }
    return 1;
}
