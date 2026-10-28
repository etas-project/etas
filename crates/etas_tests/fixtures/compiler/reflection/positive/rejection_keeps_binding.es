module tests.reflection;
import std.compiler.{parse_body, check, compile, CompileError};

agent Worker() -> i32 { return 1; }

flow verify() -> bool ![Error<CompileError>] {
    let body = match parse_body("return false;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let edited = (^Worker).with_body(body);
    let report = check(edited);
    if report.status != "rejected" || report.diagnostics.len() == 0 { return false; }
    return match compile(edited)? {
        Err(error) => error.kind == "rejected" && Worker() == 1,
        Ok(_) => false,
    };
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
