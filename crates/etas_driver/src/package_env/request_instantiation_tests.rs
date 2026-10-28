use super::frontend_environment;
use etas_frontend::{EntryPolicy, Frontend, ProjectInput, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};

struct Project(std::path::PathBuf);
impl Drop for Project {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn binary_request_template_with_an_undeclared_generic_is_rejected_at_its_import() {
    let producer = r#"module requests;
import std.compiler.{AgentDef, CompileError, compile};
public flow build<A>(definition: AgentDef<A>) -> A ![Error<CompileError>] { return compile(definition); }
"#;
    let mut input = ProjectInput::single_source(SourceInput::anonymous(producer));
    input.options.entry_policy = EntryPolicy::Optional;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let root = Project(std::env::temp_dir().join(format!(
        "etas-malformed-request-template-{}",
        std::process::id()
    )));
    std::fs::create_dir_all(root.0.join("src")).unwrap();
    std::fs::write(root.0.join("src/main.es"), producer).unwrap();
    std::fs::write(
        root.0.join("etas.toml"),
        "[package]\nname='requests'\nversion='0.1.0'\nedition='2026'\n",
    )
    .unwrap();
    let source_hash = etas_package::package_source_payload_hash(&root.0).unwrap();
    let artifact = etas_frontend::build_package_metadata_artifact_from_checked(
        etas_frontend::PackageMetadataBuildInput {
            definition_origin: None,
            package_id: "requests".into(),
            package_version: "0.1.0".into(),
            package_edition: "2026".into(),
            compiler_version: "test".into(),
            source_payload_hash: source_hash.clone(),
            manifest_hash: etas_package::package_manifest_hash(&root.0).unwrap(),
            dependency_lock_hash: etas_package::package_dependency_lock_hash(&root.0).unwrap(),
            bins: vec![],
            dependencies: vec![],
            tool_bindings: vec![],
        },
        output.checked.as_ref().unwrap(),
    )
    .unwrap();
    let path = etas_package::package_metadata_artifact_path(&root.0);
    let (header, mut metadata) =
        etas_package_metadata::package_metadata_from_artifact(&path, &artifact.bytes).unwrap();
    let summary = metadata
        .public_metadata
        .effect_summaries
        .iter_mut()
        .find(|summary| summary.item == ["requests", "build"])
        .unwrap();
    let bindings = summary.handler_requests.as_mut().unwrap().actions[0]
        .type_bindings
        .as_mut()
        .unwrap();
    assert_eq!(bindings[0].ty.kind, etas_package_metadata::TypeKind::Var);
    bindings[0].ty.name = "Undeclared".into();
    // The binary is structurally valid. Generic scope is checked by the
    // frontend environment boundary, after package and driver projection.
    let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
    let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    let package = etas_package::read_package_metadata_artifact(&root.0)
        .unwrap()
        .unwrap();
    let environment = frontend_environment(PackageEnvironmentMetadata {
        dependencies: vec![ResolvedDependency {
            identity: package.package,
            import_root: "requests".into(),
            source: ResolvedDependencySource::Path {
                path: root.0.to_string_lossy().into_owned(),
                checksum: source_hash,
            },
            dependencies: package.dependencies,
            public_metadata: package.public_metadata,
            effect_metadata: package.effect_metadata,
            tool_bindings: package.tool_bindings,
        }],
        ..Default::default()
    })
    .unwrap();
    let consumer = "module consumer;\nimport requests.build;\nagent Worker;\nflow main() -> unit { build(^Worker)?; return; }";
    let mut source = SourceInput::anonymous(consumer);
    source.id = etas_core::SourceId(509);
    let mut input = ProjectInput::single_source(source);
    input.environment = environment;
    let result = Frontend.check_project(input);
    assert!(result.checked.is_none());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                )
        })
        .unwrap();
    assert!(
        diagnostic.message.contains("Undeclared"),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic.primary.span.source, etas_core::SourceId(509));
    let span = diagnostic.primary.span.range;
    assert_eq!(
        &consumer[span.start.0 as usize..span.end.0 as usize],
        "requests.build"
    );
}
