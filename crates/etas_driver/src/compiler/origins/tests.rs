use super::*;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::provenance::dependency_id;
use etas_core::{SourceFile, SourceId};
use etas_frontend::{EntryPolicy, Frontend, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};

use crate::{
    DriverCacheMode, DriverOptions, DriverProjectMode, LoadProjectRequest, SourceDependencyMode,
    load_project,
};

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "etas-definition-origins-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn package(&self, directory: &str, name: &str, module: &str) -> LoadedDriverProject {
        let root = self.0.join(directory);
        let source = root
            .join("src")
            .join(module.replace('.', "/"))
            .with_extension("es");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(
            root.join("etas.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
        )
        .unwrap();
        std::fs::write(
            source,
            format!("module {module}; public agent Worker() -> i32 {{ return 1; }}"),
        )
        .unwrap();
        load(&root)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn load(path: &Path) -> LoadedDriverProject {
    load_project(
        &DriverOptions {
            workspace: Some(if path.is_dir() {
                path.to_owned()
            } else {
                path.parent().unwrap().to_owned()
            }),
            cache_mode: DriverCacheMode::Off,
            ..Default::default()
        },
        LoadProjectRequest {
            paths: vec![path.to_owned()],
            all: path.is_dir(),
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: SourceDependencyMode::MetadataOnly,
        },
    )
    .unwrap()
}

fn capture(loaded: &LoadedDriverProject) -> Result<SourceDefinitionProvenance, DriverError> {
    capture_source_definition_provenance(
        loaded.mode,
        loaded.package_root.as_deref(),
        loaded.source_definition_provenance.single_file_root(),
        loaded.package_environment.as_ref(),
        &loaded.loaded.input,
    )
}

fn refresh(loaded: &mut LoadedDriverProject) {
    loaded.source_definition_provenance = capture(loaded).unwrap();
    loaded.loaded.input.environment.local_source_package =
        loaded.source_definition_provenance.local_package_key();
    loaded.loaded.input.environment.dependency_source_packages = loaded
        .source_definition_provenance
        .dependency_source_packages()
        .into_iter()
        .filter(|origin| {
            loaded
                .loaded
                .input
                .environment
                .external_packages
                .iter()
                .any(|package| package.id == origin.package)
        })
        .collect();
}

fn check(loaded: &LoadedDriverProject) -> etas_frontend::CheckedProgram {
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    output.checked.unwrap()
}

fn overlay(consumer: &mut LoadedDriverProject, library: &LoadedDriverProject, alias: &str) {
    let identity = library
        .package_environment
        .as_ref()
        .unwrap()
        .current_package
        .clone()
        .unwrap();
    let dependency = ResolvedDependency {
        identity: identity.clone(),
        std_compatibility: etas_std::standard_registry()
            .version()
            .compatibility_identity(),
        import_root: alias.into(),
        source: ResolvedDependencySource::Path {
            path: library
                .package_root
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
            checksum: "locked-source-revision".into(),
        },
        dependencies: vec![],
        public_metadata: Default::default(),
        effect_metadata: Default::default(),
        tool_bindings: vec![],
    };
    let id = dependency_id(&dependency);
    consumer
        .package_environment
        .as_mut()
        .unwrap()
        .dependencies
        .push(dependency);
    consumer.loaded.input.environment.external_packages.push(
        etas_frontend::ProjectExternalPackageInput {
            id,
            name: identity.name,
            version: identity.version,
            edition: identity.edition,
            import_root: alias.into(),
            std_compatibility: etas_std::standard_registry()
                .version()
                .compatibility_identity(),
        },
    );
    let root = library.loaded.input.source_root.as_ref().unwrap();
    let source_root = consumer.loaded.input.source_root.as_ref().unwrap();
    let mut next = consumer
        .loaded
        .input
        .sources
        .iter()
        .map(|source| source.id.0)
        .max()
        .unwrap()
        + 1;
    for source in &library.loaded.input.sources {
        let path = source_root.join(source.path.as_ref().unwrap().strip_prefix(root).unwrap());
        let source = SourceInput {
            id: SourceId(next),
            path: Some(path.clone()),
            text: source.text.clone(),
            kind: SourceKind::DependencySourceOverlay {
                package: id,
                import_root: alias.into(),
            },
        };
        consumer.loaded.sources.push((
            path.clone(),
            SourceFile::new(source.id, Some(path), source.text.clone()),
        ));
        consumer.loaded.input.sources.push(source);
        next += 1;
    }
    refresh(consumer);
}

fn module_origin(loaded: &LoadedDriverProject, module: &[&str]) -> SourceModuleOrigin {
    let checked = check(loaded);
    source_definition_origins(loaded, &checked)
        .unwrap()
        .into_values()
        .find(|origin| {
            origin
                .module()
                .iter()
                .map(String::as_str)
                .eq(module.iter().copied())
        })
        .unwrap()
}

#[test]
fn dependency_source_identity_matches_its_owner_and_does_not_depend_on_import_alias() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "worker-library", "library.workers");
    let consumer = workspace.package("consumer", "consumer-app", "app.main");
    let original = module_origin(&library, &["library", "workers"]);
    let mut first = consumer.clone();
    overlay(&mut first, &library, "first_alias");
    let mut second = consumer;
    overlay(&mut second, &library, "second_alias");
    assert_eq!(module_origin(&first, &["library", "workers"]), original);
    assert_eq!(module_origin(&second, &["library", "workers"]), original);
    assert_ne!(
        module_origin(&first, &["app", "main"]).package(),
        original.package()
    );
}

#[test]
fn local_package_origin_uses_name_and_version_not_checkout_or_content() {
    let workspace = Workspace::new();
    let mut first = workspace.package("first", "same-name", "app.main");
    let second = workspace.package("second", "same-name", "app.main");
    let before = module_origin(&first, &["app", "main"]);
    assert_eq!(
        first
            .loaded
            .input
            .environment
            .source_package(&first.loaded.input.sources[0]),
        Ok(before.package())
    );
    assert_eq!(
        before.package(),
        module_origin(&second, &["app", "main"]).package()
    );
    first.loaded.input.sources[0].text =
        "module app.main; public agent Worker() -> i32 { return 2; }".into();
    assert_eq!(before, module_origin(&first, &["app", "main"]));
    first
        .package_environment
        .as_mut()
        .unwrap()
        .current_package
        .as_mut()
        .unwrap()
        .version = "0.2.0".into();
    assert_eq!(before, module_origin(&first, &["app", "main"]));
    refresh(&mut first);
    assert_ne!(
        before.package(),
        module_origin(&first, &["app", "main"]).package()
    );
}

#[test]
fn package_sources_get_checkout_independent_stable_keys_before_analysis() {
    let workspace = Workspace::new();
    let first = workspace.package("first", "same-name", "app.main");
    let second = workspace.package("second", "same-name", "app.main");
    let first_source = &first.loaded.input.sources[0];
    let second_source = &second.loaded.input.sources[0];
    let first_key = first
        .loaded
        .input
        .environment
        .stable_source_key(first_source.id)
        .unwrap();
    let second_key = second
        .loaded
        .input
        .environment
        .stable_source_key(second_source.id)
        .unwrap();
    assert_eq!(first_key, second_key);
    assert_eq!(
        etas_code::SourceRevisionKey::new(first_key, first_source.text.as_bytes()),
        etas_code::SourceRevisionKey::new(second_key, second_source.text.as_bytes())
    );
    let other = workspace.package("other", "another-name", "app.main");
    assert_ne!(
        first_key,
        other
            .loaded
            .input
            .environment
            .stable_source_key(other.loaded.input.sources[0].id)
            .unwrap()
    );
}

#[test]
fn checked_trace_clause_identity_is_stable_across_package_checkouts() {
    let workspace = Workspace::new();
    let first = workspace.package("first", "same-name", "app.main");
    let second = workspace.package("second", "same-name", "app.main");
    let source = r#"module app.main;
spec Safe: trace = -Console.stdout_write;
flow run() -> unit ![] ~ Safe { return; }
"#;
    let identity = |loaded: LoadedDriverProject| {
        let root = loaded.package_root.unwrap();
        std::fs::write(root.join("src/app/main.es"), source).unwrap();
        let checked = check(&load(&root));
        let conformance = checked
            .effects
            .trace_specs
            .conformances
            .values()
            .flatten()
            .next()
            .unwrap();
        let clause = conformance.alternatives[0][0].source_clause().unwrap();
        (clause, conformance.source_spec_instance.clone().unwrap())
    };
    assert_eq!(identity(first), identity(second));
}

#[test]
fn missing_or_ambiguous_locked_dependency_origins_are_rejected() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let mut consumer = workspace.package("consumer", "app", "app.main");
    overlay(&mut consumer, &library, "workers");
    let checked = check(&consumer);
    let saved = consumer.package_environment.take().unwrap();
    assert!(source_definition_origins(&consumer, &checked).is_ok());
    assert!(
        capture(&consumer)
            .unwrap_err()
            .to_string()
            .contains("no prepared package identity")
    );
    consumer.package_environment = Some(PackageEnvironmentMetadata {
        dependencies: vec![],
        ..saved.clone()
    });
    refresh(&mut consumer);
    assert!(
        source_definition_origins(&consumer, &checked)
            .unwrap_err()
            .to_string()
            .contains("checked environment differs")
    );
    consumer.package_environment = Some(saved);
    let other = workspace.package("other", "workers", "library.workers");
    let dependencies = &mut consumer.package_environment.as_mut().unwrap().dependencies;
    let mut duplicate = dependencies.last().unwrap().clone();
    duplicate.source = ResolvedDependencySource::Path {
        path: other.package_root.unwrap().to_str().unwrap().into(),
        checksum: "another-locked-revision".into(),
    };
    dependencies.push(duplicate.clone());
    assert!(capture(&consumer).is_ok());
    duplicate.source = ResolvedDependencySource::Vendor {
        path: "vendor/other".into(),
        checksum: "vendor-checksum".into(),
        store: None,
        definition_origin: Some(etas_code::PackageKey::from_canonical_fingerprint(
            etas_code::CodeFingerprint::from_bytes([0x42; 32]),
        )),
    };
    *consumer
        .package_environment
        .as_mut()
        .unwrap()
        .dependencies
        .last_mut()
        .unwrap() = duplicate;
    assert!(
        capture(&consumer)
            .unwrap_err()
            .to_string()
            .contains("vendored definition origin disagrees")
    );
}

#[test]
fn dependency_identity_is_name_and_version_for_all_package_sources() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let mut consumer = workspace.package("consumer", "app", "app.main");
    overlay(&mut consumer, &library, "workers");
    let expected = module_origin(&library, &["library", "workers"]).package();
    let sources = [
        ResolvedDependencySource::Path {
            path: "relative/workers".into(),
            checksum: "source-checksum".into(),
        },
        ResolvedDependencySource::Registry {
            registry: "https://registry.example".into(),
            checksum: "registry-checksum".into(),
            store: None,
        },
        ResolvedDependencySource::Git {
            url: "https://git.example/workers".into(),
            rev: "revision".into(),
            checksum: "git-checksum".into(),
            store: None,
        },
        ResolvedDependencySource::GitHubClone {
            repo: "example/workers".into(),
            rev: "revision".into(),
            checksum: "clone-checksum".into(),
            store: None,
        },
        ResolvedDependencySource::GitHubRelease {
            repo: "example/workers".into(),
            release: "release".into(),
            asset: "archive".into(),
            asset_checksum: "asset-checksum".into(),
            payload_checksum: "payload-checksum".into(),
            store: None,
        },
        ResolvedDependencySource::Vendor {
            path: "vendor/workers".into(),
            checksum: "vendor-checksum".into(),
            store: None,
            definition_origin: Some(expected),
        },
    ];
    for source in sources {
        let dependency = &mut consumer.package_environment.as_mut().unwrap().dependencies[0];
        dependency.source = source;
        let id = dependency_id(dependency);
        let provenance = capture(&consumer).unwrap();
        assert_eq!(
            provenance.dependency_package_for(id, "workers").unwrap(),
            expected
        );
    }
}

#[test]
fn relative_path_dependency_under_remote_package_keeps_its_package_identity() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let mut consumer = workspace.package("consumer", "app", "app.main");
    overlay(&mut consumer, &library, "workers");
    let dependency = &mut consumer.package_environment.as_mut().unwrap().dependencies[0];
    let mut child = dependency.clone();
    child.identity.name = "helpers".into();
    child.import_root = "helpers".into();
    child.source = ResolvedDependencySource::Path {
        path: "../helpers".into(),
        checksum: "child-checksum".into(),
    };
    let expected =
        etas_code::PackageKey::from_name_and_version(&child.identity.name, &child.identity.version)
            .unwrap();
    let child_id = dependency_id(&child);
    dependency.source = ResolvedDependencySource::Git {
        url: "https://git.example/workers".into(),
        rev: "revision".into(),
        checksum: "git-checksum".into(),
        store: None,
    };
    dependency.dependencies.push(child);
    let provenance = capture(&consumer).unwrap();
    assert_eq!(
        provenance
            .dependency_package_for(child_id, "helpers")
            .unwrap(),
        expected
    );
}

#[test]
fn checked_programs_cannot_borrow_another_loaded_sources_package_identity() {
    let workspace = Workspace::new();
    let mut loaded = workspace.package("source", "app", "app.main");
    let checked = check(&loaded);
    loaded.loaded.input.sources[0]
        .text
        .push_str("\n// a different immutable source revision\n");
    assert!(
        source_definition_origins(&loaded, &checked)
            .unwrap_err()
            .to_string()
            .contains("differs from the prepared driver input")
    );
}

#[test]
fn checked_programs_cannot_borrow_another_loaded_environment() {
    let workspace = Workspace::new();
    let mut loaded = workspace.package("source", "app", "app.main");
    let checked = check(&loaded);
    assert!(source_definition_origins(&loaded, &checked).is_ok());
    loaded.loaded.input.environment.local_source_package =
        Some(etas_code::PackageKey::from_canonical_fingerprint(
            etas_code::CodeFingerprint::from_bytes([0x42; 32]),
        ));
    assert!(
        source_definition_origins(&loaded, &checked)
            .unwrap_err()
            .to_string()
            .contains("checked environment differs from the prepared driver input")
    );
}

#[test]
fn relative_source_dependency_origins_use_the_owning_original_package_directory() {
    let workspace = Workspace::new();
    let leaf = workspace.package("parent/leaf", "leaf", "library.leaf");
    let parent = workspace.package("parent", "parent", "library.parent");
    let mut consumer = workspace.package("consumer", "app", "app.main");
    overlay(&mut consumer, &leaf, "leaf");
    let dependencies = &mut consumer.package_environment.as_mut().unwrap().dependencies;
    let mut leaf_dependency = dependencies.pop().unwrap();
    leaf_dependency.source = ResolvedDependencySource::Path {
        path: "leaf".into(),
        checksum: "leaf-revision".into(),
    };
    dependencies.push(ResolvedDependency {
        identity: parent.package_environment.unwrap().current_package.unwrap(),
        std_compatibility: etas_std::standard_registry()
            .version()
            .compatibility_identity(),
        import_root: "parent".into(),
        source: ResolvedDependencySource::Path {
            path: "../parent".into(),
            checksum: "parent-revision".into(),
        },
        dependencies: vec![leaf_dependency],
        public_metadata: Default::default(),
        effect_metadata: Default::default(),
        tool_bindings: vec![],
    });
    refresh(&mut consumer);
    assert_eq!(
        module_origin(&consumer, &["library", "leaf"]),
        module_origin(&leaf, &["library", "leaf"])
    );
}

#[test]
fn source_project_and_single_file_origins_are_explicit_filesystem_origins() {
    let workspace = Workspace::new();
    let root = workspace.0.join("source");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("main.es"),
        "module main; agent Worker() -> i32 { return 1; }",
    )
    .unwrap();
    let loaded = load(&root);
    assert_eq!(loaded.mode, DriverProjectMode::SourceProject);
    let source = module_origin(&loaded, &["main"]);
    assert_eq!(
        loaded
            .loaded
            .input
            .environment
            .source_package(&loaded.loaded.input.sources[0]),
        Ok(source.package())
    );
    let single = load(&root.join("main.es"));
    assert_eq!(single.mode, DriverProjectMode::SingleFile);
    let first = module_origin(&single, &["main"]);
    assert_eq!(
        single
            .loaded
            .input
            .environment
            .source_package(&single.loaded.input.sources[0]),
        Ok(first.package())
    );
    assert_ne!(first.package(), source.package());
    let other = root.join("other.es");
    std::fs::write(&other, &loaded.loaded.input.sources[0].text).unwrap();
    let other = load(&other);
    assert_ne!(module_origin(&other, &["main"]).package(), first.package());
}

#[test]
fn captured_origins_survive_removal_of_the_original_package_and_source_paths() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let mut consumer = workspace.package("consumer", "app", "app.main");
    overlay(&mut consumer, &library, "workers");
    let checked = check(&consumer);
    let expected = source_definition_origins(&consumer, &checked).unwrap();
    std::fs::remove_dir_all(library.package_root.as_ref().unwrap()).unwrap();
    std::fs::remove_dir_all(consumer.package_root.as_ref().unwrap()).unwrap();
    assert_eq!(
        source_definition_origins(&consumer, &checked).unwrap(),
        expected
    );
}

#[test]
fn identical_text_from_a_different_source_path_does_not_prove_loaded_ownership() {
    let workspace = Workspace::new();
    let first = workspace.package("first", "app", "app.main");
    let second = workspace.package("second", "app", "app.main");
    let checked = check(&first);
    assert!(
        source_definition_origins(&second, &checked)
            .unwrap_err()
            .to_string()
            .contains("differs from the prepared driver input")
    );
}

#[test]
fn vendor_aliases_use_the_recorded_owner_not_the_mirror_or_store_directory() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let consumer = workspace.package("consumer", "app", "app.main");
    let expected = module_origin(&library, &["library", "workers"]);
    for alias in ["first_alias", "second_alias"] {
        let mut input = consumer.clone();
        overlay(&mut input, &library, alias);
        let dependency = input
            .package_environment
            .as_mut()
            .unwrap()
            .dependencies
            .last_mut()
            .unwrap();
        dependency.source = ResolvedDependencySource::Vendor {
            path: format!(".etas/vendor/{alias}"),
            checksum: "locked-payload".into(),
            store: Some(format!(".etas/store/{alias}")),
            definition_origin: Some(expected.package()),
        };
        refresh(&mut input);
        assert_eq!(module_origin(&input, &["library", "workers"]), expected);
        let ResolvedDependencySource::Vendor {
            definition_origin, ..
        } = &mut input
            .package_environment
            .as_mut()
            .unwrap()
            .dependencies
            .last_mut()
            .unwrap()
            .source
        else {
            unreachable!()
        };
        *definition_origin = None;
        refresh(&mut input);
        let checked = check(&input);
        assert!(
            source_definition_origins(&input, &checked)
                .unwrap_err()
                .to_string()
                .contains("no canonical definition origin")
        );
    }
}

#[test]
fn locked_remote_origins_match_the_publisher_and_ignore_revision_or_cache() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let consumer = workspace.package("consumer", "app", "app.main");
    let publisher = module_origin(&library, &["library", "workers"]);
    let mut expected = None;
    for alias in ["one", "two"] {
        let mut input = consumer.clone();
        overlay(&mut input, &library, alias);
        input
            .package_environment
            .as_mut()
            .unwrap()
            .dependencies
            .last_mut()
            .unwrap()
            .source = ResolvedDependencySource::Git {
            url: "https://git.invalid/owner/workers.git".into(),
            rev: "locked-commit".into(),
            checksum: "locked-payload".into(),
            store: Some(format!("missing-cache/{alias}")),
        };
        refresh(&mut input);
        let origin = module_origin(&input, &["library", "workers"]);
        assert_eq!(origin.package(), publisher.package());
        if let Some(expected) = &expected {
            assert_eq!(&origin, expected);
        }
        expected = Some(origin.clone());
        let ResolvedDependencySource::Git { rev, .. } = &mut input
            .package_environment
            .as_mut()
            .unwrap()
            .dependencies
            .last_mut()
            .unwrap()
            .source
        else {
            unreachable!()
        };
        *rev = "different-locked-commit".into();
        refresh(&mut input);
        assert_eq!(
            module_origin(&input, &["library", "workers"]).package(),
            origin.package()
        );
    }
}

#[test]
fn all_remote_sources_share_the_published_name_version_origin() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let consumer = workspace.package("consumer", "app", "app.main");
    let publisher = module_origin(&library, &["library", "workers"]);
    let sources = [
        ResolvedDependencySource::Registry {
            registry: "https://registry.example".into(),
            checksum: "checksum".into(),
            store: None,
        },
        ResolvedDependencySource::GitHubClone {
            repo: "owner/workers".into(),
            rev: "commit".into(),
            checksum: "checksum".into(),
            store: None,
        },
        ResolvedDependencySource::GitHubRelease {
            repo: "owner/workers".into(),
            release: "v1".into(),
            asset: "package.tar".into(),
            asset_checksum: "asset-checksum".into(),
            payload_checksum: "payload-checksum".into(),
            store: None,
        },
    ];
    for source in sources {
        let mut input = consumer.clone();
        overlay(&mut input, &library, "workers");
        input
            .package_environment
            .as_mut()
            .unwrap()
            .dependencies
            .last_mut()
            .unwrap()
            .source = source;
        refresh(&mut input);
        assert_eq!(
            module_origin(&input, &["library", "workers"]).package(),
            publisher.package()
        );
    }
    let mut invalid = consumer;
    overlay(&mut invalid, &library, "workers");
    invalid
        .package_environment
        .as_mut()
        .unwrap()
        .dependencies
        .last_mut()
        .unwrap()
        .source = ResolvedDependencySource::Git {
        url: String::new(),
        rev: "commit".into(),
        checksum: "checksum".into(),
        store: None,
    };
    assert!(capture(&invalid).is_err());
}
