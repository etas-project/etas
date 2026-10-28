module tests.reflection;
agent Worker() -> i32 { return 1; }
flow main() -> unit { let definition = ^Worker(); return; }
