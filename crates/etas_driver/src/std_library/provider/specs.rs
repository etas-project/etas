use super::*;

pub(super) fn publish(
    sdk: &LoadedStdSdk,
    builder: &mut StdRegistryBuilder,
    modules: &mut BTreeMap<Vec<String>, etas_std::StdModuleId>,
    symbols: &mut BTreeMap<Vec<String>, etas_std::StdSymbolId>,
    declarations: &BTreeMap<Vec<String>, &DefinitionKey>,
    aliases: &BTreeMap<Vec<String>, Vec<String>>,
) -> Result<(), DriverError> {
    let image = sdk.image()?;
    let mut registered = std::collections::BTreeSet::new();
    for spec in &sdk.public()?.public_metadata.spec_signatures {
        let canonical = canonical_export(&spec.path, aliases)?;
        let Some(key) = declarations.get(&canonical).copied() else {
            continue;
        };
        let source_role = image
            .declarations
            .get(key)
            .and_then(|item| image.program.items.get(*item))
            .and_then(etas_hir::HirItem::symbol)
            .and_then(|symbol| image.types.source_type_roles.get(&symbol));
        if let Some(native) = source_role {
            if registered.insert(canonical.clone()) {
                builder.bind_declaration(*native, key.clone());
                let projector = TypeProjector {
                    owner: key,
                    params: &spec.generic_params,
                };
                let implementations = sdk
                    .public()?
                    .public_metadata
                    .spec_impls
                    .iter()
                    .filter(|implementation| implementation.spec == canonical)
                    .map(|implementation| {
                        Ok(etas_std::StdImplFact {
                            self_type: projector.ty(&implementation.self_type)?,
                            spec: etas_std::StdSpecRef {
                                path: canonical.clone(),
                                args: implementation
                                    .args
                                    .iter()
                                    .map(|ty| projector.ty(ty))
                                    .collect::<Result<_, DriverError>>()?,
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, DriverError>>()?;
                builder.replace_spec_impls(&canonical, implementations);
            }
            continue;
        }
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
        builder.source_visibility(
            symbol,
            match spec.visibility {
                metadata::Visibility::Public => etas_std::StdSymbolVisibility::Public,
                metadata::Visibility::Private => etas_std::StdSymbolVisibility::Private,
            },
        );
        registered.insert(canonical.clone());
        symbols.insert(canonical, symbol);
    }
    Ok(())
}
