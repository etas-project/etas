module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow normalize(value: bool) -> bool ![] { return value; }
flow Worker(f: (bool) -> bool ![], value: bool) -> bool ![] { return value; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("let definition = ^f; let source = ^normalize; return definition.id == source.id && definition.revision == source.revision && definition.dependencies == source.dependencies;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let current = compile((^Worker).with_body(body));
    if !current(normalize, false) { return 2; }
    return 0;
}
