module tests.reflection;
agent Worker() -> i32 { return 1; }
flow main() -> unit { let edited = (^Worker).with_body("return 2;"); return; }
