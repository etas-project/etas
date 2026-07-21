module tests.compiler.handlers.negative.resume_captured_by_lambda;

effect Gate {
    action request(message: string) -> bool;
}

flow ask_once() -> bool ![Gate.request]
{
    return perform Gate.request("ship");
}

flow main(args: Array<string>) -> i32 ![]
{
    let accepted = ask_once() with {
        Gate.request(req) => {
            let later: i32 -> i32 = (value: i32) => {
                resume true;
                return value;
            };
            return later(1) > 0;
        }
    };
    if accepted {
        return 0;
    }
    return 1;
}
