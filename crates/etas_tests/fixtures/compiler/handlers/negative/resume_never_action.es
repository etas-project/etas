module tests.compiler.handlers.negative.resume_never_action;

flow first_label() -> string ![Error<IndexError>]
{
    let labels = ["left", "right"];
    return labels[0];
}

flow main(args: Array<string>) -> i32 ![]
{
    let value = first_label() with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    if value == "fallback" {
        return 1;
    }
    return 0;
}
