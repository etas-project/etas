//! Installed calling interfaces and demand-loaded source behavior graphs.
use super::*;
use etas_code::CallableId;
use etas_frontend::{StandardLibraryBehaviors, StandardLibraryInterface};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct BehaviorEntry {
    pub callable: CallableId,
    pub dependencies: BTreeSet<usize>,
    pub file: StdBundleFile,
}

pub(super) fn publish(
    output: &mut StdLibraryBuildOutput,
    provider: &super::super::provider::ArtifactStdProvider,
) -> Result<
    (
        Arc<StandardLibraryInterface>,
        StdBundleFile,
        Vec<BehaviorEntry>,
    ),
    DriverError,
> {
    let signatures = provider.interface.prepare_signatures().map_err(invalid)?;
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&signatures, &mut bytes).map_err(invalid)?;
    let signatures = output.bundle_store.write(
        &output.out_dir,
        "artifacts/std.signatures.cbor".into(),
        "signatures",
        &bytes,
    )?;
    let mut interface = provider.interface.as_ref().clone();
    let mut records = BTreeMap::<CallableId, StandardLibraryBehaviors>::new();
    for method in std::mem::take(&mut interface.metadata.methods) {
        records
            .entry(CallableId::method(method.identity.clone()))
            .or_default()
            .methods
            .push(method);
    }
    interface.metadata.effect_summaries.retain(|summary| {
        let callable = summary
            .definition
            .clone()
            .and_then(|key| CallableId::declaration(key).ok());
        if let Some(callable) = callable {
            records
                .entry(callable)
                .or_default()
                .summaries
                .push(summary.clone());
            false
        } else {
            true
        }
    });
    let image = provider.sdk.image()?;
    let items = records
        .keys()
        .map(|callable| {
            let item = callable
                .as_method()
                .and_then(|id| image.methods.get(id))
                .or_else(|| {
                    callable
                        .as_declaration()
                        .and_then(|id| image.declarations.get(id))
                })
                .copied()
                .ok_or_else(|| invalid("published behavior has no checked declaration"))?;
            Ok((callable.clone(), item))
        })
        .collect::<Result<BTreeMap<_, _>, DriverError>>()?;
    let indices = records
        .keys()
        .enumerate()
        .map(|(index, callable)| (callable.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let by_item = items
        .iter()
        .map(|(callable, item)| (*item, indices[callable]))
        .collect::<BTreeMap<_, _>>();
    let mut entries = Vec::new();
    for (callable, behavior) in records {
        let dependencies = image
            .semantic_dependencies
            .closure([etas_frontend::SemanticDependency::Definition(
                items[&callable],
            )])
            .iter()
            .filter_map(|node| match node {
                etas_frontend::SemanticDependency::Definition(item) => by_item.get(item).copied(),
                _ => None,
            })
            .collect();
        let bytes = serde_json::to_vec(&behavior).map_err(invalid)?;
        let file = output.bundle_store.write(
            &output.out_dir,
            format!("artifacts/interfaces/{}.json", entries.len()).into(),
            "interface",
            &bytes,
        )?;
        entries.push(BehaviorEntry {
            callable,
            dependencies,
            file,
        });
    }
    Ok((Arc::new(interface), signatures, entries))
}

impl LoadedStdSdk {
    pub(crate) fn load_signatures(
        &self,
    ) -> Result<Arc<etas_types::ExternalPublicMetadataInput>, DriverError> {
        let file = &self
            .prepared_queries
            .as_ref()
            .expect("paired queries")
            .signatures;
        let bytes = self.load_bundle_file(file)?;
        let mut cursor = std::io::Cursor::new(bytes.as_slice());
        let input = ciborium::de::from_reader(&mut cursor).map_err(invalid)?;
        if cursor.position() != bytes.len() as u64 {
            return Err(invalid("trailing installed signature data"));
        }
        Ok(Arc::new(input))
    }

    pub(crate) fn load_behaviors(
        &self,
        registry: &etas_std::StdRegistry,
        consumer: Option<(&etas_hir::HirProgram, &etas_types::TypeFacts)>,
        profile: &etas_utils::ProfileHandle,
    ) -> Result<StandardLibraryBehaviors, DriverError> {
        let _load = profile.span("std.load_public_behaviors", "std");
        let entries = &self
            .prepared_queries
            .as_ref()
            .expect("paired queries")
            .behaviors;
        let consumer = consumer.filter(|(_, types)| types.definition_operations.is_empty());
        let selected = match consumer {
            None => (0..entries.len()).collect::<BTreeSet<_>>(),
            Some((hir, types)) => {
                let mut required = types
                    .referenced_external_methods()
                    .into_iter()
                    .map(CallableId::method)
                    .collect::<BTreeSet<_>>();
                required.extend(
                    types
                        .library_flow_calls
                        .values()
                        .map(|call| call.callable.clone()),
                );
                // Function values have no call fact; retain their resolved declaration.
                for (_, expression) in hir.exprs.iter() {
                    if let etas_hir::HirExpr::Path(path) = expression
                        && let etas_hir::ResolveResult::Resolved(symbol) = path.resolution
                        && let Some(etas_hir::SymbolDef::ImportAlias { path, .. }) =
                            hir.symbols.get(symbol).map(|s| &s.def)
                        && let Some(symbol) = registry.lookup_qualified(path)
                        && let Some(binding) = registry.body_binding(symbol.id)
                    {
                        required.insert(binding.callable.clone());
                    }
                }
                let package = self
                    .prepared_queries
                    .as_ref()
                    .expect("paired queries")
                    .interface
                    .origins
                    .package;
                let indices = entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| (&entry.callable, index))
                    .collect::<BTreeMap<_, _>>();
                let mut selected = BTreeSet::new();
                for callable in required {
                    let owner = callable
                        .as_method()
                        .map(|method| method.implementation().package())
                        .or_else(|| {
                            callable
                                .as_declaration()
                                .map(|definition| definition.package())
                        });
                    if owner != Some(package) {
                        continue;
                    }
                    let index = *indices.get(&callable).ok_or_else(|| {
                        invalid(format!(
                            "installed std callable has no published behavior: {callable:?}"
                        ))
                    })?;
                    selected.insert(index);
                    selected.extend(entries[index].dependencies.iter().copied());
                }
                selected
            }
        };
        profile.counter("std.loaded_public_behaviors", selected.len() as u64);
        let mut result = StandardLibraryBehaviors::default();
        for index in selected {
            let bytes = self.load_bundle_file(&entries[index].file)?;
            let mut behavior: StandardLibraryBehaviors =
                serde_json::from_slice(&bytes).map_err(invalid)?;
            result.methods.append(&mut behavior.methods);
            result.summaries.append(&mut behavior.summaries);
        }
        Ok(result)
    }
}
