module tests.compiler.handlers.negative.bare_handler_arm_block;

effect Gate {
    action request(message: string) -> bool;
}

flow main(args: Array<string>) -> i32 ![]
{
    let invalid = {
        Gate.request(req) => {
            resume true;
        }
    };
    return 0;
}
