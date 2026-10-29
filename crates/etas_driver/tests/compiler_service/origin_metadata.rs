use super::*;
use etas_code::descriptor::{AnnotationReferenceInfo, StaticValueInfo};
use etas_driver::{
    DriverCacheMode, DriverOptions, LoadProjectRequest, PackageMetadataRequest,
    PackageUpdateRequest, SourceDependencyMode, compiler::prepare_native_definitions,
};
use std::path::{Path, PathBuf};

pub(super) struct Workspace(pub(super) PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn options(root: &Path) -> DriverOptions {
    DriverOptions {
        workspace: Some(root.to_owned()),
        cache_mode: DriverCacheMode::Off,
        ..Default::default()
    }
}

fn workspace() -> Workspace {
    workspace_with_source("module library.api; public flow lookup() -> i32 { return 7; }")
}

pub(super) fn workspace_with_source(source: &str) -> Workspace {
    workspace_with_source_config(source, "")
}

pub(super) fn workspace_with_source_config(source: &str, configuration: &str) -> Workspace {
    workspace_with_sources(&[("library/api.es", source)], configuration)
}

pub(super) fn workspace_with_sources(sources: &[(&str, &str)], configuration: &str) -> Workspace {
    static NEXT_WORKSPACE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "etas-origin-metadata-{}-{nonce}-{}",
        std::process::id(),
        NEXT_WORKSPACE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    let root = Workspace(path);
    let library = root.0.join("library");
    std::fs::create_dir_all(library.join("src/library")).unwrap();
    std::fs::write(
        library.join("etas.toml"),
        format!("[package]\nname = \"origin-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n{configuration}"),
    )
    .unwrap();
    for (path, source) in sources {
        let path = library.join("src").join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    etas_driver::build_package_metadata(
        &options(&library),
        PackageMetadataRequest {
            package_root: library,
            bin: None,
        },
    )
    .unwrap();
    root
}

pub(super) fn consumer(workspace: &Workspace, alias: &str) -> etas_driver::LoadedDriverProject {
    let root = workspace.0.join(format!("consumer-{alias}"));
    std::fs::create_dir_all(root.join("src/app")).unwrap();
    std::fs::write(
        root.join("etas.toml"),
        format!(
            r#"
[package]
name = "consumer-{alias}"
version = "0.1.0"
edition = "2026"
[dependencies]
lib = {{ package = "origin-library", version = "0.1.0", import = "{alias}", path = "../library" }}
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        format!(
            r#"
module app.main;
import {alias}.library.api as Lib;
import {alias}.library.api.lookup as chosen;
@custom(Lib, symbol = chosen)
agent Worker() -> i32 {{ return abort("reflection must not execute the agent"); }}
flow main() -> Array<std.compiler.AnnotationInfo> {{ return (^Worker).annotations; }}
"#
        ),
    )
    .unwrap();
    etas_driver::update_package(
        &options(&root),
        PackageUpdateRequest {
            package_root: root.clone(),
        },
    )
    .unwrap();
    etas_driver::load_project(
        &options(&root),
        LoadProjectRequest {
            paths: vec![root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap()
}

pub(super) fn check(loaded: &etas_driver::LoadedDriverProject) -> etas_frontend::CheckedProgram {
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    output.checked.unwrap()
}

#[test]
fn published_definition_origins_replay_through_binary_driver_and_reflection_without_dependency_source()
 {
    let workspace = workspace();
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let expected = metadata
        .public_metadata
        .definition_origins
        .as_ref()
        .unwrap();
    for alias in ["first", "second"] {
        let loaded = consumer(&workspace, alias);
        assert_eq!(loaded.loaded.input.sources.len(), 1);
        let checked = check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1);
        let definitions =
            prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
        let expression = *checked.types.reflections.keys().next().unwrap();
        let reference = definitions.reflect_agent(expression).unwrap();
        let view = definitions.resolve(&reference).unwrap();
        let annotation = &view.annotations().unwrap()[0];
        assert!(
            matches!(annotation.arguments[0].value.as_ref(), StaticValueInfo::Reference(AnnotationReferenceInfo::Module { package, path }) if *package == expected.package && path == &["library", "api"])
        );
        assert!(
            matches!(annotation.arguments[1].value.as_ref(), StaticValueInfo::Reference(AnnotationReferenceInfo::Declaration(key)) if key.package() == expected.package && key.module() == ["library", "api"] && key.declaration() == ["lookup"])
        );
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
        assert_eq!(host.calls(), 0);
        let json = etas_interpreter::api::codec::value_json(result.value().unwrap()).unwrap();
        assert!(json.to_string().contains("StaticValue"));
        assert!(!result.events.iter().any(|event| matches!(
            event,
            etas_interpreter::api::WorkflowEvent::HostTrace(
                etas_host::TraceEvent::HostRequestStarted { .. }
            )
        )));
    }
}

#[test]
fn definition_origins_cannot_impersonate_a_different_prepared_package_owner() {
    let workspace = workspace();
    let mut loaded = consumer(&workspace, "first");
    let forged = PackageKey::from_canonical_fingerprint(CodeFingerprint::from_bytes([0x42; 32]));
    let input = &mut loaded.loaded.input.environment.external_definition_origins[0];
    input.origins.package = forged;
    for binding in &mut input.origins.declarations {
        binding.definition = DefinitionKey::new(
            forged,
            binding.definition.module().to_vec(),
            binding.definition.declaration().to_vec(),
            binding.definition.kind(),
        )
        .unwrap();
    }
    for binding in &mut input.descriptors.as_mut().unwrap().declarations {
        binding.definition = DefinitionKey::new(
            forged,
            binding.definition.module().to_vec(),
            binding.definition.declaration().to_vec(),
            binding.definition.kind(),
        )
        .unwrap();
    }
    for callable in &mut input
        .descriptors
        .as_mut()
        .unwrap()
        .headers
        .as_mut()
        .unwrap()
        .callables
    {
        let original = callable.id.as_declaration().unwrap();
        callable.id = etas_code::CallableId::declaration(
            DefinitionKey::new(
                forged,
                original.module().to_vec(),
                original.declaration().to_vec(),
                original.kind(),
            )
            .unwrap(),
        )
        .unwrap();
    }
    for snapshot in &mut input
        .descriptors
        .as_mut()
        .unwrap()
        .headers
        .as_mut()
        .unwrap()
        .snapshots
        .as_mut()
        .unwrap()
        .flows
    {
        let original = snapshot.identity.package();
        snapshot.identity = DefinitionKey::new(
            forged,
            snapshot.identity.module().to_vec(),
            snapshot.identity.declaration().to_vec(),
            snapshot.identity.kind(),
        )
        .unwrap();
        snapshot.revision.definition =
            etas_code::CallableId::declaration(snapshot.identity.clone()).unwrap();
        snapshot.dependencies.lexical_owner = snapshot.identity.clone();
        for module in &mut snapshot.dependencies.modules {
            if module.package == original {
                module.package = forged;
            }
        }
        for binding in &mut snapshot.dependencies.bindings {
            if binding.definition.package() == original {
                binding.definition = DefinitionKey::new(
                    forged,
                    binding.definition.module().to_vec(),
                    binding.definition.declaration().to_vec(),
                    binding.definition.kind(),
                )
                .unwrap();
            }
        }
    }
    let checked = check(&loaded);
    let error = prepare_native_definitions(&loaded, &checked, Default::default())
        .err()
        .expect("forged owner must be rejected");
    assert!(
        error.to_string().contains("prepared source origin"),
        "{error}"
    );
}

#[test]
fn malformed_binary_definition_kind_is_rejected_at_the_real_import_span() {
    let workspace = workspace();
    let initial = consumer(&workspace, "first");
    let consumer_root = initial.package_root.unwrap();
    let library = workspace.0.join("library");
    let artifact = etas_package::package_metadata_artifact_path(&library);
    let bytes = std::fs::read(&artifact).unwrap();
    let (header, mut metadata) =
        etas_package_metadata::package_metadata_from_artifact(&artifact, &bytes).unwrap();
    let origins = metadata
        .public_metadata
        .definition_origins
        .as_mut()
        .unwrap();
    let binding = &mut origins.declarations[0];
    binding.definition = DefinitionKey::new(
        origins.package,
        binding.definition.module().to_vec(),
        binding.definition.declaration().to_vec(),
        etas_code::DefinitionKind::Tool,
    )
    .unwrap();
    metadata
        .public_metadata
        .definition_descriptors
        .as_mut()
        .unwrap()
        .declarations[0]
        .definition = binding.definition.clone();
    metadata
        .public_metadata
        .definition_descriptors
        .as_mut()
        .unwrap()
        .headers
        .as_mut()
        .unwrap()
        .callables[0]
        .id = etas_code::CallableId::declaration(binding.definition.clone()).unwrap();
    // This forged Tool no longer claims a Flow snapshot. Keep the graph
    // internally coherent so the frontend can reject the export-kind mismatch.
    metadata
        .public_metadata
        .definition_descriptors
        .as_mut()
        .unwrap()
        .headers
        .as_mut()
        .unwrap()
        .snapshots
        .as_mut()
        .unwrap()
        .flows
        .clear();
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
    etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
    etas_package::materialize_package(etas_package::MaterializePackageOptions {
        package_root: consumer_root.clone(),
        source_config: Some(
            etas_package::PackageSourceConfig::read_project(&consumer_root).unwrap(),
        ),
    })
    .unwrap();
    etas_package::lock_package(etas_package::LockPackageOptions {
        package_root: consumer_root.clone(),
    })
    .unwrap();
    let loaded = etas_driver::load_project(
        &options(&consumer_root),
        LoadProjectRequest {
            paths: vec![consumer_root],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    assert_eq!(
        loaded
            .loaded
            .input
            .environment
            .external_definition_origins
            .len(),
        1
    );
    assert_eq!(
        loaded.loaded.input.environment.external_definition_origins[0]
            .origins
            .declarations[0]
            .definition
            .kind(),
        etas_code::DefinitionKind::Tool,
        "package preparation must preserve the tampered artifact for validation"
    );
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let source = &loaded.loaded.input.sources[0];
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("kind disagrees"))
        .expect("invalid binary metadata needs a structured import diagnostic");
    assert_eq!(
        diagnostic.code,
        etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
    );
    assert_eq!(diagnostic.primary.span.source, source.id);
    let span = diagnostic.primary.span.range;
    let text = &source.text[span.start.0 as usize..span.end.0 as usize];
    assert!(
        text.contains("first.library.api"),
        "anchor was not the resolved dependency import: {text}"
    );
}

#[test]
fn published_descriptors_preserve_recursive_public_type_closures_without_private_code_exports() {
    use etas_code::descriptor::*;
    let workspace = workspace_with_source(
        r#"
module library.api;
type Hidden = string;
flow private_helper() -> i32 { return 99; }
public type Ticket = { value: Hidden };
public enum Tree<T> { Leaf(T), Branch(Tree<T>, Tree<T>), }
public flow relay<effect E>(f: () -> unit ![E]) -> unit ![E] { f(); return; }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let package = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let descriptors = package.public_metadata.definition_descriptors.unwrap();
    descriptors.validate(DescriptorLimits::default()).unwrap();
    let tree = descriptors
        .declarations
        .iter()
        .find(|binding| binding.definition.declaration() == ["Tree"])
        .unwrap();
    assert!(
        matches!(&descriptors.graph.types[tree.ty.0 as usize], TypeInfo::Enum { parameters, variants, .. } if parameters.len() == 1 && variants.len() == 2)
    );
    assert!(descriptors.graph.types.iter().any(|ty| matches!(ty, TypeInfo::Nominal { definition, .. } if definition.declaration() == ["Hidden"])));
    assert!(!descriptors.declarations.iter().any(|binding| matches!(binding.definition.declaration(), [name] if name == "Hidden" || name == "private_helper")));
    let relay = descriptors
        .declarations
        .iter()
        .find(|binding| binding.definition.declaration() == ["relay"])
        .unwrap();
    let TypeInfo::Function(signature) = &descriptors.graph.types[relay.ty.0 as usize] else {
        panic!("relay signature descriptor");
    };
    let tail = descriptors.graph.rows[signature.effects.unwrap().0 as usize]
        .tail
        .unwrap();
    let binder = &descriptors.graph.binders[tail.0 as usize];
    assert_eq!(binder.kind, GenericParameterKind::EffectRow);
    assert_eq!(
        binder.owner,
        GenericOwner::Declaration(relay.definition.clone())
    );
    let loaded = consumer(&workspace, "first");
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = check(&loaded);
    let replayed = checked.external_definition_origins()[0]
        .descriptors
        .as_ref()
        .unwrap();
    assert_eq!(replayed, &descriptors);
    assert_eq!(
        etas_package_metadata::definition_descriptors_fingerprint(replayed),
        etas_package_metadata::definition_descriptors_fingerprint(&descriptors)
    );
}

#[test]
fn callback_handler_scope_survives_published_metadata_and_consumer_loading() {
    use etas_code::descriptor::{EffectTargetInfo, ParameterHandlerScopeInfo, TraceInfo};

    let workspace = workspace_with_source(
        r#"
module library.api;
public effect Choice { action decide() -> bool; }
public flow relay(f: () -> unit ![]) -> unit ![] {
    handle { f(); } with { Console.stdout_write(_) => { resume; } };
    return;
}
public flow choose(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { resume true; } };
    return;
}
public flow stop(f: () -> unit ![]) -> unit ![] {
    handle { f(); } with { Console.stdout_write(_) => { finish (); } };
    return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let descriptors = metadata.public_metadata.definition_descriptors.unwrap();
    let mut scopes = descriptors
        .graph
        .traces
        .iter()
        .filter_map(|trace| match trace {
            TraceInfo::ParameterCall(call)
                if call
                    .callable
                    .as_declaration()
                    .is_some_and(|id| id.declaration() == ["relay"]) =>
            {
                call.handlers.as_ref()
            }
            _ => None,
        });
    let handlers = scopes
        .next()
        .expect("checked callback scope in published trace");
    let [ParameterHandlerScopeInfo::Handle { arms }] = handlers.as_slice() else {
        panic!("expected one checked resuming handler: {handlers:?}");
    };
    assert!(
        matches!(arms.as_slice(), [arm] if matches!(&arm.action.target, EffectTargetInfo::Action { name, .. } if name == "stdout_write") && arm.completion.value() == &etas_code::descriptor::ReturnObservationInfo::Unit)
    );
    assert!(descriptors.graph.traces.iter().any(|trace| matches!(trace,
        TraceInfo::ParameterCall(call) if call.callable.as_declaration().is_some_and(|id| id.declaration() == ["choose"])
            && matches!(call.handlers.as_deref(), Some([ParameterHandlerScopeInfo::Handle { arms }])
                if matches!(arms.as_slice(), [arm] if matches!(&arm.action.target, EffectTargetInfo::Action { name, .. } if name == "decide")
                    && arm.completion.value() == &etas_code::descriptor::ReturnObservationInfo::Bool(true)))
    )), "boolean handler must survive metadata projection");
    assert!(descriptors.graph.traces.iter().any(|trace| matches!(trace,
        TraceInfo::ParameterCall(call) if call.callable.as_declaration().is_some_and(|id| id.declaration() == ["stop"])
            && matches!(call.handlers.as_deref(), Some([ParameterHandlerScopeInfo::Handle { arms }])
                if matches!(arms.as_slice(), [arm] if matches!(&arm.completion, etas_code::descriptor::ParameterHandlerCompletionInfo::Finish { .. })))
    )), "finish handler must survive metadata projection");

    let mut loaded = consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.relay;
import first.library.api.choose;
import first.library.api.stop;
flow main() -> unit { let definition = ^relay; let choice = ^choose; let stopped = ^stop; return; }
"#
    .into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = check(&loaded);
    let replayed = checked.external_definition_origins()[0]
        .descriptors
        .as_ref()
        .unwrap();
    assert_eq!(replayed, &descriptors);

    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.relay;
import std.io.println;
flow write() -> unit ![] { println("hello")?; return; }
flow main() -> unit ![] { relay(write); return; }
"#
    .into();
    let checked = check(&loaded);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let console = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(
        summary.requested_actions.effects.contains(&console),
        "{summary:?}"
    );
    assert!(
        !summary.default_actions.effects.contains(&console),
        "{summary:?}"
    );
}

#[test]
fn callback_default_actions_remain_possible_for_partial_or_mixed_external_scopes() {
    for body in [
        r#"handle { f(); } with { Console.stdout_write("other") => { resume; } };"#,
        r#"handle { f(); } with { Console.stdout_write(_) => { resume; } }; f();"#,
    ] {
        let workspace = workspace_with_source(&format!(
            "module library.api; public flow relay(f: () -> unit ![]) -> unit ![] {{ {body} return; }} public flow lookup() -> i32 {{ return 7; }}"
        ));
        let mut loaded = consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.relay;
import std.io.println;
flow write() -> unit ![] { println("hello")?; return; }
flow main() -> unit ![] { relay(write); return; }
"#
        .into();
        let checked = check(&loaded);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        let console = etas_effects::Effect::Action(etas_effects::ActionRef {
            tag: etas_effects::CONSOLE_TAG,
            action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
        });
        assert!(
            summary.requested_actions.effects.contains(&console),
            "{body}: {summary:?}"
        );
        assert!(
            summary.default_actions.effects.contains(&console),
            "{body}: {summary:?}"
        );
    }
}

#[test]
fn published_agent_headers_keep_default_and_auxiliary_methods_separate_without_private_method_exports()
 {
    use etas_code::descriptor::*;
    let workspace = workspace_with_source(
        r#"
module library.api;
import std.text as Text;
let label = "meta";
public spec Stage<I, O> I => O ![];
@custom(Text, symbol = Text.trim, details = { enabled = true, label }, labels = ["a", "b"])
public agent Worker(input: string) -> string ~ Stage { return input; }
impl agent Worker {
    public flow explain(value: string) -> string { return value; }
    private flow secret() -> i32 { return 99; }
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let stage = metadata
        .public_metadata
        .spec_signatures
        .iter()
        .find(|spec| spec.path.last().is_some_and(|name| name == "Stage"))
        .unwrap();
    let callable = stage.callable.as_ref().unwrap();
    assert!(
        matches!(&callable.params[0], etas_package::PackageTypeMetadata::Var { name } if name == "I")
    );
    assert!(
        matches!(&callable.output, etas_package::PackageTypeMetadata::Var { name } if name == "O")
    );
    let descriptors = metadata.public_metadata.definition_descriptors.unwrap();
    let headers = descriptors
        .headers
        .as_ref()
        .expect("native metadata must carry full definition headers");
    let worker = headers
        .agents
        .iter()
        .find(|agent| agent.identity.declaration() == ["Worker"])
        .unwrap();
    let default = worker.default_entry.as_ref().unwrap();
    assert_eq!(default.as_method().unwrap().name(), "run");
    assert_eq!(worker.methods.len(), 1);
    assert_eq!(worker.methods[0].as_method().unwrap().name(), "explain");
    assert!(!headers.callables.iter().any(|header| {
        header
            .id
            .as_method()
            .is_some_and(|method| method.name() == "secret")
    }));
    let callable = headers
        .callables
        .iter()
        .find(|callable| &callable.id == default)
        .unwrap();
    let TypeInfo::Function(signature) = &descriptors.graph.types[callable.signature.0 as usize]
    else {
        panic!("default callable signature");
    };
    assert_eq!(signature.parameters[0].name.as_deref(), Some("input"));
    assert_eq!(
        descriptors.graph.types[signature.output.0 as usize],
        TypeInfo::Primitive(PrimitiveInfo::String)
    );
    assert_eq!(worker.annotations.len(), 1);
    assert_eq!(worker.annotations[0].arguments.len(), 4);
    assert!(worker.specs.iter().any(|spec| matches!(spec, AttachedSpecInfo::Named(spec) if spec.definition.declaration() == ["Stage"])));
    for alias in ["first", "second"] {
        let loaded = consumer(&workspace, alias);
        assert_eq!(loaded.loaded.input.sources.len(), 1);
        let checked = check(&loaded);
        assert_eq!(
            checked.external_definition_origins()[0]
                .descriptors
                .as_ref()
                .unwrap(),
            &descriptors
        );
    }
}

#[test]
fn published_agent_snapshot_replays_the_native_revision_and_locked_dependencies() {
    let workspace = workspace_with_source(
        r#"
module library.api;
public agent Worker(value: i32) -> i32 { return value; }
impl agent Worker {
    public flow explain(value: i32) -> i32 { return value; }
    private flow secret() -> i32 { return 99; }
}
public flow lookup() -> i32 { return 7; }
flow inspect() -> unit { let definition = ^Worker; return; }
"#,
    );
    let root = workspace.0.join("library");
    let metadata = etas_package::read_package_metadata_artifact(&root)
        .unwrap()
        .unwrap();
    let headers = metadata
        .public_metadata
        .definition_descriptors
        .as_ref()
        .unwrap()
        .headers
        .as_ref()
        .unwrap();
    let snapshots = headers.snapshots.as_ref().unwrap();
    assert_eq!(snapshots.agents.len(), 1);
    let snapshot = &snapshots.agents[0];
    assert_eq!(snapshot.identity, headers.agents[0].identity);
    let loaded = etas_driver::load_project(
        &options(&root),
        LoadProjectRequest {
            paths: vec![root],
            all: false,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap();
    let checked = check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let reference = definitions.reflect_agent(expression).unwrap();
    let native = definitions.resolve(&reference).unwrap();
    assert_eq!(snapshot.revision_info(), *native.revision());
    assert_eq!(snapshot.dependencies, native.dependency_info().unwrap());
    assert_eq!(native.environment().unwrap().contract().methods().len(), 2);
    assert_eq!(headers.agents[0].methods.len(), 1);
    for alias in ["first", "second"] {
        let consumer = consumer(&workspace, alias);
        let consumer = check(&consumer);
        let replayed = consumer.external_definition_origins()[0]
            .descriptors
            .as_ref()
            .unwrap()
            .headers
            .as_ref()
            .unwrap();
        assert_eq!(replayed.snapshots.as_ref().unwrap(), snapshots);
    }
}

#[test]
fn bare_agent_metadata_does_not_invent_a_default_callable() {
    let workspace = workspace_with_source(
        r#"
module library.api;
public agent Worker;
impl agent Worker { public flow explain(value: i32) -> i32 { return value; } }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let signature = &metadata.public_metadata.agents[0];
    assert!(signature.output.is_none());
    assert!(signature.input.is_empty());
    let header = &metadata
        .public_metadata
        .definition_descriptors
        .as_ref()
        .unwrap()
        .headers
        .as_ref()
        .unwrap()
        .agents[0];
    assert!(header.default_entry.is_none());
    assert_eq!(header.methods.len(), 1);

    let mut loaded = consumer(&workspace, "first");
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Worker;
flow main() -> unit { let definition = ^Worker; return; }
"#
    .into();
    let checked = check(&loaded);
    assert!(checked.types.symbol_types.values().any(|fact| {
        matches!(fact, etas_types::SymbolTypeFact::Agent { signature } if signature.default_entry.is_none())
    }));
    assert_eq!(checked.types.reflections.len(), 1);

    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Worker;
flow main() -> unit { Worker(); return; }
"#
    .into();
    let rejected = Frontend.check_project(loaded.loaded.input.clone());
    assert!(
        rejected.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == etas_core::Severity::Error
                && diagnostic.message.contains("default")
        }),
        "{:?}",
        rejected.diagnostics
    );
    assert!(rejected.checked.is_none());
}

#[test]
fn binary_agent_metadata_cannot_add_a_default_entry_missing_from_the_definition() {
    let workspace = workspace_with_source(
        "module library.api; public agent Worker; public flow lookup() -> i32 { return 7; }",
    );
    let initial = consumer(&workspace, "first");
    let root = initial.package_root.unwrap();
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
        &artifact,
        &std::fs::read(&artifact).unwrap(),
    )
    .unwrap();
    let public = &mut metadata.public_metadata;
    let agent = &mut public.agents[0];
    agent.output = public.flows[0].output.clone();
    let mut summary = public
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "lookup"))
        .unwrap()
        .clone();
    summary.item = agent.path.clone();
    public.effect_summaries.push(summary);
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
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
        &options(&root),
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
    let output = Frontend.check_project(loaded.loaded.input.clone());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("disagree on the default entry"))
        .expect("conflicting binary agent metadata must be rejected at the environment boundary");
    assert_eq!(
        diagnostic.code,
        etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
    );
    let source = &loaded.loaded.input.sources[0];
    assert_eq!(diagnostic.primary.span.source, source.id);
    let range = diagnostic.primary.span.range;
    assert!(
        source.text[range.start.0 as usize..range.end.0 as usize].contains("first.library.api")
    );
    assert!(output.checked.is_none());
}
