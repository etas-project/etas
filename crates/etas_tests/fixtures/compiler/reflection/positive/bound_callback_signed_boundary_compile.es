module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow Worker<effect E>(value: bool, f: (i8) -> unit ![E]) -> bool ![E] {
    f(-128);
    return value;
}

flow noop(value: i8) -> unit ![] { return; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, (i8) -> unit ![]) -> bool ![] = Worker;
    let body = match parse_body("f(-0x80); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false, noop) { return 1; }
    if current(true, noop) { return 2; }
    return 0;
}
