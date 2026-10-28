module tests.reflection;
import std.compiler.{compile, CompileError};

agent Worker() -> i32 { return 7; }

flow verify() -> bool ![Error<CompileError>] {
    let candidate = handle {
        compile(^Worker)
    } with {
        Code.compile(_) => { resume Worker; }
    };
    return candidate() == 7;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
