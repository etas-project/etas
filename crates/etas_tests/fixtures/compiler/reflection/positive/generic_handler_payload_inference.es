module tests.reflection;

effect Echo { action value<T>(payload: T) -> T; }

flow main(args: Array<string>) -> i32 {
    let values = handle {
        let number = perform Echo.value(7);
        let text = perform Echo.value("ok");
        (number, text)
    } with {
        Echo.value<_>(payload) => { resume payload; }
    };
    let (number, text) = values;
    if number == 7 && text == "ok" { return 0; }
    return 1;
}
