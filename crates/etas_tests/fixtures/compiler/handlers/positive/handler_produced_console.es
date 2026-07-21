module tests.compiler.handlers.positive.handler_produced_console;

import std.io.{println};

effect Gate {
    action request(message: string) -> bool;
}

let HumanGate: ![Gate => Error<IOError>] = handler {
    Gate.request(req) => {
        println("approval requested");
        resume true;
    }
};

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("publish");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    let accepted = ask_once() with HumanGate;
    if accepted {
        return 0;
    }
    return 1;
}
