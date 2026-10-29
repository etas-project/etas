use super::*;

#[test]
fn behavioral_self_generic_method_does_not_claim_a_diverging_implementation_returns() {
    for (implementation, accepted) in [
        ("return value;", true),
        ("return abort(\"never returns\");", false),
    ] {
        let source = r#"
module app;
import std.compiler.{check, parse_body, CompileError};
spec Encoder { flow encode<A>(self, value: A) -> A ![]; }
type Packet = { number: i32 };
impl Packet ~ Encoder {
    flow encode<B>(self, value: B) -> B ![] { $implementation }
}
flow relay<T ~ Encoder, U>(receiver: T, value: U) -> U ![] { return receiver::Encoder.encode(value); }
agent Worker(value: Packet) -> i32 ![] { return relay(value, 21); }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("return 21;") {
        Ok(body) => body,
        Err(_) => { return "parse rejected"; },
    };
    let report = check((^Worker).with_body(body));
    var diagnostics = report.status;
    for diagnostic in report.diagnostics limit Iterations(16) {
        diagnostics = diagnostics + "|" + diagnostic.code + ": " + diagnostic.message;
    }
    return diagnostics;
}
"#
        .replace("$implementation", implementation);
        let result = revisions::execute(&source, Some(4_000));
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Some(etas_interpreter::api::InterpValue::String(report)) = result.value() else {
            panic!("missing report: {result:?}");
        };
        if accepted {
            assert_eq!(report, "accepted");
        } else {
            assert!(
                report.starts_with("rejected|"),
                "a pure row is not evidence of normal completion: {report}"
            );
        }
        assert!(!report.contains("CompilerServiceFailure"), "{report}");
    }
}
