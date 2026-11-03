module tests.reflection;
import std.compiler.{check, compile, parse_body, CompileError};

flow Worker<effect E>(value: bool, f: (i32) -> unit ![E]) -> bool ![E] {
    f(0x2a);
    return value;
}

flow noop(value: i32) -> unit ![] { return; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool, (i32) -> unit ![]) -> bool ![] = Worker;
    let changed = match parse_body("f(43); return value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let report = check((^original).with_body(changed));
    if report.status != "rejected" { return 1; }
    var changed_trace = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::ActionTraceNotContained" || diagnostic.code == "code::OutcomeTraceNotContained" {
            changed_trace = true;
        }
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" { return 2; }
    }
    if !changed_trace { return 3; }
    let body = match parse_body("f(42); return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false, noop) { return 4; }
    if current(true, noop) { return 5; }
    return 0;
}
