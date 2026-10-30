use super::*;

const LIBRARY: &str = r#"
module library.api;
public spec Encoder { flow encode(self) -> i32 ![]; }
public type First = { value: i32 };
public type Second = { value: i32 };
impl First ~ Encoder { flow encode(self) -> i32 ![] { return self.value; } }
impl Second ~ Encoder { flow encode(self) -> i32 ![] { return self.value + 1; } }
public flow first(value: First) -> i32 ![] { return value::Encoder.encode(); }
public flow second(value: Second) -> i32 ![] { return value::Encoder.encode(); }
public flow lookup() -> i32 ![] { return 7; }
"#;

#[test]
fn behavioral_self_type_method_descriptors_survive_real_package_loading_with_two_import_roots() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let path = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let (_, metadata) = etas_package_metadata::package_metadata_from_artifact(
        &path,
        &std::fs::read(&path).unwrap(),
    )
    .unwrap();
    let evidence = metadata
        .public_metadata
        .spec_impls
        .iter()
        .flat_map(|implementation| &implementation.methods)
        .map(|method| {
            method.callable.validate_references(Some((0, &[]))).unwrap();
            let root = method.root().unwrap();
            assert_eq!(root.params.len(), 1);
            assert_eq!(root.output.name, "i32");
            (method.identity.clone(), root.params[0].path.clone())
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(evidence.len(), 2);
    let methods = metadata
        .public_metadata
        .definition_descriptors
        .unwrap()
        .headers
        .unwrap()
        .callables
        .into_iter()
        .filter(|callable| callable.id.as_method().is_some())
        .map(|callable| callable.id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(methods.len(), 2);
    for alias in ["first_import", "second_import"] {
        let mut loaded = origin_metadata::consumer(&workspace, alias);
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import {alias}.library.api.{{First as Remote, first}};
flow main() -> i32 ![] {{ return first(Remote {{ value = 7 }}); }}
"#
        );
        let checked = origin_metadata::check(&loaded);
        assert_eq!(checked.sources.sources.len(), 1, "metadata-only consumer");
        let imported = checked
            .types
            .external_specs
            .methods
            .iter()
            .map(|method| method.identity.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(imported, evidence.keys().cloned().collect());
        let actual = checked
            .external_definition_origins()
            .iter()
            .filter_map(|origin| origin.descriptors.as_ref())
            .flat_map(|set| set.headers.iter().flat_map(|headers| &headers.callables))
            .filter(|header| header.id.as_method().is_some())
            .map(|header| header.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actual, methods,
            "import roots must not rewrite canonical method identities"
        );
        let definitions =
            prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
        for id in &methods {
            assert!(definitions.catalog().item_owner(id).is_none());
        }
    }
}

#[test]
fn behavioral_self_impl_evidence_replays_generic_method_signatures_without_source() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Echo { flow echo<T>(self, value: T) -> T ![]; }
public type Packet;
impl Packet ~ Echo { flow echo<U>(self, value: U) -> U ![] { return value; } }
public flow lookup() -> i32 ![] { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "renamed");
    loaded.loaded.input.sources[0].text = "module app.main; import renamed.library.api.{Packet, Echo, lookup}; flow main() -> i32 ![] { return lookup(); }".into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let methods = &checked.types.external_specs.methods;
    assert_eq!(methods.len(), 1);
    let method = &methods[0];
    assert_eq!(method.identity.name(), "echo");
    assert_eq!(method.signature.generic_params.len(), 1);
    assert_eq!(method.signature.generic_params[0].name, "U");
    assert_eq!(method.signature.params[0], method.self_type);
    assert_eq!(method.signature.params[1], method.signature.output);
    assert_eq!(
        method.signature.output,
        method.signature.generic_params[0].subject
    );
}

#[test]
fn behavioral_self_impl_evidence_is_rejected_by_encoder_and_at_import_span() {
    for change in [0, 1, 2] {
        let workspace = origin_metadata::workspace_with_source(LIBRARY);
        let initial = origin_metadata::consumer(&workspace, "dep");
        let root = initial.package_root.clone().unwrap();
        let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
        let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
            &artifact,
            &std::fs::read(&artifact).unwrap(),
        )
        .unwrap();
        let first = metadata.public_metadata.spec_impls[0].methods[0]
            .identity
            .clone();
        let method = &mut metadata.public_metadata.spec_impls[1].methods[0];
        let original_identity = method.identity.clone();
        if change == 0 {
            method.identity = first.clone();
        } else {
            let etas_package_metadata::CallableValueSource::Value(index) =
                method.callable.sources[0]
            else {
                panic!("owned method root");
            };
            if change == 1 {
                method.callable.values[index as usize].output = etas_package_metadata::Type {
                    kind: etas_package_metadata::TypeKind::Primitive,
                    name: "bool".into(),
                    ..Default::default()
                };
            } else {
                method.callable.values[index as usize]
                    .public_effects
                    .effects
                    .push(etas_package_metadata::EffectRef {
                        path: vec![
                            "std".into(),
                            "effects".into(),
                            "Console".into(),
                            "stdout_write".into(),
                        ],
                        args: Vec::new(),
                    });
            }
        }
        let loaded = if change == 0 {
            let error = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("conflicting published behavior graphs"),
                "{error}"
            );
            // A public environment can also arrive from an embedding, without
            // passing through our encoder. Keep its import-boundary rejection.
            let mut loaded = initial;
            let method = loaded
                .loaded
                .input
                .environment
                .external_public_metadata
                .iter_mut()
                .flat_map(|metadata| &mut metadata.spec_impls)
                .flat_map(|implementation| &mut implementation.methods)
                .find(|method| method.identity == original_identity)
                .expect("loaded binary method evidence");
            method.identity = first;
            loaded
        } else {
            let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
            let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
            etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
            etas_package::materialize_package(etas_package::MaterializePackageOptions {
                package_root: root.clone(),
                source_config: Some(
                    etas_package::PackageSourceConfig::read_project(&root).unwrap(),
                ),
            })
            .unwrap();
            etas_package::lock_package(etas_package::LockPackageOptions {
                package_root: root.clone(),
            })
            .unwrap();
            etas_driver::load_project(
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
            .unwrap()
        };
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none());
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code
                    == if change == 2 {
                        etas_core::DiagnosticCode::Effect(
                            etas_core::EffectDiagnosticCode::EffectOutsideDeclaredRow,
                        )
                    } else {
                        etas_core::DiagnosticCode::Type(
                            etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                        )
                    }
            })
            .expect("binary method evidence must be rejected at the import boundary");
        assert!(
            diagnostic.message.contains(if change == 0 {
                "method identity"
            } else if change == 1 {
                "output type differs"
            } else {
                "escaping effects exceed"
            }),
            "{:?}",
            output.diagnostics
        );
        let source = &loaded.loaded.input.sources[0];
        assert_eq!(diagnostic.primary.span.source, source.id);
        let range = diagnostic.primary.span.range;
        assert!(
            source.text[range.start.0 as usize..range.end.0 as usize].contains("dep.library.api"),
            "case {change}, anchor {:?}: {:?}",
            range,
            &source.text[range.start.0 as usize..range.end.0 as usize]
        );
    }
}

fn runtime_consumer(source: &str) -> (origin_metadata::Workspace, std::path::PathBuf) {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let root = origin_metadata::consumer(&workspace, "dep")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), source).unwrap();
    (workspace, root)
}

#[test]
fn behavioral_self_type_method_identity_survives_dependency_source_materialization() {
    let (_workspace, root) = runtime_consumer(
        r#"
module app.main;
import dep.library.api.{First, Second, first, second};
flow main() -> i32 ![] {
    return first(First { value = 7 }) + second(Second { value = 9 });
}
"#,
    );
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 2, 17);
}

#[test]
fn behavioral_self_direct_imported_impl_materializes_its_selected_module() {
    let (_workspace, root) = runtime_consumer(
        r#"
module app.main;
import dep.library.api.{First, Second, Encoder};
flow main() -> i32 ![] {
    return First { value = 7 }::Encoder.encode() + Second { value = 9 }::Encoder.encode();
}
"#,
    );
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 2, 17);
}

#[test]
fn behavioral_self_type_method_identity_survives_native_agent_revision_compilation() {
    let (_workspace, root) = runtime_consumer(
        r#"
module app.main;
import dep.library.api.{First, Second, first, second};
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: First) -> i32 ![] { return first(value); }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return Worker(value);") {
        Ok(body) => body,
        Err(_) => { return -1; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised(First { value = 7 }) + second(Second { value = 9 });
}
"#,
    );
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 2, 17);
}
