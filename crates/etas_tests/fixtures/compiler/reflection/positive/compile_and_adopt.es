module tests.reflection;
import std.compiler.{parse_body, compile, CompileError};

agent Worker() -> i32 { return 1; }

flow verify() -> bool ![Error<CompileError>] {
    var current = Worker;
    let old = current;
    let body = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    current = compile((^current).with_body(body));
    return current() == 42 && old() == 1 && Worker() == 1;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
