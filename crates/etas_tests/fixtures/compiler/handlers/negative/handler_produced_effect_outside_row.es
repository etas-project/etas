module tests.compiler.handlers.negative.handler_produced_effect_outside_row;

import std.io.{println};

effect Gate {
    action request(message: string) -> bool;
}

let BadGate: ![Gate => []] = handler {
    Gate.request(req) => {
        println("approval requested");
        resume true;
    }
};

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("ship");
}

flow main(args: Array<string>) -> i32 ![]
{
    let accepted = ask_once() with BadGate;
    if accepted {
        return 0;
    }
    return 1;
}
