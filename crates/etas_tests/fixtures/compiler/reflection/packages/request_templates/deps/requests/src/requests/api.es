module requests.api;
import std.compiler.{AgentDef, CompileError, compile};

public type Node = { next: Option<Node> };

agent Internal() -> i32 { return 3; }

public flow build_internal() -> unit ![Error<CompileError>] {
    let value = compile(^Internal);
    return;
}

public flow build<A>(definition: AgentDef<A>) -> A ![Error<CompileError>] {
    return compile(definition);
}

public flow apply<T, effect E>(callback: () -> T ![E]) -> T ![E] {
    return callback();
}

public flow wrap<T, effect E>(callback: () -> T ![E]) -> () -> T ![E] {
    return () => callback();
}

public flow shield<T, effect E>(callback: () -> T ![E]) -> unit ![E] {
    handle { callback(); } with { Code.compile(_) => { finish (); } };
    return;
}
