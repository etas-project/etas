use super::*;
use etas_driver::compiler::prepare_native_definitions;
use etas_frontend::code::definition::StaticReflectionView;

#[path = "external_spec_methods/control_self.rs"]
mod control_self;
#[path = "external_spec_methods/frozen_self.rs"]
mod frozen_self;
#[path = "external_spec_methods/implementation_replay.rs"]
mod implementation_replay;
#[path = "external_spec_methods/implicit_self.rs"]
mod implicit_self;
#[path = "external_spec_methods/invalid_self_metadata.rs"]
mod invalid_self_metadata;
#[path = "external_spec_methods/runtime_self.rs"]
mod runtime_self;
#[path = "external_spec_methods/type_method_identity.rs"]
mod type_method_identity;

#[test]
fn external_spec_method_reflection_retains_trace_spec_selection() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Quiet: trace = -Console.stdout_write;
public agent Worker;
impl agent Worker ~ Quiet { public flow run() -> bool { return true; } }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Worker as Remote, Quiet as Silent};
flow main() -> unit { let method = ^Remote::Silent.run; return; }
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    let StaticReflectionView::Callable(method) =
        definitions.catalog().static_reflection(expression).unwrap()
    else {
        panic!("callable");
    };
    let spec = method
        .id
        .as_method()
        .unwrap()
        .implementation()
        .selected_spec()
        .unwrap();
    assert_eq!(spec.declaration(), ["Quiet"]);
    assert!(definitions.catalog().item_owner(&method.id).is_none());
}

#[test]
fn external_named_type_rejects_undeclared_parameter_after_binary_decode_at_import_span() {
    let workspace = origin_metadata::workspace_with_source(
        "module library.api; public type Wrapper<T> = { value: T }; public flow lookup() -> i32 { return 7; }",
    );
    let initial = origin_metadata::consumer(&workspace, "dep");
    let root = initial.package_root.unwrap();
    let artifact = etas_package::package_metadata_artifact_path(&workspace.0.join("library"));
    let (header, mut metadata) = etas_package_metadata::package_metadata_from_artifact(
        &artifact,
        &std::fs::read(&artifact).unwrap(),
    )
    .unwrap();
    let wrapper = metadata
        .public_metadata
        .types
        .iter_mut()
        .find(|ty| ty.path.last().is_some_and(|name| name == "Wrapper"))
        .unwrap();
    assert_eq!(wrapper.generic_params.len(), 1);
    wrapper.generic_params.clear();
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
    let output = Frontend.check_project(loaded.loaded.input.clone());
    assert!(output.checked.is_none());
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts,
                )
                && diagnostic.message.contains("T")
        })
        .expect("undeclared type parameter must be rejected");
    let source = &loaded.loaded.input.sources[0];
    assert_eq!(diagnostic.primary.span.source, source.id);
    let range = diagnostic.primary.span.range;
    assert!(source.text[range.start.0 as usize..range.end.0 as usize].contains("dep.library.api"));
}

#[test]
fn external_spec_method_reflection_selects_published_implementation_and_type_arguments() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Convert<I, O>: callable I => O ![];
public agent Worker;
impl agent Worker ~ Convert<bool, bool> {
    public flow run(value: bool) -> bool { return !value; }
}
impl agent Worker ~ Convert<string, string> {
    public flow run(value: string) -> string { return value; }
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.Worker as Remote;
import dep.library.api.Convert as Contract;
flow main() -> bool {
    let first = ^Remote::Contract<bool, bool>.run;
    let second = ^Remote::Contract<string, string>.run;
    return first.id != second.id;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for expression in checked.types.reflections.keys() {
        let StaticReflectionView::Callable(callable) = definitions
            .catalog()
            .static_reflection(*expression)
            .unwrap()
        else {
            panic!("method");
        };
        assert_eq!(
            callable
                .id
                .as_method()
                .unwrap()
                .implementation()
                .selected_spec()
                .unwrap()
                .declaration(),
            ["Convert"]
        );
        assert!(definitions.catalog().item_owner(&callable.id).is_none());
        ids.insert(callable.id.clone());
    }
    assert_eq!(ids.len(), 2);
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            etas_interpreter::api::RunOptions {
                code_definitions: Some(Arc::new(
                    etas_interpreter::api::code::PreparedCodeDefinitions::new(definitions),
                )),
                ..Default::default()
            },
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(host.calls(), 0);
    assert_eq!(loaded.loaded.input.sources.len(), 1);
}

#[test]
fn external_spec_method_type_arguments_preserve_nominal_and_structural_identity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Convert<I, O>: callable I => O ![];
public type First = bool;
public type Second = bool;
public alias FirstAlias = First;
public type Wrapped<T, Ignored> = { value: T };
public alias Maybe<T> = Option<T>;
public agent Worker;
impl agent Worker ~ Convert<First, First> {
    public flow run(value: First) -> First { return value; }
}
impl agent Worker ~ Convert<Second, Second> {
    public flow run(value: Second) -> Second { return value; }
}
impl agent Worker ~ Convert<Option<bool>, Option<bool>> {
    public flow run(value: Option<bool>) -> Option<bool> { return value; }
}
impl agent Worker ~ Convert<{ value: bool }, { value: bool }> {
    public flow run(value: { value: bool }) -> { value: bool } { return value; }
}
impl agent Worker ~ Convert<Wrapped<bool, string>, Wrapped<bool, string>> {
    public flow run(value: Wrapped<bool, string>) -> Wrapped<bool, string> { return value; }
}
impl agent Worker ~ Convert<Maybe<string>, Maybe<string>> {
    public flow run(value: Maybe<string>) -> Maybe<string> { return value; }
}
impl agent Worker ~ Convert<Deque<bool>, Deque<bool>> {
    public flow run(value: Deque<bool>) -> Deque<bool> { return value; }
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Worker as Remote, Convert as Contract, First as Primary, Second as Other, FirstAlias, Wrapped, Maybe};
alias LocalAlias = Primary;
flow main() -> bool {
    let first = ^Remote::Contract<Primary, Primary>.run;
    let aliased = ^Remote::Contract<LocalAlias, FirstAlias>.run;
    let other = ^Remote::Contract<Other, Other>.run;
    let option = ^Remote::Contract<Option<bool>, Option<bool>>.run;
    let record = ^Remote::Contract<{ value: bool }, { value: bool }>.run;
    let wrapped = ^Remote::Contract<Wrapped<bool, string>, Wrapped<bool, string>>.run;
    let maybe = ^Remote::Contract<Maybe<string>, Option<string>>.run;
    let deque = ^Remote::Contract<Deque<bool>, Deque<bool>>.run;
    return first.id == aliased.id && first.id != other.id
        && option.id != record.id && wrapped.id != maybe.id && deque.id != option.id;
}
"#.into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    for arguments in [
        "bool, bool",
        "Primary, Other",
        "Option<i32>, Option<i32>",
        "Wrapped<string, string>, Wrapped<string, string>",
        "Wrapped<bool, bool>, Wrapped<bool, bool>",
    ] {
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import dep.library.api.{{Worker as Remote, Convert as Contract, First as Primary, Second as Other, Wrapped}}; flow main() -> unit {{ let method = ^Remote::Contract<{arguments}>.run; return; }}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none(), "{arguments}");
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::TypeMismatch)),
            "{arguments}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn external_spec_method_rejects_ambiguous_wrong_and_private_selection() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Convert<I, O>: callable I => O ![];
public spec Other: callable bool => bool ![];
public spec Hidden { flow hidden() -> bool; }
public agent Worker;
impl agent Worker ~ Convert<bool, bool> {
    public flow run(value: bool) -> bool { return value; }
}
impl agent Worker ~ Convert<string, string> {
    public flow run(value: string) -> string { return value; }
}
impl agent Worker ~ Hidden { private flow hidden() -> bool { return false; } }
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    for selection in [
        "Convert.run",
        "Other.run",
        "Hidden.hidden",
        "Convert<bool>.run",
        "Convert<bool, bool>.missing",
    ] {
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import dep.library.api.{{Worker as Remote, Convert, Other, Hidden}}; flow main() -> unit {{ let method = ^Remote::{selection}; return; }}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none(), "{selection}");
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::TypeMismatch)),
            "{selection}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn external_spec_method_function_arguments_compare_checked_effect_identities() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
public spec Convert<T> { flow adapt(value: T) -> T; }
public alias Writer = () -> unit ![Console.stdout_write, Error<i32>];
public agent Worker;
impl agent Worker ~ Convert<Writer> {
    public flow adapt(value: Writer) -> Writer { return value; }
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Worker as Remote, Convert as Contract, Writer};
import std.effects.Console as Logging;
alias LocalWriter = () -> unit ![Error<i32>, Logging.stdout_write];
flow main() -> bool {
    let first = ^Remote::Contract<Writer>.adapt;
    let second = ^Remote::Contract<LocalWriter>.adapt;
    return first.id == second.id;
}
"#
    .into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    for (declaration, row) in [
        ("", "Console.stderr_write, Error<i32>"),
        ("", "Console.stdout_write, Error<string>"),
        (
            "effect Console { action stdout_write(text: string) -> unit; }",
            "Console.stdout_write, Error<i32>",
        ),
    ] {
        loaded.loaded.input.sources[0].text = format!(
            "module app.main; import dep.library.api.{{Worker as Remote, Convert as Contract}}; {declaration} flow main() -> unit {{ let method = ^Remote::Contract<() -> unit ![{row}]>.adapt; return; }}"
        );
        let output = Frontend.check_project(loaded.loaded.input.clone());
        assert!(output.checked.is_none(), "{row}");
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::TypeMismatch)),
            "{row}: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn external_spec_selected_view_keeps_the_canonical_inherent_body() {
    let workspace = inherent_callable_spec_package();
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Worker as Remote, Convert as Contract};
flow main() -> bool {
    let canonical = ^Remote.run;
    let selected = ^Remote::Contract<bool>.run;
    let same_body = match canonical.body {
        Some(body) => match selected.body { Some(other) => body.text == other.text, None => false, },
        None => false,
    };
    return canonical.id != selected.id && same_body;
}
"#.into();
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let mut selected = false;
    let mut canonical = false;
    for expression in checked.types.reflections.keys() {
        let StaticReflectionView::Callable(method) = definitions
            .catalog()
            .static_reflection(*expression)
            .unwrap()
        else {
            panic!("callable");
        };
        assert!(definitions.catalog().item_owner(&method.id).is_none());
        if method
            .id
            .as_method()
            .unwrap()
            .implementation()
            .selected_spec()
            .is_some()
        {
            selected = true;
        } else {
            canonical = true;
        }
    }
    assert!(selected && canonical);
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
}

fn inherent_callable_spec_package() -> origin_metadata::Workspace {
    origin_metadata::workspace_with_source_config(
        r#"
module library.api;
public spec Convert<T>: callable T => T ![];
public agent Worker;
impl agent Worker {
    public flow run(value: bool) -> bool ~ Convert<bool> { return !value; }
}
public flow lookup() -> i32 { return 7; }
"#,
        "[reflection]\nexport_bodies = [\"library.api.Worker.run\"]\n",
    )
}

#[test]
fn compiled_body_preserves_external_spec_selected_body_ownership() {
    let workspace = inherent_callable_spec_package();
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Worker as Remote, Convert as Contract};
import std.compiler.{compile, parse_body, CompileError};
agent Target(value: bool) -> bool ![] { return value && true; }
flow main() -> bool ![Error<CompileError>] {
    let syntax = match parse_body("let method = ^Remote::Contract<bool>.run; match method.body { Some(body) => { return !value; }, None => { return value; }, }") {
        Ok(syntax) => syntax, Err(_) => { return false; },
    };
    let next = compile((^Target).with_body(syntax));
    return next(false);
}
"#.into();
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.types.reflections.len(), 1);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let result =
        revisions::execute_prepared((checked, definitions), Some(600), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::Bool(true))
    );
    assert_eq!(loaded.loaded.input.sources.len(), 1);
}
