use super::*;

const SOURCES: &[(&str, &str)] = &[
    (
        "library/api.es",
        r#"module library.api;
import library.internal.increment;
import library.types.UserId;
public flow apply(value: UserId<i32>) -> UserId<i32> ![] {
    return increment(value);
}
public flow lookup() -> i32 { return 7; }
"#,
    ),
    (
        "library/internal.es",
        r#"module library.internal;
import library.types.UserId;
public flow increment(value: UserId<i32>) -> UserId<i32> ![] {
    return UserId<i32> { value = value.value + 1 };
}
"#,
    ),
    (
        "library/types.es",
        "module library.types; public type UserId<T> = { value: T };",
    ),
];

pub(super) fn compile(root: &std::path::Path) -> etas_driver::CompileRunnableProjectResponse {
    etas_driver::compile_runnable_project_with_loaded(
        &origin_metadata::options(root),
        etas_driver::LoadProjectRequest {
            paths: vec![root.to_owned()],
            all: true,
            entry_policy: EntryPolicy::Optional,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        etas_frontend::SnapshotDetailLevel::Full,
    )
    .unwrap()
}

pub(super) fn execute(
    compiled: etas_driver::CompileRunnableProjectResponse,
    sources: usize,
    expected: i32,
) {
    let result = execute_result(compiled, sources);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(expected))
    );
}

pub(super) fn execute_result(
    compiled: etas_driver::CompileRunnableProjectResponse,
    sources: usize,
) -> etas_interpreter::api::RunResult {
    assert!(
        compiled.response.output.diagnostics.is_empty(),
        "{:?}",
        compiled.response.output.diagnostics
    );
    let checked = compiled.response.output.checked.unwrap();
    assert_eq!(
        checked.sources.sources.len(),
        sources,
        "requirements: {:?}; source paths: {:?}",
        compiled.response.output.runtime_source_requirements,
        compiled
            .loaded
            .loaded
            .sources
            .iter()
            .map(|(path, _)| path)
            .collect::<Vec<_>>(),
    );
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &compiled.loaded,
        &checked,
        Default::default(),
    )
    .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&compiled.loaded)
        .unwrap();
    revisions::execute_using_service(
        checked,
        definitions,
        etas_interpreter::api::ExecutionLimits {
            max_steps: std::num::NonZeroU64::new(4_000),
            ..Default::default()
        },
        service,
        true,
    )
}

#[test]
fn dependency_overlays_keep_canonical_imports_and_type_only_metadata_across_aliases() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    for (alias, namespace) in [("first", "first.library"), ("library", "library")] {
        let root = origin_metadata::consumer(&workspace, alias)
            .package_root
            .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
import {namespace}.api.apply;
import {namespace}.types.UserId;
import std.compiler.{{compile, CompileError}};
agent Worker(value: UserId<i32>) -> UserId<i32> ![] {{ return apply(value); }}
flow main() -> i32 ![Error<CompileError>] {{
    if (^apply).body != None {{ return -1; }}
    let revised = compile(^Worker);
    return revised(UserId<i32> {{ value = 18 }}).value;
}}
"#
            ),
        )
        .unwrap();
        let compiled = compile(&root);
        assert!(
            compiled
                .loaded
                .loaded
                .sources
                .iter()
                .all(|(path, _)| !path.ends_with("library/types.es")),
            "type-only dependencies must remain metadata-only for alias {alias}"
        );
        execute(compiled, 3, 19);
    }
}

#[test]
fn dependency_packages_with_the_same_source_module_keep_distinct_overlay_identities() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 ![] { return 7; }",
    );
    let other = workspace.0.join("other");
    std::fs::create_dir_all(other.join("src/library")).unwrap();
    std::fs::write(
        other.join("etas.toml"),
        "[package]\nname = \"other-library\"\nversion = \"0.1.0\"\nedition = \"2026\"\n",
    )
    .unwrap();
    std::fs::write(
        other.join("src/library/api.es"),
        "module library.api; public flow lookup() -> i32 ![] { return 11; }",
    )
    .unwrap();
    etas_driver::build_package_metadata(
        &origin_metadata::options(&other),
        etas_driver::PackageMetadataRequest {
            package_root: other,
            bin: None,
        },
    )
    .unwrap();
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("etas.toml"),
        r#"
[package]
name = "consumer-first"
version = "0.1.0"
edition = "2026"
[dependencies]
first = { package = "origin-library", version = "0.1.0", import = "first", path = "../library" }
second = { package = "other-library", version = "0.1.0", import = "second", path = "../other" }
[[bin]]
name = "app"
module = "app.main"
flow = "main"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup as first;
import second.library.api.lookup as second;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return first() + second(); }
flow main() -> i32 ![Error<CompileError>] {
    if (^first).body != None || (^second).body != None { return -1; }
    let revised = compile(^Worker);
    return revised();
}
"#,
    )
    .unwrap();
    etas_driver::update_package(
        &origin_metadata::options(&root),
        etas_driver::PackageUpdateRequest {
            package_root: root.clone(),
        },
    )
    .unwrap();
    execute(compile(&root), 3, 18);
}

#[test]
fn retained_definition_analysis_does_not_materialize_its_dependency_runtime_sources() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    let manifest = root.join("etas.toml");
    let content = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        format!("{content}\n[reflection]\nretain_bodies = [\"app.main.Worker\"]\n"),
    )
    .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.apply;
import first.library.types.UserId;
agent Worker(value: UserId<i32>) -> UserId<i32> ![] { return apply(value); }
flow main() -> i32 {
    if (^Worker).body == None { return -1; }
    return 19;
}
"#,
    )
    .unwrap();
    execute(compile(&root), 1, 19);
}

#[test]
fn reflecting_an_external_flow_does_not_materialize_hidden_runtime_source() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public flow lookup() -> i32 ![] { return 7; }",
    );
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.lookup;
flow main() -> i32 {
    if (^lookup).body != None { return -1; }
    return 19;
}
"#,
    )
    .unwrap();
    execute(compile(&root), 1, 19);
}

#[test]
fn package_local_namespace_bindings_do_not_leak_into_the_consumer() {
    let workspace = origin_metadata::workspace_with_sources(SOURCES, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import library.api.apply;
flow main() -> i32 { return 0; }
"#,
    )
    .unwrap();
    let compiled = compile(&root);
    assert_eq!(compiled.loaded.loaded.sources.len(), 1);
    let output = &compiled.response.output;
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Syntax(etas_core::SyntaxDiagnosticCode::InvalidItem)
                && diagnostic
                    .message
                    .contains("missing imported module `library.api.apply`")
                && diagnostic.primary.span.source == compiled.loaded.loaded.input.sources[0].id
        }),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn dependency_overlay_resolves_grouped_and_wildcard_imports_in_its_package_namespace() {
    let sources = SOURCES
        .iter()
        .map(|(path, source)| {
            (
                *path,
                if *path == "library/api.es" {
                    r#"module library.api;
import library.internal.{increment};
import library.types.*;
public flow apply(value: UserId<i32>) -> UserId<i32> ![] { return increment(value); }
public flow lookup() -> i32 { return 7; }
"#
                } else {
                    *source
                },
            )
        })
        .collect::<Vec<_>>();
    let workspace = origin_metadata::workspace_with_sources(&sources, "");
    let root = origin_metadata::consumer(&workspace, "first")
        .package_root
        .unwrap();
    std::fs::write(
        root.join("src/app/main.es"),
        r#"module app.main;
import first.library.api.apply;
import first.library.types.UserId;
import std.compiler.{compile, CompileError};
agent Worker() -> i32 ![] { return apply(UserId<i32> { value = 18 }).value; }
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return revised();
}
"#,
    )
    .unwrap();
    execute(compile(&root), 3, 19);
}

#[test]
fn dependency_reexports_materialize_the_defining_module_without_exposing_its_body() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                "module library.api; public import library.internal.increment as apply; public flow lookup() -> i32 { return 7; }",
            ),
            (
                "library/internal.es",
                "module library.internal; public flow increment(value: i32) -> i32 ![] { return value + 1; }",
            ),
        ],
        "",
    );
    for (import, callable) in [
        ("import first.library.api.apply;", "apply"),
        ("import first.library.api.{apply};", "apply"),
        ("import first.library.api.*;", "apply"),
        ("import first.library.api as Api;", "Api.apply"),
    ] {
        let root = origin_metadata::consumer(&workspace, "first")
            .package_root
            .unwrap();
        std::fs::write(
            root.join("src/app/main.es"),
            format!(
                r#"module app.main;
{import}
import std.compiler.{{compile, CompileError}};
agent Worker() -> i32 ![] {{ return {callable}(18); }}
flow main() -> i32 ![Error<CompileError>] {{
    if (^{callable}).body != None {{ return -1; }}
    let revised = compile(^Worker);
    return revised();
}}
"#
            ),
        )
        .unwrap();
        let compiled = compile(&root);
        assert!(
            compiled.response.output.diagnostics.is_empty(),
            "{import}: {:?}",
            compiled.response.output.diagnostics
        );
        assert!(
            compiled
                .loaded
                .loaded
                .sources
                .iter()
                .all(|(path, _)| !path.ends_with("library/api.es"))
        );
        execute(compiled, 2, 19);
    }
}
