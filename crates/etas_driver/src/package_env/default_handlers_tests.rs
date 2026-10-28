use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use etas_frontend::{EntryPolicy, Frontend, ProjectInput, ProjectOutput, SourceInput};
use etas_package::{PackageEnvironmentMetadata, ResolvedDependency, ResolvedDependencySource};

use super::frontend_environment;

struct PackageFixture(PathBuf);

impl PackageFixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "etas-default-handler-replay-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            "[package]\nname = \"compiler-api\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
        )
        .unwrap();
        Self(root)
    }

    fn dependency(&self, body: &str) -> ResolvedDependency {
        let source = format!(
            r#"
module compiler_api;
import std.compiler.check;
import std.compiler.CompileError;
import std.compiler.CheckReport;
agent Worker() -> unit {{ return; }}
{body}
"#
        );
        fs::write(self.0.join("src/main.es"), &source).unwrap();
        let output = check(&source, Default::default());
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let artifact = etas_frontend::build_package_metadata_artifact(
            etas_frontend::PackageMetadataBuildInput {
                definition_origin: None,
                package_id: "compiler-api".into(),
                package_version: "0.1.0".into(),
                package_edition: "2026".into(),
                compiler_version: "test".into(),
                source_payload_hash: etas_package::package_source_payload_hash(&self.0).unwrap(),
                manifest_hash: etas_package::package_manifest_hash(&self.0).unwrap(),
                dependency_lock_hash: etas_package::package_dependency_lock_hash(&self.0).unwrap(),
                bins: Vec::new(),
                dependencies: Vec::new(),
                tool_bindings: Vec::new(),
            },
            &output,
        )
        .unwrap();
        let path = etas_package::package_metadata_artifact_path(&self.0);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.bytes).unwrap();
        let index = etas_package::read_package_metadata_artifact(&self.0)
            .unwrap()
            .unwrap();
        ResolvedDependency {
            identity: index.package,
            import_root: "compiler_api".into(),
            source: ResolvedDependencySource::Path {
                path: self.0.to_string_lossy().into_owned(),
                checksum: etas_package::package_source_payload_hash(&self.0).unwrap(),
            },
            dependencies: index.dependencies,
            public_metadata: index.public_metadata,
            effect_metadata: index.effect_metadata,
            tool_bindings: index.tool_bindings,
        }
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn check(source: &str, environment: etas_frontend::ProjectEnvironmentInput) -> ProjectOutput {
    let mut input = ProjectInput::single_source(SourceInput::anonymous(source));
    input.options.entry_policy = EntryPolicy::Optional;
    input.environment = environment;
    Frontend.check_project(input)
}

fn replay(dependency: ResolvedDependency, body: &str) -> ProjectOutput {
    let environment = frontend_environment(PackageEnvironmentMetadata {
        dependencies: vec![dependency],
        ..Default::default()
    })
    .unwrap();
    check(
        &format!(
            r#"
module consumer;
import compiler_api.inspect;
import std.compiler.CheckReport;
import std.compiler.CompileError;
{body}
"#
        ),
        environment,
    )
}

fn summary(output: &ProjectOutput) -> &etas_effects::EffectSummary {
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.as_ref().unwrap();
    let item = checked
        .hir
        .items
        .iter()
        .find_map(|(id, item)| match item {
            etas_hir::HirItem::Flow(flow)
                if checked
                    .hir
                    .symbols
                    .get(flow.symbol)
                    .is_some_and(|symbol| symbol.name == "main") =>
            {
                Some(id)
            }
            _ => None,
        })
        .unwrap();
    &checked.effects.item_effects[&item]
}

#[test]
fn binary_replay_preserves_default_handler_effect_causality() {
    let fixture = PackageFixture::new();
    let dependency = fixture.dependency(
        "public flow inspect() -> CheckReport ![Error<CompileError>] { return check(^Worker); }",
    );
    let defaults = dependency
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "inspect"))
        .unwrap()
        .default_handler_effects
        .as_ref()
        .unwrap();
    assert!(defaults.direct_effects.effects.is_empty());
    assert_eq!(defaults.handlers.len(), 1);
    assert_eq!(defaults.default_actions.effects.len(), 1);
    let output = replay(
        dependency,
        "flow main(report: CheckReport) -> CheckReport { return handle { return inspect(); } with { Code.check(_) => { resume report; } }; }",
    );
    let summary = summary(&output);
    assert!(summary.escaping_effects.is_empty(), "{summary:#?}");
    assert!(summary.default_actions.is_empty());
    assert!(summary.default_handler_effects.is_none());
    assert_eq!(summary.requested_actions.effects.iter().count(), 1);
    assert!(
        !matches!(&summary.support, etas_effects::InterpreterSupport::RequiresHost(host) if host.kinds.contains(&etas_effects::HostRequirementKind::Compiler))
    );
}

#[test]
fn binary_replay_does_not_erase_independent_source_error() {
    let fixture = PackageFixture::new();
    let dependency = fixture.dependency("public flow inspect(error: CompileError, fail: bool) -> CheckReport ![Error<CompileError>] { if fail { perform Error<CompileError>.raise(error); } return check(^Worker); }");
    let output = replay(
        dependency,
        "flow main(report: CheckReport, error: CompileError) -> CheckReport ![Error<CompileError>] { return handle { return inspect(error, true); } with { Code.check(_) => { resume report; } }; }",
    );
    let summary = summary(&output);
    assert_eq!(summary.escaping_effects.effects.iter().count(), 1);
    assert!(matches!(
        summary.escaping_effects.effects.iter().next(),
        Some(etas_effects::Effect::Error(_))
    ));
    assert!(summary.default_actions.is_empty());
}

#[test]
fn missing_default_handler_metadata_is_not_replayed_as_a_pure_call() {
    let fixture = PackageFixture::new();
    let mut dependency = fixture.dependency(
        "public flow inspect() -> CheckReport ![Error<CompileError>] { return check(^Worker); }",
    );
    let summary = dependency
        .public_metadata
        .effect_summaries
        .iter_mut()
        .find(|summary| summary.item.last().is_some_and(|name| name == "inspect"))
        .unwrap();
    summary.default_handler_effects = None;
    let output = replay(
        dependency,
        "flow main() -> CheckReport ![Error<CompileError>] { return inspect(); }",
    );
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn binary_replay_keeps_custom_handling_distinct_from_default_handling() {
    let fixture = PackageFixture::new();
    let dependency = fixture.dependency("public flow inspect(report: CheckReport) -> CheckReport { return handle { return check(^Worker); } with { Code.check(_) => { resume report; } }; }");
    let output = replay(
        dependency,
        "flow main(report: CheckReport) -> CheckReport { return inspect(report); }",
    );
    let summary = summary(&output);
    assert!(summary.escaping_effects.is_empty());
    assert!(summary.default_actions.is_empty());
    assert_eq!(summary.handled_actions.effects.iter().count(), 1);
    assert!(
        matches!(&summary.support, etas_effects::InterpreterSupport::RequiresInterpreterOrchestration(requirements) if !requirements.host.kinds.contains(&etas_effects::HostRequirementKind::Compiler)),
        "{summary:#?}"
    );
}

#[test]
fn duplicate_default_handler_evidence_is_rejected_at_the_import() {
    let fixture = PackageFixture::new();
    let mut dependency = fixture.dependency(
        "public flow inspect() -> CheckReport ![Error<CompileError>] { return check(^Worker); }",
    );
    let summary = dependency
        .public_metadata
        .effect_summaries
        .iter_mut()
        .find(|summary| summary.item.last().is_some_and(|name| name == "inspect"))
        .unwrap();
    let defaults = summary.default_handler_effects.as_mut().unwrap();
    defaults.handlers.push(defaults.handlers[0].clone());
    let output = replay(
        dependency,
        "flow main() -> CheckReport ![Error<CompileError>] { return inspect(); }",
    );
    assert!(output.checked.is_none());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                )
                && diagnostic
                    .message
                    .contains("duplicate default handler contribution")
        })
        .unwrap_or_else(|| panic!("{:?}", output.diagnostics));
    let span = diagnostic.primary.span;
    let source = output
        .sources
        .as_ref()
        .unwrap()
        .files
        .iter()
        .find(|source| source.id == span.source)
        .unwrap();
    assert_eq!(
        source.text[span.range.start.to_usize()..span.range.end.to_usize()].trim(),
        "compiler_api.inspect"
    );
}
