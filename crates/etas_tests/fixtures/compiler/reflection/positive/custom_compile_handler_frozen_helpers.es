module tests.reflection;
import std.compiler.{AgentDef, CompileError, compile};

agent Leaf() -> i32 { return 7; }

flow build<A>(definition: AgentDef<A>) -> A ![Error<CompileError>] {
    return compile(definition);
}

flow make<A>(definition: AgentDef<A>) -> () -> A ![Error<CompileError>] {
    return () => build(definition);
}

agent Worker() -> i32 ![Error<CompileError>] {
    let first = handle { build(^Leaf) } with { Code.compile(_) => { resume Leaf; } };
    let request = make(^Leaf);
    let second = handle { request() } with { Code.compile(_) => { resume Leaf; } };
    if first() == 7 && second() == 7 { return 0; }
    return 1;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let definition = ^Worker;
    let body = match definition.body {
        Some(body) => body,
        None => { return 2; },
    };
    let candidate = compile(definition.with_body(body));
    return candidate();
}
