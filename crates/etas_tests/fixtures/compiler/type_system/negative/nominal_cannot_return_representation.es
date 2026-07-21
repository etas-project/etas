module tests.compiler.type_system.negative.nominal_cannot_return_representation;

type HeaderName = string;

flow raw(header: HeaderName) -> string
{
    return header;
}

flow main(args: Array<string>) -> i32
{
    return 0;
}
