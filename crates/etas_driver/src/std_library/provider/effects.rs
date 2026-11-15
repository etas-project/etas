use super::*;
use etas_std::{EffectDecl, StdDecl, StdModuleId, StdSymbolId, StdSymbolKind};

pub(super) fn publish(
    sdk: &LoadedStdSdk,
    builder: &mut StdRegistryBuilder,
    modules: &mut BTreeMap<Vec<String>, StdModuleId>,
    symbols: &mut BTreeMap<Vec<String>, StdSymbolId>,
    declarations: &BTreeMap<Vec<String>, &DefinitionKey>,
    aliases: &BTreeMap<Vec<String>, Vec<String>>,
) -> Result<(), DriverError> {
    let mut registered = std::collections::BTreeSet::new();
    for effect in &sdk.public.public_metadata.effects {
        if effect.visibility != metadata::Visibility::Public {
            continue;
        }
        let canonical = canonical_export(&effect.path, aliases)?;
        let Some(key) = declarations.get(&canonical).copied() else {
            continue;
        };
        if !registered.insert(canonical.clone()) {
            continue;
        }
        let projector = TypeProjector {
            owner: key,
            params: &effect.generic_params,
        };
        let params = effect
            .generic_params
            .iter()
            .map(|param| projector.param(param))
            .collect::<Result<Vec<_>, _>>()?;
        let extends = sdk
            .public
            .effect_metadata
            .extensions
            .iter()
            .filter(|edge| edge.child == canonical)
            .map(|edge| edge.parent.join("."))
            .collect();
        let source_role = sdk
            .image
            .declarations
            .get(key)
            .and_then(|item| sdk.image.program.items.get(*item))
            .and_then(etas_hir::HirItem::symbol)
            .and_then(|symbol| sdk.image.types.source_type_roles.get(&symbol));
        if let Some(native) = source_role {
            if symbols.get(&canonical) != Some(native) {
                return Err(invalid(
                    "checked std effect differs from its execution role",
                ));
            }
            builder.bind_declaration(*native, key.clone());
            builder.source_effect_declaration(*native, params, extends);
        } else {
            if symbols.contains_key(&canonical) {
                return Err(invalid(
                    "std effect replaces a native contract without a checked role",
                ));
            }
            let (name, parent) = canonical
                .split_last()
                .ok_or_else(|| invalid("std effect has no path"))?;
            let module = *modules.entry(parent.to_vec()).or_insert_with(|| {
                builder.module(&parent.iter().map(String::as_str).collect::<Vec<_>>(), "")
            });
            let mut declaration = EffectDecl::standard(name, &[]);
            declaration.params = params;
            declaration.extends = extends;
            let id = builder.symbol(
                module,
                name,
                StdSymbolKind::Effect,
                StdDecl::Effect(declaration),
                "",
            );
            builder.bind_declaration(id, key.clone());
            symbols.insert(canonical, id);
        }
    }
    Ok(())
}
