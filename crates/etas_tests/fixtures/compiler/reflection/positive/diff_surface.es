module tests.reflection;
import std.compiler.{parse_body, diff, ChangeReport};

agent Worker() -> i32 { return 1; }

flow main(args: Array<string>) -> i32 {
    let body = match parse_body("return 42;") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let original = ^Worker;
    let edited = original.with_body(body);
    let changes: ChangeReport = diff(original, edited);
    let unchanged = diff(original, original);
    if changes.schema == "etas.code.change-report.v1"
        && changes.identity == original.identity
        && changes.old_revision == original.revision
        && changes.new_revision == edited.revision
        && changes.old_body_available && changes.new_body_available
        && changes.body_changed == Some(true)
        && !changes.signature_changed && !changes.dependencies_changed
        && !changes.declared_bound_changed
        && changes.escaping_changed == None
        && changes.requested_changed == None
        && changes.default_handled_changed == None
        && changes.determinism_changed == None
        && changes.trace_changed == None
        && unchanged.body_changed == Some(false)
        && unchanged.escaping_changed == Some(false)
        && unchanged.requested_changed == Some(false)
        && unchanged.default_handled_changed == Some(false)
        && unchanged.determinism_changed == Some(false)
        && unchanged.trace_changed == Some(false)
    { return 0; }
    return 1;
}
