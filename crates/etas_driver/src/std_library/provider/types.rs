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
        let source_role = sdk
            .image
            .declarations
            .get(key)
            .and_then(|item| sdk.image.program.items.get(*item))
            .and_then(etas_hir::HirItem::symbol)
            .and_then(|symbol| sdk.image.types.source_type_roles.get(&symbol));
        if registered.contains(&canonical) {
            continue;
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
        let params = signature
            .generic_params
            .iter()
            .map(|param| projector.param(param))
            .collect::<Result<Vec<_>, _>>()?;
        let kind = match ty.kind {
            metadata::TypeKind::Enum => TypeDeclKind::Enum,
            metadata::TypeKind::Nominal => TypeDeclKind::Struct,
            _ => {
                return Err(invalid(format!(
                    "std source type {} requires its declaration contract before publication",
                    canonical.join(".")
                )));
            }
        };
        let id = if let Some(native) = source_role {
            if symbols.get(&canonical) != Some(native) {
                return Err(invalid(
                    "checked std role differs from its native canonical declaration",
                ));
            }
            builder.bind_declaration(*native, key.clone());
            builder.source_type_parameters(*native, params.clone());
            if !ty.projected_fields.is_empty() {
                builder.source_type_projected_fields(
                    *native,
                    ty.projected_fields
                        .iter()
                        .map(|field| {
                            Ok(etas_std::StdRecordField {
                                name: field.name.clone(),
                                ty: projector.ty(&field.ty)?,
                            })
                        })
                        .collect::<Result<_, DriverError>>()?,
                );
            }
            *native
        } else {
            if symbols.contains_key(&canonical) {
                return Err(invalid(format!(
                    "std source type {} replaces a native type without a checked role contract",
                    canonical.join(".")
                )));
            }
            let mut declaration = TypeDecl::generic_params(name, params.clone(), kind);
            declaration.representation =
                ty.children.first().map(|ty| projector.ty(ty)).transpose()?;
            let id = builder.symbol(
                module,
                name,
                StdSymbolKind::Type,
                StdDecl::Type(declaration),
                "",
            );
            builder.bind_declaration(id, key.clone());
            symbols.insert(canonical.clone(), id);
            id
        };
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
            constructor.output =
                if source_role.is_some() && canonical == ["std", "result", "Result"] {
                    StdType::Result {
                        ok: Box::new(StdType::Var(params[0].name.clone())),
                        err: Box::new(StdType::Var(params[1].name.clone())),
                    }
                } else if source_role.is_some() && canonical == ["std", "option", "Option"] {
                    StdType::Option(Box::new(StdType::Var(params[0].name.clone())))
                } else if params.is_empty() {
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
            let mut variant_path = canonical.clone();
            variant_path.push(variant.name.clone());
            symbols.insert(variant_path, constructor);
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
