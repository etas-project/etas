module tests.reflection;
import std.compiler.{compile, CompileError};

agent Worker() -> i32 { return 7; }
agent Other() -> i32 { return 99; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let candidate = handle {
        let deferred = () => compile(^Other);
        compile(^Worker)
    } with {
        Code.compile(_) => { resume Worker; }
    };
    if candidate() == 7 { return 0; }
    return 1;
}
