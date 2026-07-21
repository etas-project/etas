module tests.compiler.handlers.negative.implicit_handler_fallback;

effect Error {
    action stop() -> never;
}

let StopFallback: ![Error for string] = handler {
    Error.stop() => {
        "fallback"
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    let value = handle perform Error.stop() with StopFallback;
    if value == "fallback" {
        return 0;
    }
    return 1;
}
