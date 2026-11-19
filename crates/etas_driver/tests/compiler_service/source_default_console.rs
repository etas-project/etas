use super::*;
use etas_interpreter::Interpreter;
use etas_interpreter::api::{EntryPoint, InterpValue, RunOptions};

#[tokio::test(flavor = "current_thread")]
async fn checked_console_default_runs_after_explicit_handlers() {
    let registry = etas_std::standard_registry();
    let symbol = registry
        .lookup_qualified(&["std", "effects", "actions", "Console", "stdout_write"])
        .unwrap();
    let etas_std::StdDecl::EffectAction(action) = symbol.decl.clone() else {
        unreachable!()
    };
    let mut builder = etas_std::StdRegistryBuilder::from_registry(registry.clone());
    builder.source_action_signature(
        symbol.id,
        action.type_params,
        action.effect_args,
        action.selector_param_names,
        action.params,
        action.output,
        Some(etas_std::StdEffectRowExpr {
            effects: vec![etas_std::StdEffectRef::typed(
                &["Error"],
                etas_std::StdType::parse("std.io.IOError"),
            )],
            tail: None,
        }),
    );
    let mut session = etas_frontend::FrontendSession::with_std_registry(std::sync::Arc::new(
        builder.try_finish().unwrap(),
    ));
    let project = session.open_project(ProjectInput::single_source(SourceInput::anonymous(
        r#"
module app;
import std.effects.Console;
import std.io.IOError;
flow main() -> i32 ![Error<IOError>] {
    handle { perform Console.stdout_write("intercepted"); } with {
        Console.stdout_write(_) => { resume; }
    };
    perform Console.stdout_write("delivered");
    return 42;
}
"#,
    )));
    let output = session
        .check(project, etas_frontend::CheckRequest::one_shot())
        .unwrap()
        .output;
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let checked = output.checked.unwrap();
    let host = no_host::NoHost::with_console();
    let result = Interpreter
        .run_checked(
            &checked,
            EntryPoint {
                item: checked.entry.unwrap(),
            },
            Vec::new(),
            &host,
            RunOptions::default(),
        )
        .await
        .unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::i32(42)));
    assert_eq!(host.calls(), 1);
    assert_eq!(
        host.console_events(),
        vec![etas_host::console::ConsoleOperation::WriteStdout {
            text: "delivered".into(),
            newline: false,
        }]
    );
}
