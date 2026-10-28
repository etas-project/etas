module tests.reflection;
import std.compiler.{AgentDef, CompileError, check, compile, parse_body};

agent Worker() -> i32 { return 7; }

flow build<A>(definition: AgentDef<A>) -> A ![Error<CompileError>] {
    return compile(definition);
}

agent Runner() -> i32 ![Error<CompileError>] {
    let value = handle { compile(^Worker) } with { Code.compile(_) => { resume Worker; } };
    return value();
}

flow main(args: Array<string>) -> i32 ![Error<CompileError>] {
    let body = match parse_body("let value = handle { build(^Worker) } with { Code.compile(_) => { resume Worker; } }; return value();") {
        Ok(body) => body,
        Err(_) => { return 1; },
    };
    let report = check((^Runner).with_body(body));
    if report.status != "rejected" { return 2; }
    var has_outcome_diagnostic = false;
    for diagnostic in report.diagnostics limit Iterations(64) {
        if diagnostic.code == "code::MissingOutcomeSensitiveSummary" { has_outcome_diagnostic = true; }
        if diagnostic.code == "type::IncompleteTypeFacts" || diagnostic.code == "effect::IncompleteEffectFacts" { return 3; }
    }
    if has_outcome_diagnostic { return 0; }
    return 4;
}
