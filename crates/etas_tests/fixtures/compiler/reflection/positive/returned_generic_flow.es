module tests.reflection;
import std.compiler.{compile, parse_body, CompileError};

flow helper<T, effect E>(value: T, callback: () -> unit ![E]) -> T ![E] {
    callback();
    return value;
}
flow noop() -> unit ![] { return; }
flow Worker() -> ((bool, () -> unit ![]) -> bool ![]) ![] {
    let selected: (bool, () -> unit ![]) -> bool ![] = helper;
    return selected;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("let renamed: (bool, () -> unit ![]) -> bool ![] = helper; return renamed;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let current = compile((^Worker).with_body(body));
    let returned = current();
    if !returned(true, noop) { return 2; }
    if returned(false, noop) { return 3; }
    return 0;
}
