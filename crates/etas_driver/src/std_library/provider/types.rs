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
    let image = sdk.image()?;
    let public = &sdk.public()?.public_metadata;
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
        let Some(key) = declarations.get(&canonical).copied() else {
            continue;
        };
        let source_role = image
            .declarations
            .get(key)
            .and_then(|item| image.program.items.get(*item))
            .and_then(etas_hir::HirItem::symbol)
            .and_then(|symbol| image.types.source_type_roles.get(&symbol));
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
            metadata::TypeKind::Alias => TypeDeclKind::Alias,
            _ if source_role.is_some()
                && etas_std::StdSourceRole::from_path(&canonical.join("."))
                    .and_then(etas_std::StdSourceRole::primitive_type)
                    .is_some() =>
            {
                TypeDeclKind::Primitive
            }
            metadata::TypeKind::Nominal if source_role.is_none() && ty.children.is_empty() => {
                TypeDeclKind::Support
            }
            metadata::TypeKind::Nominal => TypeDeclKind::Struct,
            _ if source_role.is_some() => TypeDeclKind::Support,
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
            let alias = matches!(kind, TypeDeclKind::Alias | TypeDeclKind::Support)
                .then(|| projector.ty(ty))
                .transpose()?;
            builder.source_type_shape(*native, kind, alias);
            if etas_std::StdSourceRole::from_path(&canonical.join("."))
                .is_some_and(etas_std::StdSourceRole::is_represented_type)
                && let Some(representation) = ty.children.first()
            {
                builder.source_type_representation(*native, projector.ty(representation)?);
            }
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
            // Native observations attach to the selected source declaration;
            // their public fields and generic binders remain source-owned.
            declaration.code_role = etas_std::CodeTypeRole::from_source_path(&canonical.join("."));
            if ty.kind == metadata::TypeKind::Alias {
                declaration.canonical_type = Some(projector.ty(ty)?);
            } else if kind == TypeDeclKind::Support {
                declaration = declaration.intrinsic_only();
            } else {
                declaration.representation =
                    ty.children.first().map(|ty| projector.ty(ty)).transpose()?;
            }
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
        builder.source_visibility(
            id,
            match signature.visibility {
                metadata::Visibility::Public => etas_std::StdSymbolVisibility::Public,
                metadata::Visibility::Private => etas_std::StdSymbolVisibility::Private,
            },
        );
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
