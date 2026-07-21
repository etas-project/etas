module tests.compiler.handlers.negative.top_level_handle_expression;

effect Gate {
    action request(message: string) -> bool;
}

let AutoGate: ![Gate => []] = handler {
    Gate.request(req) => {
        resume true;
    }
};

handle perform Gate.request("ship") with AutoGate;

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
