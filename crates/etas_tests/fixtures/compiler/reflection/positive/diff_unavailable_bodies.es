module tests.reflection;
import std.compiler.diff;

agent Worker() -> i32 { return 7; }

flow main(args: Array<string>) -> i32 {
    let definition = ^Worker;
    let changes = diff(definition, definition);
    if !changes.old_body_available && !changes.new_body_available
        && changes.body_changed == None && !changes.signature_changed
    { return 0; }
    return 1;
}
