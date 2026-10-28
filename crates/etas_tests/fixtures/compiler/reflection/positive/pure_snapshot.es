module tests.reflection;

agent Worker() -> i32 {
    return abort("reflection must not execute this body");
}

flow verify() -> bool {
    let definition = ^Worker;
    return definition.signature != None
        && definition.body != None
        && definition.identity == (^Worker).identity
        && definition.revision == (^Worker).revision;
}

flow main(args: Array<string>) -> i32 {
    if verify() { return 0; }
    return 1;
}
