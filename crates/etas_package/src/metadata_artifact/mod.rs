mod checksum;
mod container;
mod convert;
mod encode;
#[cfg(any(test, feature = "test-support"))]
mod fixture;
mod header;

pub use container::{
    metadata_artifact_hash, package_dependency_lock_hash, package_manifest_hash,
    package_metadata_artifact_path, package_source_payload_hash, read_package_metadata_artifact,
    read_package_metadata_artifact_info,
};
pub use encode::resolved_dependency_metadata;
#[cfg(any(test, feature = "test-support"))]
pub use fixture::write_fixture_package_metadata_artifact;
pub use header::{MetadataArtifactHeader, MetadataArtifactInfo, MetadataSectionKind};

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::metadata::{
        PackageActionTraceMetadata, PackageEffectRefMetadata, PackageEffectRowMetadata,
        PackageEffectSummaryMetadata, PackageFlowSignatureMetadata, PackageIdentity, PackageIndex,
        PackageLatentFlowSummaryMetadata, PackagePublicMetadata, PackageTypeMetadata,
    };

    use super::*;

    #[test]
    fn selector_bound_metadata_survives_binary_and_package_conversion() {
        let mut metadata = etas_package_metadata::PackageMetadata {
            version: 1,
            package: etas_package_metadata::PackageIdentity {
                name: "dependency".into(),
                version: "1.0.0".into(),
                edition: "2026".into(),
            },
            ..Default::default()
        };
        metadata.public_metadata.trace_spec_summaries =
            vec![etas_package_metadata::TraceSpecSummary {
                trace_spec: vec!["dependency".into(), "Safe".into()],
                definition: etas_code::DefinitionKey::new(
                    etas_code::PackageKey::from_canonical_fingerprint(
                        etas_code::CodeFingerprint::from_bytes([7; 32]),
                    ),
                    vec!["dependency".into()],
                    vec!["Safe".into()],
                    etas_code::DefinitionKind::Spec,
                )
                .unwrap(),
                alternatives: vec![vec![etas_package_metadata::TraceSpecClause {
                    pattern: Some(etas_package_metadata::EffectRow {
                        effects: vec![etas_package_metadata::EffectRef {
                            path: vec!["Fs".into(), "read".into()],
                            args: vec![etas_package_metadata::EffectArg::default()],
                        }],
                        tail: None,
                    }),
                    selector_bounds: vec![etas_package_metadata::TraceSelectorBound {
                        binding: "R".into(),
                        effect_path: vec!["Fs".into(), "read".into()],
                        spec_path: vec!["dependency".into(), "Within".into()],
                        spec_args: vec![etas_package_metadata::Type {
                            kind: etas_package_metadata::TypeKind::Primitive,
                            name: "string".into(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }]],
            }];
        let path = std::path::Path::new("dependency/package.etasmeta");
        let header = etas_package_metadata::MetadataArtifactHeader {
            artifact_schema_version: etas_package_metadata::ARTIFACT_SCHEMA_VERSION,
            compiler_version: "test".into(),
            package_id: "dependency".into(),
            package_version: "1.0.0".into(),
            source_payload_hash: "source".into(),
            manifest_hash: "manifest".into(),
            dependency_lock_hash: "lock".into(),
            created_target: "test".into(),
        };
        let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
        let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
        let (_, decoded) =
            etas_package_metadata::package_metadata_from_artifact(path, &bytes).unwrap();
        let index = super::convert::package_index_from_metadata(decoded, path).unwrap();
        let bound =
            &index.public_metadata.trace_spec_summaries[0].alternatives[0][0].selector_bounds[0];
        assert_eq!(bound.binding, "R");
        assert_eq!(
            bound.spec_args,
            [PackageTypeMetadata::Primitive {
                name: "string".into()
            }]
        );
        assert_eq!(
            super::encode::package_index_to_metadata(&index).unwrap(),
            metadata
        );
    }

    #[test]
    fn package_metadata_artifact_round_trips_complete_builtin_std_dependency() {
        let root = temp_dir("builtin-std-selector-roundtrip");
        fs::write(
            root.join("etas.toml"),
            "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
        )
        .unwrap();
        let index = PackageIndex {
            version: 1,
            package: PackageIdentity {
                name: "consumer".into(),
                version: "0.1.0".into(),
                edition: "2026".into(),
            },
            dependencies: vec![crate::metadata::builtin_std_dependency(
                "0.1".into(),
                "2026".into(),
            )],
            external_modules: Vec::new(),
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
            bins: Vec::new(),
        };
        write_fixture_package_metadata_artifact(&root, &index).unwrap();
        let decoded = read_package_metadata_artifact(&root).unwrap().unwrap();
        assert_eq!(decoded, index);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn package_metadata_artifact_round_trips_package_index() {
        let reference = etas_code::residual::identity::ResidualObligationRef::new(
            etas_code::residual::identity::ObligationId::from_canonical_fingerprint(
                etas_code::CodeFingerprint::from_bytes([42; 32]),
            ),
            Default::default(),
        )
        .unwrap();
        let root = temp_dir("metadata-artifact-roundtrip");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "artifact"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(root.join("src").join("lib.es"), "module artifact;\n").unwrap();
        let index = PackageIndex {
            version: 1,
            package: PackageIdentity {
                name: "artifact".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            dependencies: Vec::new(),
            external_modules: Vec::new(),
            public_metadata: PackagePublicMetadata {
                flows: vec![PackageFlowSignatureMetadata {
                    generic_params: Vec::new(),
                    path: vec!["artifact".to_owned(), "decode".to_owned()],
                    param_names: vec!["value".to_owned()],
                    params: vec![PackageTypeMetadata::Trust {
                        wrapper: "Untrusted".to_owned(),
                        inner: Box::new(PackageTypeMetadata::Primitive {
                            name: "string".to_owned(),
                        }),
                    }],
                    output: PackageTypeMetadata::Primitive {
                        name: "string".to_owned(),
                    },
                    effects: None,
                    visibility: "public".to_owned(),
                }],
                effect_summaries: vec![PackageEffectSummaryMetadata {
                    residual_checks: vec![reference.clone()],
                    behavior: None,
                    handler_requests: None,
                    returned_callables: None,
                    default_handler_effects: None,
                    item: vec!["artifact".to_owned(), "apply".to_owned()],
                    public_effects: PackageEffectRowMetadata {
                        effects: vec![PackageEffectRefMetadata {
                            path: vec!["Network".to_owned()],
                            args: Vec::new(),
                        }],
                        tail: None,
                    },
                    requested_actions: PackageEffectRowMetadata::default(),
                    handled_requested_actions: PackageEffectRowMetadata::default(),
                    latent_flows: vec![PackageLatentFlowSummaryMetadata {
                        residual_checks: vec![reference],
                        declared_bound: PackageEffectRowMetadata {
                            effects: vec![PackageEffectRefMetadata {
                                path: vec!["Network".to_owned()],
                                args: Vec::new(),
                            }],
                            tail: None,
                        },
                        inferred_effects: PackageEffectRowMetadata {
                            effects: vec![PackageEffectRefMetadata {
                                path: vec!["Network".to_owned()],
                                args: Vec::new(),
                            }],
                            tail: None,
                        },
                    }],
                    action_trace: PackageActionTraceMetadata::Empty,
                }],
                ..Default::default()
            },
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
            bins: Vec::new(),
        };

        let info = write_fixture_package_metadata_artifact(&root, &index).unwrap();
        assert!(info.artifact_hash.starts_with("blake3:"));
        assert!(package_metadata_artifact_path(&root).is_file());
        let decoded = read_package_metadata_artifact(&root).unwrap().unwrap();
        assert_eq!(decoded, index);
    }

    #[test]
    fn package_metadata_artifact_rejects_trailing_bytes() {
        let root = temp_dir("metadata-artifact-trailing-bytes");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "artifact"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(root.join("src").join("lib.es"), "module artifact;\n").unwrap();
        let index = PackageIndex {
            version: 1,
            package: PackageIdentity {
                name: "artifact".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            dependencies: Vec::new(),
            external_modules: Vec::new(),
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
            bins: Vec::new(),
        };
        write_fixture_package_metadata_artifact(&root, &index).unwrap();
        let path = package_metadata_artifact_path(&root);
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(0);
        fs::write(&path, bytes).unwrap();

        let error = read_package_metadata_artifact(&root).unwrap_err();

        assert!(error.to_string().contains("undeclared trailing byte"));
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("etas-package-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
