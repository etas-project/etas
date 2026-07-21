module tests.compiler.handlers.positive.index_error_inline_handler;

flow first_or_default() -> string ![]
{
    let labels = ["left", "right"];
    return handle {
        labels[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "default";
        }
    };
}

flow main(args: Array<string>) -> i32 ![]
{
    let value = first_or_default();
    if value == "default" {
        return 1;
    }
    return 0;
}
