module tests.reflection;
import std.compiler.{check, parse_body};

flow Worker(value: bool, f: () -> unit ![]) -> bool ![] {
    if value { f(); }
    return value;
}

flow main(args: Array<string>) -> i32 {
    let body = match parse_body("f(); return value;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let report = check((^Worker).with_body(body));
    if report.status != "rejected" { return 2; }
    if report.diagnostics.len() == 0 { return 3; }
    return 0;
}
