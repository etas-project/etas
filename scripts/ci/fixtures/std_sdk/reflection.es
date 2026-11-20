module app.reflection;

flow answer() -> i32 ![] {
    return 42;
}

flow main(args: Array<string>) -> i32 ![] {
    let definition = ^answer;
    return if definition.id == (^answer).id {
        answer()
    } else {
        0
    };
}
