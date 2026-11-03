module tests.reflection;
import std.compiler.{check, compile, parse_body, CompileError};

flow Worker<T, effect E>(value: T) -> T ![E] { return value; }

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let original: (bool) -> bool ![Console.stdout_write] = Worker;
    let changed_effects = match parse_body("perform Console.stdout_write(\"extra\"); return value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let report = check((^original).with_body(changed_effects));
    if report.status != "rejected" { return 3; }
    var contract_rejected = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::AdditionalRequestedActions" || diagnostic.code == "code::ActionTraceNotContained" {
            contract_rejected = true;
        }
    }
    if !contract_rejected { return 4; }
    let wrong_type = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let type_report = check((^original).with_body(wrong_type));
    if type_report.status != "rejected" { return 5; }
    var type_rejected = false;
    for diagnostic in type_report.diagnostics limit Iterations(64) {
        if diagnostic.code == "type::TypeMismatch" { type_rejected = true; }
    }
    if !type_rejected { return 6; }
    let body = match parse_body("return !value;") {
        Ok(body) => body,
        Err(_) => { return abort("fixture body must parse"); },
    };
    let current = compile((^original).with_body(body));
    if !current(false) { return 1; }
    if current(true) { return 2; }
    return 0;
}
