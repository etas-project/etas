//! Export the selected SDK query view for the existing per-module migration audit.
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io,
    path::PathBuf,
};

use etas_driver::std_library::provider::{ArtifactStdProvider, StdProvider};
use etas_std::migration::StdMigrationBaseline;

fn qualified(key: &etas_code::DefinitionKey) -> String {
    [key.module(), key.declaration()].concat().join(".")
}

#[derive(PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
struct SourceMethod<'a> {
    name: &'a str,
    specification: Option<String>,
    // Distinguishes impls on the same type with different generic constraints.
    implementation: etas_code::CodeFingerprint,
}

fn compare(
    provider: &ArtifactStdProvider,
    artifact: &StdMigrationBaseline,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let registry = etas_std::standard_registry();
    let legacy = StdMigrationBaseline::from_registry(&registry);
    let diff = legacy.compare(artifact);
    let mut legacy_method_exports = BTreeMap::new();
    for symbol in registry.symbols() {
        if let etas_std::StdDecl::Flow(flow) = &symbol.decl
            && let Some(method) = &flow.source_method
        {
            legacy_method_exports.insert(symbol.qualified_path.join("."), method);
        }
    }
    let metadata = &provider.sdk.public()?.public_metadata;
    let image = provider.sdk.image()?;
    let mut source_flows = BTreeMap::new();
    for (definition, item) in &image.declarations {
        if let etas_hir::HirItem::Flow(flow) = &image.program.items[*item]
            && image
                .program
                .symbols
                .get(flow.symbol)
                .expect("checked flow symbol")
                .visibility
                == etas_hir::Visibility::Public
        {
            let body = match flow.body {
                etas_hir::HirFlowBody::Block(_) | etas_hir::HirFlowBody::Expr { .. } => "etas",
                etas_hir::HirFlowBody::Runtime { .. } => "runtime",
                etas_hir::HirFlowBody::Decl { .. } => "declaration",
            };
            source_flows.insert(qualified(definition), body);
        }
    }
    let mut source_methods = BTreeMap::<String, BTreeSet<_>>::new();
    for method in &metadata.methods {
        let owner = method.identity.implementation();
        source_methods
            .entry(qualified(owner.subject()))
            .or_default()
            .insert(SourceMethod {
                name: method.identity.name(),
                specification: owner.selected_spec().map(qualified),
                implementation: owner.header(),
            });
    }
    let source_reexports = metadata
        .re_exports
        .iter()
        .map(|alias| (alias.exported.join("."), alias.from.join(".")))
        .collect::<BTreeMap<_, _>>();
    Ok(serde_json::json!({
        "schema": "etas.std.sdk-api-diff.v1",
        "artifact_hash": provider.sdk.manifest.artifact_hash,
        "registry_diff": diff,
        "legacy_method_exports": legacy_method_exports,
        "source_flows": source_flows,
        "source_methods_by_owner": source_methods,
        "source_reexports": source_reexports,
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let usage = "usage: std_sdk_inventory <sysroot> [--compare-legacy]";
    let sysroot = PathBuf::from(arguments.next().ok_or(usage)?);
    let compare_legacy = match arguments.next() {
        None => false,
        Some(flag) if flag == "--compare-legacy" && arguments.next().is_none() => true,
        _ => return Err(usage.into()),
    };
    let StdProvider::Artifact(provider) = StdProvider::artifact(Some(&sysroot))? else {
        unreachable!()
    };
    let artifact = StdMigrationBaseline::from_registry(&provider.registry);
    if compare_legacy {
        serde_json::to_writer_pretty(io::stdout().lock(), &compare(&provider, &artifact)?)?;
    } else {
        serde_json::to_writer_pretty(io::stdout().lock(), &artifact)?;
    }
    Ok(())
}
