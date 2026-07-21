module tests.compiler.handlers.positive.nested_handlers;

effect Gate {
    action request(message: string) -> bool;
}

let IndexFallback: ![Error<IndexError> => [] for string] = handler {
    Error<IndexError>.raise(err) => {
        finish "missing";
    }
};

let AutoGate: ![Gate => []] = handler {
    Gate.request(req) => {
        resume true;
    }
};

flow guarded_label() -> string ![Gate.request, Error<IndexError>]
{
    let ok = perform Gate.request("read label");
    let labels = ["left", "right"];
    if ok {
        return labels[0];
    }
    return labels[1];
}

flow main(args: Array<string>) -> i32 ![]
{
    let value = handle {
        guarded_label() with AutoGate
    } with IndexFallback;
    if value == "missing" {
        return 1;
    }
    return 0;
}
