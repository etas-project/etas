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
        loaded.package_environment.as_ref(),
        &loaded.loaded.input,
    )
}

fn refresh(loaded: &mut LoadedDriverProject) {
    loaded.source_definition_provenance = capture(loaded).unwrap();
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
fn package_origin_is_neither_a_display_name_nor_a_source_content_hash() {
    let workspace = Workspace::new();
    let mut first = workspace.package("first", "same-name", "app.main");
    let second = workspace.package("second", "same-name", "app.main");
    let before = module_origin(&first, &["app", "main"]);
    assert_ne!(
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
            .contains("no matching locked package origin")
    );
    consumer.package_environment = Some(saved);
    let other = workspace.package("other", "workers", "library.workers");
    let dependencies = &mut consumer.package_environment.as_mut().unwrap().dependencies;
    let mut duplicate = dependencies.last().unwrap().clone();
    duplicate.source = ResolvedDependencySource::Path {
        path: other.package_root.unwrap().to_str().unwrap().into(),
        checksum: "another-locked-revision".into(),
    };
    dependencies.push(duplicate);
    assert!(
        capture(&consumer)
            .unwrap_err()
            .to_string()
            .contains("binding is ambiguous")
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
    let single = load(&root.join("main.es"));
    assert_eq!(single.mode, DriverProjectMode::SingleFile);
    let first = module_origin(&single, &["main"]);
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
fn locked_remote_origins_do_not_depend_on_import_alias_or_local_cache() {
    let workspace = Workspace::new();
    let library = workspace.package("library", "workers", "library.workers");
    let consumer = workspace.package("consumer", "app", "app.main");
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
        assert_ne!(
            module_origin(&input, &["library", "workers"]).package(),
            origin.package()
        );
    }
}
