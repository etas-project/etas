module tests.reflection;
import std.compiler.{compile, CompileError};

agent Worker() -> i32 { return 7; }
agent Other() -> i32 { return 99; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let deferred = () => compile(^Other);
    let candidate = handle {
        let first = compile(^Worker);
        deferred()
    } with {
        Code.compile(_) => { resume Worker; }
    };
    return candidate();
}
