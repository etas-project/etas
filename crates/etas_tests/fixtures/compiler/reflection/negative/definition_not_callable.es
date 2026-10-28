module tests.reflection;
agent Worker() -> i32 { return 1; }
flow main() -> i32 { return (^Worker)(); }
