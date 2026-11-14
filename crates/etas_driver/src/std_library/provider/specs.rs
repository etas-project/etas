use super::*;

pub(super) fn publish(
    sdk: &LoadedStdSdk,
    builder: &mut StdRegistryBuilder,
    modules: &mut BTreeMap<Vec<String>, etas_std::StdModuleId>,
    symbols: &mut BTreeMap<Vec<String>, etas_std::StdSymbolId>,
    declarations: &BTreeMap<Vec<String>, &DefinitionKey>,
    aliases: &BTreeMap<Vec<String>, Vec<String>>,
) -> Result<(), DriverError> {
    let mut registered = std::collections::BTreeSet::new();
    for spec in &sdk.public.public_metadata.spec_signatures {
        if spec.visibility != metadata::Visibility::Public {
            continue;
        }
        let canonical = canonical_export(&spec.path, aliases)?;
        let Some(key) = declarations.get(&canonical).copied() else {
            continue;
        };
        if symbols.contains_key(&canonical) {
            if registered.contains(&canonical) {
                continue;
            }
            return Err(invalid(format!(
                "std source spec {} replaces a native contract without a checked role binding",
                canonical.join(".")
            )));
        }
        let (name, parent) = canonical
            .split_last()
            .ok_or_else(|| invalid("std spec has no path"))?;
        let module = *modules.entry(parent.to_vec()).or_insert_with(|| {
            builder.module(&parent.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        let projector = TypeProjector {
            owner: key,
            params: &spec.generic_params,
        };
        let params = spec
            .generic_params
            .iter()
            .map(|param| projector.param(param))
            .collect::<Result<_, _>>()?;
        let symbol = builder.symbol(
            module,
            name,
            etas_std::StdSymbolKind::Type,
            etas_std::StdDecl::Type(etas_std::TypeDecl::generic_params(
                name,
                params,
                etas_std::TypeDeclKind::Spec,
            )),
            "",
        );
        builder.bind_declaration(symbol, key.clone());
        registered.insert(canonical.clone());
        symbols.insert(canonical, symbol);
    }
    Ok(())
}
