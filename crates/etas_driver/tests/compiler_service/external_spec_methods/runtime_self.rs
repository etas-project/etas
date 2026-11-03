use super::*;

const LIBRARY: &str = r#"
module library.api;
public spec Encoder { flow encode(self) -> i32 ![]; }
public flow twice<T ~ Encoder>(value: T) -> i32 ![] {
    value::Encoder.encode();
    return value::Encoder.encode();
}
public flow lookup() -> i32 ![] { return 7; }
"#;

fn root(workspace: &origin_metadata::Workspace, main: &str) -> std::path::PathBuf {
    let root = origin_metadata::consumer(workspace, "dep")
        .package_root
        .unwrap();
    std::fs::write(root.join("src/app/main.es"), main).unwrap();
    root
}

#[test]
fn behavioral_self_runtime_overlay_executes_a_metadata_generic_wrapper() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let root = root(
        &workspace,
        r#"
module app.main;
import dep.library.api.{Encoder, twice};
type Packet = { number: i32 };
impl Packet ~ Encoder { flow encode(self) -> i32 ![] { return self.number; } }
flow relay<U ~ Encoder>(value: U) -> i32 ![] { return twice(value); }
flow main() -> i32 ![] { return relay(Packet { number = 21 }); }
"#,
    );
    let compiled = dependency_namespaces::compile(&root);
    let checked = compiled.response.output.checked.as_ref().unwrap();
    let reachable = checked
        .checked_call_reachability(checked.entry.unwrap())
        .expect("runtime overlay must retain checked implementation body facts");
    let implementation = &checked.types.spec_impls[0].methods[0];
    let etas_hir::SymbolDef::Item { item } =
        checked.hir.symbols.get(implementation.symbol).unwrap().def
    else {
        panic!("checked implementation must target a normalized callable");
    };
    assert!(reachable.contains(&item));
    assert!(checked.effects.item_effects.contains_key(&item));
    dependency_namespaces::execute(compiled, 2, 21);
}

#[test]
fn behavioral_self_frozen_agent_keeps_external_method_evidence_after_compilation() {
    let workspace = origin_metadata::workspace_with_source(LIBRARY);
    let root = root(
        &workspace,
        r#"
module app.main;
import dep.library.api.{Encoder, twice};
import std.compiler.{compile, parse_body, CompileError};
type Packet = { number: i32 };
impl Packet ~ Encoder { flow encode(self) -> i32 ![] { return self.number; } }
flow relay<U ~ Encoder>(value: U) -> i32 ![] { return twice(value); }
agent Worker(value: Packet) -> i32 ![] { return relay(value); }
flow main() -> i32 ![Error<CompileError>] {
    let body = match parse_body("return Worker(value);") {
        Ok(body) => body,
        Err(_) => { return -1; },
    };
    let revised = compile((^Worker).with_body(body));
    return revised(Packet { number = 21 });
}
"#,
    );
    dependency_namespaces::execute(dependency_namespaces::compile(&root), 2, 21);
}
