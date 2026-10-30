use etas_frontend::{EntryPolicy, Frontend, ProjectEnvironmentInput, ProjectInput, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};
use etas_package_metadata::CallableContractTarget;

const SOURCE: &str = r#"module library;
import std.io.println;
public type Container<T> = { value: T };
public alias Packet = Container<string>;
public spec Encoder { flow encode(self) -> i32 ![]; }
public spec Safe: trace = +Console.stdout_write;
impl Packet ~ Encoder {
    flow encode(self) -> i32 ![] ~ Safe { println("encoded")?; return 42; }
}
public flow make() -> Packet { return Container<string> { value = "body" }; }
"#;

fn environment() -> ProjectEnvironmentInput {
    environment_with_root("library")
}

fn environment_with_root(import_root: &str) -> ProjectEnvironmentInput {
    let mut input = ProjectInput::single_source(SourceInput::anonymous(SOURCE));
    input.options.entry_policy = EntryPolicy::Optional;
    let checked = Frontend.check_project(input);
    assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
    let fixture = super::callback_replay_tests::PackageFixture::named("library", SOURCE);
    let source_hash = etas_package::package_source_payload_hash(&fixture.0).unwrap();
    let artifact = etas_frontend::build_package_metadata_artifact_from_checked(
        etas_frontend::PackageMetadataBuildInput {
            definition_origin: Some(etas_code::PackageKey::from_canonical_fingerprint(
                etas_code::CodeFingerprint::of_bytes(
                    "etas.test.method-contract-package",
                    b"library",
                ),
            )),
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
    assert_eq!(identity, &metadata.spec_impls[0].methods[0].identity);
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
fn replay_checks_method_trace_contract_against_owned_behavior_not_just_public_row() {
    let mut environment = environment();
    let summary = &mut environment.external_public_metadata[0].trace_spec_summaries[0];
    summary.clauses[0].kind = etas_frontend::ProjectExternalTraceSpecClauseKindInput::Deny;
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
