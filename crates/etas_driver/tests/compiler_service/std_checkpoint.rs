use super::*;
use etas_frontend::code::service::CandidatePublisher;
use etas_interpreter::api::{InterpValue, RunOptions, codec};
use std::{path::Path, sync::Arc};

fn prepare(sysroot: &Path) -> Arc<etas_interpreter::api::code::PreparedCodeRevision> {
    use etas_driver::std_library::provider::StdProvider;
    use etas_driver::{CompileProjectRequest, DriverCacheMode, DriverOptions, check_project_once};
    let options = DriverOptions {
        std_provider: StdProvider::artifact(Some(sysroot)).unwrap(),
        cache_mode: DriverCacheMode::Off,
        ..Default::default()
    };
    let mut input = ProjectInput::single_source(SourceInput::anonymous(
        r#"
module app;
import std.runtime.checkpoint_apply;
flow main(value: i32) -> i32 ![] {
    let add = (number: i32) => number + 1;
    return checkpoint_apply(add, value);
}
flow inspect() -> unit { let definition = ^main; return; }
"#,
    ));
    input.options.entry_policy = EntryPolicy::Optional;
    let package = PackageKey::from_name_and_version("std-checkpoint-app", "0.1.0").unwrap();
    input.environment.local_source_package = Some(package);
    let output = check_project_once(
        &options,
        CompileProjectRequest {
            input,
            snapshot_detail: Default::default(),
        },
    )
    .unwrap()
    .response
    .output;
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let origins = DefinitionOrigins {
        source_modules: DefinitionOrigins::declared_source_modules(&checked, package).unwrap(),
        standard_package: etas_frontend::code::identity::standard_package_key(
            &checked.std_registry,
        )
        .unwrap(),
        external_definitions: DefinitionOrigins::checked_external_definitions(&checked).unwrap(),
    };
    let reflection = *checked.types.reflections.keys().next().unwrap();
    let catalog =
        Arc::new(DefinitionCatalog::build(checked, &origins, Default::default()).unwrap());
    let store = NativeDefinitionStore::new(
        catalog,
        Arc::new(NativeCodeSyntaxOperations::default()),
        Default::default(),
    );
    let definition = store.reflect_flow(reflection).unwrap();
    let StdProvider::Artifact(artifact) = &options.std_provider else {
        unreachable!()
    };
    let mut session = etas_frontend::FrontendSession::with_std_registry(artifact.registry.clone())
        .with_standard_library(artifact.executable_library(&options.profile).unwrap());
    let candidate = store
        .compile_definition(&definition, &mut session, Default::default())
        .unwrap();
    InterpreterCandidatePublisher.publish(candidate).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn source_std_checkpoint_resumes_under_a_fresh_prepared_revision_in_another_process() {
    const ROOT: &str = "ETAS_STD_REVISION_CHECKPOINT_ROOT";
    if let Some(root) = std::env::var_os(ROOT) {
        let root = std::path::PathBuf::from(root);
        let prepared = prepare(&root.join("sdk"));
        let mut revisions = etas_interpreter::api::code::PreparedRevisionSet::new(
            prepared.type_bridge().original_program().clone(),
        );
        revisions.insert(prepared.clone()).unwrap();
        let revisions = Arc::new(revisions);
        let document = codec::checkpoint_file_from_bytes(
            &std::fs::read(root.join("checkpoint.bin")).unwrap(),
            Default::default(),
        )
        .unwrap();
        let checkpoint = codec::checkpoint_from_json_with_revisions(
            &Default::default(),
            &document,
            prepared.program(),
            Some(&revisions),
        )
        .unwrap();
        let host = no_host::NoHost::with_checkpoint();
        let result = prepared
            .create_resume(
                &checkpoint,
                &host,
                RunOptions {
                    checkpoint_revisions: Some(revisions),
                    replay_key_provider: Arc::new(
                        etas_host::replay::DirectoryReplayKeyProvider::open(&root.join("keys"))
                            .unwrap(),
                    ),
                    ..Default::default()
                },
            )
            .execute()
            .await
            .unwrap();
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.value(), Some(&InterpValue::i32(42)));
        assert!(result.checkpoints.is_empty());
        assert_eq!(host.calls(), 0);
        return;
    }

    let workspace = etas_host::TestWorkspace::create("std-revision-checkpoint").unwrap();
    let root = workspace.path();
    let maintained = etas_std::library::source_checkout_manifest();
    let source = root.join("source");
    fn copy(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target.join(entry.file_name()));
            } else {
                std::fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
            }
        }
    }
    copy(
        &maintained.parent().unwrap().join("library"),
        &source.join("library"),
    );
    std::fs::copy(maintained, source.join("etas.toml")).unwrap();
    std::fs::write(
        source.join("library/std/runtime/probe.es"),
        r#"module std.runtime;
import std.runtime.checkpoint.checkpoint;
type Pending<T> = { value: T };
flow saved<T>(value: T) -> T {
    let pending = Pending { value = value };
    checkpoint("inside-std-revision");
    return pending.value;
}
public flow checkpoint_apply<T>(action: (T) -> T ![], value: T) -> T {
    let first = action(value);
    return action(saved(first));
}
"#,
    )
    .unwrap();
    etas_driver::build_standard_library(
        &etas_driver::DriverOptions {
            cache_mode: etas_driver::DriverCacheMode::Off,
            ..Default::default()
        },
        etas_driver::StdLibraryBuildRequest {
            manifest_path: source.join("etas.toml"),
            out_dir: root.join("sdk/std"),
        },
    )
    .unwrap();
    let prepared = prepare(&root.join("sdk"));
    let host = no_host::NoHost::with_checkpoint();
    let result = prepared
        .create_run(
            vec![InterpValue::i32(40)],
            &host,
            RunOptions {
                replay_key_provider: Arc::new(
                    etas_host::replay::DirectoryReplayKeyProvider::open(&root.join("keys"))
                        .unwrap(),
                ),
                ..Default::default()
            },
        )
        .execute()
        .await
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(42)));
    assert_eq!(result.checkpoints.len(), 1);
    assert_eq!(host.calls(), 0);
    let document = codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0]).unwrap();
    assert!(
        serde_json::to_string(&*document)
            .unwrap()
            .contains("standard_library")
    );
    let bytes = codec::checkpoint_file_to_bytes(document, Default::default()).unwrap();
    std::fs::write(root.join("checkpoint.bin"), bytes).unwrap();
    drop(result);
    drop(prepared);
    // The receiver reloads the installed artifact and recompiles the caller;
    // no live checked program, closure or PreparedCodeRevision is transferred.
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "std_checkpoint::source_std_checkpoint_resumes_under_a_fresh_prepared_revision_in_another_process", "--nocapture"])
        .env(ROOT, root).output().unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}
