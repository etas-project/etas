module tests.reflection;
import std.compiler.{compile, CompileError};

type UserId = string;
agent Worker(value: UserId) -> (() -> UserId ![]) { return () => value; }

flow verify() -> bool ![Error<CompileError>] {
    let value = UserId("stable identity");
    let revised = compile(^Worker);
    return revised(value)() == value && Worker(value)() == value;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
