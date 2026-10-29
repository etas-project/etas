module tests.reflection;
import std.compiler.diff;

flow Worker(value: i32) -> i32 ![] { return abort("diff does not invoke its target"); }

flow main(args: Array<string>) -> i32 {
    let definition = ^Worker;
    let report = diff(definition, definition);
    if report.old_body_available || report.new_body_available { return 1; }
    if report.body_changed != None { return 2; }
    if report.identity != definition.dependencies.lexical_owner { return 3; }
    if definition.effects.requested != None { return 4; }
    if report.requested_changed != None { return 5; }
    return 0;
}
