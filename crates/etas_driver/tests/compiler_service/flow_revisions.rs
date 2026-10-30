use super::*;

#[test]
fn metadata_only_callback_loop_preserves_origin_and_rejects_unproved_outcomes() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide() -> unit; }
public effect Other { action decide() -> unit; }
public flow each(f: () -> unit ![Choice.decide], values: Array<bool>) -> unit ![] {
    handle { for value in values limit Iterations(8) { f(); } } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow extra(f: () -> unit ![Choice.decide], values: Array<bool>) -> unit ![] {
    handle { perform Other.decide(); } with { Other.decide() => { resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    let descriptors = loaded
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter()
        .find_map(|origin| origin.descriptors.as_ref())
        .unwrap();
    let header = descriptors
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["each"])
        })
        .unwrap();
    assert!(header.analysis.outcomes.is_none());
    assert!(matches!(
        header.analysis.trace.map(|id| &descriptors.graph.traces[id.0 as usize]),
        Some(etas_code::descriptor::TraceInfo::Widened { parameter_calls, .. })
            if parameter_calls.len() == 1 && parameter_calls[0].position == 0
    ));
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{each, extra, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide], values: Array<bool>) -> unit ![] { each(f, values); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("each(f, values); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    if report.status != "rejected" { return "loop accepted without proof"; }
    let changed = match parse_body("extra(f, values); return;") { Ok(body) => body, Err(_) => { return "parse extra"; }, };
    if check((^normalize).with_body(changed)).status != "rejected" { return "extra action accepted"; }
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String(
            "code::UnprovenParametricDeterminism".into()
        ))
    );
}

#[test]
fn compiled_dependency_callback_respects_branch_choice() {
    use etas_host::console::ConsoleOperation;
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, RunOptions, code::PreparedCodeDefinitions},
    };
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide() -> unit; }
public flow choose(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide], enabled: bool) -> unit ![] {
    handle { if enabled { f(); } else { g(); } } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{choose, Choice};
import std.compiler.{compile, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide], enabled: bool) -> unit ![] { choose(f, g, enabled); return; }
flow f() -> unit ![Choice.decide] { perform Choice.decide(); return; }
flow g() -> unit ![Choice.decide] { return; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("choose(f, g, enabled); return;") { Ok(body) => body, Err(_) => { return false; }, };
    let candidate = compile((^normalize).with_body(body));
    candidate(f, g, true);
    candidate(f, g, false);
    return true;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let host = CompilerHost::new(no_host::NoHost::with_console(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    for (effect, action) in [("Code", "compile"), ("Console", "stdout_write")] {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow(effect, action));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    assert_eq!(
        host.inner().console_events(),
        vec![ConsoleOperation::WriteStdout {
            text: "handled".into(),
            newline: true
        }]
    );
}

#[test]
fn metadata_only_reflection_preserves_callback_branch_choices() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide() -> unit; }
public flow choose(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide], enabled: bool) -> unit ![] {
    handle { if enabled { f(); } else { g(); } } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow both(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide], enabled: bool) -> unit ![] {
    handle { f(); g(); } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{choose, both, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide], enabled: bool) -> unit ![] { choose(f, g, enabled); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("choose(f, g, enabled); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    let changed = match parse_body("both(f, g, enabled); return;") { Ok(body) => body, Err(_) => { return "parse both"; }, };
    if check((^normalize).with_body(changed)).status != "rejected" { return "sequential calls accepted"; }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut incomplete = loaded.clone();
    let set = incomplete
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let root = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["choose"])
        })
        .unwrap()
        .analysis
        .outcomes
        .unwrap();
    let etas_code::descriptor::TraceInfo::Choice(branches) = &mut set.graph.traces[root.0 as usize]
    else {
        panic!("expected outcome choice");
    };
    assert_eq!(branches.len(), 3);
    branches.pop();
    let checked = origin_metadata::check(&incomplete);
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &incomplete,
        &checked,
        Default::default(),
    )
    .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_ne!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_callback_preserves_generic_nominal_selector() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public type Region<T> = { value: T };
public effect Choice { action decide<T>() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide<Region<i32>>]) -> unit ![] {
    handle { f(); } with { Choice.decide<Region<i32>>() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice, Region};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide<Region<i32>>]) -> unit ![] { wrapped(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut forged = loaded.clone();
    let set = forged
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let requested = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["wrapped"])
        })
        .unwrap()
        .analysis
        .effects
        .requested
        .unwrap();
    let original = set.graph.rows[requested.0 as usize]
        .effects
        .iter()
        .find_map(|effect| match &effect.arguments[..] {
            [etas_code::descriptor::StaticEffectArgInfo::Type(ty)] => Some(*ty),
            _ => None,
        })
        .expect("requested action has a type selector");
    let etas_code::descriptor::TypeInfo::Applied {
        constructor,
        arguments,
    } = &set.graph.types[original.0 as usize]
    else {
        panic!("expected generic nominal");
    };
    assert_eq!(arguments.len(), 1);
    let constructor = *constructor;
    let other_arg = etas_code::descriptor::TypeInfoRef(set.graph.types.len() as u32);
    set.graph
        .types
        .push(etas_code::descriptor::TypeInfo::Primitive(
            etas_code::descriptor::PrimitiveInfo::U32,
        ));
    let altered = etas_code::descriptor::TypeInfoRef(set.graph.types.len() as u32);
    set.graph
        .types
        .push(etas_code::descriptor::TypeInfo::Applied {
            constructor,
            arguments: vec![other_arg],
        });
    let row = &mut set.graph.rows[requested.0 as usize];
    let selector = row
        .effects
        .iter_mut()
        .find_map(|effect| match &mut effect.arguments[..] {
            [etas_code::descriptor::StaticEffectArgInfo::Type(ty)] => Some(ty),
            _ => None,
        })
        .unwrap();
    *selector = altered;
    let checked = origin_metadata::check(&forged);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&forged, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_ne!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_callback_preserves_nominal_type_selector_identity() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public type Region = string;
public type Other = string;
public effect Choice { action decide<T>() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide<Region>]) -> unit ![] {
    handle { f(); } with { Choice.decide<Region>() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice, Region};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide<Region>]) -> unit ![] { wrapped(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut forged = loaded.clone();
    let set = forged
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let other = set.graph.types.iter().position(|ty| matches!(ty,
        etas_code::descriptor::TypeInfo::Nominal { definition, .. } if definition.declaration() == ["Other"]
    )).expect("published Other declaration");
    let requested = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["wrapped"])
        })
        .unwrap()
        .analysis
        .effects
        .requested
        .unwrap();
    let row = &mut set.graph.rows[requested.0 as usize];
    let selector = row
        .effects
        .iter_mut()
        .find_map(|effect| match &mut effect.arguments[..] {
            [etas_code::descriptor::StaticEffectArgInfo::Type(ty)] => Some(ty),
            _ => None,
        })
        .expect("requested action has a nominal selector");
    *selector = etas_code::descriptor::TypeInfoRef(other as u32);
    let checked = origin_metadata::check(&forged);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&forged, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_ne!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_callback_preserves_nested_type_selector() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide<T>() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide<Array<i32>>]) -> unit ![] {
    handle { f(); } with { Choice.decide<Array<i32>>() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide<Array<i32>>]) -> unit ![] { wrapped(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_callback_preserves_type_selector() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide<T>() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide<i32>]) -> unit ![] {
    handle { f(); } with { Choice.decide<i32>() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide<i32>]) -> unit ![] { wrapped(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut forged = loaded.clone();
    let set = forged
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let requested = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["wrapped"])
        })
        .unwrap()
        .analysis
        .effects
        .requested
        .unwrap();
    let other = etas_code::descriptor::TypeInfoRef(set.graph.types.len() as u32);
    set.graph
        .types
        .push(etas_code::descriptor::TypeInfo::Primitive(
            etas_code::descriptor::PrimitiveInfo::U32,
        ));
    let row = &mut set.graph.rows[requested.0 as usize];
    let selector = row
        .effects
        .iter_mut()
        .find_map(|effect| match &mut effect.arguments[..] {
            [etas_code::descriptor::StaticEffectArgInfo::Type(ty)] => Some(ty),
            _ => None,
        })
        .expect("requested action has a type selector");
    *selector = other;
    let checked = origin_metadata::check(&forged);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&forged, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_ne!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_rejects_reordered_distinct_callbacks() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.println;
public effect Choice { action decide() -> unit; }
public flow pair(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); g(); } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow reversed(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide]) -> unit ![] {
    handle { g(); f(); } with { Choice.decide() => { println("handled")?; resume; } }; return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{pair, reversed, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide], g: () -> unit ![Choice.decide]) -> unit ![] { pair(f, g); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("pair(f, g); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    let changed = match parse_body("reversed(f, g); return;") { Ok(body) => body, Err(_) => { return "parse reversed"; }, };
    if check((^normalize).with_body(changed)).status != "rejected" { return "reordering accepted"; }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_replays_each_callback_call_in_order() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.println;
public effect Choice { action decide() -> unit; }
public flow twice(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); f(); } with { Choice.decide() => { println("handled")?; resume; } };
    return;
}
public flow once(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("handled")?; resume; } };
    return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{twice, once, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] { twice(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("twice(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    let reduced = match parse_body("once(f); return;") { Ok(body) => body, Err(_) => { return "parse reduced"; }, };
    if check((^normalize).with_body(reduced)).status != "rejected" { return "missing callback accepted"; }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut incomplete = loaded.clone();
    let set = incomplete
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let root = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["twice"])
        })
        .unwrap()
        .analysis
        .outcomes
        .unwrap();
    let etas_code::descriptor::TraceInfo::Choice(branches) = &mut set.graph.traces[root.0 as usize]
    else {
        panic!("expected outcome choice");
    };
    assert_eq!(branches.len(), 3);
    branches.pop();
    let checked = origin_metadata::check(&incomplete);
    let definitions = etas_driver::compiler::prepare_native_definitions(
        &incomplete,
        &checked,
        Default::default(),
    )
    .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String(
            "code::UnprovenParametricDeterminism".into()
        ))
    );
}

#[test]
fn metadata_only_effectful_handler_callback_with_argument_replays() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
public effect Choice { action decide() -> unit; }
public flow wrapped(f: (bool) -> unit ![Choice.decide], enabled: bool) -> unit ![] {
    handle { f(enabled); } with { Choice.decide() => { println("first")?; eprintln("second")?; resume; } };
    return;
}

public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: (bool) -> unit ![Choice.decide], enabled: bool) -> unit ![] { wrapped(f, enabled); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f, enabled); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    let changed = match parse_body("wrapped(f, true); return;") { Ok(body) => body, Err(_) => { return "parse changed"; }, };
    if check((^normalize).with_body(changed)).status != "rejected" { return "changed argument accepted"; }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
}

#[test]
fn metadata_only_callback_literal_arguments_are_preserved() {
    for (ty, value, changed) in [("bool", "true", "false"), ("i32", "-42", "17")] {
        let source = format!(
            r#"module library.api;
import std.io.println;
public effect Choice {{ action decide() -> unit; }}
public flow wrapped(f: ({ty}) -> unit ![Choice.decide]) -> unit ![] {{
    handle {{ f({value}); }} with {{ Choice.decide() => {{ println("handled")?; resume; }} }};
    return;
}}
public flow wrapped_other(f: ({ty}) -> unit ![Choice.decide]) -> unit ![] {{
    handle {{ f({changed}); }} with {{ Choice.decide() => {{ println("handled")?; resume; }} }};
    return;
}}
public flow lookup() -> i32 {{ return 7; }}
"#
        );
        let workspace = origin_metadata::workspace_with_source(&source);
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"
module app.main;
import first.library.api.{{wrapped, wrapped_other, Choice}};
import std.compiler.{{check, parse_body, CompileError}};
flow normalize(f: ({ty}) -> unit ![Choice.decide]) -> unit ![] {{ wrapped(f); return; }}
flow main() -> string ![Error<CompileError>] {{
    let body = match parse_body("wrapped(f); return;") {{ Ok(body) => body, Err(_) => {{ return "parse"; }}, }};
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) {{ if diagnostic.severity == "error" {{ return diagnostic.code; }} }}
    let altered = match parse_body("wrapped_other(f); return;") {{ Ok(body) => body, Err(_) => {{ return "parse changed"; }}, }};
    if check((^normalize).with_body(altered)).status != "rejected" {{ return "changed argument accepted"; }}
    return report.status;
}}
"#
        );
        loaded
            .loaded
            .input
            .options
            .reflection
            .retain_bodies
            .push("app.main.normalize".into());
        let checked = origin_metadata::check(&loaded);
        let definitions = etas_driver::compiler::prepare_native_definitions(
            &loaded,
            &checked,
            Default::default(),
        )
        .unwrap();
        let result = revisions::execute_prepared(
            (checked, definitions),
            Some(20000),
            Default::default(),
            true,
        );
        assert!(
            result.diagnostics.is_empty(),
            "{ty}: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.value(),
            Some(&InterpValue::String("accepted".into())),
            "{ty}"
        );
    }
}

#[test]
fn compiled_dependency_callback_preserves_argument_and_handler() {
    use etas_host::console::ConsoleOperation;
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, RunOptions, code::PreparedCodeDefinitions},
    };
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.println;
public type Region<T> = { value: T };
public effect Choice { action decide<T>() -> unit; }
public flow wrapped(f: (bool) -> unit ![Choice.decide<Region<i32>>], enabled: bool) -> unit ![] {
    handle { f(enabled); f(enabled); } with { Choice.decide<Region<i32>>() => { println("handled")?; resume; } };
    return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice, Region};
import std.compiler.{compile, parse_body, CompileError};
flow normalize(f: (bool) -> unit ![Choice.decide<Region<i32>>], enabled: bool) -> unit ![] { wrapped(f, enabled); return; }
flow callback(enabled: bool) -> unit ![Choice.decide<Region<i32>>] { if enabled { perform Choice.decide<Region<i32>>(); } return; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("wrapped(f, enabled); return;") { Ok(body) => body, Err(_) => { return false; }, };
    let candidate = compile((^normalize).with_body(body));
    candidate(callback, true);
    candidate(callback, false);
    return true;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let host = CompilerHost::new(no_host::NoHost::with_console(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    for (effect, action) in [("Code", "compile"), ("Console", "stdout_write")] {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow(effect, action));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    assert_eq!(
        host.inner().console_events(),
        vec![
            ConsoleOperation::WriteStdout {
                text: "handled".into(),
                newline: true
            };
            2
        ]
    );
}

#[test]
fn compile_reuses_checked_dependency_effect_identity_with_source_overlay() {
    use etas_host::console::ConsoleOperation;
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, RunOptions, code::PreparedCodeDefinitions},
    };
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.io.{println, eprintln};
public effect Choice { action decide() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("first")?; eprintln("second")?; resume; } };
    return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice};
import std.compiler.{compile, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] { wrapped(f); return; }
flow callback() -> unit ![Choice.decide] { perform Choice.decide(); return; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return false; }, };
    let candidate = compile((^normalize).with_body(body));
    candidate(callback);
    return true;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    let checked = origin_metadata::check(&loaded);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let service = NativeCompilerService::new(definitions.clone(), Default::default())
        .with_project_sources(&loaded)
        .unwrap();
    let host = CompilerHost::new(no_host::NoHost::with_console(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    for (effect, action) in [
        ("Code", "compile"),
        ("Console", "stdout_write"),
        ("Console", "stderr_write"),
    ] {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow(effect, action));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    assert_eq!(
        host.inner().console_events(),
        vec![
            ConsoleOperation::WriteStdout {
                text: "first".into(),
                newline: true
            },
            ConsoleOperation::WriteStderr {
                text: "second".into(),
                newline: true
            },
        ]
    );
}
use etas_interpreter::api::InterpValue;

#[test]
fn metadata_only_effectful_handler_callback_contract_replays() {
    let workspace = origin_metadata::workspace_with_source(
        r#"
module library.api;
import std.io.{println, eprintln};
public effect Choice { action decide() -> unit; }
public flow wrapped(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("first")?; eprintln("second")?; resume; } };
    return;
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"
module app.main;
import first.library.api.{wrapped, Choice};
import std.compiler.{check, parse_body, CompileError};
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] { wrapped(f); return; }
flow main() -> string ![Error<CompileError>] {
    let body = match parse_body("wrapped(f); return;") { Ok(body) => body, Err(_) => { return "parse"; }, };
    let report = check((^normalize).with_body(body));
    for diagnostic in report.diagnostics limit Iterations(64) { if diagnostic.severity == "error" { return diagnostic.code; } }
    let unsafe_body = match parse_body("f(); return;") { Ok(body) => body, Err(_) => { return "parse unsafe"; }, };
    let unsafe_report = check((^normalize).with_body(unsafe_body));
    if unsafe_report.status != "rejected" { return "unsafe accepted"; }
    return report.status;
}
"#.into();
    loaded
        .loaded
        .input
        .options
        .reflection
        .retain_bodies
        .push("app.main.normalize".into());
    assert!(
        !loaded
            .loaded
            .input
            .environment
            .external_effect_metadata
            .tags
            .iter()
            .any(|tag| tag.path == ["library", "api", "Choice"])
    );
    assert!(
        loaded
            .loaded
            .input
            .environment
            .external_effect_metadata
            .tags
            .iter()
            .any(|tag| tag.path == ["first", "library", "api", "Choice"])
    );
    let checked = origin_metadata::check(&loaded);
    assert_eq!(checked.sources.sources.len(), 1);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&loaded, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String("accepted".into()))
    );
    let mut forged = loaded.clone();
    let set = forged
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let mut replaced = 0;
    for trace in &mut set.graph.traces {
        if let etas_code::descriptor::TraceInfo::Event(event) = trace
            && let etas_code::descriptor::EffectTargetInfo::Action { name, .. } = &mut event.target
            && name == "stderr_write"
        {
            *name = "stdout_write".into();
            replaced += 1;
        }
    }
    assert!(replaced > 0);
    let checked = origin_metadata::check(&forged);
    let definitions =
        etas_driver::compiler::prepare_native_definitions(&forged, &checked, Default::default())
            .unwrap();
    let result = revisions::execute_prepared(
        (checked, definitions),
        Some(20000),
        Default::default(),
        true,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(
        result.value(),
        Some(&InterpValue::String(
            "code::UnprovenParametricDeterminism".into()
        ))
    );
}

#[test]
fn source_compile_preserves_effectful_handler_arm_order_at_host_boundary() {
    use etas_host::console::ConsoleOperation;
    use etas_interpreter::{
        Interpreter,
        api::{EntryPoint, RunOptions, code::PreparedCodeDefinitions},
    };

    let (checked, definitions) = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, check, parse_body, CompileError};
import std.io.{println, eprintln};
effect Choice { action decide() -> unit; }
flow normalize(f: () -> unit ![Choice.decide]) -> unit ![] {
    handle { f(); } with { Choice.decide() => { println("first")?; eprintln("second")?; resume; } };
    return;
}
flow callback() -> unit ![Choice.decide] { perform Choice.decide(); return; }
flow main() -> bool ![Error<CompileError>] {
    let wrong = match parse_body("handle { f(); } with { Choice.decide() => { eprintln(\"second\")?; println(\"first\")?; resume; } }; return;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    if check((^normalize).with_body(wrong)).status != "rejected" { return false; }
    let body = match parse_body("handle { f(); } with { Choice.decide() => { println(\"first\")?; eprintln(\"second\")?; resume; } }; return;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let draft = (^normalize).with_body(body);
    if check(draft).status != "accepted" { return false; }
    let current = compile(draft);
    current(callback);
    return true;
}
"#,
        &["app.normalize"],
    );
    let service = NativeCompilerService::new(definitions.clone(), Default::default());
    let host = CompilerHost::new(no_host::NoHost::with_console(), service.clone())
        .with_candidate_resolver(Arc::new(service));
    let mut options = RunOptions {
        code_definitions: Some(Arc::new(PreparedCodeDefinitions::new(definitions))),
        ..Default::default()
    };
    for (effect, action) in [
        ("Code", "check"),
        ("Code", "compile"),
        ("Console", "stdout_write"),
        ("Console", "stderr_write"),
    ] {
        options
            .host_context
            .authority
            .grants
            .push(HostActionGrant::allow(effect, action));
    }
    let result = runtime()
        .block_on(Interpreter.run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            vec![],
            &host,
            options,
        ))
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    assert_eq!(
        host.inner().console_events(),
        vec![
            ConsoleOperation::WriteStdout {
                text: "first".into(),
                newline: true
            },
            ConsoleOperation::WriteStderr {
                text: "second".into(),
                newline: true
            },
        ]
    );
    assert_eq!(host.inner().calls(), 2);
}

#[test]
fn flow_compile_contracts_survive_source_package_binary_projection_and_metadata_only_replay() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{compile, CompileError, CallableDef};
import std.io.println;
private flow hidden(value: i32) -> i32 ![] { println("frozen request")?; return value; }
public flow make() -> ((i32) -> i32 ![]) ![Error<CompileError>] { return compile(^hidden); }
public flow revise<F>(draft: CallableDef<F>) -> F ![Error<CompileError>] { let candidate = compile(draft); return candidate; }
public flow choose<F>(draft: CallableDef<F>, replacement: CallableDef<F>, replace: bool) -> F ![Error<CompileError>] {
    let other = compile(replacement);
    return handle { compile(draft) } with {
        Code.compile(_) => { if replace { finish other; } resume compile(draft); }
    };
}
public flow choose_value<F>(draft: CallableDef<F>, replacement: F) -> F {
    return handle { compile(draft) } with { Code.compile(_) => { finish replacement; } };
}
public flow lookup() -> i32 { return 7; }
"#,
    );
    let metadata = etas_package::read_package_metadata_artifact(&workspace.0.join("library"))
        .unwrap()
        .unwrap();
    let producer = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "make"))
        .unwrap();
    let graph = producer.returned_callables.as_ref().unwrap();
    assert!(matches!(
        graph.sources.as_slice(),
        [etas_package_metadata::CallableValueSource::FrozenContract(
            _
        )]
    ));
    let chosen = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| {
            summary
                .item
                .last()
                .is_some_and(|name| name == "choose_value")
        })
        .unwrap();
    assert!(
        matches!(chosen.returned_callables.as_ref().unwrap().sources.as_slice(), [etas_package_metadata::CallableValueSource::Parameter(parameter)] if parameter.scope == etas_package_metadata::CallableValueScope::Root && parameter.position == 1)
    );
    let request = metadata
        .public_metadata
        .effect_summaries
        .iter()
        .find(|summary| summary.item.last().is_some_and(|name| name == "revise"))
        .unwrap()
        .handler_requests
        .as_ref()
        .unwrap()
        .actions[0]
        .signature
        .as_ref()
        .unwrap();
    assert_eq!(request.params.len(), 1);
    assert!(
        matches!(&request.output, etas_package::PackageTypeMetadata::Var { name } if name == "F")
    );
    for (expression, capture, console, denied) in [
        ("make()", true, true),
        ("revise(^local)", true, true),
        ("choose(^pure, ^local, true)", true, true),
        ("choose_value(^pure, local)", false, true),
        ("choose_value(^local, pure)", false, false),
        (
            "handle { revise(^local) } with { Code.compile(_) => { resume local; } }",
            false,
            true,
        ),
        (
            "handle { revise(^pure) } with { Code.compile(_) => { resume pure; } }",
            false,
            false,
        ),
    ]
    .into_iter()
    .flat_map(|(expression, capture, console)| {
        [false, true].map(move |denied| (expression, capture, console, denied))
    }) {
        let invocation = if capture {
            format!(
                "let result = {expression}?; return match result {{ Ok(candidate) => candidate(4), Err(_) => 0, }};"
            )
        } else {
            format!("let candidate = {expression}; return candidate(4);")
        };
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.{{make, revise, choose, choose_value}};
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow local(value: i32) -> i32 ![] {{ println("local frozen request")?; return value; }}
flow pure(value: i32) -> i32 ![] {{ return value; }}
flow main() -> i32 {} {{
    {invocation}
}}
"#,
            if denied { "~ Quiet" } else { "" }
        );
        let output = Frontend.check_project(loaded.loaded.input);
        if denied && console {
            assert!(
                output.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(
                        etas_core::EffectDiagnosticCode::TraceSpecDenied
                    )),
                "{:?}",
                output.diagnostics
            );
        } else {
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            let checked = output.checked.unwrap();
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            assert!(summary.escaping_effects.is_empty());
            assert_eq!(
                summary.requested_actions.effects.iter().count(),
                if console { 2 } else { 1 },
                "{summary:#?}"
            );
            assert_eq!(summary.default_actions.is_empty(), !console);
        }
    }
}

#[test]
fn external_callable_definition_parameter_keeps_its_frozen_action_contract() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public flow invoke(draft: CallableDef<(i32) -> i32 ![]>, value: i32) -> i32 ![Error<CompileError>] {
    let next = compile(draft);
    return next(value);
}
"#,
    );
    for denied in [false, true] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.invoke;
import std.io.println;
spec Quiet: trace = -Console.stdout_write;
flow local(value: i32) -> i32 ![] {{ println("frozen")?; return value; }}
flow main() -> i32 {} {{
    let result = invoke(^local, 4)?;
    return match result {{ Ok(value) => value, Err(_) => 0, }};
}}
"#,
            if denied { "~ Quiet" } else { "" }
        );
        let output = Frontend.check_project(loaded.loaded.input);
        if denied {
            assert!(
                output.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == etas_core::DiagnosticCode::Effect(
                        etas_core::EffectDiagnosticCode::TraceSpecDenied
                    )),
                "{:?}",
                output.diagnostics
            );
        } else {
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            let checked = output.checked.unwrap();
            assert_eq!(checked.sources.sources.len(), 1);
            let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
            let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
                tag: etas_effects::CONSOLE_TAG,
                action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
            });
            assert!(
                summary.requested_actions.effects.contains(&stdout),
                "{summary:?}"
            );
        }
    }
}

#[test]
fn external_callable_definition_specializes_effect_row_contract() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public flow invoke<effect E>(draft: CallableDef<(i32) -> i32 ![E]>, value: i32) -> i32 ![E, Error<CompileError>] {
    let next = compile(draft); return next(value);
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.invoke;
flow local(value: i32) -> i32 ![Console.stdout_write] { perform Console.stdout_write("value"); return value; }
flow main() -> i32 ![Console.stdout_write] { let result = invoke(^local, 4)?; return match result { Ok(value) => value, Err(_) => 0, }; }
"#.into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let summary = &checked.effects.item_effects[&checked.entry.unwrap()];
    let stdout = etas_effects::Effect::Action(etas_effects::ActionRef {
        tag: etas_effects::CONSOLE_TAG,
        action: etas_effects::CONSOLE_STDOUT_WRITE_ACTION,
    });
    assert!(
        summary.escaping_effects.effects.contains(&stdout),
        "{summary:?}"
    );
    assert!(
        summary.requested_actions.effects.contains(&stdout),
        "{summary:?}"
    );
}

#[test]
fn external_callable_definition_alias_preserves_parameter_contract() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public alias Draft = CallableDef<(i32) -> i32 ![]>;
public flow invoke(draft: Draft, value: i32) -> i32 ![Error<CompileError>] {
    let next = compile(draft);
    return next(value);
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.invoke;
flow local(value: i32) -> i32 ![] { return value; }
flow main() -> i32 { let result = invoke(^local, 4)?; return match result { Ok(value) => value, Err(_) => 0, }; }
"#.into();
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output.checked.is_some());
}

#[test]
fn forged_callable_definition_parameter_role_is_rejected_at_import() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public flow invoke(draft: CallableDef<(i32) -> i32 ![]>, value: i32) -> i32 ![Error<CompileError>] {
    let next = compile(draft);
    return next(value);
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text =
        "module app.main; import first.library.api.invoke; flow main() -> unit { return; }".into();
    let set = loaded
        .loaded
        .input
        .environment
        .external_definition_origins
        .iter_mut()
        .find_map(|origin| origin.descriptors.as_mut())
        .unwrap();
    let header = set
        .headers
        .as_ref()
        .unwrap()
        .callables
        .iter()
        .find(|header| {
            header
                .id
                .as_declaration()
                .is_some_and(|id| id.declaration() == ["invoke"])
        })
        .unwrap();
    let etas_code::descriptor::TypeInfo::Function(signature) =
        &set.graph.types[header.signature.0 as usize]
    else {
        panic!("missing callable signature")
    };
    let parameter = signature.parameters[0].ty;
    let etas_code::descriptor::TypeInfo::Builtin { constructor, .. } =
        &mut set.graph.types[parameter.0 as usize]
    else {
        panic!("missing callable definition role")
    };
    assert_eq!(
        *constructor,
        etas_code::descriptor::BuiltinTypeInfo::CallableDefinition
    );
    *constructor = etas_code::descriptor::BuiltinTypeInfo::Option;
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(output.checked.is_none());
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Type(etas_core::TypeDiagnosticCode::IncompleteTypeFacts)
            && diagnostic
                .message
                .contains("trace parameter call requires a checked callback function")),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn external_descriptor_contract_cannot_be_rebound_to_an_unrelated_parameter_type() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public flow invoke(draft: CallableDef<(i32) -> i32 ![]>, value: i32) -> i32 ![Error<CompileError>] {
    let next = compile(draft);
    return next(value);
}
"#,
    );
    for wrapped in [true, false] {
        let mut loaded = origin_metadata::consumer(&workspace, "first");
        loaded.loaded.input.sources[0].text = format!(
            r#"module app.main;
import first.library.api.invoke;
flow local(value: i32) -> i32 ![] {{ return value; }}
flow main() -> i32 {{ let result = invoke({}, 4)?; return match result {{ Ok(value) => value, Err(_) => 0, }}; }}
"#,
            if wrapped { "Some(local)" } else { "local" }
        );
        let signature = loaded
            .loaded
            .input
            .environment
            .external_public_metadata
            .iter_mut()
            .flat_map(|metadata| &mut metadata.flows)
            .find(|flow| flow.path.last().is_some_and(|name| name == "invoke"))
            .unwrap();
        let etas_frontend::ProjectExternalTypeInput::Applied { args, .. } = &signature.params[0]
        else {
            panic!("expected CallableDef parameter")
        };
        let inner = args[0].clone();
        signature.params[0] = if wrapped {
            etas_frontend::ProjectExternalTypeInput::Option(Box::new(inner))
        } else {
            inner
        };
        let output = Frontend.check_project(loaded.loaded.input);
        assert!(
            output.checked.is_none(),
            "mismatched descriptor and public signature were accepted"
        );
        assert!(
            output.diagnostics.iter().any(|diagnostic| diagnostic.code
                == etas_core::DiagnosticCode::Type(
                    etas_core::TypeDiagnosticCode::IncompleteTypeFacts
                )),
            "{:?}",
            output.diagnostics
        );
    }
}

#[test]
fn external_callable_definition_cannot_change_its_inner_signature() {
    let workspace = origin_metadata::workspace_with_source(
        r#"module library.api;
import std.compiler.{CallableDef, CompileError, compile};
public flow invoke(draft: CallableDef<(i32) -> i32 ![]>, value: i32) -> i32 ![Error<CompileError>] {
    let next = compile(draft); return next(value);
}
"#,
    );
    let mut loaded = origin_metadata::consumer(&workspace, "first");
    loaded.loaded.input.sources[0].text = r#"module app.main;
import first.library.api.invoke;
flow local(value: bool) -> i32 ![] { return 1; }
flow main() -> i32 { let result = invoke(^local, 4)?; return match result { Ok(value) => value, Err(_) => 0, }; }
"#.into();
    let signature = loaded
        .loaded
        .input
        .environment
        .external_public_metadata
        .iter_mut()
        .flat_map(|metadata| &mut metadata.flows)
        .find(|flow| flow.path.last().is_some_and(|name| name == "invoke"))
        .unwrap();
    let etas_frontend::ProjectExternalTypeInput::Applied { args, .. } = &mut signature.params[0]
    else {
        panic!("expected CallableDef parameter")
    };
    let etas_frontend::ProjectExternalTypeInput::Function { input, .. } = &mut args[0] else {
        panic!("expected function contract")
    };
    input[0] = etas_frontend::ProjectExternalTypeInput::Primitive("bool".into());
    let output = Frontend.check_project(loaded.loaded.input);
    assert!(
        output.checked.is_none(),
        "mismatched inner callable signature was accepted"
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| diagnostic.code
            == etas_core::DiagnosticCode::Effect(
                etas_core::EffectDiagnosticCode::IncompleteEffectFacts
            )
            && diagnostic
                .message
                .contains("matching checked callable parameter interface")),
        "{:?}",
        output.diagnostics
    );
}

#[test]
fn reflected_pure_finite_for_loop_can_be_recompiled() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, compile, parse_body, CompileError};
flow target(values: Array<bool>) -> bool ![] {
    for value in values limit Iterations(8) { let copy = value; }
    return true;
}
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("for value in values limit Iterations(8) { let local = value; } return true;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let draft = (^target).with_body(body);
    if check(draft).status != "accepted" { return false; }
    let candidate = compile(draft);
    return candidate([true, false]);
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn reflected_bounded_loop_rejects_unproven_early_return_paths() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, parse_body};
flow target(values: Array<bool>) -> bool {
    for value in values limit Iterations(2) { if value { return true; } }
    return false;
}
flow main() -> string {
    let same = match parse_body("for value in values limit Iterations(2) { if value { return true; } } return false;") {
        Ok(body) => body, Err(_) => { return "parse-same"; },
    };
    let changed = match parse_body("for value in values limit Iterations(2) { if value { return false; } } return true;") {
        Ok(body) => body, Err(_) => { return "parse-changed"; },
    };
    if check((^target).with_body(same)).status != "rejected" { return "same-accepted"; }
    if check((^target).with_body(changed)).status != "rejected" { return "changed-accepted"; }
    return "ok";
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::String("ok".into())));
}

#[test]
fn reflected_bounded_loop_rejects_unproven_action_outcomes() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, parse_body};
flow target(values: Array<bool>) -> unit ![Console.stdout_write, Console.stderr_write] {
    for value in values limit Iterations(2) {
        perform Console.stdout_write("a");
        perform Console.stderr_write("b");
    }
    return;
}
flow main() -> string {
    let same = match parse_body("for value in values limit Iterations(2) { perform Console.stdout_write(\"a\"); perform Console.stderr_write(\"b\"); } return;") {
        Ok(body) => body, Err(_) => { return "parse-same"; },
    };
    let changed = match parse_body("for value in values limit Iterations(2) { perform Console.stderr_write(\"b\"); perform Console.stdout_write(\"a\"); } return;") {
        Ok(body) => body, Err(_) => { return "parse-changed"; },
    };
    if check((^target).with_body(same)).status != "rejected" { return "same-accepted"; }
    if check((^target).with_body(changed)).status != "rejected" { return "changed-accepted"; }
    return "ok";
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::String("ok".into())));
}

#[test]
fn instantiated_effect_row_flow_revision_uses_bound_effect_environment() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, compile, parse_body, CompileError};
import std.io.println;
flow identity<effect E>(value: bool) -> bool ![E] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let original: (bool) -> bool ![] = identity;
    let effectful: (bool) -> bool ![Console.stdout_write] = identity;
    if check(^effectful).status != "accepted" { return false; }
    let body = match parse_body("return !value;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let draft = (^original).with_body(body);
    if check(draft).status != "accepted" { return false; }
    if check((^effectful).with_body(body)).status != "accepted" { return false; }
    let changed = match parse_body("println(\"changed\")?; return value;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    if check((^effectful).with_body(changed)).status != "rejected" { return false; }
    let candidate = compile(draft);
    return !candidate(true) && candidate(false) && original(true);
}
"#,
        &["app.identity"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn instantiated_generic_flow_revision_uses_bound_type_environment() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, compile, parse_body, CompileError};
flow identity<T>(value: T) -> T ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let original: (bool) -> bool ![] = identity;
    let integer: (i32) -> i32 ![] = identity;
    let body = match parse_body("return !value;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let draft = (^original).with_body(body);
    if check(draft).status != "accepted" { return false; }
    if check((^integer).with_body(body)).status != "rejected" { return false; }
    let candidate = compile(draft);
    return !candidate(true) && candidate(false) && original(true) && integer(42) == 42;
}
"#,
        &["app.identity"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn spec_ordinary_flow_revision_example_compiles_and_runs() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
flow normalize(text: string) -> string ![] {
    return std.text.trim(text);
}
flow RunAfterFlowRevision(text: string) -> bool {
    var current = normalize;
    let definition = ^current;
    let original_revision = definition.revision.key;
    let replacement: string =
        "let cleaned = std.text.trim(text); return cleaned;";
    match std.compiler.parse_body(replacement) {
        Ok(syntax) => {
            let edited = definition.with_body(syntax);
            match std.compiler.compile(edited)? {
                Ok(candidate) => { current = candidate; },
                Err(_) => {},
            };
        },
        Err(_) => {},
    };
    return (^current).revision.key != original_revision && current(text) == "hello";
}
flow main() -> bool { return RunAfterFlowRevision("  hello  "); }
"#,
        &["app.normalize"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_revision_with_checked_limit_constructor_executes() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{check, compile, parse_body, CompileError};
flow target(flag: bool) -> bool ![] { let limit = Iterations(8); return flag; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("let limit = Iterations(9); return flag;") {
        Ok(body) => body, Err(_) => { return false; },
    };
    let draft = (^target).with_body(body);
    if check(draft).status != "accepted" { return false; }
    let replacement = compile(draft);
    return replacement(true) && !replacement(false);
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_direct_actions_use_real_reports_and_reject_invalid_candidates() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{parse_body, CompileError};
flow normalize(text: string) -> string ![] { return text; }
flow main() -> bool ![Error<CompileError>] {
    let original = ^normalize;
    let accepted = perform Code.check(original);
    let syntax = match parse_body("return 42;") { Ok(body) => body, Err(_) => { return false; }, };
    let draft = original.with_body(syntax);
    let rejected = perform Code.check(draft);
    let result = (perform Code.compile(draft))?;
    return accepted.status == "accepted" && rejected.status == "rejected"
        && match result { Err(error) => error.kind == "rejected", Ok(_) => false, };
}
"#,
        &["app.normalize"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_compile_publishes_without_invoking_the_candidate() {
    for expression in ["compile(^target)", "perform Code.compile(^target)"] {
        let prepared = source::prepared_with_bodies(
            &format!(
                r#"
module app;
import std.compiler.{{compile, CompileError}};
flow target() -> i32 ![] {{ return abort("compilation must not invoke this body"); }}
flow main() -> bool ![Error<CompileError>] {{ let candidate = {expression}; return true; }}
"#
            ),
            &["app.target"],
        );
        let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
        assert!(
            result.diagnostics.is_empty(),
            "{expression}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
    }
}

#[test]
fn successive_flow_revisions_keep_the_original_contract_and_pinned_aliases() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, check, parse_body, CompileError};
flow target(flag: bool) -> bool ![] { return flag; }
flow main() -> bool ![Error<CompileError>] {
    var current = target;
    let original = current;
    let first_body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return false; }, };
    current = compile((^current).with_body(first_body));
    let first = current;
    let first_view = ^current;
    let second_body = match parse_body("return true;") { Ok(body) => body, Err(_) => { return false; }, };
    let second_draft = first_view.with_body(second_body);
    if check(second_draft).status != "accepted" { return false; }
    current = compile(second_draft);
    let second_view = ^current;
    let first_text = match first_view.body { Some(body) => body.text, None => { return false; }, };
    let second_text = match second_view.body { Some(body) => body.text, None => { return false; }, };
    return original(true) && !first(true) && current(false)
        && first_view.id == second_view.id
        && first_view.revision.key != second_view.revision.key
        && first_view.dependencies == second_view.dependencies
        && first_text != second_text;
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_source_compile_handler_can_resume_a_pinned_compatible_flow_revision() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
flow target(flag: bool) -> bool ![] { return flag; }
flow main() -> bool ![Error<CompileError>] {
    let body = match parse_body("return false;") { Ok(body) => body, Err(_) => { return false; }, };
    let pinned = compile((^target).with_body(body));
    let current = handle { compile(^target) } with { Code.compile(_) => { resume pinned; } };
    return !current(true) && target(true);
}
"#,
        &["app.target"],
    );
    let result = revisions::execute_prepared(prepared, Some(20000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn ordinary_flow_source_edit_check_compile_and_call_preserve_old_bindings() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, check, parse_body, CompileError};
flow normalize(text: string) -> string ![] { return text; }
flow main() -> bool ![Error<CompileError>] {
    var current = normalize;
    let old = current;
    let syntax = match parse_body("return \"changed\";") { Ok(body) => body, Err(_) => { return false; }, };
    let draft = (^normalize).with_body(syntax);
    let report = check(draft);
    if report.status != "accepted" { return false; }
    current = compile(draft);
    return current("input") == "changed" && old("input") == "input" && normalize("input") == "input";
}
"#,
        &["app.normalize"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn compiled_flow_source_preserves_nominal_arguments_and_results() {
    let prepared = source::prepared_with_bodies(
        r#"
module app;
import std.compiler.{compile, parse_body, CompileError};
type UserId = string;
flow identity(value: UserId) -> UserId ![] { return value; }
flow main() -> bool ![Error<CompileError>] {
    let syntax = match parse_body("return value;") { Ok(body) => body, Err(_) => { return false; }, };
    let candidate = compile((^identity).with_body(syntax));
    let value = UserId("native");
    return candidate(value) == value;
}
"#,
        &["app.identity"],
    );
    let result = revisions::execute_prepared(prepared, Some(10000), Default::default(), true);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}
