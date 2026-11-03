use super::*;

#[test]
fn behavioral_self_malformed_binary_evidence_is_rejected_at_the_real_import() {
    for corruption in ["undeclared-type", "unknown-method", "generic-kind"] {
        let workspace = origin_metadata::workspace_with_source(
            r#"
module library.api;
public spec Encoder { flow encode<T>(self, value: T) -> T ![]; }
public flow forward<R ~ Encoder>(receiver: R) -> i32 ![] { return receiver::Encoder.encode(7); }
public flow lookup() -> i32 { return 7; }
"#,
        );
        let initial = origin_metadata::consumer(&workspace, "dep");
        let root = initial.package_root.unwrap();
        let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
        let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
            &artifact,
            &std::fs::read(&artifact).unwrap(),
        )
        .unwrap();
        let summary = metadata
            .public_metadata
            .effect_summaries
            .iter_mut()
            .find(|summary| summary.item == ["library", "api", "forward"])
            .unwrap();
        let etas_package_metadata::ActionTrace::ParameterCall {
            parameter: etas_package_metadata::CallableRequestTarget::SpecMethod(method),
        } = &mut summary.action_trace
        else {
            panic!("checked method evidence")
        };
        match corruption {
            "undeclared-type" => method.receiver.name = "Undeclared".into(),
            "unknown-method" => method.method = "absent".into(),
            "generic-kind" => {
                method.generic_arguments[0] =
                    etas_package_metadata::SpecMethodGenericArgument::EffectRow(Default::default())
            }
            _ => unreachable!(),
        }
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
            etas_driver::LoadProjectRequest {
                paths: vec![root],
                all: true,
                entry_policy: EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap();
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none(), "accepted {corruption}");
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code
                    == etas_core::DiagnosticCode::Type(
                        etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                    )
            })
            .unwrap_or_else(|| panic!("{corruption}: {:?}", output.diagnostics));
        let source = &loaded.loaded.input.sources[0];
        assert_eq!(diagnostic.primary.span.source, source.id, "{corruption}");
        let range = diagnostic.primary.span.range;
        assert!(
            source.text[range.start.0 as usize..range.end.0 as usize].contains("dep.library.api"),
            "{corruption}: {diagnostic:?}"
        );
    }
}
