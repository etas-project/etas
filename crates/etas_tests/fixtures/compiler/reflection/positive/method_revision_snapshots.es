module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

agent Worker(value: bool) -> bool ![] { return value; }
impl agent Worker {
    public flow helper(value: bool) -> bool ![] { return value; }
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original = ^Worker;
    let old_method = ^Worker.helper;
    let body = match parse_body("return !value;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let current = compile(original.with_body(body));
    let definition = ^current;
    let helper = ^current.helper;
    let method = match definition.methods.get(0) { Some(method) => method, None => { return 2; }, };
    if (^current.run).revision != definition.revision { return 3; }
    if (^Worker.run).revision != original.revision { return 4; }
    if helper.revision != method.revision || helper.dependencies != method.dependencies { return 5; }
    if helper.revision.key == old_method.revision.key { return 6; }
    if helper.revision.content != old_method.revision.content { return 7; }
    if helper.body != None || method.body != None { return 8; }
    if helper.dependencies.lexical_owner != original.dependencies.lexical_owner { return 9; }
    if definition.revision == original.revision { return 10; }
    return 0;
}
