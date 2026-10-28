mod support;

use std::path::{Path, PathBuf};

use etas_cache::ArtifactKey;
use etas_core::{DiagnosticCode, SourceId, TypeDiagnosticCode};
use etas_frontend::{
    BODY_UNIT_KIND, CheckRequest, EnvironmentChange, ExternalModuleId, ExternalPackageId,
    ExternalSymbolId, FrontendSession, FrontendSessionOptions, ModulePath, ProjectEntry,
    ProjectEnvironmentInput, ProjectExternalExportInput, ProjectExternalModuleInput,
    ProjectExternalPackageInput, ProjectInput, ProjectOutput, ProjectRevision, SourceChange,
    SourceInput, SourceKind, SourceVersion, UnitId, UnitKind, UnitTarget,
};
use etas_utils::UnitKey;
use support::{fixture, path_str, run_cli};

#[test]
fn compiler_incremental_fixture_projects_have_architecture_coverage() {
    let projects = incremental_project_roots();
    assert_eq!(
        projects.len(),
        3,
        "incremental compiler fixtures should contain three independent projects"
    );

    for project in &projects {
        let name = project.file_name().unwrap().to_string_lossy();
        assert!(
            project.join("etas.toml").exists(),
            "{name} should include an etas.toml manifest"
        );
        assert!(
            project.join("src/app/main.es").exists(),
            "{name} should expose app.main"
        );
        let source_lines = count_source_lines(&project.join("src"));
        assert!(
            source_lines >= 1000,
            "{name} should contain at least 1000 non-empty, non-comment source lines; got {source_lines}"
        );
        assert!(
            project_sources(project)
                .iter()
                .any(|source| read(source).contains("import std.")),
            "{name} should exercise std import resolution"
        );
    }

    let body = fixture("compiler/incremental/body_reuse_workload");
    assert!(
        body.join("edits/body_only_edit/src/app/body/workload.es")
            .exists()
    );
    assert!(
        body.join("edits/diagnostic_error/src/app/body/workload.es")
            .exists()
    );
    assert!(
        body.join("edits/diagnostic_fixed/src/app/body/workload.es")
            .exists()
    );

    let import = fixture("compiler/incremental/import_invalidation_workload");
    assert!(
        import
            .join("edits/import_edge_change/src/app/consumer.es")
            .exists()
    );

    let module_parts = fixture("compiler/incremental/module_part_merge_workload");
    let merged_part_count = project_sources(&module_parts)
        .iter()
        .filter(|source| read(source).contains("module app.pipeline;"))
        .count();
    assert!(
        merged_part_count >= 4,
        "module_part_merge_workload should have one canonical module file plus multiple ModulePart files"
    );
    assert!(
        module_parts
            .join("edits/snapshot_delta/src/app/pipeline/part_b.es")
            .exists()
    );
    assert!(module_parts.join("dependency_import_source.es").exists());
    assert!(module_parts.join("dependency-manifest.json").exists());
}

#[test]
fn compiler_incremental_fixture_projects_check_as_cli_workspaces() {
    let mut failures = Vec::new();

    for project in incremental_project_roots() {
        let name = project.file_name().unwrap().to_string_lossy();
        let (code, stdout, stderr) =
            run_cli(["etas", "--workspace", path_str(&project), "check", "--all"]);
        if code != 0 || !stdout.contains("checked ") {
            failures.push(format!(
                "{name} should check as a base incremental fixture workspace\nexit: {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "incremental compiler fixtures exposed project-check failures:\n\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn compiler_incremental_body_fixture_reuses_unchanged_bodies_and_replaces_diagnostics() {
    let root = fixture("compiler/incremental/body_reuse_workload");
    let (input, sources) = project_input_from_workspace(&root);
    let changed = source_id(&sources, "src/app/body/workload.es");
    let unchanged = source_id(&sources, "src/app/body/chain.es");

    // Complete-reuse assertions require an admitted working set, not an LRU miss.
    let mut session = FrontendSession::with_options(
        FrontendSessionOptions::memory_only().with_memory_cache_byte_budget(256 * 1024 * 1024),
    )
    .unwrap();
    let project = session.open_project(input);
    let first = session
        .check(project, CheckRequest::default())
        .expect("initial body reuse fixture check should run");
    assert!(
        first.output.checked.is_some(),
        "{:?}",
        first.diagnostics.diagnostics
    );
    let usage = first.memory.cache_after.as_ref().unwrap();
    assert_eq!(
        usage.evicted_artifacts, 0,
        "reuse workload must fit its explicit cache budget: {usage:?}"
    );
    assert_eq!(
        usage.rejected_insertions, 0,
        "reuse workload must be admitted: {usage:?}"
    );
    let unchanged_body_keys = body_artifact_keys_for_source(&first.output, unchanged);
    let first_manifest = session.artifact_manifest(project).unwrap().clone();
    assert!(
        unchanged_body_keys.len() > 50,
        "fixture should provide many unchanged body artifacts"
    );

    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(1),
        &root.join("edits/body_only_edit/src/app/body/workload.es"),
    );
    let second = session
        .check(project, CheckRequest::incremental())
        .expect("body-only edit incremental check should run");
    assert!(
        second.output.checked.is_some(),
        "{:?}",
        second.diagnostics.diagnostics
    );
    let first_signatures = &first.output.signature_types.as_ref().unwrap().store;
    let second_signatures = &second.output.signature_types.as_ref().unwrap().store;
    let changed_signature_types = first_signatures
        .iter()
        .zip(second_signatures.iter())
        .filter(|((_, first), (_, second))| first != second)
        .take(8)
        .collect::<Vec<_>>();
    assert!(
        first_signatures.len() == second_signatures.len() && changed_signature_types.is_empty(),
        "body-only edit changed declaration type universe: {} -> {} types; differences: {changed_signature_types:?}",
        first_signatures.len(),
        second_signatures.len()
    );
    let second_manifest = session.artifact_manifest(project).unwrap();
    for record in &first_manifest.records {
        if !unchanged_body_keys
            .iter()
            .any(|(key, _)| *key == record.key.to_cache_key())
        {
            continue;
        }
        for dependency in &record.dependencies {
            if let Some(expected) = dependency.fingerprint {
                assert_eq!(
                    second_manifest
                        .get(&dependency.key)
                        .map(|record| record.fingerprint),
                    Some(expected),
                    "unchanged body dependency changed: {}",
                    dependency.key.to_cache_key()
                );
            }
        }
    }
    for (type_key, effect_key) in &unchanged_body_keys {
        assert!(
            second.cache.reused_artifacts.contains(type_key),
            "unchanged sibling source TypeFacts should be reused after body-only edit: {type_key}; retained revision: {}; initial cache: {:?}",
            first.memory.retained_revision_estimated_bytes,
            first.memory.cache_after
        );
        assert!(
            second.cache.reused_artifacts.contains(effect_key),
            "unchanged sibling source EffectFacts should be reused after body-only edit: {effect_key}"
        );
    }
    let second_delta = second
        .delta
        .expect("body edit should produce a semantic delta");
    assert_eq!(second_delta.changed_sources, vec![changed]);
    assert!(
        second_delta
            .diagnostics
            .republish_sources
            .contains(&changed)
    );

    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(2),
        &root.join("edits/diagnostic_error/src/app/body/workload.es"),
    );
    let error = session
        .check(project, CheckRequest::incremental())
        .expect("diagnostic edit incremental check should run");
    assert!(error.output.checked.is_none());
    assert!(
        error
            .output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code
                == DiagnosticCode::Type(TypeDiagnosticCode::TypeMismatch)),
        "diagnostic edit should produce a type mismatch: {:?}",
        error.diagnostics.diagnostics
    );
    assert!(
        error
            .delta
            .expect("diagnostic edit delta")
            .diagnostics
            .republish_sources
            .contains(&changed)
    );

    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(3),
        &root.join("edits/diagnostic_fixed/src/app/body/workload.es"),
    );
    let fixed = session
        .check(project, CheckRequest::incremental())
        .expect("diagnostic fix incremental check should run");
    assert!(
        fixed.output.checked.is_some(),
        "{:?}",
        fixed.diagnostics.diagnostics
    );
    assert!(
        fixed
            .delta
            .expect("diagnostic fix delta")
            .diagnostics
            .republish_sources
            .contains(&changed)
    );
}

#[test]
fn compiler_incremental_fixture_recomputes_body_facts_with_zero_cache_capacity() {
    let root = fixture("compiler/incremental/body_reuse_workload");
    let (input, sources) = project_input_from_workspace(&root);
    let changed = source_id(&sources, "src/app/body/workload.es");
    let unchanged = source_id(&sources, "src/app/body/chain.es");
    let mut session = FrontendSession::with_options(
        FrontendSessionOptions::memory_only().with_memory_cache_byte_budget(0),
    )
    .unwrap();
    let project = session.open_project(input);
    let first = session.check(project, CheckRequest::default()).unwrap();
    assert!(first.output.checked.is_some(), "{:?}", first.diagnostics);
    let unchanged_keys = body_artifact_keys_for_source(&first.output, unchanged);
    assert!(unchanged_keys.len() > 50);
    assert_eq!(first.memory.cache_after.as_ref().unwrap().entries, 0);
    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(1),
        &root.join("edits/body_only_edit/src/app/body/workload.es"),
    );
    let second = session.check(project, CheckRequest::incremental()).unwrap();
    assert!(second.output.checked.is_some(), "{:?}", second.diagnostics);
    assert_eq!(
        second
            .memory
            .cache_after
            .as_ref()
            .unwrap()
            .estimated_retained_bytes,
        0
    );
    for (types, effects) in unchanged_keys {
        assert!(!second.cache.reused_artifacts.contains(&types));
        assert!(!second.cache.reused_artifacts.contains(&effects));
    }
    assert!(
        second
            .pipeline_records
            .iter()
            .any(|record| record.pass == "TypeCheckBodyPass")
    );
}

#[test]
fn compiler_incremental_import_fixture_tracks_reverse_import_and_reuses_unrelated_bodies() {
    let root = fixture("compiler/incremental/import_invalidation_workload");
    let (input, sources) = project_input_from_workspace(&root);
    let changed = source_id(&sources, "src/app/consumer.es");
    let unrelated = source_id(&sources, "src/app/unrelated.es");

    let mut session = FrontendSession::new();
    let project = session.open_project(input);
    let first = session
        .check(project, CheckRequest::default())
        .expect("initial import fixture check should run");
    assert!(
        first.output.checked.is_some(),
        "{:?}",
        first.diagnostics.diagnostics
    );
    let unrelated_body_keys = body_artifact_keys_for_source(&first.output, unrelated);
    assert!(!unrelated_body_keys.is_empty());

    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(1),
        &root.join("edits/import_edge_change/src/app/consumer.es"),
    );
    let second = session
        .check(project, CheckRequest::incremental())
        .expect("import edge incremental check should run");
    assert!(
        second.output.checked.is_some(),
        "{:?}",
        second.diagnostics.diagnostics
    );
    for (type_key, effect_key) in &unrelated_body_keys {
        assert!(
            second.cache.reused_artifacts.contains(type_key),
            "unrelated TypeFacts should be reused after import edge change: {type_key}"
        );
        assert!(
            second.cache.reused_artifacts.contains(effect_key),
            "unrelated EffectFacts should be reused after import edge change: {effect_key}"
        );
    }

    let delta = second
        .delta
        .expect("import edge change should produce a delta");
    let affected = module_paths_for_delta(&second.output, &delta.affected_modules);
    assert!(
        affected.contains(&"app.consumer".to_owned()),
        "{affected:?}"
    );
    assert!(affected.contains(&"app.main".to_owned()), "{affected:?}");
    assert!(
        !affected.contains(&"app.unrelated".to_owned()),
        "reverse-import invalidation should not pull unrelated modules into the affected set: {affected:?}"
    );
}

#[test]
fn compiler_incremental_module_part_fixture_refreshes_snapshot_delta_and_dependency_imports() {
    let root = fixture("compiler/incremental/module_part_merge_workload");
    let (input, sources) = project_input_from_workspace(&root);
    let changed = source_id(&sources, "src/app/pipeline/part_b.es");

    let mut session = FrontendSession::new();
    let project = session.open_project(input);
    let first = session
        .check(project, CheckRequest::default())
        .expect("initial module part fixture check should run");
    assert!(
        first.output.checked.is_some(),
        "{:?}",
        first.diagnostics.diagnostics
    );
    let snapshot = first.snapshot.expect("initial snapshot should exist");
    let pipeline = snapshot
        .modules
        .by_path
        .get(&ModulePath {
            segments: vec!["app".to_owned(), "pipeline".to_owned()],
        })
        .and_then(|module| snapshot.modules.modules.get(*module))
        .expect("app.pipeline module should be indexed");
    assert!(
        pipeline.parts.len() >= 4,
        "app.pipeline should merge the canonical file plus multiple ModulePart sources"
    );

    apply_replace(
        &mut session,
        project,
        changed,
        SourceVersion(1),
        &root.join("edits/snapshot_delta/src/app/pipeline/part_b.es"),
    );
    let second = session
        .check(project, CheckRequest::incremental())
        .expect("module part incremental check should run");
    assert!(
        second.output.checked.is_some(),
        "{:?}",
        second.diagnostics.diagnostics
    );
    let delta = second
        .delta
        .expect("module part edit should produce a delta");
    assert_eq!(delta.changed_sources, vec![changed]);
    assert!(!delta.affected_bodies.is_empty());
    assert_eq!(
        second.snapshot.expect("updated snapshot").revision,
        ProjectRevision(1)
    );

    let dependency_output = etas_frontend::Frontend.check_project(ProjectInput {
        project_root: root.clone(),
        source_root: Some(root.join("src")),
        options: Default::default(),
        environment: external_environment(),
        sources: vec![SourceInput {
            id: SourceId(1),
            path: Some(root.join("src/app/dependency_bridge.es")),
            text: read(&root.join("dependency_import_source.es")),
            kind: SourceKind::SourceProjectFile,
        }],
        entry: ProjectEntry {
            module: Some(ModulePath {
                segments: vec!["app".to_owned(), "dependency_bridge".to_owned()],
            }),
            flow: "main".to_owned(),
        },
    });
    assert!(
        dependency_output.checked.is_some(),
        "{:?}",
        dependency_output.diagnostics
    );
    let resolved = dependency_output
        .resolved_imports
        .as_ref()
        .expect("dependency imports should resolve");
    assert_eq!(resolved.re_exports.len(), 1);
    assert_eq!(resolved.re_exports[0].target.segments, ["dep", "math"]);
    assert!(
        resolved.wildcard_imports.iter().any(|wildcard| {
            wildcard.exported_names.iter().any(|name| name == "Number")
                && wildcard.exported_names.iter().any(|name| name == "add")
        }),
        "dependency wildcard re-export should expose dependency names"
    );
}

fn incremental_project_roots() -> Vec<PathBuf> {
    let root = fixture("compiler/incremental");
    let mut projects = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir() && path.join("etas.toml").exists())
        .collect::<Vec<_>>();
    projects.sort();
    projects
}

fn project_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_las_sources(&root.join("src"), &mut files);
    files.sort();
    files
}

fn collect_las_sources(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_las_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "es") {
            files.push(path);
        }
    }
}

fn count_source_lines(root: &Path) -> usize {
    let mut files = Vec::new();
    collect_las_sources(root, &mut files);
    files
        .iter()
        .map(|path| {
            read(path)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with("//"))
                .count()
        })
        .sum()
}

fn project_input_from_workspace(root: &Path) -> (ProjectInput, Vec<(SourceId, PathBuf)>) {
    let sources = project_sources(root);
    let mut source_inputs = Vec::new();
    let mut source_paths = Vec::new();
    for (index, path) in sources.into_iter().enumerate() {
        let source = SourceId(index as u32 + 1);
        source_paths.push((source, path.clone()));
        source_inputs.push(SourceInput {
            id: source,
            path: Some(path.clone()),
            text: read(&path),
            kind: SourceKind::SourceProjectFile,
        });
    }

    (
        ProjectInput {
            project_root: root.to_path_buf(),
            source_root: Some(root.join("src")),
            options: Default::default(),
            environment: ProjectEnvironmentInput::default(),
            sources: source_inputs,
            entry: ProjectEntry {
                module: Some(ModulePath {
                    segments: vec!["app".to_owned(), "main".to_owned()],
                }),
                flow: "main".to_owned(),
            },
        },
        source_paths,
    )
}

fn source_id(sources: &[(SourceId, PathBuf)], suffix: &str) -> SourceId {
    sources
        .iter()
        .find_map(|(source, path)| path.ends_with(suffix).then_some(*source))
        .unwrap_or_else(|| panic!("missing fixture source ending in {suffix}"))
}

fn body_artifact_keys_for_source(
    output: &ProjectOutput,
    source: SourceId,
) -> Vec<(ArtifactKey, ArtifactKey)> {
    let units = output.units.as_ref().expect("unit tree should exist");
    let mut keys = units
        .nodes
        .iter()
        .filter_map(|(unit, node)| {
            (node.kind == UnitKind::Body && node.source == Some(source))
                .then_some(UnitKey::new(BODY_UNIT_KIND, unit.0 as u64))
        })
        .map(|unit| {
            (
                frontend_body_artifact_key("type_facts", unit),
                frontend_body_artifact_key("effect_facts", unit),
            )
        })
        .collect::<Vec<_>>();
    keys.sort_by(|left, right| left.0.to_string().cmp(&right.0.to_string()));
    keys
}

fn frontend_body_artifact_key(kind: &str, unit: UnitKey) -> ArtifactKey {
    ArtifactKey::new(
        "frontend",
        kind,
        format!(
            "unit:{}:{}:{}",
            unit.kind.namespace, unit.kind.name, unit.id
        ),
    )
}

fn module_paths_for_delta(output: &ProjectOutput, modules: &[UnitKey]) -> Vec<String> {
    let units = output.units.as_ref().expect("unit tree should exist");
    let module_index = output.modules.as_ref().expect("module index should exist");
    let mut paths = modules
        .iter()
        .filter_map(|unit| units.nodes.get(UnitId(unit.id as u32)))
        .filter_map(|node| match node.target {
            UnitTarget::Module(module) => module_index.modules.get(module),
            _ => None,
        })
        .map(|module| module.path.segments.join("."))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn apply_replace<S: etas_frontend::FrontendArtifactStore + 'static>(
    session: &mut FrontendSession<S>,
    project: etas_frontend::ProjectSessionId,
    source: SourceId,
    version: SourceVersion,
    path: &Path,
) {
    session
        .apply_changes(
            project,
            etas_frontend::ProjectChangeSet {
                revision: ProjectRevision(version.0),
                source_changes: vec![SourceChange::Replace {
                    source,
                    version,
                    text: read(path),
                }],
                dependency_overlay_changes: Vec::new(),
                environment_change: None::<EnvironmentChange>,
                option_changes: Vec::new(),
            },
        )
        .expect("fixture source replacement should be accepted");
}

fn external_environment() -> ProjectEnvironmentInput {
    use etas_frontend::{
        ProjectExternalEffectSummaryInput, ProjectExternalFlowSignatureInput,
        ProjectExternalNamedSignatureInput, ProjectExternalPublicMetadataInput,
        ProjectExternalTypeInput,
    };
    ProjectEnvironmentInput {
        external_packages: vec![ProjectExternalPackageInput {
            id: ExternalPackageId(0),
            name: "dep".into(),
            version: "1.0.0".into(),
            edition: "2026".into(),
            import_root: "dep".into(),
        }],
        external_modules: vec![ProjectExternalModuleInput {
            package: Some(ExternalPackageId(0)),
            id: ExternalModuleId(1),
            path: ModulePath {
                segments: vec!["dep".to_owned(), "math".to_owned()],
            },
            exports: vec![
                ProjectExternalExportInput {
                    symbol: ExternalSymbolId(0),
                    name: "Number".to_owned(),
                    visibility: etas_hir::Visibility::Public,
                },
                ProjectExternalExportInput {
                    symbol: ExternalSymbolId(1),
                    name: "add".to_owned(),
                    visibility: etas_hir::Visibility::Public,
                },
            ],
        }],
        external_public_metadata: vec![ProjectExternalPublicMetadataInput {
            package: ExternalPackageId(0),
            types: vec![ProjectExternalNamedSignatureInput {
                path: vec!["dep".into(), "math".into(), "Number".into()],
                generic_params: Vec::new(),
                visibility: "public".into(),
                ty: Some(ProjectExternalTypeInput::Primitive("i32".into())),
            }],
            flows: vec![ProjectExternalFlowSignatureInput {
                path: vec!["dep".into(), "math".into(), "add".into()],
                generic_params: Vec::new(),
                param_names: vec!["left".into(), "right".into()],
                params: vec![ProjectExternalTypeInput::Primitive("i32".into()); 2],
                output: ProjectExternalTypeInput::Primitive("i32".into()),
                effects: Some(Default::default()),
                visibility: "public".into(),
            }],
            effect_summaries: vec![ProjectExternalEffectSummaryInput {
                handler_requests: None,
                item: vec!["dep".into(), "math".into(), "add".into()],
                returned_callables: None,
                default_handler_effects: None,
                public_effects: Default::default(),
                requested_actions: Default::default(),
                handled_requested_actions: Default::default(),
                latent_flows: Vec::new(),
                action_trace: Default::default(),
            }],
            values: Vec::new(),
            enums: Vec::new(),
            agents: Vec::new(),
            tools: Vec::new(),
            tool_schemas: Vec::new(),
            effects: Vec::new(),
            actions: Vec::new(),
            trace_specs: Vec::new(),
            spec_signatures: Vec::new(),
            spec_impls: Vec::new(),
            type_spec_satisfactions: Vec::new(),
            callable_spec_satisfactions: Vec::new(),
            trace_spec_conformances: Vec::new(),
            action_summaries: Vec::new(),
            trace_spec_summaries: Vec::new(),
            re_exports: Vec::new(),
        }],
        ..ProjectEnvironmentInput::default()
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}
