module tests.compiler.handlers.negative.resume_twice;

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
            let repeat = () => { resume true; };
            repeat();
            repeat();
        }
    };
    if accepted {
        return 0;
    }
    return 1;
}
