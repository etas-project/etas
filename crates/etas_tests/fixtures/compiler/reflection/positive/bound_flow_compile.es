module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow Worker<T>(value: T) -> T ![] { return value; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool) -> bool ![] = Worker;
    let body = match parse_body("return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false) { return 1; }
    if current(true) { return 2; }
    if !original(true) { return 3; }
    return 0;
}
