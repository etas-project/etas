module tests.reflection;
import std.compiler.{parse_body, compile, CompileError};

agent Worker() -> i32 { return 1; }

flow verify() -> bool ![Error<CompileError>] {
    let first_body = match parse_body("return 2;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let second_body = match parse_body("return 3;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let first = compile((^Worker).with_body(first_body));
    let second = compile((^first).with_body(second_body));
    return Worker() == 1 && first() == 2 && second() == 3
        && (^first).identity == (^second).identity
        && (^first).revision != (^second).revision;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
