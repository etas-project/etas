module tests.reflection;
import std.compiler.{parse_body, diff, ChangeReport};
import std.runtime.checkpoint;

agent Worker() -> i32 { return 1; }

flow changes() -> ChangeReport {
    let body = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return abort("valid body must parse"); },
    };
    return diff(^Worker, (^Worker).with_body(body));
}

flow main(args: Array<string>) -> i32 {
    let report = changes();
    checkpoint("definition changes");
    if report.schema == "etas.code.change-report.v2"
        && report.identity == (^Worker).dependencies.lexical_owner
        && report.body_changed == Some(true)
        && !report.signature_changed
        && report.escaping_changed == None && report.trace_changed == None
    { return 0; }
    return 1;
}
