module tests.reflection;
import std.compiler.diff;
agent Worker() -> i32 { return 1; }
agent Other() -> i32 { return 2; }
flow main() -> unit { let changes = diff(^Worker, ^Other); return; }
