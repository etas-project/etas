module tests.reflection;
import std.compiler.{parse_body, check, CompileError};

agent Worker;

flow verify() -> bool ![Error<CompileError>] {
    let original = ^Worker;
    if original.signature != None || original.body != None { return false; }
    let body = match parse_body("return 1;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let edited = original.with_body(body);
    let report = check(edited);
    return edited.signature == None && report.status == "rejected";
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
