module tests.reflection;
import std.compiler.compile;

agent Worker() -> i32 { return 7; }
agent Other() -> i32 { return 9; }

flow main(args: Array<string>) -> i32 {
    let candidate = handle {
        let first = compile(^Worker);
        compile(^Other)
    } with {
        Code.compile(_) => { resume Worker; }
    };
    return candidate();
}
