module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow normalize(value: bool) -> bool ![] { return value; }
flow Worker() -> ((bool) -> bool ![]) ![] { return normalize; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("return normalize;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let current = compile((^Worker).with_body(body));
    let returned = current();
    if !returned(true) { return 6; }
    if returned(false) { return 7; }
    let definition = ^returned;
    let source = ^normalize;
    if definition.id != source.id { return 2; }
    if definition.revision != source.revision { return 3; }
    if definition.dependencies != source.dependencies { return 4; }
    if definition.body != None { return 5; }
    return 0;
}
