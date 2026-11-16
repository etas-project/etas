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
    publish_actions(sdk, builder, modules, symbols, declarations)
}

fn publish_actions(
    sdk: &LoadedStdSdk,
    builder: &mut StdRegistryBuilder,
    modules: &mut BTreeMap<Vec<String>, StdModuleId>,
    symbols: &mut BTreeMap<Vec<String>, StdSymbolId>,
    declarations: &BTreeMap<Vec<String>, &DefinitionKey>,
) -> Result<(), DriverError> {
    for action in &sdk.public.public_metadata.actions {
        if action.visibility != metadata::Visibility::Public {
            continue;
        }
        let (name, owner) = action
            .path
            .split_last()
            .ok_or_else(|| invalid("source action has no owner"))?;
        let key = *declarations
            .get(owner)
            .ok_or_else(|| invalid("source action has no checked effect owner"))?;
        let item = sdk.image.declarations[key];
        let etas_hir::HirItem::Effect(effect) = &sdk.image.program.items[item] else {
            return Err(invalid("source action owner is not an effect"));
        };
        let source = effect
            .body
            .actions()
            .iter()
            .find(|source| {
                sdk.image
                    .program
                    .symbols
                    .get(source.symbol)
                    .unwrap()
                    .name
                    .rsplit('.')
                    .next()
                    == Some(name.as_str())
            })
            .ok_or_else(|| invalid("source action has no checked declaration"))?;
        let native = sdk
            .image
            .program
            .standard_action_targets
            .get(&source.symbol);
        let projector = TypeProjector {
            owner: key,
            params: &action.generic_params,
        };
        let selectors = action
            .effect_args
            .iter()
            .map(|kind| match kind {
                metadata::ActionArgKind::Type => Ok(etas_std::EffectActionArgKind::Type),
                _ => Err(invalid(
                    "native source action has an unsupported static selector",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let parameters = action
            .generic_params
            .iter()
            .map(|p| projector.param(p))
            .collect::<Result<_, _>>()?;
        let inputs = action
            .params
            .iter()
            .map(|p| projector.ty(p))
            .collect::<Result<_, _>>()?;
        let output = projector.ty(action
            .output
            .as_ref()
            .ok_or_else(|| invalid("source action has no checked result"))?)?;
        let module = *modules.entry(owner.to_vec()).or_insert_with(|| {
            builder.module(&owner.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        let symbol = if let Some(native) = native {
            builder.source_action_signature(
                *native,
                parameters,
                selectors,
                action.selector_param_names.clone(),
                inputs,
                output,
            );
            builder.re_export(module, name, *native).map_err(invalid)?;
            *native
        } else {
            let mut declaration =
                etas_std::EffectActionDecl::typed(&owner.join("."), name, inputs, output);
            declaration.type_params = parameters;
            declaration.effect_args = selectors;
            declaration.selector_param_names = action.selector_param_names.clone();
            builder.symbol(
                module,
                name,
                StdSymbolKind::EffectAction,
                StdDecl::EffectAction(declaration),
                "",
            )
        };
        symbols.insert(action.path.clone(), symbol);
    }
    Ok(())
}
