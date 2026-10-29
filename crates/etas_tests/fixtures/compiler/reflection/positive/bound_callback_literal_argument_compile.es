module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow Worker<T, effect E>(value: T, f: (bool) -> unit ![E]) -> T ![E] {
    f(true);
    return value;
}

flow noop(value: bool) -> unit ![] { return; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, (bool) -> unit ![]) -> bool ![] = Worker;
    let body = match parse_body("f(true); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false, noop) { return 1; }
    if current(true, noop) { return 2; }
    return 0;
}
