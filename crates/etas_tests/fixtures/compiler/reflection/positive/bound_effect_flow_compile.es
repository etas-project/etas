module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow Worker<T, effect E>(value: T, f: () -> unit ![E]) -> T ![E] {
    f();
    return value;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, () -> unit ![]) -> bool ![] = Worker;
    let body = match parse_body("f(); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    let noop = () => { return; };
    if !current(false, noop) { return 1; }
    if current(true, noop) { return 2; }
    return 0;
}
