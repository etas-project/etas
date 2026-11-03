module tests.reflection;
import std.compiler.diff;

flow first(value: i32) -> i32 { return value; }
flow second(value: string) -> string { return value; }
flow main() -> unit { let changes = diff(^first, ^second); return; }
