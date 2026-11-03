use super::*;

const LIBRARY: &str = r#"
module library.api;
public spec Encoder { flow encode(self) -> i32 ![]; }
public flow twice<T ~ Encoder>(value: T) -> i32 ![] {
    value::Encoder.encode();
    return value::Encoder.encode();
}
public flow captured<T ~ Encoder>(value: T) -> (() -> i32 ![]) ![] {
    return () => value::Encoder.encode();
}
public flow lookup() -> i32 ![] { return 7; }
"#;

#[test]
fn behavioral_self_metadata_only_check_relocates_frozen_method_evidence() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    for (alias, original_body, expected) in [
        ("direct", "return value.number;", "accepted"),
        (
            "delegated",
            "return relay(value);",
            "rejected|code::MissingOutcomeSensitiveSummary: outcome-sensitive behavior is unavailable; compatibility is not proved",
        ),
        (
            "returned",
            "let next = delayed(value); return next();",
            "rejected|code::MissingOutcomeSensitiveSummary: outcome-sensitive behavior is unavailable; compatibility is not proved",
        ),
    ] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        loaded.loaded.input.sources[0].text = r#"
module app.main;
import $alias.library.api.{Encoder, twice, captured};
import std.compiler.{check, parse_body, CompileError};
type Packet = { number: i32 };
impl Packet ~ Encoder { flow encode(self) -> i32 ![] { return self.number; } }
flow relay<T ~ Encoder>(value: T) -> i32 ![] { return twice(value); }
flow delayed<U ~ Encoder>(value: U) -> (() -> i32 ![]) ![] { return captured(value); }
agent Worker(value: Packet) -> i32 ![] { $body }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("return Worker(value);") {
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
        .replace("$alias", alias)
        .replace("$body", original_body);
        let checked = origin_metadata::check(&loaded);
        assert_eq!(
            checked.sources.sources.len(),
            1,
            "dependency must stay metadata-only"
        );
        assert!(
            !checked.effects.method_evidence.is_empty(),
            "freeze must exercise actual metadata evidence"
        );
        assert!(
            !checked.effects.metadata_callable_graphs.is_empty(),
            "returned method evidence must not be dropped from metadata replay"
        );
        let definitions =
            prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
        let service = NativeCompilerService::new(definitions.clone(), Default::default())
            .with_project_sources(&loaded)
            .unwrap();
        let result = revisions::execute_using_service(
            checked,
            definitions,
            etas_interpreter::api::ExecutionLimits {
                max_steps: std::num::NonZeroU64::new(4_000),
                ..Default::default()
            },
            service,
            true,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(
            result.value(),
            Some(&etas_interpreter::api::InterpValue::String(expected.into())),
            "{alias}"
        );
    }
}

#[test]
fn behavioral_self_frozen_metadata_method_requests_still_violate_a_forbidding_trace_spec() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Encoder, captured};
import std.compiler.{check, parse_body, CompileError};
import std.io.println;
type Packet = { number: i32 };
impl Packet ~ Encoder {
    flow encode(self) -> i32 ![] { println("forbidden")?; return self.number; }
}
flow delayed<U ~ Encoder>(value: U) -> (() -> i32 ![]) ![] { return captured(value); }
spec Silent: trace = -Console.stdout_write;
agent Worker(value: Packet) -> i32 ![] ~ Silent { return value.number; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("let next = delayed(value); return next();") {
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
    .into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let result = revisions::execute_using_service(
        checked,
        definitions,
        etas_interpreter::api::ExecutionLimits {
            max_steps: std::num::NonZeroU64::new(4_000),
            ..Default::default()
        },
        service,
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let Some(etas_interpreter::api::InterpValue::String(report)) = result.value() else {
        panic!("missing check report: {result:?}");
    };
    assert!(report.starts_with("rejected|"), "{report}");
    assert!(report.contains("effect::TraceSpecDenied"), "{report}");
    assert!(!report.contains("CompilerServiceFailure"), "{report}");
}
