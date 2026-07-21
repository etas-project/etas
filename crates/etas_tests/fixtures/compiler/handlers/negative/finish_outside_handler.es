module tests.compiler.handlers.negative.finish_outside_handler;

flow main(args: Array<string>) -> i32 ![]
{
    finish 1;
}
