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
        PackageEffectRefMetadata, PackageEffectRowMetadata, PackageEffectSummaryMetadata,
        PackageFlowSignatureMetadata, PackageIdentity, PackageIndex,
        PackageLatentFlowSummaryMetadata, PackagePublicMetadata, PackageTypeMetadata,
    };

    use super::*;

    #[test]
    fn package_metadata_artifact_round_trips_package_index() {
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
                    item: vec!["artifact".to_owned(), "apply".to_owned()],
                    public_effects: PackageEffectRowMetadata {
                        effects: vec![PackageEffectRefMetadata {
                            path: vec!["Network".to_owned()],
                            args: Vec::new(),
                        }],
                    },
                    requested_actions: PackageEffectRowMetadata::default(),
                    handled_requested_actions: PackageEffectRowMetadata::default(),
                    latent_flows: vec![PackageLatentFlowSummaryMetadata {
                        declared_bound: PackageEffectRowMetadata {
                            effects: vec![PackageEffectRefMetadata {
                                path: vec!["Network".to_owned()],
                                args: Vec::new(),
                            }],
                        },
                        inferred_effects: PackageEffectRowMetadata {
                            effects: vec![PackageEffectRefMetadata {
                                path: vec!["Network".to_owned()],
                                args: Vec::new(),
                            }],
                        },
                    }],
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
