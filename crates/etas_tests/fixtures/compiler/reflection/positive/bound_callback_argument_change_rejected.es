module tests.reflection;
import std.compiler.{check, parse_body};

flow Worker<T, effect E>(value: T, f: (bool) -> unit ![E]) -> T ![E] {
    f(true);
    return value;
}

flow main(args: Array<string>) -> i32 {
    let original: (bool, (bool) -> unit ![]) -> bool ![] = Worker;
    let body = match parse_body("f(false); return value;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let report = check((^original).with_body(body));
    if report.status != "rejected" { return 2; }
    var changed_trace = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::ActionTraceNotContained" || diagnostic.code == "code::OutcomeTraceNotContained" {
            changed_trace = true;
        }
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" { return 3; }
    }
    if !changed_trace { return 4; }
    return 0;
}
