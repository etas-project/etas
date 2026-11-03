module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};
import std.runtime.checkpoint;

flow Worker(value: bool) -> bool ![] {
    checkpoint("source flow");
    return value;
}

flow replacement(original: (bool) -> bool ![]) -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let body = match parse_body("checkpoint(\"compiled flow\"); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    return compile((^original).with_body(body));
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let current = replacement(Worker);
    let alias = current;
    if !current(false) { return 2; }
    if alias(true) { return 3; }
    return 0;
}
