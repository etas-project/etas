module tests.reflection;
import std.compiler.{check, parse_body};

flow Worker<T, effect E>(value: T, f: () -> unit ![E]) -> T ![E] {
    f();
    return value;
}

flow main(args: Array<string>) -> i32 {
    let original: (bool, () -> unit ![]) -> bool ![] = Worker;
    let body = match parse_body("return !value;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let report = check((^original).with_body(body));
    if report.status != "rejected" { return 2; }
    if report.diagnostics.len() == 0 { return 3; }
    return 0;
}
