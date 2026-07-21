module tests.compiler.handlers.negative.handler_bare_effect_tag;

effect Gate {
    action request(message: string) -> bool;
}

flow main(args: Array<string>) -> i32 ![]
{
    let invalid: ![Gate => []] = handler {
        Gate(req) => {
            resume true;
        }
    };
    return 0;
}
