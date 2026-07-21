module tests.compiler.handlers.negative.resume_outside_handler;

flow main(args: Array<string>) -> i32 ![]
{
    resume true;
    return 0;
}
