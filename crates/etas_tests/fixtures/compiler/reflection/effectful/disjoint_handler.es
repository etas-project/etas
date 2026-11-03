module tests.reflection;
import std.compiler.{check, compile, parse_body, CompileError};
import std.io.println;

flow Worker<effect E>(f: () -> unit ![E]) -> unit ![E] {
    handle { f(); } with {
        Error<i32>.raise(_) => { finish (); }
    };
    return;
}

flow WorkerImplicit<effect E>(f: () -> unit ![E]) -> unit ![E] {
    handle { f(); } with {
        Error<i32>.raise(_) => { finish (); }
    };
}

flow helper<effect E>(f: () -> unit ![E]) -> unit ![E] {
    handle { f(); } with {
        Error<i32>.raise(_) => { finish (); }
    };
}

flow WorkerForward<effect E>(f: () -> unit ![E]) -> unit ![E] {
    helper<![E]>(f);
}

flow helper_argument<effect E>(f: (bool) -> unit ![E], value: bool) -> unit ![E] {
    f(value);
}

flow WorkerArgument<effect E>(f: (bool) -> unit ![E]) -> unit ![E] {
    helper_argument<![E]>(f, true);
}

flow emit() -> unit ![Console.stdout_write] {
    println("disjoint handler")?;
    return;
}

flow emit_bool(value: bool) -> unit ![Console.stdout_write] {
    if value { println("forwarded true")?; }
    return;
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>, Console.stdout_write] {
    let original: (() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write] = Worker;
    let body = match parse_body("handle { f(); } with { Error<i32>.raise(_) => { finish (); } }; return;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let report = check((^original).with_body(body));
    if report.status != "accepted" { return 1; }
    let intercepting = match parse_body("handle { f(); } with { Console.stdout_write(_) => { resume; } }; return;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let intercepted = check((^original).with_body(intercepting));
    if intercepted.status != "rejected" { return 2; }
    var blocked = false;
    for diagnostic in intercepted.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" || diagnostic.code == "code::ActionTraceNotContained" || diagnostic.code == "code::OutcomeTraceNotContained" {
            blocked = true;
        }
    }
    if !blocked { return 3; }
    let current = compile((^original).with_body(body));
    current(emit);
    let implicit: (() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write] = WorkerImplicit;
    let implicit_body = match parse_body("handle { f(); } with { Error<i32>.raise(_) => { finish (); } };") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    if check((^implicit).with_body(implicit_body)).status != "accepted" { return 4; }
    let implicit_current = compile((^implicit).with_body(implicit_body));
    implicit_current(emit);
    let forward: (() -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write] = WorkerForward;
    let forward_body = match parse_body("helper<![E]>(f);") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let forward_report = check((^forward).with_body(forward_body));
    if forward_report.status != "accepted" { return 5; }
    let forward_current = compile((^forward).with_body(forward_body));
    forward_current(emit);
    let argument: ((bool) -> unit ![Console.stdout_write]) -> unit ![Console.stdout_write] = WorkerArgument;
    let argument_body = match parse_body("helper_argument<![E]>(f, true);") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    if check((^argument).with_body(argument_body)).status != "accepted" { return 6; }
    let changed_argument = match parse_body("helper_argument<![E]>(f, false);") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    if check((^argument).with_body(changed_argument)).status != "rejected" { return 7; }
    let argument_current = compile((^argument).with_body(argument_body));
    argument_current(emit_bool);
    return 0;
}
