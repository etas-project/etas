use super::*;
use etas_driver::{
    LoadProjectRequest, PackageMetadataRequest, PackageUpdateRequest, SourceDependencyMode,
};
use std::path::{Path, PathBuf};

const BASE: &str = r#"
module library.api;
public spec Allowed;
public spec Strong ~ Allowed;
public type Value = i32;
impl Value ~ Strong;
public type Bounded<T ~ Allowed> = { value: T };
public type Wrapper<T> = { value: T };
public alias Restricted<T ~ Allowed> = Wrapper<T>;
public flow needs<T ~ Allowed>(value: T) -> T { return value; }
public enum Tree<T> { Leaf(T), Branch { children: Array<Tree<T>> }, }
public enum Response { Empty, Success(string), Failure { code: i32, message: string }, }
public agent Remote(value: Response) -> Response { return value; }
public flow lookup() -> i32 { return 7; }
"#;

fn package(root: &Path, name: &str, dependency: &str, module: &str, source: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::create_dir_all(path.join("src")).unwrap();
    std::fs::write(
        path.join("etas.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n{dependency}"
        ),
    )
    .unwrap();
    let file = path.join("src").join(module);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&path),
        PackageUpdateRequest {
            package_root: path.clone(),
        },
    )
    .unwrap();
    path
}

fn relay(workspace: &origin_metadata::Workspace, source: &str) -> PathBuf {
    let path = package(
        &workspace.0,
        "relay-library",
        r#"
[dependencies]
base = { package = "origin-library", version = "0.1.0", import = "base", path = "../library" }
"#,
        "relay/api.es",
        source,
    );
    etas_driver::build_package_metadata(
        &origin_metadata::options(&path),
        PackageMetadataRequest {
            package_root: path.clone(),
            bin: None,
        },
    )
    .unwrap();
    path
}

fn consumer(
    workspace: &origin_metadata::Workspace,
    source: &str,
) -> etas_driver::LoadedDriverProject {
    let path = package(
        &workspace.0,
        "final-consumer",
        r#"
[dependencies]
relay = { package = "relay-library", version = "0.1.0", import = "middle", path = "../relay-library" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
        "app/main.es",
        source,
    );
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&path),
        LoadProjectRequest {
            paths: vec![path],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    std::fs::remove_dir_all(workspace.0.join("library/src")).unwrap();
    std::fs::remove_dir_all(workspace.0.join("relay-library/src")).unwrap();
    loaded
}

fn run(loaded: etas_driver::LoadedDriverProject) {
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
fn facade_reexports_preserve_enum_layouts_and_agent_headers_without_source() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
public import base.library.api.{Tree, Response, Remote};
public flow lookup() -> i32 { return 7; }
"#,
    );
    run(consumer(
        &workspace,
        r#"
module app.main;
import middle.relay.api.{Tree as T, Response, Remote};
flow main() -> bool {
    let tree: T<i32> = T.Branch<i32> { children = [T.Leaf(7)] };
    let response: Response = Response.Failure { code = 503, message = "unavailable" };
    let definition = ^Remote;
    if definition.signature == None || definition.body != None { return false; }
    if !match tree { T.Leaf(_) => false, T.Branch { children } => children.len() == 1 } { return false; }
    return match response { Response.Empty => false, Response.Success(_) => false, Response.Failure { code, message } => code == 503 && message == "unavailable" };
}
"#,
    ));
}

#[test]
fn transitive_spec_evidence_does_not_require_a_direct_dependency_import() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
import base.library.api.{Value, Allowed, Bounded};
public alias PublicValue = Value;
public alias PublicBox<T ~ Allowed> = Bounded<T>;
public enum Envelope<T ~ Allowed> { Empty, Item { value: T }, }
public agent Remote(value: Envelope<Value>) -> Envelope<Value> { return value; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    run(consumer(
        &workspace,
        r#"
module app.main;
import middle.relay.api.{Envelope, PublicValue, PublicBox as Public, Remote};
flow main() -> bool {
    let value: Envelope<PublicValue> = Envelope.Item { value = PublicValue(42) };
    let boxed: Public<PublicValue> = Public<PublicValue>({ value = PublicValue(42) });
    let inferred: Public<PublicValue> = Public { value = PublicValue(42) };
    let definition = ^Remote;
    if definition.signature == None || definition.body != None { return false; }
    if boxed.value != inferred.value { return false; }
    return match value { Envelope.Empty => false, Envelope.Item { value } => value == PublicValue(42) };
}
"#,
    ));
}

#[test]
fn a_transitive_generic_alias_cannot_bypass_the_original_nominal_spec_bound() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
import base.library.api.{Bounded, Allowed};
public alias PublicBox<T ~ Allowed> = Bounded<T>;
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = consumer(
        &workspace,
        r#"
module app.main;
import middle.relay.api.PublicBox;
flow main() -> PublicBox<i32> { return PublicBox<i32>({ value = 42 }); }
"#,
    );
    for source in [
        "module app.main; import middle.relay.api.PublicBox; flow main() -> PublicBox<i32> { return PublicBox<i32>({ value = 42 }); }",
        "module app.main; import middle.relay.api.PublicBox; flow main() -> PublicBox<i32> { return PublicBox { value = 42 }; }",
    ] {
        loaded.loaded.input.sources[0].text = source.into();
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(
            output.checked.is_none(),
            "unproved source-independent spec evidence must be rejected"
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("spec")),
            "{:?}",
            output.diagnostics
        );
    }
}

#[test]
fn compiled_body_uses_facade_enum_and_agent_identities_without_dependency_sources() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
public import base.library.api.{Tree, Remote};
public flow lookup() -> i32 { return 7; }
"#,
    );
    let loaded = consumer(
        &workspace,
        r#"
module app.main;
import middle.relay.api.{Tree as T, Remote};
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: bool) -> bool ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let tree: T<i32> = T.Leaf(7); let definition = ^Remote; return match tree { T.Leaf(value) => value == 7 && definition.signature != None && definition.body == None, T.Branch { children } => false };") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let next = compile((^Worker).with_body(body));
    return next(true);
}

"#,
    );
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

#[test]
fn alias_own_bounds_and_scoped_evidence_survive_two_binary_metadata_layers() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
public import base.library.api.{Restricted, Wrapper, Allowed, Value, needs};
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = consumer(
        &workspace,
        r#"
module app.main;
import middle.relay.api.{Restricted, Wrapper, Allowed, Value, needs};
flow wrap<T ~ Allowed>(value: T) -> Restricted<T> { return Restricted { value = needs(value) }; }
flow main() -> bool {
    let record: Wrapper<Value> = Restricted<Value> { value = Value(42) };
    let call = Restricted<Value>({ value = Value(42) });
    return record.value == call.value;
}
"#,
    );
    run(loaded.clone());
    for body in [
        "flow main() -> Wrapper<i32> { return Restricted<i32>({ value = 42 }); }",
        "flow main() -> Wrapper<i32> { return Restricted { value = 42 }; }",
        "flow invalid(value: Restricted<i32>) -> unit { return; } flow main() -> unit { return; }",
        "flow invalid<T>(value: Restricted<T>) -> unit { return; } flow main() -> unit { return; }",
        "flow invalid<T>(value: T) -> T { return needs(value); } flow main() -> unit { return; }",
    ] {
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import middle.relay.api.{{Restricted, Wrapper, Allowed, Value, needs}}; {body}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(
            output.checked.is_none(),
            "unproved alias bound was accepted: {body}"
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|d| d.message.contains("spec")),
            "{:?}",
            output.diagnostics
        );
    }
}

#[test]
fn transitive_reexport_rejects_a_tampered_callable_definition_signature() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.CallableDef;
public flow invoke(draft: CallableDef<(i32) -> i32 ![]>) -> unit ![] { return; }
"#,
    );
    relay(
        &workspace,
        r#"module relay.api;
public import base.library.api.invoke;
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = consumer(
        &workspace,
        "module app.main; import middle.relay.api.invoke; flow main() -> unit { return; }",
    );
    let baseline = Frontend.check_project(loaded.loaded.input.clone());
    assert!(
        baseline.diagnostics.is_empty(),
        "{:?}",
        baseline.diagnostics
    );
    assert!(baseline.checked.is_some());
    let signatures = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter_mut()
        .flat_map(|metadata| &mut metadata.flows)
        .filter(|flow| flow.path.last().is_some_and(|name| name == "invoke"))
        .collect::<Vec<_>>();
    assert_eq!(
        signatures.len(),
        1,
        "expected canonical declaration, not a duplicate re-export signature"
    );
    let etas_frontend::ProjectExternalTypeInput::Applied { args, .. } =
        &mut signatures.into_iter().next().unwrap().params[0]
    else {
        panic!("expected CallableDef")
    };
    let etas_frontend::ProjectExternalTypeInput::Function { input, .. } = &mut args[0] else {
        panic!("expected function")
    };
    input[0] = etas_frontend::ProjectExternalTypeInput::Primitive("bool".into());
    let source = loaded.loaded.input.sources[0].clone();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(
        output.checked.is_none(),
        "tampered transitive callable signature was accepted"
    );
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                )
                && diagnostic.message.contains("parameter type")
        })
        .unwrap_or_else(|| panic!("{:?}", output.diagnostics));
    assert_eq!(diagnostic.primary.span.source, source.id);
    let span = diagnostic.primary.span.range;
    assert!(
        source.text[span.start.0 as usize..span.end.0 as usize].contains("middle.relay.api.invoke")
    );
}

#[test]
fn malformed_transitive_binary_metadata_is_anchored_to_the_introducing_import() {
    let workspace = origin_metadata::workspace_with_source(BASE);
    relay(
        &workspace,
        r#"
module relay.api;
import base.library.api.Value;
public alias PublicValue = Value;
public flow lookup() -> i32 { return 7; }
"#,
    );
    let root = package(
        &workspace.0,
        "final-consumer",
        r#"
[dependencies]
relay = { package = "relay-library", version = "0.1.0", import = "middle", path = "../relay-library" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
        "app/main.es",
        "module app.main; import middle.relay.api.lookup; flow main() -> i32 { return lookup(); }",
    );
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("relay-library"));
    let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
        &artifact,
        &std::fs::read(&artifact).unwrap(),
    )
    .unwrap();
    metadata
        .dependencies
        .iter_mut()
        .find(|dependency| dependency.identity.name == "origin-library")
        .unwrap()
        .public_metadata
        .flows
        .iter_mut()
        .find(|flow| flow.path.last().is_some_and(|name| name == "lookup"))
        .unwrap()
        .output = Some(etas_package_metadata::Type {
        kind: etas_package_metadata::TypeKind::Primitive,
        name: "invalid_primitive".into(),
        ..Default::default()
    });
    let bytes = etas_package_metadata::encode_metadata_artifact(
        &header,
        etas_package_metadata::package_metadata_to_sections(&metadata).unwrap(),
    )
    .unwrap();
    etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
    etas_package::materialize_package(etas_package::MaterializePackageOptions {
        package_root: root.clone(),
        source_config: Some(etas_package::PackageSourceConfig::read_project(&root).unwrap()),
    })
    .unwrap();
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: root.clone(),
    })
    .unwrap();
    let loaded = etas_driver::load_project(
        &origin_metadata::options(&root),
        LoadProjectRequest {
            paths: vec![root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let source = &loaded.loaded.input.sources[0];
    assert!(loaded.loaded.input.environment.external_public_metadata.iter().flat_map(|metadata| &metadata.flows).any(|flow| {
        matches!(&flow.output, etas_frontend::ProjectExternalTypeInput::Primitive(name) if name == "invalid_primitive")
    }), "driver must preserve the malformed transitive binary metadata for boundary validation: {:?}", loaded.loaded.input.environment.external_public_metadata.iter().flat_map(|metadata| &metadata.flows).map(|flow| (&flow.path, &flow.output)).collect::<Vec<_>>());
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("invalid_primitive"))
        .expect("malformed transitive metadata must produce a structured diagnostic");
    assert_eq!(
        diagnostic.code,
        etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
    );
    assert_eq!(diagnostic.primary.span.source, source.id);
    let span = diagnostic.primary.span.range;
    assert!(
        source.text[span.start.0 as usize..span.end.0 as usize].contains("middle.relay.api.lookup")
    );
}
