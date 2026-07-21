module tests.compiler.handlers.negative.reject_with_handler_keyword;

flow first_label() -> string ![Error<IndexError>]
{
    let labels = ["left"];
    return labels[2];
}

flow main(args: Array<string>) -> i32 ![]
{
    let value = first_label() with handler {
        Error<IndexError>.raise(error) => {
            finish "fallback";
        }
    };
    if value == "fallback" {
        return 0;
    }
    return 1;
}
