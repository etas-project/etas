module tests.reflection;
import std.compiler.{parse_body, compile, CompileError};

flow helper() -> i32 { return 7; }
agent Worker() -> i32 { return helper(); }

flow verify() -> bool ![Error<CompileError>] {
    let helper = () => 999;
    let body = match parse_body("return helper();") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised() == 7 && helper() == 999;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
