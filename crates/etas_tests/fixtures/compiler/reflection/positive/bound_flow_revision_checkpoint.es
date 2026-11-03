module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};
import std.runtime.checkpoint;

flow Worker<T>(value: T) -> T ![] { return value; }

flow replacement(original: (bool) -> bool ![]) -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let body = match parse_body("checkpoint(\"compiled bound flow\"); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    return compile((^original).with_body(body));
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool) -> bool ![] = Worker;
    let current = replacement(original);
    if !current(false) { return 1; }
    if current(true) { return 2; }
    return 0;
}
