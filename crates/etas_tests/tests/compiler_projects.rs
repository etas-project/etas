mod support;

use std::path::{Path, PathBuf};

use support::{fixture, path_str, run_cli};

#[test]
fn compiler_project_positive_application_fixtures_are_layered_workspaces() {
    let projects = project_roots("compiler/projects/positive");
    assert_eq!(
        projects.len(),
        27,
        "expected 27 positive project-level compiler fixtures, including dependency-exported alias/nominal types, indexed imported alias records, metadata replay, std substrate, full Phase 1 Agent runtime, and multi-agent system projects"
    );

    for project in projects {
        let name = project.file_name().unwrap().to_string_lossy();
        assert!(
            project.join("etas.toml").exists(),
            "{name} should include an etas.toml manifest"
        );
        assert!(
            project.join("etas.lock").exists(),
            "{name} should include an etas.lock for reproducible application package checks"
        );
        assert_package_manifest_matches_spec(&project);
        assert_lockfile_matches_manifest(&project);

        let sources = las_sources(&project);
        assert!(
            sources.len() >= 3,
            "{name} should be a multi-file application fixture"
        );
        assert!(
            project.join("src/app/main.es").exists(),
            "{name} should expose app.main as the application entry module"
        );

        let mut saw_app_import = false;
        let mut saw_public_api = false;
        for source_path in &sources {
            let source = std::fs::read_to_string(source_path).unwrap();
            assert!(
                source.contains("module "),
                "{} should declare its module",
                source_path.display()
            );
            assert!(
                !uses_refinement_type_syntax(&source),
                "{} should not use removed refinement type syntax",
                source_path.display()
            );
            assert!(
                !uses_source_prompt_keyword(&source),
                "{} should not use removed source-level `prompt` syntax; construct `Prompt` values through ordinary flows",
                source_path.display()
            );
            assert!(
                !uses_raw_dependency_import(&source),
                "{} should import logical module paths only; dependency locations belong in etas.toml/etas.lock",
                source_path.display()
            );
            saw_app_import |= source.contains("import app.");
            saw_public_api |= source.contains("public type ") || source.contains("public flow ");
        }

        let main = std::fs::read_to_string(project.join("src/app/main.es")).unwrap();
        assert!(
            main.contains("flow main(args: Array<string>) -> i32"),
            "{name} should use the current command-line entry shape"
        );
        assert!(
            saw_app_import,
            "{name} should exercise cross-module application imports"
        );
        assert!(
            saw_public_api,
            "{name} should expose public module API imported by other modules"
        );
        if name.starts_with("effect_") {
            assert_project_uses_real_effect_surface(&project);
            if name.starts_with("effect_agent") {
                assert_project_uses_agent_call_surface(&project);
            }
        }
    }
}

#[test]
fn compiler_project_metadata_replay_does_not_reanalyze_dependency_source() {
    let root = fixture("compiler/projects/positive/dependency_spec_trace_metadata_replay");
    let dependency_source = std::fs::read_to_string(
        root.join(".etas/vendor/trace-contracts/src/trace_contract/contract.es"),
    )
    .unwrap();
    assert!(
        dependency_source.contains("policy LegacyContract")
            && dependency_source.contains("follows LegacyContract"),
        "metadata replay fixture dependency source should contain obsolete syntax that would fail if reanalyzed"
    );

    let index = std::fs::read_to_string(root.join(".etas/package-index.json")).unwrap();
    for required in [
        "\"trace_specs\"",
        "\"spec_signatures\"",
        "\"spec_impls\"",
        "\"trace_spec_conformances\"",
        "\"trace_spec_summaries\"",
        "\"ExternalBox\"",
        "\"ExternalStringBox\"",
        "\"kind\": \"alias\"",
        "\"kind\": \"applied\"",
    ] {
        assert!(
            index.contains(required),
            "metadata replay fixture should provide dependency fact `{required}` through package metadata"
        );
    }

    let (code, stdout, stderr) =
        run_cli(["etas", "--workspace", path_str(&root), "check", "--all"]);
    assert_eq!(
        code, 0,
        "metadata replay consumer should check from dependency metadata without reanalyzing source\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn compiler_project_positive_application_fixtures_check_as_workspaces() {
    let mut failures = Vec::new();

    for project in project_roots("compiler/projects/positive") {
        let name = project.file_name().unwrap().to_string_lossy();
        let (code, stdout, stderr) =
            run_cli(["etas", "--workspace", path_str(&project), "check", "--all"]);

        if code != 0 || !stdout.contains("checked ") {
            failures.push(format!(
                "{name} should compile as a project-level application fixture\nexit: {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "project-level application fixtures exposed frontend compiler gaps:\n\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn compiler_project_boundary_negative_fixtures_report_diagnostics() {
    for name in [
        "entry_flow_missing",
        "source_root_mismatch",
        "duplicate_dependency_import_root",
    ] {
        let root = fixture(&format!("compiler/projects/negative/{name}"));
        assert_package_manager_owned_fixture_shape(&root, name);
    }

    let cases = [
        ProjectNegativeCase {
            name: "raw_https_import",
            diagnostic: "expected import path segment",
        },
        ProjectNegativeCase {
            name: "raw_relative_import",
            diagnostic: "expected import path segment",
        },
        ProjectNegativeCase {
            name: "entry_return_type_mismatch",
            diagnostic: "entry flow `main` must resolve to `flow main(args: Array[string]) -> i32`",
        },
    ];
    let mut failures = Vec::new();

    for case in cases {
        let root = fixture(&format!("compiler/projects/negative/{}", case.name));
        assert_package_manager_owned_fixture_shape(&root, case.name);

        let (code, _stdout, stderr) = if case.name == "entry_return_type_mismatch" {
            run_cli([
                "etas",
                "--workspace",
                path_str(&root),
                "check",
                "--phase1",
                "--all",
            ])
        } else {
            run_cli(["etas", "--workspace", path_str(&root), "check", "--all"])
        };

        if code != 1 {
            failures.push(format!(
                "{} should fail source-project validation through public `etas check`, got exit {code}",
                case.name
            ));
            continue;
        }
        if !stderr.contains(case.diagnostic) {
            failures.push(format!(
                "{} did not report expected frontend project diagnostic `{}`:\n{stderr}",
                case.name, case.diagnostic
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "project boundary negative fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn compiler_project_negative_fixtures_report_project_diagnostics() {
    let cases = [
        ProjectNegativeCase {
            name: "missing_module",
            diagnostic: "missing imported module `app.missing`",
        },
        ProjectNegativeCase {
            name: "missing_exported_item",
            diagnostic: "missing exported item `missing`",
        },
        ProjectNegativeCase {
            name: "private_import",
            diagnostic: "imported item `helper` from module `app.util` is private",
        },
        ProjectNegativeCase {
            name: "duplicate_top_level_item",
            diagnostic: "duplicate top-level declaration `helper`",
        },
        ProjectNegativeCase {
            name: "duplicate_module",
            diagnostic: "ambiguous module files for `app.main`",
        },
        ProjectNegativeCase {
            name: "module_path_mismatch",
            diagnostic: "module declaration does not match source path; expected `app.main`",
        },
        ProjectNegativeCase {
            name: "wildcard_ambiguity",
            diagnostic: "wildcard import ambiguity for `helper`",
        },
        ProjectNegativeCase {
            name: "dependency_alias_nominal_types",
            diagnostic: "type::TypeMismatch",
        },
    ];

    for case in cases {
        let root = fixture(&format!("compiler/projects/negative/{}", case.name));
        let (code, stdout, stderr) =
            run_cli(["etas", "--workspace", path_str(&root), "check", "--all"]);

        assert_eq!(code, 1, "{name} should fail:\n{stderr}", name = case.name);
        assert!(
            stdout.contains("checked "),
            "{} should still report project check summary on stdout:\n{stdout}",
            case.name
        );
        assert!(
            stderr.contains(case.diagnostic),
            "{} did not report expected diagnostic `{}`:\n{stderr}",
            case.name,
            case.diagnostic
        );
    }
}

#[test]
fn compiler_project_effect_negative_fixtures_report_effect_diagnostics() {
    let cases = [
        "effect_missing_console_row_01",
        "effect_missing_io_error_03",
        "effect_checked_index_04",
        "effect_checked_index_05",
        "effect_agent_console_06",
        "effect_agent_console_07",
        "effect_public_api_08",
        "effect_pipeline_09",
        "effect_entry_10",
    ];
    let mut failures = Vec::new();

    for name in cases {
        let root = fixture(&format!("compiler/projects/negative/{name}"));
        assert_project_uses_real_effect_surface(&root);
        if name.starts_with("effect_agent") {
            assert_project_uses_agent_call_surface(&root);
        }

        let (code, _stdout, stderr) =
            run_cli(["etas", "--workspace", path_str(&root), "check", "--all"]);

        if code != 1 {
            failures.push(format!(
                "{name} should fail effect checking through public `etas check`, got exit {code}"
            ));
            continue;
        }
        if !stderr.contains("effect::EffectOutsideDeclaredRow")
            && !stderr.contains("inferred effect escapes the declared effect row")
        {
            failures.push(format!(
                "{name} did not report the expected effect row diagnostic:\n{stderr}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "project-level effect negative fixture regressions:\n{}",
        failures.join("\n\n")
    );
}

struct ProjectNegativeCase {
    name: &'static str,
    diagnostic: &'static str,
}

fn project_roots(relative: &str) -> Vec<PathBuf> {
    let root = fixture(relative);
    let mut projects = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir() && path.join("etas.toml").exists())
        .collect::<Vec<_>>();
    projects.sort();
    projects
}

fn las_sources(root: &Path) -> Vec<PathBuf> {
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

fn assert_package_manifest_matches_spec(project: &Path) {
    let name = project.file_name().unwrap().to_string_lossy();
    let manifest = std::fs::read_to_string(project.join("etas.toml")).unwrap();
    assert!(
        manifest.contains("[package]\n"),
        "{name} should declare a [package] section"
    );
    assert!(
        manifest.contains("version = \"0.1.0\""),
        "{name} should declare a SemVer package version"
    );
    assert!(
        manifest.contains("edition = \"2026\""),
        "{name} should declare the current language edition"
    );
    assert!(
        manifest.contains("[source]\nroot = \"src\""),
        "{name} should declare the package source root"
    );
    assert!(
        manifest.contains("[dependencies]\nstd = { version = \"0.1\" }"),
        "{name} should declare std as a package dependency"
    );
    assert!(
        manifest.contains("[[bin]]"),
        "{name} should declare a runnable package entry"
    );
    assert!(
        manifest.contains("module = \"app.main\""),
        "{name} should select app.main as the runnable entry module"
    );
    assert!(
        manifest.contains("flow = \"main\""),
        "{name} should select main as the runnable entry flow"
    );
}

fn assert_lockfile_matches_manifest(project: &Path) {
    let name = project.file_name().unwrap().to_string_lossy();
    let lockfile = std::fs::read_to_string(project.join("etas.lock")).unwrap();
    assert!(
        lockfile.contains("version = 1"),
        "{name} lockfile should declare lockfile schema version"
    );
    assert!(
        lockfile.contains("manifest_fingerprint = "),
        "{name} lockfile should include the manifest dependency fingerprint"
    );
    assert!(
        lockfile.contains("name = \"std\"") && lockfile.contains("source = \"builtin\""),
        "{name} lockfile should include the compiler-known std package"
    );
    assert!(
        lockfile.contains("import_root = \"std\""),
        "{name} lockfile should record the std import root explicitly"
    );
}

fn assert_project_uses_real_effect_surface(project: &Path) {
    let name = project.file_name().unwrap().to_string_lossy();
    let sources = las_sources(project)
        .into_iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        sources.contains("Console.stdout_write")
            || sources.contains("Agentic")
            || sources.contains("Error<IndexError>")
            || sources.contains("[0]")
            || sources.contains("std.io.{println}")
            || sources.contains("ProjectEffectAgent.run("),
        "{name} should include explicit effect rows instead of being a pure project fixture"
    );
    assert!(
        sources.contains("std.io.{println}")
            || sources.contains("agent ProjectEffectAgent")
            || sources.contains("[0]"),
        "{name} should exercise real effect-producing syntax such as std.io, agent declarations, or checked indexing"
    );
    assert!(
        !sources.contains("\neffect Web")
            && !sources.contains("\neffect Error[")
            && !sources.contains("ProjectMemorySchema"),
        "{name} should not inline a fake effect registry"
    );
}

fn assert_project_uses_agent_call_surface(project: &Path) {
    let name = project.file_name().unwrap().to_string_lossy();
    let sources = las_sources(project)
        .into_iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        sources.contains("agent ProjectEffectAgent")
            && sources.contains("model = \"gpt-5.5\"")
            && sources.contains("ProjectEffectAgent.run("),
        "{name} should exercise SPEC agent declaration, model config, and agent call lowering"
    );
    assert!(
        sources.contains("![") || project.to_string_lossy().contains("/positive/"),
        "{name} negative agent project should make the omitted or incomplete residual row explicit"
    );
}

fn assert_package_manager_owned_fixture_shape(root: &Path, case_name: &str) {
    let manifest = std::fs::read_to_string(root.join("etas.toml")).unwrap();
    match case_name {
        "raw_https_import" => {
            assert!(
                las_sources(root)
                    .iter()
                    .any(|path| std::fs::read_to_string(path)
                        .unwrap()
                        .contains("import \"https://")),
                "{case_name} should encode a raw URL source import"
            );
        }
        "raw_relative_import" => {
            assert!(
                las_sources(root)
                    .iter()
                    .any(|path| std::fs::read_to_string(path)
                        .unwrap()
                        .contains("import \"../")),
                "{case_name} should encode a relative filesystem source import"
            );
        }
        "entry_flow_missing" => {
            assert!(
                manifest.contains("flow = \"serve\""),
                "{case_name} should encode a manifest entry flow that is absent from app.main"
            );
        }
        "entry_return_type_mismatch" => {
            let main = std::fs::read_to_string(root.join("src/app/main.es")).unwrap();
            assert!(
                main.contains("flow main(args: Array<string>) -> string"),
                "{case_name} should encode an entry return type that is incompatible with runnable main"
            );
        }
        "source_root_mismatch" => {
            assert!(
                manifest.contains("root = \"appsrc\"") && root.join("src/app/main.es").exists(),
                "{case_name} should encode a source root that differs from the physical source tree"
            );
        }
        "duplicate_dependency_import_root" => {
            let duplicate_count = manifest.matches("import = \"tools\"").count();
            assert_eq!(
                duplicate_count, 2,
                "{case_name} should encode two dependencies with the same logical import root"
            );
        }
        _ => panic!("unhandled project boundary negative fixture `{case_name}`"),
    }
}

fn uses_refinement_type_syntax(source: &str) -> bool {
    source
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with("type ") && line.contains(" where "))
}

fn uses_source_prompt_keyword(source: &str) -> bool {
    source.lines().map(str::trim).any(|line| {
        line == "prompt"
            || line.starts_with("prompt ")
            || line.starts_with("prompt{")
            || line.starts_with("prompt {")
    })
}

fn uses_raw_dependency_import(source: &str) -> bool {
    source.lines().map(str::trim).any(|line| {
        line.starts_with("import \"")
            || line.starts_with("import '../")
            || line.starts_with("import \"../")
            || line.starts_with("import \"./")
    })
}
