module tests.compiler.handlers.negative.finish_type_mismatch;

let BadFallback: ![Error<IndexError> => [] for string] = handler {
    Error<IndexError>.raise(err) => {
        let wrong: i32 = 1;
        finish wrong;
    }
};

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
