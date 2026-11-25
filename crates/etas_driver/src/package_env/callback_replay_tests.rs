use etas_frontend::{EntryPolicy, Frontend, ProjectInput, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};

use super::frontend_environment;

pub(super) struct PackageFixture(pub(super) std::path::PathBuf);

impl PackageFixture {
    fn new(source: &str) -> Self {
        Self::named("callbacks", source)
    }

    pub(super) fn named(name: &str, source: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-closed-callback-replay-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.es"), source).unwrap();
        std::fs::write(
            root.join("etas.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n"),
        )
        .unwrap();
        Self(root)
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn default_callable_values_survive_source_binary_driver_metadata_replay() {
    let producer = r#"module defaults;
public type Codec<T> = { encode: (T) -> T ![] };
flow make_codec<T>() -> Codec<T> ![] { return Codec { encode= (value: T) => value }; }
public flow apply<T>(value: T, codec: Codec<T> = make_codec<T>()) -> T ![] {
    return (codec.encode)(value);
}
public flow seeded<T>(seed: T, callback: () -> T ![] = () => seed) -> T ![] { return callback(); }
public type Owner<T> = { value: T };
impl Owner<T> {
    public flow apply(self, codec: Codec<T> = make_codec<T>()) -> T ![] { return (codec.encode)(self.value); }
}
"#;
    let mut input = ProjectInput::single_source(SourceInput::anonymous(producer));
    input.options.entry_policy = EntryPolicy::Optional;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let package = PackageFixture::named("defaults", producer);
    let source_hash = etas_package::package_source_payload_hash(&package.0).unwrap();
    let artifact = etas_frontend::build_package_metadata_artifact_from_checked(
        etas_frontend::PackageMetadataBuildInput {
            definition_origin: Some(
                etas_code::PackageKey::from_name_and_version("defaults", "0.1.0").unwrap(),
            ),
            package_id: "defaults".into(),
            package_version: "0.1.0".into(),
            package_edition: "2026".into(),
            compiler_version: "test".into(),
            source_payload_hash: source_hash.clone(),
            manifest_hash: etas_package::package_manifest_hash(&package.0).unwrap(),
            dependency_lock_hash: etas_package::package_dependency_lock_hash(&package.0).unwrap(),
            bins: Vec::new(),
            dependencies: Vec::new(),
            tool_bindings: Vec::new(),
        },
        output.checked.as_ref().unwrap(),
    )
    .unwrap();
    let path = etas_package::package_metadata_artifact_path(&package.0);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, artifact.bytes).unwrap();
    let metadata = etas_package::read_package_metadata_artifact(&package.0)
        .unwrap()
        .unwrap();
    let default = &metadata
        .public_metadata
        .flows
        .iter()
        .find(|signature| signature.path == ["defaults", "apply"])
        .unwrap()
        .parameter_defaults[0];
    assert!(
        default
            .callable_value
            .as_ref()
            .is_some_and(|graph| !graph.source_projections.is_empty())
    );
    assert!(
        metadata
            .public_metadata
            .methods
            .iter()
            .find(|method| method.identity.name() == "apply")
            .unwrap()
            .parameter_defaults[0]
            .callable_value
            .is_some()
    );
    let environment = frontend_environment(PackageEnvironmentMetadata {
        dependencies: vec![ResolvedDependency {
            identity: metadata.package,
            std_compatibility: metadata.std_compatibility,
            import_root: "defaults".into(),
            source: ResolvedDependencySource::Path {
                path: package.0.to_string_lossy().into_owned(),
                checksum: source_hash,
            },
            dependencies: metadata.dependencies,
            public_metadata: metadata.public_metadata,
            effect_metadata: metadata.effect_metadata,
            tool_bindings: metadata.tool_bindings,
        }],
        ..Default::default()
    })
    .unwrap();
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import defaults.{apply, seeded, Owner};
flow main() -> i32 ![] {
    let owner: Owner<i32> = Owner { value= 3 };
    return apply(1) + seeded(2) + owner.apply();
}
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output.checked.is_some());
}

#[test]
fn closed_callback_templates_survive_source_binary_driver_metadata_replay() {
    let producer = r#"module callbacks;
public flow twice(f: () -> unit ![]) -> unit { f(); f(); }
public flow relay(g: () -> unit ![]) -> unit { let alias = g; twice(alias); }
public flow ignore(unused: () -> unit ![]) -> unit { return; }
public flow invoke_pair<A, B, effect E>(jobs: (() -> A ![E], () -> B ![E])) -> (A, B) ![E] {
    let (first, second) = jobs;
    return (first(), second());
}
public flow capture_pair<A, B, effect E>(jobs: (() -> A ![E], () -> B ![E])) -> (() -> (A, B) ![E]) ![] {
    let (first, second) = jobs;
    return () => (first(), second());
}
public flow invoke_fields<A, B, effect E>(jobs: { first: () -> A ![E], second: () -> B ![E] }) -> (A, B) ![E] {
    return ((jobs.first)(), (jobs.second)());
}
public flow relay_fields<A, B, effect E>(jobs: { first: () -> A ![E], second: () -> B ![E] }) -> { first: () -> A ![E], second: () -> B ![E] } ![] {
    return jobs;
}
public flow capture_fields<A, B, effect E>(jobs: { first: () -> A ![E], second: () -> B ![E] }) -> (() -> (A, B) ![E]) ![] {
    return () => ((jobs.first)(), (jobs.second)());
}
"#;
    let mut input = ProjectInput::single_source(SourceInput::anonymous(producer));
    input.options.entry_policy = EntryPolicy::Optional;
    let output = Frontend.check_project(input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let package = PackageFixture::new(producer);
    let source_hash = etas_package::package_source_payload_hash(&package.0).unwrap();
    let artifact = etas_frontend::build_package_metadata_artifact_from_checked(
        etas_frontend::PackageMetadataBuildInput {
            definition_origin: None,
            package_id: "callbacks".into(),
            package_version: "0.1.0".into(),
            package_edition: "2026".into(),
            compiler_version: "test".into(),
            source_payload_hash: source_hash.clone(),
            manifest_hash: etas_package::package_manifest_hash(&package.0).unwrap(),
            dependency_lock_hash: etas_package::package_dependency_lock_hash(&package.0).unwrap(),
            bins: Vec::new(),
            dependencies: Vec::new(),
            tool_bindings: Vec::new(),
        },
        output.checked.as_ref().unwrap(),
    )
    .unwrap();
    let path = etas_package::package_metadata_artifact_path(&package.0);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, artifact.bytes).unwrap();
    let metadata = etas_package::read_package_metadata_artifact(&package.0)
        .unwrap()
        .unwrap();
    let relay = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item == ["callbacks", "relay"])
        .unwrap();
    let etas_package::PackageActionTraceMetadata::Seq { children: trace } = &relay.action_trace
    else {
        panic!("{relay:?}");
    };
    assert_eq!(trace.iter().filter(|trace| matches!(trace, etas_package::PackageActionTraceMetadata::ParameterCall { parameter: etas_package_metadata::CallableRequestTarget::Parameter(parameter) } if parameter == "g")).count(), 2);
    assert!(relay.requested_actions.effects.is_empty());
    assert!(relay.requested_actions.tail.is_none());
    let environment = frontend_environment(PackageEnvironmentMetadata {
        dependencies: vec![ResolvedDependency {
            identity: metadata.package,
            std_compatibility: metadata.std_compatibility,
            import_root: "callbacks".into(),
            source: ResolvedDependencySource::Path {
                path: package.0.to_string_lossy().into_owned(),
                checksum: source_hash,
            },
            dependencies: metadata.dependencies,
            public_metadata: metadata.public_metadata,
            effect_metadata: metadata.effect_metadata,
            tool_bindings: metadata.tool_bindings,
        }],
        ..Default::default()
    })
    .unwrap();
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"module consumer;
import callbacks.{relay, ignore, invoke_pair, capture_pair, invoke_fields, capture_fields, relay_fields};
import std.io.println;
flow main() -> unit {
    relay(() => { println("handled")?; return; });
    ignore(() => { println("not invoked")?; return; });
    let paired: (i32, bool) = invoke_pair((() => { println("paired")?; return 7; }, () => true));
    let run = capture_pair((() => 7, () => { println("captured")?; return true; }));
    let captured: (i32, bool) = run();
    let fields: (i32, bool) = invoke_fields({ first= () => { println("first field")?; return 7; }, second= () => true });
    let run_fields = capture_fields({ first= () => 7, second= () => { println("captured field")?; return true; } });
    let captured_fields: (i32, bool) = run_fields();
    let returned_fields = relay_fields({ first= () => { println("returned first")?; return 7; }, second= () => { println("returned second")?; return true; } });
    let returned: (i32, bool) = ((returned_fields.first)(), (returned_fields.second)());
}
"#,
    ));
    input.environment = environment;
    let output = Frontend.check_project(input);
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != etas_core::Severity::Error),
        "{:?}",
        output.diagnostics
    );
    let checked = output.checked.unwrap();
    assert_eq!(
        checked.sources.sources.len(),
        1,
        "consumer must not reanalyze dependency source"
    );
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    assert!(summary.escaping_effects.is_empty(), "{summary:?}");
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
    let mut events = 0;
    let mut pending = vec![&summary.action_trace];
    while let Some(trace) = pending.pop() {
        match trace.structure() {
            etas_effects::ActionTraceDomain::Event(event) => {
                events += usize::from(event.action == stdout)
            }
            etas_effects::ActionTraceDomain::Seq(children)
            | etas_effects::ActionTraceDomain::Choice(children) => pending.extend(children),
            etas_effects::ActionTraceDomain::Repeat(inner) => pending.push(inner),
            etas_effects::ActionTraceDomain::Empty => {}
            other => panic!("unresolved callback trace after replay: {other:?}"),
        }
    }
    assert_eq!(events, 8, "{summary:?}");
}
