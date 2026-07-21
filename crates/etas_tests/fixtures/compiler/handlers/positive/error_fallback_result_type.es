module tests.compiler.handlers.positive.error_fallback_result_type;

let IndexFallback: ![Error<IndexError> => [] for string] = handler {
    Error<IndexError>.raise(err) => {
        finish "fallback";
    }
};

flow first_label() -> string ![Error<IndexError>]
{
    let labels = ["left", "right"];
    return labels[0];
}

flow main(args: Array<string>) -> i32 ![]
{
    let value = first_label() with IndexFallback;
    if value == "fallback" {
        return 1;
    }
    return 0;
}
