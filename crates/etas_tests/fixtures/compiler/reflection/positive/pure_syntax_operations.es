module tests.reflection;
import std.compiler.{parse_body, format};

agent Worker() -> i32 { return 1; }

flow verify() -> bool {
    let original = ^Worker;
    let syntax = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let edited = original.with_body(syntax);
    return format(syntax) == "return 42;"
        && edited.effects.escaping == None
        && original.effects.escaping != None
        && edited.identity == original.identity
        && edited.revision != original.revision;
}

flow main(args: Array<string>) -> i32 {
    if verify() { return 0; }
    return 1;
}
