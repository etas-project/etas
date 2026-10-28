module tests.reflection;
import std.compiler.{compile, CompileError};

agent Worker() -> i32 { return 7; }

flow verify() -> bool ![Error<CompileError>] {
    let candidates = handle {
        let first = compile(^Worker);
        let second = perform Code.compile(^Worker);
        (first, second)
    } with {
        Code.compile(_) => { resume Worker; }
    };
    let (first, second) = candidates;
    return first() == 7 && second() == 7;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    if verify() { return 0; }
    return 1;
}
