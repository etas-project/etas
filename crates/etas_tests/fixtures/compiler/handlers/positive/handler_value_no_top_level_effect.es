module tests.compiler.handlers.positive.handler_value_no_top_level_effect;

effect Gate {
    action request(message: string) -> bool;
}

let AutoGate: ![Gate => []] = handler {
    Gate.request(req) => {
        resume true;
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
