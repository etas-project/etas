use std::{collections::BTreeMap, path::Path, sync::Arc};

use etas_code::{CallableId, DefinitionKey, descriptor::GenericOwner};
use etas_package_metadata as metadata;
use etas_std::{StdRegistry, StdRegistryBuilder, StdType};

use super::sdk::{LoadedStdSdk, STD_TOOLCHAIN_ID, load_standard_library_sdk};
use crate::DriverError;

#[derive(Clone, Debug, Default)]
pub enum StdProvider {
    #[default]
    Legacy,
    Artifact(Arc<ArtifactStdProvider>),
}

#[derive(Clone, Debug)]
pub struct ArtifactStdProvider {
    pub sdk: LoadedStdSdk,
    pub registry: Arc<StdRegistry>,
    pub library: Arc<etas_frontend::StandardLibrary>,
    pub identity: String,
    pub bootstrap_identity: String,
}

impl StdProvider {
    pub fn artifact(sysroot: Option<&Path>) -> Result<Self, DriverError> {
        let sysroot = match sysroot {
            Some(path) => path.to_owned(),
            None => {
                let binary = std::env::current_exe().map_err(|source| DriverError::Io {
                    path: "<current executable>".into(),
                    source,
                })?;
                binary
                    .parent()
                    .and_then(Path::parent)
                    .ok_or_else(|| invalid("toolchain executable has no installation prefix"))?
                    .join("lib/etas")
                    .join(STD_TOOLCHAIN_ID)
            }
        };
        let sdk = load_standard_library_sdk(&sysroot)?;
        let registry = Arc::new(query_registry(&sdk)?);
        let checked = sdk
            .image
            .clone()
            .into_checked_from_image(&sdk.root)
            .map_err(invalid)?;
        let package = etas_frontend::ProjectExternalPackageInput {
            id: etas_frontend::ExternalPackageId(0),
            name: sdk.manifest.package.clone(),
            version: sdk.manifest.version.clone(),
            edition: sdk.manifest.edition.clone(),
            import_root: "std".into(),
            std_compatibility: sdk.manifest.std_contract.clone(),
        };
        let index = etas_package::package_index_from_metadata(
            sdk.public.clone(),
            &sdk.root.join(&sdk.manifest.artifact),
        )?;
        let public_metadata = crate::package_env::external_public_metadata_input(
            &package,
            std::slice::from_ref(&package),
            &index.public_metadata,
        );
        let library = Arc::new(etas_frontend::StandardLibrary {
            program: checked,
            package,
            metadata: public_metadata,
            origins: sdk
                .public
                .public_metadata
                .definition_origins
                .clone()
                .ok_or_else(|| invalid("std artifact has no canonical declaration origins"))?,
            descriptors: Arc::new(
                sdk.public
                    .public_metadata
                    .definition_descriptors
                    .clone()
                    .ok_or_else(|| {
                        invalid("std artifact has no canonical public definition descriptors")
                    })?,
            ),
        });
        let identity = metadata::blake3_hash(&serde_json::to_vec(&sdk.manifest).map_err(invalid)?);
        let bootstrap_identity = metadata::blake3_hash(
            &serde_json::to_vec(&sdk.image.native_registry).map_err(invalid)?,
        );
        Ok(Self::Artifact(Arc::new(ArtifactStdProvider {
            sdk,
            registry,
            library,
            identity,
            bootstrap_identity,
        })))
    }

    pub(crate) fn configure(&self, input: &mut etas_frontend::ProjectInput) {
        if let Self::Artifact(provider) = self {
            if input.options.standard_library_mode
                != etas_frontend::StandardLibraryMode::SourceBootstrap
            {
                input.options.standard_library_mode = etas_frontend::StandardLibraryMode::Artifact;
            }
            input.options.standard_library_artifact = Some(
                if input.options.standard_library_mode
                    == etas_frontend::StandardLibraryMode::SourceBootstrap
                {
                    provider.bootstrap_identity.clone()
                } else {
                    provider.identity.clone()
                },
            );
        }
    }
}

/// Project structured checked package interfaces into the existing std query
/// API. Every source flow keeps its canonical bundle callable, never a native
/// intrinsic substitute or a handwritten source signature.
fn query_registry(sdk: &LoadedStdSdk) -> Result<StdRegistry, DriverError> {
    let native = sdk
        .image
        .native_registry
        .clone()
        .into_registry()
        .map_err(invalid)?;
    let mut modules = native
        .modules()
        .map(|m| (m.path.clone(), m.id))
        .collect::<BTreeMap<_, _>>();
    let mut symbols = native
        .symbols()
        .map(|s| (s.qualified_path.clone(), s.id))
        .collect::<BTreeMap<_, _>>();
    let mut builder = StdRegistryBuilder::from_registry(native);
    let declarations = sdk
        .image
        .declarations
        .keys()
        .map(|key| {
            let mut path = key.module().to_vec();
            path.extend_from_slice(key.declaration());
            (path, key)
        })
        .collect::<BTreeMap<_, _>>();
    let exports = &sdk.public.public_metadata.re_exports;
    let aliases = exports
        .iter()
        .map(|export| (export.exported.clone(), export.from.clone()))
        .collect();
    let mut flows = sdk
        .public
        .public_metadata
        .flows
        .iter()
        .map(|flow| {
            let canonical = canonical_export(&flow.path, &aliases)?;
            Ok((flow, canonical))
        })
        .collect::<Result<Vec<_>, DriverError>>()?;
    flows.sort_by(|(left, from), (right, to)| {
        (left.path != *from, &left.path).cmp(&(right.path != *to, &right.path))
    });
    let mut registered = std::collections::BTreeSet::new();
    for (flow, canonical) in flows {
        if flow.visibility != metadata::Visibility::Public {
            continue;
        }
        // Re-exports share the original symbol and checked body identity.
        if (symbols.contains_key(&canonical) && !declarations.contains_key(&canonical))
            || !registered.insert(canonical.clone())
        {
            continue;
        }
        let key = *declarations.get(&canonical).ok_or_else(|| {
            invalid(format!(
                "std source {} has no canonical checked body",
                flow.path.join(".")
            ))
        })?;
        let (name, module) = flow
            .path
            .split_last()
            .ok_or_else(|| invalid("std flow has an empty path"))?;
        let module = *modules.entry(module.to_vec()).or_insert_with(|| {
            builder.module(&module.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        let projector = TypeProjector {
            owner: key,
            params: &flow.generic_params,
        };
        let summary = sdk
            .public
            .public_metadata
            .effect_summaries
            .iter()
            .find(|s| s.item == flow.path)
            .ok_or_else(|| {
                invalid(format!(
                    "std source {} has no checked effect contract",
                    flow.path.join(".")
                ))
            })?;
        let effects = projector.row(Some(&summary.public_effects))?;
        let requests = projector.row(Some(&summary.requested_actions))?;
        let declaration = etas_std::FlowDecl {
            name: name.clone(),
            type_params: flow
                .generic_params
                .iter()
                .map(|p| projector.param(p))
                .collect::<Result<_, _>>()?,
            params: flow
                .input
                .iter()
                .map(|ty| projector.ty(ty))
                .collect::<Result<_, _>>()?,
            output: projector.ty(flow
                .output
                .as_ref()
                .ok_or_else(|| invalid("std flow has no output type"))?)?,
            public_effects: effects.effects,
            public_effect_tail: effects.tail,
            requested_actions: requests.effects,
            requested_action_tail: requests.tail,
            source_method: None,
        };
        let symbol = builder.source_flow(
            module,
            name,
            declaration,
            CallableId::declaration(key.clone()).map_err(invalid)?,
            "",
        );
        if flow.path[..flow.path.len() - 1] == ["std", "prelude"] {
            builder.prelude(name, symbol);
        }
        symbols.insert(flow.path.clone(), symbol);
        symbols.insert(canonical, symbol);
    }
    for export in &sdk.public.public_metadata.re_exports {
        let target = *symbols
            .get(&canonical_export(&export.from, &aliases)?)
            .ok_or_else(|| {
                invalid(format!(
                    "std re-export has no target {}",
                    export.from.join(".")
                ))
            })?;
        let (name, path) = export
            .exported
            .split_last()
            .ok_or_else(|| invalid("std re-export has no path"))?;
        if path == ["std", "prelude"] {
            builder.prelude(name, target);
        }
        if symbols.get(&export.exported) == Some(&target) {
            continue;
        }
        let module = *modules.entry(path.to_vec()).or_insert_with(|| {
            builder.module(&path.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        builder.re_export(module, name, target).map_err(invalid)?;
        symbols.insert(export.exported.clone(), target);
    }
    builder.try_finish().map_err(invalid)
}

fn canonical_export(
    path: &[String],
    exports: &BTreeMap<Vec<String>, Vec<String>>,
) -> Result<Vec<String>, DriverError> {
    let mut canonical = path;
    for _ in 0..=exports.len() {
        match exports.get(canonical) {
            Some(target) if target != canonical => canonical = target,
            _ => return Ok(canonical.to_vec()),
        }
    }
    Err(invalid(format!(
        "std re-export cycle at {}",
        path.join(".")
    )))
}

struct TypeProjector<'a> {
    owner: &'a DefinitionKey,
    params: &'a [metadata::GenericParam],
}

impl TypeProjector<'_> {
    fn param(
        &self,
        param: &metadata::GenericParam,
    ) -> Result<etas_std::StdGenericParam, DriverError> {
        Ok(etas_std::StdGenericParam {
            name: param.name.clone(),
            kind: match param.kind {
                metadata::GenericParamKind::Type => etas_std::StdGenericParamKind::Type,
                metadata::GenericParamKind::Effect => etas_std::StdGenericParamKind::EffectRow,
            },
            bounds: param
                .bounds
                .iter()
                .map(|bound| {
                    Ok(etas_std::StdSpecRef {
                        path: bound.spec.clone(),
                        args: bound
                            .args
                            .iter()
                            .map(|ty| self.ty(ty))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, DriverError>>()?,
        })
    }

    fn ty(&self, ty: &metadata::Type) -> Result<StdType, DriverError> {
        use metadata::TypeKind as K;
        let child = |index: usize| {
            self.ty(ty
                .children
                .get(index)
                .ok_or_else(|| invalid("std type contract is missing its operand"))?)
                .map(Box::new)
        };
        Ok(match ty.kind {
            K::Primitive => StdType::Primitive(primitive(&ty.name)?),
            K::Parameter => {
                let param = ty
                    .parameter
                    .as_ref()
                    .ok_or_else(|| invalid("std type parameter has no canonical binder"))?;
                if param.owner != GenericOwner::Declaration(self.owner.clone()) {
                    return Err(invalid(
                        "std type parameter belongs to a different callable",
                    ));
                }
                StdType::Var(
                    self.params
                        .get(param.position as usize)
                        .ok_or_else(|| invalid("std type parameter has no declared binder"))?
                        .name
                        .clone(),
                )
            }
            K::Var
                if self.params.iter().any(|param| {
                    param.kind == metadata::GenericParamKind::Type && param.name == ty.name
                }) =>
            {
                StdType::Var(ty.name.clone())
            }
            K::Named | K::Nominal | K::Enum => StdType::Named(ty.path.join(".")),
            K::Alias => {
                return self.ty(ty
                    .children
                    .first()
                    .ok_or_else(|| invalid("std alias has no target"))?);
            }
            K::Applied => StdType::NamedApplied {
                name: ty.path.join("."),
                args: ty
                    .children
                    .iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Result<_, _>>()?,
            },
            K::Array => StdType::Array(child(0)?),
            K::List => StdType::List(child(0)?),
            K::Set => StdType::Set(child(0)?),
            K::Range => StdType::Range(child(0)?),
            K::Slice => StdType::Slice(child(0)?),
            K::Option => StdType::Option(child(0)?),
            K::Map => StdType::Map {
                key: child(0)?,
                value: child(1)?,
            },
            K::Store => StdType::Store {
                key: child(0)?,
                value: child(1)?,
            },
            K::Result => StdType::Result {
                ok: child(0)?,
                err: child(1)?,
            },
            K::Tuple => StdType::Tuple(
                ty.children
                    .iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Result<_, _>>()?,
            ),
            K::Record => StdType::Record(
                ty.fields
                    .iter()
                    .map(|f| {
                        Ok(etas_std::StdRecordField {
                            name: f.name.clone(),
                            ty: self.ty(&f.ty)?,
                        })
                    })
                    .collect::<Result<_, DriverError>>()?,
            ),
            K::Function => {
                let (output, input) = ty
                    .children
                    .split_last()
                    .ok_or_else(|| invalid("std function type has no result"))?;
                StdType::Function {
                    input: input
                        .iter()
                        .map(|ty| self.ty(ty))
                        .collect::<Result<_, _>>()?,
                    output: Box::new(self.ty(output)?),
                    effects: self.row(ty.effects.as_ref())?,
                }
            }
            K::Task => StdType::Task {
                output: child(0)?,
                effects: self.row(ty.effects.as_ref())?,
            },
            K::Prompt => StdType::Prompt,
            K::PromptPart => StdType::PromptPart,
            K::Message => StdType::Message(child(0)?),
            K::MemorySelection => StdType::MemorySelection(child(0)?),
            K::MemoryRegion => StdType::MemoryRegion(child(0)?),
            K::Trusted | K::Untrusted | K::Secret | K::Public | K::Sanitized => StdType::Trust {
                wrapper: match ty.kind {
                    K::Trusted => etas_std::StdTrustWrapper::Trusted,
                    K::Untrusted => etas_std::StdTrustWrapper::Untrusted,
                    K::Secret => etas_std::StdTrustWrapper::Secret,
                    K::Public => etas_std::StdTrustWrapper::Public,
                    _ => etas_std::StdTrustWrapper::Sanitized,
                },
                inner: child(0)?,
            },
            K::Existential | K::Var | K::Handler | K::ResourceHandle => {
                return Err(invalid(format!(
                    "std query type {:?} requires its declaration contract before publication",
                    ty.kind
                )));
            }
        })
    }

    fn row(
        &self,
        row: Option<&metadata::EffectRow>,
    ) -> Result<etas_std::StdEffectRowExpr, DriverError> {
        let Some(row) = row else {
            return Ok(Default::default());
        };
        Ok(etas_std::StdEffectRowExpr {
            tail: row.tail.clone(),
            effects: row
                .effects
                .iter()
                .map(|effect| {
                    Ok(etas_std::StdEffectRef {
                        path: effect.path.clone(),
                        args: effect
                            .args
                            .iter()
                            .map(|arg| {
                                Ok(match arg.kind {
                                    metadata::EffectArgKind::Type => etas_std::StdStaticArg::Type(
                                        self.ty(arg.ty.as_ref().ok_or_else(|| {
                                            invalid("std effect argument has no type")
                                        })?)?,
                                    ),
                                    metadata::EffectArgKind::Path => {
                                        etas_std::StdStaticArg::Path(arg.path.clone())
                                    }
                                    metadata::EffectArgKind::String => {
                                        etas_std::StdStaticArg::String(arg.value.clone())
                                    }
                                    metadata::EffectArgKind::Int => {
                                        etas_std::StdStaticArg::Int(arg.value.clone())
                                    }
                                    metadata::EffectArgKind::Wildcard => {
                                        etas_std::StdStaticArg::Wildcard
                                    }
                                })
                            })
                            .collect::<Result<_, DriverError>>()?,
                    })
                })
                .collect::<Result<_, DriverError>>()?,
        })
    }
}

fn primitive(name: &str) -> Result<etas_std::StdPrimitiveType, DriverError> {
    use etas_std::StdPrimitiveType as P;
    Ok(match name {
        "bool" => P::Bool,
        "i8" => P::I8,
        "i16" => P::I16,
        "i32" => P::I32,
        "i64" => P::I64,
        "i128" => P::I128,
        "isize" => P::ISize,
        "u8" => P::U8,
        "u16" => P::U16,
        "u32" => P::U32,
        "u64" => P::U64,
        "u128" => P::U128,
        "usize" => P::USize,
        "f32" => P::F32,
        "f64" => P::F64,
        "char" => P::Char,
        "string" => P::String,
        "bytes" => P::Bytes,
        "unit" => P::Unit,
        "never" => P::Never,
        _ => return Err(invalid(format!("unknown std primitive type `{name}`"))),
    })
}

fn invalid(error: impl std::fmt::Display) -> DriverError {
    DriverError::InvalidInput(error.to_string())
}
