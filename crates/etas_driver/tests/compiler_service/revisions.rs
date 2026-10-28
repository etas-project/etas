use super::*;
use etas_interpreter::{
    Interpreter,
    api::code::PreparedCodeDefinitions,
    api::{EntryPoint, InterpValue, RunOptions},
};

pub(super) fn execute(source: &str, max_steps: Option<u64>) -> etas_interpreter::api::RunResult {
    execute_configured(source, max_steps, Default::default(), true)
}

pub(super) fn execute_with_limits(
    source: &str,
    limits: etas_interpreter::api::ExecutionLimits,
) -> etas_interpreter::api::RunResult {
    execute_prepared_with_limits(source::prepared(source), limits, Default::default(), true)
}

fn execute_configured(
    source: &str,
    max_steps: Option<u64>,
    config: NativeCompilerConfig,
    with_resolver: bool,
) -> etas_interpreter::api::RunResult {
    execute_prepared(source::prepared(source), max_steps, config, with_resolver)
}

pub(super) fn execute_prepared(
    prepared: (etas_frontend::CheckedProgram, Arc<NativeDefinitionStore>),
    max_steps: Option<u64>,
    config: NativeCompilerConfig,
    with_resolver: bool,
) -> etas_interpreter::api::RunResult {
    let limits = etas_interpreter::api::ExecutionLimits {
        max_steps: max_steps.map(|limit| std::num::NonZeroU64::new(limit).unwrap()),
        ..Default::default()
    };
    execute_prepared_with_limits(prepared, limits, config, with_resolver)
}

fn execute_prepared_with_limits(
    (checked, definitions): (etas_frontend::CheckedProgram, Arc<NativeDefinitionStore>),
    limits: etas_interpreter::api::ExecutionLimits,
    config: NativeCompilerConfig,
    with_resolver: bool,
) -> etas_interpreter::api::RunResult {
    let service = NativeCompilerService::new(definitions.clone(), config);
    execute_using_service(checked, definitions, limits, service, with_resolver)
}

pub(super) fn execute_using_service(
    checked: etas_frontend::CheckedProgram,
    definitions: Arc<NativeDefinitionStore>,
    limits: etas_interpreter::api::ExecutionLimits,
    service: NativeCompilerService,
    with_resolver: bool,
) -> etas_interpreter::api::RunResult {
    let host = CompilerHost::new(no_host::NoHost::default(), service.clone());
    let host = if with_resolver {
        host.with_candidate_resolver(Arc::new(service))
    } else {
        host
    };
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        execution_limits: limits,
        ..Default::default()
    };
    options
        .host_context
        .authority
        .grants
        .push(HostActionGrant::allow("Code", "compile"));
    options
        .host_context
        .authority
        .grants
        .push(HostActionGrant::allow("Code", "check"));
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert_eq!(
        host.inner().calls(),
        0,
        "pure revisions must not execute Host requests"
    );
    result
}

#[test]
fn compiled_body_calls_the_original_declaration_instead_of_itself() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> string { return "original"; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("return Worker();") {
        Ok(body) => body,
        Err(_) => { return "invalid body"; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised();
}
"#,
        Some(300),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("original".into()))
    );
}

#[test]
fn compiled_auxiliary_method_uses_its_own_managed_compiler_mediation() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
impl agent Worker {
    flow revise() -> i32 ![Error<CompileError>] {
        let body = match parse_body("return 3;") {
            Ok(body) => body,
            Err(_) => { return -1; },
        };
        let next = compile((^Worker).with_body(body));
        return next();
    }
}
flow main() -> i32 ![Error<CompileError>] {
    let first = compile(^Worker);
    return first.revise();
}
"#,
        Some(500),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(3)));
}

#[test]
fn compiled_body_aliases_and_source_wrappers_keep_the_original_declaration() {
    for body in [
        "let original = Worker; return original();",
        "return source_wrapper();",
    ] {
        let source = format!(
            r#"
module app;
import std.compiler.{{compile, parse_body, CompileError}};
agent Worker() -> string {{ return "original"; }}
flow source_wrapper() -> string {{ return Worker(); }}
flow main() -> string ![Error<CompileError>] {{
    let body = match parse_body({body:?}) {{ Ok(body) => body, Err(_) => {{ return "invalid body"; }}, }};
    let revised = compile((^Worker).with_body(body));
    return revised();
}}
"#
        );
        let result = execute(&source, Some(300));
        assert!(
            result.diagnostics.is_empty(),
            "{body}: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.value(),
            Some(&InterpValue::String("original".into())),
            "{body}"
        );
    }
}

#[test]
fn compiled_bare_agent_run_can_call_the_original_run_method() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker;
impl agent Worker { flow run() -> string { return "original"; } }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("return Worker.run();") { Ok(body) => body, Err(_) => { return "invalid body"; }, };
    let revised = compile((^Worker).with_body(body));
    return revised.run();
}
"#,
        Some(300),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("original".into()))
    );
}

#[test]
fn frozen_auxiliary_methods_do_not_require_a_default_agent_entry() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Namespace;
impl agent Namespace { flow echo() -> string { return "original"; } }
agent Worker() -> string { return Namespace.echo(); }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("let namespace = Namespace; return namespace.echo();") { Ok(body) => body, Err(_) => { return "invalid body"; }, };
    let revised = compile((^Worker).with_body(body));
    return revised();
}
"#,
        Some(300),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("original".into()))
    );
}

#[test]
fn frozen_calls_preserve_enum_constructor_identity_across_both_boundaries() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
enum Status { Ready, Rejected }
agent Worker() -> Status { return Status.Ready; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return Worker();") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    return revised() == Status.Ready;
}
"#,
        Some(300),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn source_compile_requires_the_native_lease_resolver() {
    let result = execute_configured(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] { let revised = compile(^Worker); return revised(); }
"#,
        None,
        Default::default(),
        false,
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("without a native checked candidate resolver")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn compiled_agent_checkpoint_requires_native_revisions_instead_of_the_original_declaration() {
    let source = r#"
module app;
import std.compiler.{compile, CompileError};
import std.runtime.checkpoint;
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    checkpoint("pinned-revision");
    return revised();
}
"#;
    let (checked, definitions) = source::prepared(source);
    let result = execute_prepared(
        (checked.clone(), definitions),
        None,
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(1)));
    assert_eq!(result.checkpoints.len(), 1);
    let artifact =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    let error =
        etas_interpreter::api::codec::checkpoint_from_json(&artifact, &checked).unwrap_err();
    assert!(
        error
            .message()
            .contains("requires freshly prepared revisions"),
        "{error}"
    );
}

#[test]
fn a_bare_agents_compiled_run_method_uses_the_candidate_method_body() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker;
impl agent Worker { flow run() -> i32 { return 1; } }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return 9;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^Worker).with_body(body));
    return revised.run();
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(9)));
}

#[test]
fn retaining_executable_agents_keeps_the_compiler_candidate_lease_charged() {
    let result = execute_configured(
        r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let first = compile(^Worker);
    let second = compile(^Worker);
    return first() + second();
}
"#,
        None,
        NativeCompilerConfig {
            max_retained_candidates: 1,
            ..Default::default()
        },
        true,
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("compiler retained output limit exceeded")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn successive_source_revisions_leave_old_agent_aliases_pinned() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> i32 ![Error<CompileError>] {
    let first_body = match parse_body("return 2;") { Ok(body) => body, Err(_) => { return -1; }, };
    let second_body = match parse_body("return 3;") { Ok(body) => body, Err(_) => { return -2; }, };
    let first = compile((^Worker).with_body(first_body));
    let alias = first;
    let second = compile((^Worker).with_body(second_body));
    return first() * 100 + second() * 10 + alias();
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(232)));
}

#[test]
fn source_revision_method_selection_uses_the_frozen_callable_identity() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: i32) -> i32 { return value; }
impl agent Worker { flow echo(value: i32) -> i32 { return value; } }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return 2;") { Ok(body) => body, Err(_) => { return -1; }, };
    let revised = compile((^Worker).with_body(body));
    return revised(10) + revised.echo(10);
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(12)));
}

#[test]
fn source_compile_does_not_publish_an_arithmetic_rewrite_without_outcome_evidence() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: i32) -> i32 { return value + 1; }
flow main() -> bool {
    let body = match parse_body("return value + 2;") { Ok(body) => body, Err(_) => { return false; }, };
    match compile((^Worker).with_body(body))? {
        Err(error) => { return error.kind == "rejected"; },
        Ok(_) => { return false; },
    }
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn source_revision_calls_convert_nominal_arguments_and_returned_record_fields() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
type Profile = { id: UserId };
agent Worker(value: UserId) -> Profile { return Profile { id = value }; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return Profile { id = value };") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let original = UserId("u1");
    let result = revised(original);
    return result.id == original;
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn inline_revision_calls_do_not_reset_the_invocation_step_budget() {
    let calls = "revised();".repeat(80);
    let source = format!(
        r#"
module app;
import std.compiler.{{compile, parse_body, CompileError}};
agent Worker() -> i32 {{ return 1; }}
flow main() -> i32 ![Error<CompileError>] {{
    let body = match parse_body("return 2;") {{ Ok(body) => body, Err(_) => {{ return -1; }}, }};
    let revised = compile((^Worker).with_body(body));
    {calls}
    return 0;
}}
"#
    );
    let result = execute(&source, Some(140));
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("maximum interpreter execution steps (140)")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn source_reflection_of_an_original_agent_binding_uses_its_checked_definition() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> bool ![Error<CompileError>] {
    let current = Worker;
    let definition = ^current;
    let method = ^current.run;
    let body = match definition.body { Some(body) => body, None => { return false; }, };
    let method_body = match method.body { Some(body) => body, None => { return false; }, };
    let revised = compile(definition);
    return body.text == method_body.text && revised() == 1;
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn source_reflection_resolves_top_level_agent_bindings_through_the_checked_global_path() {
    let result = execute(
        r#"
module app;
agent Worker() -> i32 { return 1; }
let current = Worker;
flow main() -> bool {
    let definition = ^current;
    return match definition.body { Some(_) => true, None => false, };
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_compiled_agent_auxiliary_method_keeps_its_own_retained_body() {
    let source = r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
impl agent Worker { flow echo() -> i32 { return 7; } }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return 2;") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let primary = ^revised.run;
    let auxiliary = ^revised.echo;
    let primary_body = match primary.body { Some(body) => body, None => { return false; }, };
    let auxiliary_body = match auxiliary.body { Some(body) => body, None => { return false; }, };
    return revised() == 2 && revised.echo() == 7 && primary_body.text != auxiliary_body.text;
}
"#;
    let result = execute_prepared(
        source::prepared_with_bodies(source, &["app.Worker", "app.Worker.echo"]),
        None,
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn reflection_of_a_compiled_agent_reads_the_pinned_body_and_can_compile_its_successor() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> bool ![Error<CompileError>] {
    var current = Worker;
    let original = ^current;
    let first_body = match parse_body("return 2;") { Ok(body) => body, Err(_) => { return false; }, };
    current = compile(original.with_body(first_body));
    let old_alias = current;
    let first = ^current;
    let method = ^current.run;
    let body = match first.body { Some(body) => body, None => { return false; }, };
    let method_body = match method.body { Some(body) => body, None => { return false; }, };
    if body.text != method_body.text { return false; }
    let second_body = match parse_body("return 3;") { Ok(body) => body, Err(_) => { return false; }, };
    current = compile(first.with_body(second_body));
    let old_definition = ^old_alias;
    let old_body = match old_definition.body { Some(body) => body, None => { return false; }, };
    let new_definition = ^current;
    let new_body = match new_definition.body { Some(body) => body, None => { return false; }, };
    if first.revision.key == original.revision.key || new_definition.revision.key == first.revision.key { return false; }
    if first.revision.parent != Some(original.revision.key) || new_definition.revision.parent != Some(first.revision.key) { return false; }
    if old_definition.revision.key != first.revision.key { return false; }
    return old_alias() == 2 && current() == 3 && old_body.text == body.text && new_body.text != body.text;
}
"#,
        None,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn compiled_reflection_does_not_expose_an_unretained_generated_body() {
    let source = r#"
module app;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 { return 1; }
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let definition = ^revised;
    let method = ^revised.run;
    return revised() == 1 && match definition.body { None => true, Some(_) => false, }
        && match method.body { None => true, Some(_) => false, };
}
"#;
    let result = execute_prepared(
        source::prepared_with_bodies(source, &[]),
        None,
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn an_active_revision_keeps_the_original_declaration_and_agent_alias_bindings() {
    for body in [
        "let definition = ^Worker; return definition.revision.key;",
        "let current = Worker; let definition = ^current; return definition.revision.key;",
    ] {
        let source = format!(
            r#"
module app;
import std.compiler.{{compile, parse_body, CompileError}};
agent Worker() -> string {{ return "baseline"; }}
flow main() -> bool ![Error<CompileError>] {{
    let body = match parse_body({body:?}) {{ Ok(body) => body, Err(_) => {{ return false; }}, }};
    let revised = compile((^Worker).with_body(body));
    let definition = ^revised;
    let original = ^Worker;
    return revised() == original.revision.key && revised() != definition.revision.key;
}}
"#
        );
        let result = execute(&source, None);
        assert!(
            result.diagnostics.is_empty(),
            "{body}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value(), Some(&InterpValue::Bool(true)), "{body}");
    }
}

#[test]
fn an_active_revision_method_body_uses_the_original_syntax_backend() {
    let source = r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> string { return "baseline"; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let current = Worker; let method = ^current.run; return match method.body { Some(body) => body.text, None => \"missing\", };") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let definition = ^Worker;
    let expected = match definition.body { Some(body) => body.text, None => { return false; }, };
    return revised() == expected;
}
"#;
    let result = execute(source, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn an_active_revision_can_edit_its_definition_without_changing_the_frozen_backend() {
    let source = r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Worker() -> Option<string> { return None; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let definition = ^Worker; let syntax = match parse_body(\"return None;\") { Ok(body) => body, Err(_) => { return None; }, }; let edited = definition.with_body(syntax); return edited.revision.parent;") { Ok(body) => body, Err(_) => { return false; }, };
    let revised = compile((^Worker).with_body(body));
    let definition = ^Worker;
    return revised() == Some(definition.revision.key);
}
"#;
    let result = execute(source, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn an_active_revision_reflects_other_declarations_from_the_frozen_environment() {
    let source = r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
agent Helper() -> string { return "helper"; }
agent Worker() -> string { return "worker"; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let definition = ^Helper; return definition.revision.key;") { Ok(body) => body, Err(_) => { return false; }, };
    let original = ^Helper;
    let revised = compile((^Worker).with_body(body));
    return revised() == original.revision.key;
}
"#;
    let result = execute(source, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn an_active_revision_can_compile_and_call_another_revision_in_the_same_machine() {
    let worker_body = "let edited = definition.with_body(syntax); let next = handle { compile(edited) } with { Error<CompileError>.raise(_) => { finish Helper; } }; return next();";
    let source = format!(
        r#"
module app;
import std.compiler.{{compile, parse_body, AgentDef, SyntaxBlock, CompileError}};
agent Helper() -> string {{ return "baseline"; }}
agent Worker(definition: AgentDef<Helper>, syntax: SyntaxBlock) -> string {{
    compile(definition)?;
    return "baseline";
}}
flow main() -> bool ![Error<CompileError>] {{
    let worker_body = match parse_body({worker_body:?}) {{ Ok(body) => body, Err(_) => {{ return false; }}, }};
    let helper_body = match parse_body("return \"successor\";") {{ Ok(body) => body, Err(_) => {{ return false; }}, }};
    let revised = compile((^Worker).with_body(worker_body));
    return revised(^Helper, helper_body) == "successor";
}}
"#
    );
    let prepared = source::prepared_with_bodies(&source, &["app.Worker", "app.Helper"]);
    let expression = prepared
        .0
        .types
        .reflections
        .iter()
        .find_map(|(expr, fact)| match fact.target {
            etas_types::CheckedReflectionTarget::Agent { declaration, .. }
                if prepared.0.symbols.get(declaration).unwrap().name == "Worker" =>
            {
                Some(*expr)
            }
            _ => None,
        })
        .unwrap();
    let definition = prepared.1.reflect_agent(expression).unwrap();
    let edited = prepared
        .1
        .with_body(
            &definition,
            prepared
                .1
                .syntax_operations()
                .parse_body(worker_body)
                .unwrap(),
        )
        .unwrap();
    let overlay = prepared
        .1
        .resolve(&edited)
        .unwrap()
        .body_overlay(prepared.1.syntax_operations())
        .unwrap();
    let analysis = etas_frontend::FrontendSession::new()
        .analyze_body_overlay(overlay)
        .unwrap();
    let checked = analysis.output().checked.as_ref().unwrap();
    let exits = checked.effects.item_effects[&checked.entry.unwrap()]
        .exits
        .as_ref()
        .unwrap();
    assert!(exits.gaps.is_empty(), "{:?}", exits.gaps);
    let result = execute_prepared(prepared, None, Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn nested_revisions_relocate_nominal_arguments_and_returned_record_members() {
    let worker_body = "let edited = definition.with_body(syntax); let next = handle { compile(edited) } with { Error<CompileError>.raise(_) => { finish Helper; } }; return next(id);";
    let helper_body = "return Profile { id = value, marker = \"successor\" };";
    let source = format!(
        r#"
module app;
import std.compiler.{{compile, parse_body, AgentDef, SyntaxBlock, CompileError}};
type UserId = string;
type Profile = {{ id: UserId, marker: string }};
agent Helper(value: UserId) -> Profile {{ return Profile {{ id = value, marker = "baseline" }}; }}
agent Worker(definition: AgentDef<Helper>, syntax: SyntaxBlock, id: UserId) -> Profile {{
    compile(definition)?;
    return Profile {{ id = id, marker = "baseline" }};
}}
flow main() -> bool ![Error<CompileError>] {{
    let worker_body = match parse_body({worker_body:?}) {{ Ok(body) => body, Err(_) => {{ return false; }}, }};
    let helper_body = match parse_body({helper_body:?}) {{ Ok(body) => body, Err(_) => {{ return false; }}, }};
    let revised = compile((^Worker).with_body(worker_body));
    let id = UserId("u1");
    let result = revised(^Helper, helper_body, id);
    return result.id == id && result.marker == "successor";
}}
"#
    );
    let result = execute_prepared(
        source::prepared_with_bodies(&source, &["app.Worker", "app.Helper"]),
        None,
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn nested_revision_compilation_and_calls_keep_the_original_step_budget() {
    let worker_body = format!(
        "let edited = definition.with_body(syntax); let next = handle {{ compile(edited) }} with {{ Error<CompileError>.raise(_) => {{ finish Helper; }} }}; {} return next();",
        "next();".repeat(80)
    );
    let source = format!(
        r#"
module app;
import std.compiler.{{compile, parse_body, AgentDef, SyntaxBlock, CompileError}};
agent Helper() -> string {{ return "baseline"; }}
agent Worker(definition: AgentDef<Helper>, syntax: SyntaxBlock) -> string {{ compile(definition)?; return "baseline"; }}
flow main() -> string ![Error<CompileError>] {{
    let worker_body = match parse_body({worker_body:?}) {{ Ok(body) => body, Err(_) => {{ return "invalid worker"; }}, }};
    let helper_body = match parse_body("return \"successor\";") {{ Ok(body) => body, Err(_) => {{ return "invalid helper"; }}, }};
    let revised = compile((^Worker).with_body(worker_body));
    return revised(^Helper, helper_body);
}}
"#
    );
    let result = execute_prepared(
        source::prepared_with_bodies(&source, &["app.Worker", "app.Helper"]),
        Some(300),
        Default::default(),
        true,
    );
    assert!(result.value().is_none());
    assert!(
        result.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("maximum interpreter execution steps (300)")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                etas_interpreter::api::WorkflowEvent::HostTrace(
                    etas_host::TraceEvent::HostRequestStarted {
                        kind: etas_host::HostRequestKind::Compiler,
                        ..
                    }
                )
            ))
            .count(),
        2
    );
}
