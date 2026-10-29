module tests.reflection;
import std.compiler.{check, compile, parse_body, CompileError};

flow Worker<effect E>(value: bool, f: (bool) -> unit ![E]) -> bool ![E] {
    f(value);
    return value;
}

flow noop(value: bool) -> unit ![] { return; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, (bool) -> unit ![]) -> bool ![] = Worker;
    let changed = match parse_body("value = !value; f(value); return value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let report = check((^original).with_body(changed));
    if report.status != "rejected" { return 1; }
    var unproved = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" { unproved = true; }
    }
    if !unproved { return 4; }
    let body = match parse_body("f(value); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false, noop) { return 2; }
    if current(true, noop) { return 3; }
    return 0;
}
