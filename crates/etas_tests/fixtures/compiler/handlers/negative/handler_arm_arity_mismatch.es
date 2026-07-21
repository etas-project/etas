module tests.compiler.handlers.negative.handler_arm_arity_mismatch;

effect Gate {
    action request(message: string) -> bool;
}

let BadGate: ![Gate => []] = handler {
    Gate.request() => {
        resume true;
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
