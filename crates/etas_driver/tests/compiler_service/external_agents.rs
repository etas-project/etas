use super::*;
use etas_driver::compiler::prepare_native_definitions;

#[path = "external_agents/frozen_callables.rs"]
mod frozen_callables;

#[path = "external_agents/enums.rs"]
mod enums;

#[path = "external_agents/transitive.rs"]
mod transitive;

#[path = "external_agents/source_compilation.rs"]
mod source_compilation;

#[test]
fn a_newly_compiled_body_can_reflect_an_external_agent_without_source_overlay() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Remote(value: bool) -> bool ![] { return value; } public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Remote;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: bool) -> bool ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let definition = ^Remote; return definition.signature != None && definition.body == None;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let next = compile((^Worker).with_body(body));
    return next(true);
}
"#.into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
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
        1
    );
}

#[test]
fn legacy_external_agent_headers_cannot_fabricate_revision_coordinates() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Remote(value: bool) -> bool { return value; } public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = "module app.main; import first.library.api.Remote; flow main() -> unit { let definition = ^Remote; return; }".into();
    let headers = loaded.loaded.input.environment.external_definition_origins[0]
        .descriptors
        .as_mut()
        .unwrap()
        .headers
        .as_mut()
        .unwrap();
    headers.snapshots = None;
    headers.bodies = None;
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let error = definitions.reflect_agent(expression).unwrap_err();
    assert!(
        matches!(error, etas_frontend::code::definition::NativeDefinitionError::Projection(
        etas_frontend::code::descriptor::ProjectionError::MissingIdentity(ref reason)
    ) if reason.contains("revision coordinates"))
    );
}

#[test]
fn recursive_public_types_replay_and_reflect_without_loading_dependency_source() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public type Node = { next: Option<Node> };
public type Directory = { entries: Array<Entry> };
public type Entry = { parent: Option<Directory> };
public type Chain<T> = { value: T, next: Option<Chain<T>> };
public agent Remote(node: Node) -> Node { return node; }
public flow identity<T>(chain: Chain<T>) -> Chain<T> { return chain; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    std::fs::remove_dir_all(workspace.0.join("library/src")).unwrap();
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{Remote, Node, Directory, Entry, Chain, identity};
flow main() -> bool {
    let node: Node = Node { next = None };
    let entry: Entry = Entry { parent = None };
    let directory: Directory = Directory { entries = [entry] };
    let chain: Chain<i32> = Chain<i32> { value = 7, next = None };
    let definition = ^Remote;
    let callable = ^identity;
    return definition.signature != None && definition.body == None
        && callable.signature.parameters.len() == 1
        && node.next == None && directory.entries.len() == 1
        && chain.value == 7 && chain.next == None;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            etas_interpreter::api::RunOptions {
                code_definitions: Some(Arc::new(
                    etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions),
                )),
                ..Default::default()
            },
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(host.calls(), 0);
}

#[test]
fn metadata_only_draft_compilation_requires_a_real_locked_source_environment() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public agent Remote(value: bool) -> bool ![] { return value; } public flow lookup() -> i32 { return 7; }",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Remote;
import std.compiler.{compile, parse_body, CompileError};
flow main() -> bool ![] {
    let body = match parse_body("return value;") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let result = compile((^Remote).with_body(body))?;
    return match result {
        Ok(_) => false,
        Err(error) => match error.diagnostics.get(0) {
            Some(diagnostic) => diagnostic.code == "code::MissingSourceEnvironment",
            None => false,
        },
    };
}

"#
    .into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
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
        1
    );
}

#[test]
fn published_enum_constructors_and_named_payloads_run_without_dependency_source() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public enum Response { Empty, Success(string), Failure { code: i32, message: string }, }
public agent Remote(value: Response) -> Response { return value; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    std::fs::remove_dir_all(workspace.0.join("library/src")).unwrap();
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{Remote, Response};
flow main() -> bool {
    let response: Response = Response.Failure { message = "unavailable", code = 503 };
    let success: Response = Response.Success("ok");
    let empty: Response = Response.Empty;
    if !match success { Response.Success(text) => text == "ok", _ => false } { return false; }
    if !match empty { Response.Empty => true, _ => false } { return false; }
    let definition = ^Remote;
    if definition.signature == None || definition.body != None { return false; }
    return match response {
        Response.Empty => false,
        Response.Success(_) => false,
        Response.Failure { code, message } => code == 503 && message == "unavailable",
    };
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            etas_interpreter::api::RunOptions {
                code_definitions: Some(Arc::new(
                    etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions),
                )),
                ..Default::default()
            },
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(host.calls(), 0);
}
