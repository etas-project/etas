use etas_frontend::{EntryPolicy, Frontend, ProjectEnvironmentInput, ProjectInput, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};
use etas_package_metadata::CallableContractTarget;

const SOURCE: &str = r#"module library;
import std.io.println;
public type Container<T> = { value: T };
public alias Packet = Container<string>;
public spec Encoder { flow encode(self) -> i32 ![]; }
public spec Safe: trace = +Console.stdout_write;
impl Packet {
    public flow new(value: string) -> Packet ![] { return Container<string> { value }; }
    public flow copy(self) -> Packet ![] { return self; }
}
impl Packet ~ Encoder {
    flow encode(self) -> i32 ![] ~ Safe { println("encoded")?; return 42; }
}
public flow make() -> Packet { return Container<string> { value = "body" }; }
"#;

fn environment() -> ProjectEnvironmentInput {
    environment_with_root("library")
}

fn environment_with_root(import_root: &str) -> ProjectEnvironmentInput {
    environment_from_source(SOURCE, import_root)
}

fn environment_from_source(source: &str, import_root: &str) -> ProjectEnvironmentInput {
    let mut input = ProjectInput::single_source(SourceInput::anonymous(source));
    input.options.entry_policy = EntryPolicy::Optional;
    let checked = Frontend.check_project(input);
    assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
    let fixture = super::callback_replay_tests::PackageFixture::named("library", source);
    let source_hash = etas_package::package_source_payload_hash(&fixture.0).unwrap();
    let artifact = etas_frontend::build_package_metadata_artifact_from_checked(
        etas_frontend::PackageMetadataBuildInput {
            definition_origin: Some(
                etas_code::PackageKey::from_name_and_version("library", "0.1.0").unwrap(),
            ),
            package_id: "library".into(),
            package_version: "0.1.0".into(),
            package_edition: "2026".into(),
            compiler_version: etas_frontend::FRONTEND_COMPILER_VERSION.into(),
            source_payload_hash: source_hash,
            manifest_hash: etas_package::package_manifest_hash(&fixture.0).unwrap(),
            dependency_lock_hash: etas_package::package_dependency_lock_hash(&fixture.0).unwrap(),
            bins: vec![],
            dependencies: vec![],
            tool_bindings: vec![],
        },
        checked.checked.as_ref().unwrap(),
    )
    .unwrap();
    let path = etas_package::package_metadata_artifact_path(&fixture.0);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, artifact.bytes).unwrap();
    let metadata = etas_package::read_package_metadata_artifact(&fixture.0)
        .unwrap()
        .unwrap();
    super::frontend_environment(PackageEnvironmentMetadata {
        dependencies: vec![ResolvedDependency {
            identity: metadata.package,
            std_compatibility: metadata.std_compatibility,
            import_root: import_root.into(),
            source: ResolvedDependencySource::Path {
                path: "/no-dependency-source".into(),
                checksum: "blake3:source".into(),
            },
            dependencies: metadata.dependencies,
            public_metadata: metadata.public_metadata,
            effect_metadata: metadata.effect_metadata,
            tool_bindings: metadata.tool_bindings,
        }],
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn binary_generic_tool_invocation_specializes_the_consumers_checked_surface() {
    let environment = environment_from_source(
        r#"module library;
import std.agent.{ToolCall, ToolResult};
public flow execute<C>(call: ToolCall<C>) -> ToolResult<C> {
    return std.tool.invoke<C>(call);
}


"#,
        "native",
    );
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module app.main;
import native.library.execute;
import std.agent.ToolCall;
import std.json.JsonValue;
import std.io.println;
tool echo(text: string) -> string ![] { println(text)?; return text; }
tool fail() -> string ![Error<i32>] { return perform Error<i32>.raise(99); }
@tools([echo, fail])
agent Runner() -> unit ![Error<i32>] {
    let call = ToolCall<Runner.run> { id = "generic", tool = std.tool.reference(echo), args = JsonValue.Object({"text" => JsonValue.String("hi")}) };
    execute(call);
    execute(call);
}
flow main() -> unit {
    handle Runner.run() with { Error<i32>.raise(_) => { finish (); } };
}
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    let runner = checked
        .hir
        .items
        .iter()
        .find_map(|(item, declaration)| match declaration {
            etas_hir::HirItem::Agent(agent)
                if checked.hir.symbols.get(agent.symbol).unwrap().name == "Runner" =>
            {
                Some(item)
            }
            _ => None,
        })
        .unwrap();
    let summary = &checked.effects.item_effects[&runner];
    assert!(
        summary
            .escaping_effects
            .effects
            .iter()
            .any(|effect| matches!(effect, etas_effects::Effect::Error(_))),
        "{summary:?}"
    );
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    let mut pending = vec![&summary.action_trace];
    let mut writes = 0;
    while let Some(trace) = pending.pop() {
        match trace {
            etas_effects::ActionTraceDomain::Event(event) => {
                writes += usize::from(event.action == stdout)
            }
            etas_effects::ActionTraceDomain::Seq(children)
            | etas_effects::ActionTraceDomain::Choice(children) => pending.extend(children),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("tool trace was not specialized: {other:?}"),
        }
    }
    assert_eq!(writes, 2, "{summary:?}");
}

#[test]
fn binary_imported_agent_context_replays_its_exposed_tools() {
    let environment = environment_from_source(
        r#"module library;
tool echo(text: string) -> string ![] { std.io.println(text)?; return text; }
@tools([echo])
public agent Writer() -> string { return perform infer<string>(Prompt.new()); }
"#,
        "native",
    );
    assert!(
        environment.external_modules.iter().all(|module| module
            .exports
            .iter()
            .all(|export| export.name != "echo"
                || export.visibility == etas_hir::Visibility::Private)),
        "publishing an agent tool contract must not make its declaration public"
    );
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import native.library.Writer;
import std.agent.{ToolCall, ToolRef};
import std.json.JsonValue;
flow main() -> string {
    return handle Writer.run() with {
        Agentic.infer<Writer.run, string>(_, _) => {
            let call = ToolCall<Writer.run> {
                id = "simulated",
                tool = ToolRef { id = "unregistered", revision = "unregistered" },
                args = JsonValue.Null,
            };
            std.tool.invoke(call);
            resume "handled";
        }
    };
}
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.default_actions.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.action_trace.action_set().contains(&stdout),
        "{summary:?}"
    );
}

#[test]
fn binary_agent_context_keeps_its_canonical_method_across_import_roots() {
    let source = r#"module library;
public agent Writer() -> string {
    return perform infer<string>(Prompt.new());
}
"#;
    let mut identities = Vec::new();
    for root in ["native", "alternate"] {
        let mut input = ProjectInput::single_source(SourceInput::anonymous(format!(
            "module app.main; import {root}.library.Writer; flow main() -> string {{ return Writer.run(); }}"
        )));
        input.environment = environment_from_source(source, root);
        let output = Frontend.check_project(input);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let checked = output.checked.unwrap();
        assert_eq!(checked.sources.sources.len(), 1);
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        let mut contexts = Vec::new();
        for action in summary.requested_actions.effects.iter() {
            if let etas_effects::Effect::AppliedAction(action) = action
                && action.action.tag == etas_effects::AGENTIC_TAG
            {
                let etas_types::EffectArgRef::Type(context) = action.args[0] else {
                    panic!("agentic context is not a checked type");
                };
                let Some(etas_types::Type::AgentContext(etas_types::AgentContextRef::Portable(
                    method,
                ))) = checked.type_store.get(context)
                else {
                    panic!("agentic context is not a portable method");
                };
                contexts.push((**method).clone());
            }
        }
        assert_eq!(contexts.len(), 2, "{summary:?}");
        assert_eq!(contexts[0], contexts[1]);
        identities.push(contexts[0].clone());
    }
    assert_eq!(identities[0], identities[1]);
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        "module app.main; import native.library.Writer; flow main() -> string { return Writer.run() with { Agentic.infer<_, string>(_, _) => resume \"mock\"; }; }",
    ));
    input.environment = environment_from_source(source, "native");
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert_eq!(summary.requested_actions.effects.len(), 1, "{summary:#?}");
    assert!(summary.default_actions.is_empty(), "{summary:#?}");
    assert_eq!(
        summary.action_trace.action_set(),
        summary.requested_actions.effects
    );
    assert!(
        matches!(&summary.support, etas_effects::InterpreterSupport::RequiresInterpreterOrchestration(requirements) if requirements.host.kinds.is_empty())
    );
}

#[test]
fn binary_spec_method_labels_bind_declared_names_not_implementation_locals() {
    let environment = environment_from_source(
        r#"module library;
import std.io.println;
public type Packet = { value: i32 };
public spec Offset { flow offset(self, amount: i32) -> i32 ![]; }
impl Packet ~ Offset {
    flow offset(self, input: i32) -> i32 ![] {
        println("offset")?;
        return self.value + input;
    }
}
public flow make() -> Packet { return Packet { value = 4 }; }
"#,
        "alternate",
    );
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import alternate.library.{Offset as Add, make};
flow bounded<T ~ Add>(value: T) -> i32 { return value.offset(amount = 2); }
flow selected<T ~ Add>(value: T) -> i32 { return value::Add.offset(amount = 3); }
flow main() -> i32 { return bounded(make()) + selected(make()) + make().offset(amount = 5); }
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    assert_eq!(checked.types.checked_method_calls.len(), 3);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty());
    assert_eq!(summary.requested_actions.effects.iter().count(), 1);
}

#[test]
fn binary_closed_owner_does_not_capture_a_methods_same_named_generic() {
    let environment = environment_from_source(
        r#"module library;
public spec Allowed;
impl string ~ Allowed;
public type Container<T ~ Allowed> = { value: T };
public alias Packet = Container<string>;
impl Packet {
    public flow copy(self) -> Packet { return self; }
    public flow choose<T>(self, input: T) -> T { return input; }
}
public flow make() -> Packet { return Container<string> { value = "closed" }; }
"#,
        "alternate",
    );
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import alternate.library.make;
flow main() -> i32 { return make().copy().choose<i32>(8); }
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    assert_eq!(checked.types.checked_method_calls.len(), 2);
    for call in checked.types.checked_method_calls.values() {
        assert!(call.owner_bindings.is_empty());
        assert!(matches!(
            call.target,
            etas_types::method::MethodTarget::External { .. }
        ));
    }
}

fn consumer(environment: ProjectEnvironmentInput) -> etas_frontend::ProjectOutput {
    let mut source = SourceInput::anonymous(
        r#"module consumer;
import library.{Encoder as Codec, make as packet};
flow main() -> i32 { let value = packet(); return value::Codec.encode(); }
"#,
    );
    source.id = etas_core::SourceId(72);
    let mut input = ProjectInput::single_source(source);
    input.environment = environment;
    Frontend.check_project(input)
}

#[test]
fn method_trace_contract_survives_source_binary_driver_and_aliased_consumer() {
    let environment = environment();
    let metadata = &environment.external_public_metadata[0];
    let CallableContractTarget::Method(identity) = &metadata.trace_spec_conformances[0].item else {
        panic!("method contract must retain MethodId");
    };
    assert_eq!(identity, &metadata.spec_impls[0].methods[0]);
    assert_eq!(
        identity.implementation().subject().declaration(),
        ["Container"]
    );
    let output = consumer(environment);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    assert_eq!(checked.sources.sources.len(), 1);
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty());
    assert_eq!(
        summary.requested_actions.effects.iter().count(),
        1,
        "{summary:?}"
    );
    assert!(
        matches!(
            summary.action_trace,
            etas_effects::ActionTraceDomain::Event(_)
        ),
        "{summary:?}"
    );
}

#[test]
fn binary_method_table_replays_associated_inherent_and_default_spec_chain_without_source() {
    for root in ["library", "alternate"] {
        let environment = environment_with_root(root);
        let path = if root == "library" {
            "library"
        } else {
            "alternate.library"
        };
        let mut input = ProjectInput::single_source(SourceInput::anonymous(format!(
            "module consumer; import {path}.Packet; flow main() -> i32 {{ return Packet.new(value = \"input\").copy().encode(); }}"
        )));
        input.environment = environment;
        let output = Frontend.check_project(input);
        assert!(
            output.diagnostics.is_empty(),
            "{root}: {:?}",
            output.diagnostics
        );
        let checked = output.checked.unwrap();
        assert_eq!(
            checked.sources.sources.len(),
            1,
            "metadata-only checking must not read dependency bodies"
        );
        assert_eq!(checked.types.checked_method_calls.len(), 3);
        assert!(
            checked
                .types
                .checked_method_calls
                .values()
                .all(|fact| matches!(
                    fact.target,
                    etas_types::method::MethodTarget::External { .. }
                ))
        );
        let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
        assert!(summary.escaping_effects.is_empty());
        assert_eq!(
            summary.requested_actions.effects.iter().count(),
            1,
            "{summary:?}"
        );
        assert!(
            matches!(
                summary.action_trace,
                etas_effects::ActionTraceDomain::Event(_)
            ),
            "{summary:?}"
        );
    }
}

#[test]
fn replay_checks_method_trace_contract_against_owned_behavior_not_just_public_row() {
    let mut environment = environment();
    let summary = &mut environment.external_public_metadata[0].trace_spec_summaries[0];
    summary.alternatives[0][0].kind = etas_frontend::ProjectExternalTraceSpecClauseKindInput::Deny;
    let output = consumer(environment);
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(etas_core::EffectDiagnosticCode::TraceSpecDenied)),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn unowned_method_contract_fails_at_real_consumer_import() {
    let mut environment = environment();
    let CallableContractTarget::Method(identity) =
        &mut environment.external_public_metadata[0].trace_spec_conformances[0].item
    else {
        panic!("method target")
    };
    *identity =
        etas_code::MethodId::new(identity.implementation().clone(), "not_encode".into()).unwrap();
    let output = consumer(environment);
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                )
        })
        .expect("malformed contract must fail closed");
    assert!(
        diagnostic
            .message
            .contains("published implementation owner"),
        "{diagnostic:?}"
    );
    assert_eq!(diagnostic.primary.span.source, etas_core::SourceId(72));
    assert!(
        diagnostic.primary.span.range.start.0 > 0,
        "must point at import, not an invented zero span"
    );
}

#[test]
fn remapped_dependency_import_root_does_not_change_method_contract_identity() {
    let original = environment();
    let remapped = environment_with_root("alternate");
    assert_eq!(
        original.external_public_metadata[0].trace_spec_conformances[0].item,
        remapped.external_public_metadata[0].trace_spec_conformances[0].item
    );
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import alternate.library.{Encoder as Codec, make as packet};
flow main() -> i32 { let value = packet(); return value::Codec.encode(); }
"#,
    ));
    input.environment = remapped;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.checked.unwrap().sources.sources.len(), 1);
}
