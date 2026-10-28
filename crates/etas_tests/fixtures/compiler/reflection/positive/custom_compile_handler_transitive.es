module tests.reflection;
import std.compiler.{AgentDef, CompileError, compile};

agent Worker() -> i32 { return 7; }

flow build<A>(definition: AgentDef<A>) -> A ![Error<CompileError>] {
    return compile(definition);
}

flow apply<T, effect E>(callback: () -> T ![E]) -> T ![E] {
    return callback();
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let request = () => build(^Worker);
    let candidate = handle {
        apply(request)
    } with {
        Code.compile(_) => { resume Worker; }
    };
    if candidate() == 7 { return 0; }
    return 1;
}
