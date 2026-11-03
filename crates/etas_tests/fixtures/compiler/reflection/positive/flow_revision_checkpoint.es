module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};
import std.runtime.checkpoint;

flow Worker(value: bool) -> bool ![] {
    checkpoint("source flow");
    return value;
}

flow replacement() -> ((bool) -> bool ![]) ![Error<CompileError>] {
    let body = match parse_body("checkpoint(\"compiled flow\"); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    return compile((^Worker).with_body(body));
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let current = replacement();
    let alias = current;
    if !current(false) { return 2; }
    if alias(true) { return 3; }
    if (^current).revision == (^Worker).revision { return 4; }
    return 0;
}
