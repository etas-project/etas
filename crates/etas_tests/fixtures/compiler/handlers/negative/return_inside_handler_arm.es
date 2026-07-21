module tests.compiler.handlers.negative.return_inside_handler_arm;

effect Gate {
    action request(message: string) -> bool;
}

let BadGate: ![Gate => []] = handler {
    Gate.request(req) => {
        return true;
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
