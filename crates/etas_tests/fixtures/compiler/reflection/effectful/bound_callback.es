module tests.reflection;
import std.compiler.{check, compile, parse_body, CompileError};
import std.io.println;

flow Worker<effect E>(f: () -> unit ![E]) -> unit ![E] {
    f();
    return;
}

flow emit() -> unit ![Console.stdout_write] {
    println("effectful callback")?;
    return;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>, Console.stdout_write] {
    let original: (() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write] = Worker;
    let intercepted = match parse_body("handle { f(); } with { Console.stdout_write(_) => { resume; } }; return;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let report = check((^original).with_body(intercepted));
    if report.status != "rejected" { return 1; }
    var unproved_or_changed = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" || diagnostic.code == "code::ActionTraceNotContained" || diagnostic.code == "code::OutcomeTraceNotContained" {
            unproved_or_changed = true;
        }
    }
    if !unproved_or_changed { return 2; }
    let extra_action = match parse_body("f(); println(\"extra\")?; return;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let extra_report = check((^original).with_body(extra_action));
    if extra_report.status != "rejected" { return 3; }
    var weakened = false;
    for diagnostic in extra_report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::WeakenedDeterminism" { weakened = true; }
    }
    if !weakened { return 4; }
    let body = match parse_body("f(); return;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    current(emit);
    return 0;
}
