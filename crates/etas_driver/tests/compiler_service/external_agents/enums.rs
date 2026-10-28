use super::*;

const LIBRARY: &str = r#"
module library.api;
public spec Allowed;
public spec Strong ~ Allowed;
public spec Within<T>;
public spec StrongWithin<T> ~ Within<Array<T>>;
public type Value = i32;
impl Value ~ Strong;
impl Value ~ StrongWithin<i32>;
public enum Box<T ~ Allowed> { Empty, Named { value: T }, Positional(T), }
public enum Scoped<T ~ Within<Array<i32>>> { Entry { value: T }, }
public enum Tree<T> { Leaf(T), Branch { children: Array<Tree<T>> }, }
public enum Directory { Entries(Array<Entry>), }
public enum Entry { Parent(Option<Directory>), }
public agent Remote(value: bool) -> bool { return value; }
public flow lookup() -> i32 { return 7; }
"#;

#[test]
fn metadata_only_recursive_enums_and_constructor_aliases_preserve_layouts() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    std::fs::remove_dir_all(workspace.0.join("library/src")).unwrap();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{Box as B, Value, Tree, Directory, Entry, Strong, Scoped};
flow box_value<T ~ Strong>(value: T) -> B<T> { return B.Named { value }; }
flow main() -> bool {
    let boxed: B<Value> = box_value(Value(42));
    let tree: Tree<i32> = Tree.Branch<i32> { children = [Tree.Leaf(7)] };
    let directory: Directory = Directory.Entries([Entry.Parent(None)]);
    let value = Value(8);
    let positional: B<Value> = B.Positional(value);
    let scoped: Scoped<Value> = Scoped.Entry { value = Value(5) };
    if !match scoped { Scoped.Entry { value } => value == Value(5) } { return false; }
    if !match boxed { B.Named { value } => value == Value(42), _ => false } { return false; }
    if !match positional { B.Positional(item) => item == value, _ => false } { return false; }
    if !match directory { Directory.Entries(entries) => entries.len() == 1 } { return false; }
    return match tree {
        Tree.Leaf(_) => false,
        Tree.Branch { children } => match children.get(0) {
            Some(Tree.Leaf(n)) => n == 7,
            _ => false,
        },
    };
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            Default::default(),
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(host.calls(), 0);
}

#[test]
fn metadata_only_enum_constructors_enforce_bounds_fields_payloads_and_coverage() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    for (body, expected) in [
        (
            "let value: Box<i32> = Box.Empty;",
            "does not satisfy spec bound",
        ),
        (
            "let value: Box<i32> = Box.Named { value = 42 };",
            "does not satisfy spec bound",
        ),
        (
            "let value: Box<i32> = Box.Positional(42);",
            "does not satisfy spec bound",
        ),
        (
            "let value: Box<Value> = Box.Named { wrong = Value(1) };",
            "unknown variant field",
        ),
        (
            "let value: Box<Value> = Box.Named {};",
            "missing variant field",
        ),
        (
            "let value: Box<Value> = Box.Named { value = Value(1), value = Value(2) };",
            "duplicate variant field",
        ),
        (
            "let value: Box<Value> = Box.Named(Value(1));",
            "requires brace construction",
        ),
        (
            "let value: Box<Value> = Box.Positional { value = Value(1) };",
            "cannot use named-field construction",
        ),
        (
            "let value: Tree<i32> = Tree.Leaf(\"wrong\");",
            "does not match",
        ),
        (
            "let value: Tree<bool> = Tree.Leaf(true); match value { Tree.Leaf(true) => (), Tree.Branch { children: _ } => () };",
            "does not cover all values",
        ),
    ] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import first.library.api.{{Box, Value, Tree}}; flow main() -> unit {{ {body} return; }}"
        );
        let output = etas_frontend::Frontend.check_project(loaded.loaded.input);
        assert!(output.checked.is_none(), "accepted {body}");
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{body}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn malformed_binary_enum_layouts_are_rejected_at_the_resolved_import() {
    for corruption in [
        "missing",
        "owner",
        "unbound",
        "missing-super",
        "cyclic-super",
    ] {
        let workspace = origin_metadata::workspace_with_source(LIBRARY);
        let initial = origin_metadata::consumer(&workspace, "first");
        let root = initial.package_root.unwrap();
        let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
        let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
            &artifact,
            &std::fs::read(&artifact).unwrap(),
        )
        .unwrap();
        let declaration = metadata
            .public_metadata
            .enums
            .iter_mut()
            .find(|declaration| declaration.path.last().is_some_and(|name| name == "Box"))
            .unwrap();
        match corruption {
            "missing" => declaration.ty = None,
            "owner" => declaration.ty.as_mut().unwrap().path.push("Other".into()),
            "unbound" => {
                declaration.ty.as_mut().unwrap().fields[1].ty.fields[0].ty =
                    etas_package_metadata::Type {
                        kind: etas_package_metadata::TypeKind::Var,
                        name: "Undeclared".into(),
                        ..Default::default()
                    }
            }
            "missing-super" => metadata
                .public_metadata
                .spec_signatures
                .iter_mut()
                .find(|spec| spec.path.last().is_some_and(|name| name == "Strong"))
                .unwrap()
                .super_specs[0]
                .spec
                .push("Missing".into()),
            "cyclic-super" => {
                let spec = metadata
                    .public_metadata
                    .spec_signatures
                    .iter_mut()
                    .find(|spec| spec.path.last().is_some_and(|name| name == "Strong"))
                    .unwrap();
                spec.super_specs[0].spec = spec.path.clone();
            }
            _ => unreachable!(),
        }
        let sections = etas_package_metadata::package_metadata_to_sections(&metadata).unwrap();
        let bytes = etas_package_metadata::encode_metadata_artifact(&header, sections).unwrap();
        etas_package_metadata::write_metadata_artifact_file(&artifact, &bytes).unwrap();
        etas_package::materialize_package(etas_package::MaterializePackageOptions {
            package_root: root.clone(),
            source_config: Some(etas_package::PackageSourceConfig::read_project(&root).unwrap()),
        })
        .unwrap();
        etas_package::lock_package(etas_package::LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let loaded = etas_driver::load_project(
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
        .unwrap();
        assert_eq!(loaded.loaded.input.sources.len(), 1);
        let source = loaded.loaded.input.sources[0].clone();
        let output = Frontend.check_project(loaded.loaded.input);
        assert!(output.checked.is_none(), "accepted {corruption}");
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code
                    == etas_core::DiagnosticCode::Type(
                        etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                    )
            })
            .unwrap();
        assert_eq!(diagnostic.primary.span.source, source.id);
        let range = diagnostic.primary.span.range;
        assert!(
            source.text[range.start.0 as usize..range.end.0 as usize].contains("first.library.api"),
            "{:?}",
            diagnostic
        );
    }
}

#[test]
fn a_compiled_revision_can_construct_and_match_metadata_only_recursive_enums() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Tree;
import std.compiler.{compile, parse_body, CompileError};
agent Worker(value: Tree<i32>) -> Tree<i32> ![] { return value; }
flow wrap(value: Tree<i32>) -> Tree<i32> { return Tree.Branch<i32> { children = [value] }; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return wrap(value);") {
        Ok(body) => body,
        Err(_) => { return false; },
    };
    let next = compile((^Worker).with_body(body));
    let value = next(Tree.Leaf(7));
    return match value {
        Tree.Leaf(_) => false,
        Tree.Branch { children } => match children.get(0) {
            Some(Tree.Leaf(n)) => n == 7,
            _ => false,
        },
    };
}

"#
    .into();
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

#[test]
fn metadata_only_enum_checkpoint_preserves_pending_constructor_identity() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.Tree;
import std.runtime.checkpoint;
flow pause() -> Array<Tree<i32>> { checkpoint("enum-field"); return [Tree.Leaf(7)]; }
flow main() -> bool {
    let value: Tree<i32> = Tree.Branch<i32> { children = pause() };
    return match value { Tree.Leaf(_) => false, Tree.Branch { children } => children.len() == 1 };
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let host = no_host::NoHost::with_checkpoint();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            Default::default(),
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    let artifact =
        etas_interpreter::api::codec::checkpoint_artifact_json(&[], "main", &result.checkpoints[0])
            .unwrap();
    let checkpoint =
        etas_interpreter::api::codec::checkpoint_from_json(&artifact, &checked).unwrap();
    let resumed = runtime()
        .block_on(etas_interpreter::Interpreter.resume_checkpoint(
            &checked,
            &checkpoint,
            &host,
            Default::default(),
        ))
        .unwrap();
    assert!(resumed.diagnostics.is_empty(), "{:?}", resumed.diagnostics);
    assert_eq!(
        resumed.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    let mut tampered = artifact;
    fn erase_constructor(value: &mut serde_json::Value) -> bool {
        if value.get("kind").and_then(serde_json::Value::as_str) == Some("record_field") {
            value["variant_symbol"] = serde_json::Value::Null;
            return true;
        }
        match value {
            serde_json::Value::Object(fields) => fields.values_mut().any(erase_constructor),
            serde_json::Value::Array(elements) => elements.iter_mut().any(erase_constructor),
            _ => false,
        }
    }
    assert!(erase_constructor(&mut tampered));
    let error =
        etas_interpreter::api::codec::checkpoint_from_json(&tampered, &checked).unwrap_err();
    assert!(
        error.to_string().contains("named enum continuation"),
        "{error}"
    );
    assert_eq!(host.calls(), 0);
}
