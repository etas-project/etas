module tests.reflection;
import std.compiler.diff;
agent Worker() -> i32 { return 1; }
flow main() -> unit {
    var changes = diff(^Worker, ^Worker);
    changes.signature_changed = true;
    return;
}
