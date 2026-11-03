module tests.reflection;
import std.compiler.{parse_body, diff, ChangeReport};

flow Worker(value: string) -> string ![] { return abort("diff does not invoke its target"); }

flow main(args: Array<string>) -> i32 {
    let body = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let original = ^Worker;
    let edited = original.with_body(body);
    let report: ChangeReport = diff(original, edited);
    let same = diff(original, original);
    if report.schema == "etas.code.change-report.v2"
        && report.identity == original.dependencies.lexical_owner
        && report.old_revision == original.revision && report.new_revision == edited.revision
        && report.old_body_available && report.new_body_available
        && report.body_changed == Some(true) && !report.signature_changed
        && !report.dependencies_changed && !report.declared_bound_changed
        && report.escaping_changed == None && report.requested_changed == None
        && report.default_handled_changed == None && report.determinism_changed == None
        && report.trace_changed == None && same.body_changed == Some(false)
        && same.requested_changed == Some(false)
    { return 0; }
    return 1;
}
