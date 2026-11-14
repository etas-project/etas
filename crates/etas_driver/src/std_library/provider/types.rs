use super::*;
use etas_code::{DefinitionKey, DefinitionKind};
use etas_std::{
    FlowDecl, StdDecl, StdModuleId, StdSymbolId, StdSymbolKind, TypeDecl, TypeDeclKind,
};

pub(super) fn publish(
    sdk: &LoadedStdSdk,
    builder: &mut StdRegistryBuilder,
    modules: &mut BTreeMap<Vec<String>, StdModuleId>,
    symbols: &mut BTreeMap<Vec<String>, StdSymbolId>,
    declarations: &BTreeMap<Vec<String>, &DefinitionKey>,
    aliases: &BTreeMap<Vec<String>, Vec<String>>,
) -> Result<(), DriverError> {
    let public = &sdk.public.public_metadata;
    let mut types = public
        .types
        .iter()
        .chain(&public.enums)
        .map(|ty| Ok((ty, canonical_export(&ty.path, aliases)?)))
        .collect::<Result<Vec<_>, DriverError>>()?;
    types.sort_by(|(left, from), (right, to)| {
        (left.path != *from, &left.path).cmp(&(right.path != *to, &right.path))
    });
    let mut registered = std::collections::BTreeSet::new();
    for (signature, canonical) in types {
        if signature.visibility != metadata::Visibility::Public {
            continue;
        }
        let Some(key) = declarations.get(&canonical).copied() else {
            continue;
        };
        if symbols.contains_key(&canonical) {
            // Re-exports retain the original slot. Native role replacement
            // requires a separately checked source role contract.
            if registered.contains(&canonical) {
                continue;
            }
            return Err(invalid(format!(
                "std source type {} replaces a native type without a checked role contract",
                canonical.join(".")
            )));
        }
        let (name, parent) = canonical
            .split_last()
            .ok_or_else(|| invalid("std type has no path"))?;
        let module = *modules.entry(parent.to_vec()).or_insert_with(|| {
            builder.module(&parent.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        let projector = TypeProjector {
            owner: key,
            params: &signature.generic_params,
        };
        let ty = signature
            .ty
            .as_ref()
            .ok_or_else(|| invalid("std source type has no checked shape"))?;
        let kind = match ty.kind {
            metadata::TypeKind::Nominal => TypeDeclKind::Struct,
            metadata::TypeKind::Enum => TypeDeclKind::Enum,
            _ => {
                return Err(invalid(format!(
                    "std source type {} requires its declaration contract before publication",
                    canonical.join(".")
                )));
            }
        };
        let mut declaration = TypeDecl::generic_params(
            name,
            signature
                .generic_params
                .iter()
                .map(|p| projector.param(p))
                .collect::<Result<_, _>>()?,
            kind,
        );
        declaration.representation = ty.children.first().map(|ty| projector.ty(ty)).transpose()?;
        let params = declaration.params.clone();
        let id = builder.symbol(
            module,
            name,
            StdSymbolKind::Type,
            StdDecl::Type(declaration),
            "",
        );
        builder.bind_declaration(id, key.clone());
        symbols.insert(canonical.clone(), id);
        registered.insert(canonical.clone());
        if kind != TypeDeclKind::Enum {
            continue;
        }
        for variant in &ty.fields {
            let (fields, names) = match variant.ty.kind {
                metadata::TypeKind::Tuple => (variant.ty.children.iter().collect::<Vec<_>>(), None),
                metadata::TypeKind::Record => (
                    variant.ty.fields.iter().map(|field| &field.ty).collect(),
                    Some(
                        variant
                            .ty
                            .fields
                            .iter()
                            .map(|field| field.name.clone())
                            .collect(),
                    ),
                ),
                _ => return Err(invalid("std enum variant has no checked payload shape")),
            };
            let mut constructor = FlowDecl::pure(&variant.name, &[], "unit");
            constructor.type_params = params.clone();
            constructor.params = fields
                .into_iter()
                .map(|ty| projector.ty(ty))
                .collect::<Result<_, _>>()?;
            constructor.output = if params.is_empty() {
                StdType::Named(canonical.join("."))
            } else {
                StdType::NamedApplied {
                    name: canonical.join("."),
                    args: params
                        .iter()
                        .map(|param| StdType::Var(param.name.clone()))
                        .collect(),
                }
            };
            let constructor = builder
                .enum_constructor(id, constructor, "")
                .map_err(invalid)?;
            let mut declaration = key.declaration().to_vec();
            declaration.push(variant.name.clone());
            builder.bind_declaration(
                constructor,
                DefinitionKey::new(
                    key.package(),
                    key.module().to_vec(),
                    declaration,
                    DefinitionKind::EnumVariant,
                )
                .map_err(invalid)?,
            );
            if let Some(names) = names {
                builder.named_enum_fields(constructor, names);
            }
        }
    }
    Ok(())
}
