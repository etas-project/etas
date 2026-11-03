use super::*;
use etas_driver::compiler::prepare_native_definitions;
use etas_frontend::code::definition::StaticReflectionView;

#[test]
fn runnable_project_loads_external_spec_implementation_module() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                r#"
module library.api;
public spec Identity<T> { flow id(input: T) -> T; }
public type Token = i32;
public flow lookup() -> i32 { return 7; }
"#,
            ),
            (
                "library/impls.es",
                r#"
module library.impls;
import library.api.{Identity, Token};
impl Token ~ Identity<Token> { flow id(input: Token) -> Token { return input; } }
"#,
            ),
        ],
        "",
    );
    let _ = origin_metadata::consumer(&workspace, "dep");
    let root = workspace.0.join("consumer-dep");
    std::fs::write(
        root.join("src/app/main.es"),
        r#"
module app.main;
import dep.library.api.{Identity, Token, lookup};
flow main(args: Array<string>) -> i32 {
    if lookup() != 7 { return 2; }
    let value = Token(7);
    let result = value::Identity<Token>.id();
    if result == value { return 0; }
    return 1;
}
"#,
    )
    .unwrap();
    let compiled = etas_driver::compile_runnable_project_with_loaded(
        &origin_metadata::options(&root),
        etas_driver::LoadProjectRequest {
            paths: vec![root],
            all: false,
            entry_policy: EntryPolicy::Runnable,
            flow: None,
            bin: None,
            source_dependency_mode: etas_driver::SourceDependencyMode::MetadataOnly,
        },
        etas_frontend::SnapshotDetailLevel::Full,
    )
    .unwrap();
    assert!(
        compiled.response.output.diagnostics.is_empty(),
        "{:?}",
        compiled.response.output.diagnostics
    );
    assert!(
        compiled
            .loaded
            .loaded
            .input
            .sources
            .iter()
            .any(|source| source.text.contains("module library.impls"))
    );
    let checked = compiled.response.output.checked.unwrap();
    let host = no_host::NoHost::default();
    let result = runtime()
        .block_on(etas_interpreter::Interpreter.run_checked(
            &checked,
            etas_interpreter::api::EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![etas_interpreter::api::InterpValue::Array(vec![].into())],
            &host,
            Default::default(),
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::i32(0))
    );
}

#[test]
fn reflected_flow_can_use_imported_parameterized_behavioral_spec() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                r#"
module library.api;
public spec Identity<T> { flow id(input: T) -> T; }
public type Token = i32;
public type Other = i32;
public flow lookup() -> i32 { return 7; }
"#,
            ),
            (
                "library/other_impls.es",
                r#"
module library.other_impls;
import library.api.{Identity, Other};
impl Other ~ Identity<Other> {
    flow id(input: Other) -> Other { return input; }
}
"#,
            ),
            (
                "library/impls.es",
                r#"
module library.impls;
import library.api.{Identity, Token};
impl Token ~ Identity<Token> {
    flow id(input: Token) -> Token { return input; }
}
"#,
            ),
        ],
        "",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    let implementation_modules = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter()
        .flat_map(|metadata| &metadata.spec_impls)
        .filter(|implementation| implementation.methods.iter().any(|name| name == "id"))
        .map(|implementation| implementation.implementation_module.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        implementation_modules,
        [
            vec!["dep".into(), "library".into(), "impls".into()],
            vec!["dep".into(), "library".into(), "other_impls".into()]
        ]
        .into_iter()
        .collect()
    );
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Identity, Token};
import std.compiler.{check, compile, parse_body, CompileError};
flow apply(input: Token) -> Token { return input::Identity<Token>.id(); }
flow main() -> string ![Error<CompileError>] {
    let good = match parse_body("return input::Identity<Token>.id();") {
        Ok(body) => body, Err(_) => { return "good parse"; },
    };
    if check((^apply).with_body(good)).status != "accepted" { return "good rejected"; }
    let candidate = compile((^apply).with_body(good));
    if candidate(Token(7)) != Token(7) { return "candidate result"; }
    let bad = match parse_body("return input::Identity<i32>.id();") {
        Ok(body) => body, Err(_) => { return "bad parse"; },
    };
    if check((^apply).with_body(bad)).status != "rejected" { return "bad accepted"; }
    return "ok";
}
"#
    .into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.apply".into());
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let expression = *checked.types.reflections.keys().next().unwrap();
    assert!(matches!(
        definitions.catalog().static_reflection(expression),
        Ok(StaticReflectionView::Callable(_))
    ));
    assert_eq!(loaded.loaded.input.sources.len(), 1);
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let result = revisions::execute_using_service(
        checked,
        definitions,
        etas_interpreter::api::ExecutionLimits {
            max_steps: std::num::NonZeroU64::new(20_000),
            ..Default::default()
        },
        service,
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::String("ok".into()))
    );
}

#[test]
fn reflected_instantiated_generic_flow_dispatches_external_spec_method() {
    let workspace = origin_metadata::workspace_with_sources(
        &[
            (
                "library/api.es",
                r#"
module library.api;
public spec Identity<T> { flow id(input: T) -> T; }
public type Token = i32;
"#,
            ),
            (
                "library/impls.es",
                r#"
module library.impls;
import library.api.{Identity, Token};
impl Token ~ Identity<Token> { flow id(input: Token) -> Token { return input; } }
"#,
            ),
        ],
        "",
    );
    let mut loaded = origin_metadata::consumer(&workspace, "dep");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import dep.library.api.{Identity, Token};
import std.compiler.{check, compile, parse_body, CompileError};
flow apply<T ~ Identity<T>>(input: T) -> T { return input::Identity<T>.id(); }
flow main() -> string ![Error<CompileError>] {
    let original: (Token) -> Token ![] = apply;
    let body = match parse_body("return input::Identity<T>.id();") {
        Ok(body) => body, Err(_) => { return "parse"; },
    };
    let draft = (^original).with_body(body);
    if check(draft).status != "accepted" { return "rejected"; }
    let candidate = compile(draft);
    if candidate(Token(7)) != Token(7) { return "candidate"; }
    return "ok";
}
"#
    .into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.apply".into());
    let checked = origin_metadata::check(&loaded);
    let definitions = prepare_native_definitions(&loaded, &checked, Default::default()).unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let result = revisions::execute_using_service(
        checked,
        definitions,
        etas_interpreter::api::ExecutionLimits {
            max_steps: std::num::NonZeroU64::new(20_000),
            ..Default::default()
        },
        service,
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&etas_interpreter::api::InterpValue::String("ok".into()))
    );
}

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
