module tests.reflection;
import std.compiler.{parse_body, diff};

flow Worker<T, effect E>(value: T, callback: () -> unit ![E]) -> T ![E] {
    callback();
    callback();
    return value;
}

flow Boolean(value: bool) -> bool ![] { return value; }

flow main(args: Array<string>) -> i32 {
    let selected: (bool, () -> unit ![Console.stdout_write]) -> bool ![Console.stdout_write] = Worker;
    let original = ^selected;
    let body = match parse_body("return true;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let edited = original.with_body(body);
    let changed = diff(original, edited);
    let unchanged = diff(original, original);
    if original.signature.generics.len() == 0
        && original.signature.parameters.len() == 2
        && original.signature.output == (^Boolean).signature.output
        && original.body != None && edited.body != None
        && original.effects.escaping != None && original.effects.requested != None
        && edited.effects.escaping == None && edited.effects.requested == None
        && changed.body_changed == Some(true) && !changed.signature_changed
        && !changed.dependencies_changed && !changed.declared_bound_changed
        && unchanged.body_changed == Some(false)
        && unchanged.requested_changed == Some(false)
    { return 0; }
    return 1;
}
