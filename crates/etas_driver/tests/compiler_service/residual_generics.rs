use super::*;
use etas_interpreter::api::InterpValue;

#[test]
fn bound_generic_guarded_flow_compiles_and_executes_its_residual_plan() {
    let source = r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type Label = string;
effect Gate { action check<T>(value: T) -> unit; action send<T>(value: T) -> unit; }
spec Safe<T>: trace = Gate.check<Array<T>> >> Gate.send<Array<T>>;
flow guarded<T>(value: T) -> i32 ![] ~ Safe<T> {
    handle { perform Gate.check<Array<T>>([value]); perform Gate.send<Array<T>>([value]); } with {
        Gate.check<Array<T>>(_) => { resume (); }
        Gate.send<Array<T>>(_) => { resume (); }
    };
    return 7;
}
flow main() -> i32 ![Error<CompileError>] {
    let original: (Label) -> i32 ![] = guarded;
    let body = match parse_body("handle { perform Gate.check<Array<Label>>([value]); perform Gate.send<Array<Label>>([value]); } with { Gate.check<Array<Label>>(_) => { resume (); } Gate.send<Array<Label>>(_) => { resume (); } }; return 9;") {
        Ok(body) => body, Err(_) => { return -1; },
    };
    let current = compile((^original).with_body(body));
    return current(Label("checked"));
}
"#;
    let mut input = ProjectInput::single_source(SourceInput::anonymous(source));
    let package = PackageKey::from_name_and_version("app", "1.0.0").unwrap();
    input.environment.local_source_package = Some(package);
    input
        .environment
        .stable_source_keys
        .push(etas_frontend::ProjectStableSourceKeyInput {
            source: input.sources[0].id,
            key: etas_code::StableSourceKey::new(
                etas_code::StableSourceOrigin::Package(package),
                &["src".into(), "main.es".into()],
            )
            .unwrap(),
        });
    input.options.residual_target = etas_interpreter::interpreter_residual_capabilities();
    input
        .options
        .reflection
        .retain_bodies
        .push("app.guarded".into());
    let prepared = source::prepared_input(input);
    let result = revisions::execute_prepared(prepared, Some(20_000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(9)));
}
