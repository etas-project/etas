module tests.compiler.handlers.negative.handler_handled_row_mismatch;

let BadHandler: ![Error<IOError> => [] for string] = handler {
    Error<IndexError>.raise(err) => {
        finish "fallback";
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
