use std::{path::Path, path::PathBuf};

use crate::{
    BinTarget, PackageError,
    metadata::{
        PackageActionSummaryMetadata, PackageAnnotationArgMetadata, PackageAnnotationFieldMetadata,
        PackageAnnotationMetadata, PackageAnnotationValueKindMetadata,
        PackageAnnotationValueMetadata, PackageCallableSpecSatisfactionMetadata,
        PackageEffectActionArgKindMetadata, PackageEffectActionSignatureMetadata,
        PackageEffectArgMetadata, PackageEffectExtensionMetadata, PackageEffectMetadata,
        PackageEffectRefMetadata, PackageEffectRowMetadata, PackageEffectSummaryMetadata,
        PackageEffectTagMetadata, PackageExternalExportMetadata, PackageExternalModuleMetadata,
        PackageExternalModuleOwnerMetadata, PackageIdentity, PackageIndex,
        PackageLatentFlowSummaryMetadata, PackageNamedSignatureMetadata, PackagePublicMetadata,
        PackageReExportMetadata, PackageRecordFieldMetadata, PackageSpecBoundMetadata,
        PackageSpecImplMetadata, PackageSpecKindMetadata, PackageSpecMethodMetadata,
        PackageSpecSignatureMetadata, PackageToolBindingMetadata, PackageToolSchemaMetadata,
        PackageTraceSpecClauseKindMetadata, PackageTraceSpecClauseMetadata,
        PackageTraceSpecConformanceMetadata, PackageTraceSpecConformanceTargetMetadata,
        PackageTraceSpecSummaryMetadata, PackageTypeMetadata, PackageTypeSpecSatisfactionMetadata,
    },
};

pub(super) fn package_index_from_metadata(
    metadata: etas_package_metadata::PackageMetadata,
    path: &Path,
) -> Result<PackageIndex, PackageError> {
    Ok(PackageIndex {
        version: metadata.version,
        package: package_identity_from_metadata(metadata.package),
        dependencies: metadata
            .dependencies
            .into_iter()
            .map(|dependency| resolved_dependency_from_metadata(dependency, path))
            .collect::<Result<Vec<_>, _>>()?,
        external_modules: metadata
            .external_modules
            .into_iter()
            .map(external_module_from_metadata)
            .collect(),
        public_metadata: public_metadata_from_metadata(metadata.public_metadata, path)?,
        effect_metadata: effect_metadata_from_metadata(metadata.effect_metadata),
        tool_bindings: metadata
            .tool_bindings
            .into_iter()
            .map(tool_binding_from_metadata)
            .collect(),
        bins: metadata
            .bins
            .into_iter()
            .map(bin_target_from_metadata)
            .collect(),
    })
}

fn public_metadata_from_metadata(
    metadata: etas_package_metadata::PublicMetadata,
    path: &Path,
) -> Result<PackagePublicMetadata, PackageError> {
    Ok(PackagePublicMetadata {
        modules: metadata
            .modules
            .into_iter()
            .map(external_module_from_metadata)
            .collect(),
        types: metadata
            .types
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        values: metadata
            .values
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        enums: metadata
            .enums
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        flows: metadata
            .flows
            .into_iter()
            .map(|signature| flow_signature_from_metadata(signature, path))
            .collect::<Result<Vec<_>, _>>()?,
        agents: metadata
            .agents
            .into_iter()
            .map(|signature| agent_signature_from_metadata(signature, path))
            .collect::<Result<Vec<_>, _>>()?,
        tools: metadata
            .tools
            .into_iter()
            .map(|signature| tool_signature_from_metadata(signature, path))
            .collect::<Result<Vec<_>, _>>()?,
        effects: metadata
            .effects
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        actions: metadata
            .actions
            .into_iter()
            .map(|signature| action_signature_from_metadata(signature, path))
            .collect::<Result<Vec<_>, _>>()?,
        trace_specs: metadata
            .trace_specs
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        spec_signatures: metadata
            .spec_signatures
            .into_iter()
            .map(|signature| spec_signature_from_metadata(signature, path))
            .collect::<Result<Vec<_>, _>>()?,
        spec_impls: metadata
            .spec_impls
            .into_iter()
            .map(spec_impl_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        type_spec_satisfactions: metadata
            .type_spec_satisfactions
            .into_iter()
            .map(type_spec_satisfaction_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        callable_spec_satisfactions: metadata
            .callable_spec_satisfactions
            .into_iter()
            .map(callable_spec_satisfaction_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        trace_spec_conformances: metadata
            .trace_spec_conformances
            .into_iter()
            .map(trace_spec_conformance_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        protocols: metadata
            .protocols
            .into_iter()
            .map(named_signature_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        effect_summaries: metadata
            .effect_summaries
            .into_iter()
            .map(effect_summary_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        action_summaries: metadata
            .action_summaries
            .into_iter()
            .map(action_summary_from_metadata)
            .collect(),
        tool_schemas: metadata
            .tool_schemas
            .into_iter()
            .map(|schema| tool_schema_from_metadata(schema, path))
            .collect::<Result<Vec<_>, _>>()?,
        trace_spec_summaries: metadata
            .trace_spec_summaries
            .into_iter()
            .map(trace_spec_summary_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        re_exports: metadata
            .re_exports
            .into_iter()
            .map(re_export_from_metadata)
            .collect(),
        annotations: metadata
            .annotations
            .into_iter()
            .map(annotation_from_metadata)
            .collect(),
        fingerprint: metadata.fingerprint,
    })
}

fn annotation_from_metadata(
    annotation: etas_package_metadata::AnnotationMetadata,
) -> PackageAnnotationMetadata {
    PackageAnnotationMetadata {
        item: annotation.item,
        path: annotation.path,
        args: annotation
            .args
            .into_iter()
            .map(annotation_arg_from_metadata)
            .collect(),
    }
}

fn annotation_arg_from_metadata(
    arg: etas_package_metadata::AnnotationArgMetadata,
) -> PackageAnnotationArgMetadata {
    PackageAnnotationArgMetadata {
        name: arg.name,
        value: annotation_value_from_metadata(arg.value),
    }
}

fn annotation_field_from_metadata(
    field: etas_package_metadata::AnnotationFieldMetadata,
) -> PackageAnnotationFieldMetadata {
    PackageAnnotationFieldMetadata {
        name: field.name,
        value: annotation_value_from_metadata(field.value),
    }
}

fn annotation_value_from_metadata(
    value: etas_package_metadata::AnnotationValueMetadata,
) -> PackageAnnotationValueMetadata {
    PackageAnnotationValueMetadata {
        kind: annotation_value_kind_from_metadata(value.kind),
        value: value.value,
        path: value.path,
        elements: value
            .elements
            .into_iter()
            .map(annotation_value_from_metadata)
            .collect(),
        fields: value
            .fields
            .into_iter()
            .map(annotation_field_from_metadata)
            .collect(),
    }
}

fn annotation_value_kind_from_metadata(
    kind: etas_package_metadata::AnnotationValueKind,
) -> PackageAnnotationValueKindMetadata {
    match kind {
        etas_package_metadata::AnnotationValueKind::Unit => {
            PackageAnnotationValueKindMetadata::Unit
        }
        etas_package_metadata::AnnotationValueKind::Bool => {
            PackageAnnotationValueKindMetadata::Bool
        }
        etas_package_metadata::AnnotationValueKind::Int => PackageAnnotationValueKindMetadata::Int,
        etas_package_metadata::AnnotationValueKind::Float => {
            PackageAnnotationValueKindMetadata::Float
        }
        etas_package_metadata::AnnotationValueKind::String => {
            PackageAnnotationValueKindMetadata::String
        }
        etas_package_metadata::AnnotationValueKind::Char => {
            PackageAnnotationValueKindMetadata::Char
        }
        etas_package_metadata::AnnotationValueKind::Path => {
            PackageAnnotationValueKindMetadata::Path
        }
        etas_package_metadata::AnnotationValueKind::Array => {
            PackageAnnotationValueKindMetadata::Array
        }
        etas_package_metadata::AnnotationValueKind::List => {
            PackageAnnotationValueKindMetadata::List
        }
        etas_package_metadata::AnnotationValueKind::Set => PackageAnnotationValueKindMetadata::Set,
        etas_package_metadata::AnnotationValueKind::Tuple => {
            PackageAnnotationValueKindMetadata::Tuple
        }
        etas_package_metadata::AnnotationValueKind::Record => {
            PackageAnnotationValueKindMetadata::Record
        }
        etas_package_metadata::AnnotationValueKind::Constructor => {
            PackageAnnotationValueKindMetadata::Constructor
        }
        etas_package_metadata::AnnotationValueKind::Limit => {
            PackageAnnotationValueKindMetadata::Limit
        }
    }
}

fn package_identity_from_metadata(
    identity: etas_package_metadata::PackageIdentity,
) -> PackageIdentity {
    PackageIdentity {
        name: identity.name,
        version: identity.version,
        edition: identity.edition,
    }
}

fn resolved_dependency_from_metadata(
    dependency: etas_package_metadata::ResolvedDependency,
    path: &Path,
) -> Result<crate::metadata::ResolvedDependency, PackageError> {
    Ok(crate::metadata::ResolvedDependency {
        identity: package_identity_from_metadata(dependency.identity),
        import_root: dependency.import_root,
        source: resolved_source_from_metadata(dependency.source),
        dependencies: dependency
            .dependencies
            .into_iter()
            .map(|dependency| resolved_dependency_from_metadata(dependency, path))
            .collect::<Result<Vec<_>, _>>()?,
        public_metadata: public_metadata_from_metadata(dependency.public_metadata, path)?,
        effect_metadata: effect_metadata_from_metadata(dependency.effect_metadata),
        tool_bindings: dependency
            .tool_bindings
            .into_iter()
            .map(tool_binding_from_metadata)
            .collect(),
    })
}

fn resolved_source_from_metadata(
    source: etas_package_metadata::ResolvedDependencySource,
) -> crate::metadata::ResolvedDependencySource {
    match source {
        etas_package_metadata::ResolvedDependencySource::Builtin { checksum } => {
            crate::metadata::ResolvedDependencySource::Builtin { checksum }
        }
        etas_package_metadata::ResolvedDependencySource::Registry {
            registry,
            checksum,
            store,
        } => crate::metadata::ResolvedDependencySource::Registry {
            registry,
            checksum,
            store,
        },
        etas_package_metadata::ResolvedDependencySource::Git {
            url,
            rev,
            checksum,
            store,
        } => crate::metadata::ResolvedDependencySource::Git {
            url,
            rev,
            checksum,
            store,
        },
        etas_package_metadata::ResolvedDependencySource::GitHubClone {
            repo,
            rev,
            checksum,
            store,
        } => crate::metadata::ResolvedDependencySource::GitHubClone {
            repo,
            rev,
            checksum,
            store,
        },
        etas_package_metadata::ResolvedDependencySource::GitHubRelease {
            repo,
            release,
            asset,
            asset_checksum,
            payload_checksum,
            store,
        } => crate::metadata::ResolvedDependencySource::GitHubRelease {
            repo,
            release,
            asset,
            asset_checksum,
            payload_checksum,
            store,
        },
        etas_package_metadata::ResolvedDependencySource::Path { path, checksum } => {
            crate::metadata::ResolvedDependencySource::Path { path, checksum }
        }
        etas_package_metadata::ResolvedDependencySource::Vendor {
            path,
            checksum,
            store,
        } => crate::metadata::ResolvedDependencySource::Vendor {
            path,
            checksum,
            store,
        },
    }
}

fn external_module_from_metadata(
    module: etas_package_metadata::ExternalModule,
) -> PackageExternalModuleMetadata {
    PackageExternalModuleMetadata {
        package: module.package.map(module_owner_from_metadata),
        id: module.id,
        path: module.path,
        exports: module
            .exports
            .into_iter()
            .map(external_export_from_metadata)
            .collect(),
    }
}

fn module_owner_from_metadata(
    owner: etas_package_metadata::ExternalModuleOwner,
) -> PackageExternalModuleOwnerMetadata {
    PackageExternalModuleOwnerMetadata {
        identity: package_identity_from_metadata(owner.identity),
        import_root: owner.import_root,
    }
}

fn external_export_from_metadata(
    export: etas_package_metadata::ExternalExport,
) -> PackageExternalExportMetadata {
    PackageExternalExportMetadata {
        id: export.id,
        name: export.name,
        visibility: visibility_from_metadata(export.visibility),
    }
}

fn named_signature_from_metadata(
    signature: etas_package_metadata::NamedSignature,
) -> Result<PackageNamedSignatureMetadata, PackageError> {
    Ok(PackageNamedSignatureMetadata {
        path: signature.path,
        visibility: visibility_from_metadata(signature.visibility),
        ty: signature.ty.map(type_from_metadata).transpose()?,
    })
}

fn flow_signature_from_metadata(
    signature: etas_package_metadata::CallableSignature,
    path: &Path,
) -> Result<crate::metadata::PackageFlowSignatureMetadata, PackageError> {
    Ok(crate::metadata::PackageFlowSignatureMetadata {
        path: signature.path,
        param_names: signature.param_names,
        params: signature
            .input
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: signature
            .output
            .map(type_from_metadata)
            .transpose()?
            .ok_or_else(|| missing(path, "flow signature is missing output type"))?,
        effects: signature
            .effects
            .map(effect_row_from_metadata)
            .transpose()?,
        visibility: visibility_from_metadata(signature.visibility),
    })
}

fn agent_signature_from_metadata(
    signature: etas_package_metadata::CallableSignature,
    path: &Path,
) -> Result<crate::metadata::PackageAgentSignatureMetadata, PackageError> {
    Ok(crate::metadata::PackageAgentSignatureMetadata {
        path: signature.path,
        param_names: signature.param_names,
        input: signature
            .input
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: signature
            .output
            .map(type_from_metadata)
            .transpose()?
            .ok_or_else(|| missing(path, "agent signature is missing output type"))?,
        effects: signature
            .effects
            .map(effect_row_from_metadata)
            .transpose()?,
        visibility: visibility_from_metadata(signature.visibility),
    })
}

fn tool_signature_from_metadata(
    signature: etas_package_metadata::CallableSignature,
    path: &Path,
) -> Result<crate::metadata::PackageToolSignatureMetadata, PackageError> {
    Ok(crate::metadata::PackageToolSignatureMetadata {
        path: signature.path,
        param_names: signature.param_names,
        input: signature
            .input
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: signature
            .output
            .map(type_from_metadata)
            .transpose()?
            .ok_or_else(|| missing(path, "tool signature is missing output type"))?,
        effects: signature
            .effects
            .map(effect_row_from_metadata)
            .transpose()?,
        visibility: visibility_from_metadata(signature.visibility),
    })
}

fn spec_signature_from_metadata(
    signature: etas_package_metadata::SpecSignature,
    path: &Path,
) -> Result<PackageSpecSignatureMetadata, PackageError> {
    Ok(PackageSpecSignatureMetadata {
        path: signature.path,
        visibility: visibility_from_metadata(signature.visibility),
        kind: spec_kind_from_metadata(signature.kind),
        param_names: signature.param_names,
        callable: signature
            .callable
            .map(|callable| flow_signature_from_metadata(callable, path))
            .transpose()?,
        methods: signature
            .methods
            .into_iter()
            .map(|method| spec_method_from_metadata(method, path))
            .collect::<Result<Vec<_>, _>>()?,
        super_specs: signature
            .super_specs
            .into_iter()
            .map(spec_bound_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn spec_method_from_metadata(
    method: etas_package_metadata::SpecMethod,
    path: &Path,
) -> Result<PackageSpecMethodMetadata, PackageError> {
    Ok(PackageSpecMethodMetadata {
        name: method.name,
        path: method.path,
        signature: method
            .signature
            .map(|signature| flow_signature_from_metadata(signature, path))
            .transpose()?,
    })
}

fn spec_bound_from_metadata(
    bound: etas_package_metadata::SpecBound,
) -> Result<PackageSpecBoundMetadata, PackageError> {
    Ok(PackageSpecBoundMetadata {
        spec: bound.spec,
        args: bound
            .args
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn spec_impl_from_metadata(
    implementation: etas_package_metadata::SpecImpl,
) -> Result<PackageSpecImplMetadata, PackageError> {
    Ok(PackageSpecImplMetadata {
        self_type: type_from_metadata(implementation.self_type)?,
        spec: implementation.spec,
        args: implementation
            .args
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        methods: implementation.methods,
    })
}

fn type_spec_satisfaction_from_metadata(
    fact: etas_package_metadata::TypeSpecSatisfaction,
) -> Result<PackageTypeSpecSatisfactionMetadata, PackageError> {
    Ok(PackageTypeSpecSatisfactionMetadata {
        self_type: type_from_metadata(fact.self_type)?,
        spec: fact.spec,
        args: fact
            .args
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn callable_spec_satisfaction_from_metadata(
    fact: etas_package_metadata::CallableSpecSatisfaction,
) -> Result<PackageCallableSpecSatisfactionMetadata, PackageError> {
    Ok(PackageCallableSpecSatisfactionMetadata {
        item: fact.item,
        spec: fact.spec,
        args: fact
            .args
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn trace_spec_conformance_from_metadata(
    fact: etas_package_metadata::TraceSpecConformance,
) -> Result<PackageTraceSpecConformanceMetadata, PackageError> {
    Ok(PackageTraceSpecConformanceMetadata {
        item: fact.item,
        target: match fact.target {
            etas_package_metadata::TraceSpecConformanceTarget::Inline => {
                PackageTraceSpecConformanceTargetMetadata::Inline
            }
            etas_package_metadata::TraceSpecConformanceTarget::Named { spec, args } => {
                PackageTraceSpecConformanceTargetMetadata::Named {
                    spec,
                    args: args
                        .into_iter()
                        .map(type_from_metadata)
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
        },
    })
}

fn spec_kind_from_metadata(kind: etas_package_metadata::SpecKind) -> PackageSpecKindMetadata {
    match kind {
        etas_package_metadata::SpecKind::Type => PackageSpecKindMetadata::Type,
        etas_package_metadata::SpecKind::Callable => PackageSpecKindMetadata::Callable,
        etas_package_metadata::SpecKind::Trace => PackageSpecKindMetadata::Trace,
    }
}

fn action_signature_from_metadata(
    signature: etas_package_metadata::ActionSignature,
    path: &Path,
) -> Result<PackageEffectActionSignatureMetadata, PackageError> {
    let signature = PackageEffectActionSignatureMetadata {
        path: signature.path,
        params: signature
            .params
            .into_iter()
            .map(type_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        effect_args: signature
            .effect_args
            .into_iter()
            .map(action_arg_kind_from_metadata)
            .collect(),
        selector_param_names: signature.selector_param_names,
        selector_defaults: signature
            .selector_defaults
            .into_iter()
            .map(|arg| arg.map(effect_arg_from_metadata).transpose())
            .collect::<Result<Vec<_>, _>>()?,
        output: signature
            .output
            .map(type_from_metadata)
            .transpose()?
            .ok_or_else(|| missing(path, "action signature is missing output type"))?,
        returns_never: signature.returns_never,
        visibility: visibility_from_metadata(signature.visibility),
    };
    validate_action_selector_metadata(&signature, path)?;
    Ok(signature)
}

fn validate_action_selector_metadata(
    signature: &PackageEffectActionSignatureMetadata,
    path: &Path,
) -> Result<(), PackageError> {
    if signature.selector_param_names.len() != signature.effect_args.len() {
        return Err(missing(
            path,
            &format!(
                "action signature `{}` selector_param_names length {} does not match effect_args length {}",
                signature.path.join("."),
                signature.selector_param_names.len(),
                signature.effect_args.len()
            ),
        ));
    }
    if signature.selector_defaults.len() != signature.effect_args.len() {
        return Err(missing(
            path,
            &format!(
                "action signature `{}` selector_defaults length {} does not match effect_args length {}",
                signature.path.join("."),
                signature.selector_defaults.len(),
                signature.effect_args.len()
            ),
        ));
    }
    for (index, (kind, default)) in signature
        .effect_args
        .iter()
        .zip(&signature.selector_defaults)
        .enumerate()
    {
        let Some(default) = default else {
            continue;
        };
        if !effect_arg_matches_action_arg_kind(default, kind) {
            return Err(missing(
                path,
                &format!(
                    "action signature `{}` selector default at index {index} does not match selector kind",
                    signature.path.join(".")
                ),
            ));
        }
    }
    Ok(())
}

fn effect_arg_matches_action_arg_kind(
    arg: &PackageEffectArgMetadata,
    kind: &PackageEffectActionArgKindMetadata,
) -> bool {
    if matches!(arg, PackageEffectArgMetadata::Wildcard) {
        return true;
    }
    match kind {
        PackageEffectActionArgKindMetadata::Type => {
            matches!(arg, PackageEffectArgMetadata::Type { .. })
        }
        PackageEffectActionArgKindMetadata::MemoryPlace => {
            matches!(arg, PackageEffectArgMetadata::Path { .. })
        }
        PackageEffectActionArgKindMetadata::StaticResourcePath { .. } => {
            matches!(arg, PackageEffectArgMetadata::Path { .. })
        }
        PackageEffectActionArgKindMetadata::StringPattern => {
            matches!(
                arg,
                PackageEffectArgMetadata::String { .. } | PackageEffectArgMetadata::Path { .. }
            )
        }
    }
}

fn action_arg_kind_from_metadata(
    kind: etas_package_metadata::ActionArgKind,
) -> PackageEffectActionArgKindMetadata {
    match kind {
        etas_package_metadata::ActionArgKind::Type => PackageEffectActionArgKindMetadata::Type,
        etas_package_metadata::ActionArgKind::MemoryPlace => {
            PackageEffectActionArgKindMetadata::MemoryPlace
        }
        etas_package_metadata::ActionArgKind::StaticResourcePath { ty } => {
            PackageEffectActionArgKindMetadata::StaticResourcePath { ty }
        }
        etas_package_metadata::ActionArgKind::StringPattern => {
            PackageEffectActionArgKindMetadata::StringPattern
        }
    }
}

fn type_from_metadata(
    ty: etas_package_metadata::Type,
) -> Result<PackageTypeMetadata, PackageError> {
    let child = |index: usize, children: &[etas_package_metadata::Type], label: &str| {
        children.get(index).cloned().ok_or_else(|| {
            missing(
                Path::new(".etas/package.etasmeta"),
                &format!("type `{:?}` is missing {label} child", ty.kind),
            )
        })
    };
    match ty.kind {
        etas_package_metadata::TypeKind::Primitive => {
            Ok(PackageTypeMetadata::Primitive { name: ty.name })
        }
        etas_package_metadata::TypeKind::Var => Ok(PackageTypeMetadata::Var { name: ty.name }),
        etas_package_metadata::TypeKind::Named if ty.children.is_empty() => {
            Ok(PackageTypeMetadata::Named { path: ty.path })
        }
        etas_package_metadata::TypeKind::Applied => Ok(PackageTypeMetadata::Applied {
            path: ty.path,
            args: ty
                .children
                .into_iter()
                .map(type_from_metadata)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        etas_package_metadata::TypeKind::Alias => Ok(PackageTypeMetadata::Alias {
            path: ty.path,
            target: Box::new(type_from_metadata(child(0, &ty.children, "target")?)?),
        }),
        etas_package_metadata::TypeKind::Nominal => {
            if ty.children.len() > 1 {
                return Err(missing(
                    Path::new(etas_package_metadata::PACKAGE_METADATA_FILE),
                    "nominal type has more than one representation child",
                ));
            }
            Ok(PackageTypeMetadata::Nominal {
                path: ty.path,
                representation: if ty.children.is_empty() {
                    None
                } else {
                    Some(Box::new(type_from_metadata(child(
                        0,
                        &ty.children,
                        "representation",
                    )?)?))
                },
            })
        }
        etas_package_metadata::TypeKind::Named => Err(missing(
            Path::new(etas_package_metadata::PACKAGE_METADATA_FILE),
            "named type cannot carry children",
        )),
        etas_package_metadata::TypeKind::Array => Ok(PackageTypeMetadata::Array {
            element: Box::new(type_from_metadata(child(0, &ty.children, "element")?)?),
        }),
        etas_package_metadata::TypeKind::List => Ok(PackageTypeMetadata::List {
            element: Box::new(type_from_metadata(child(0, &ty.children, "element")?)?),
        }),
        etas_package_metadata::TypeKind::Map => Ok(PackageTypeMetadata::Map {
            key: Box::new(type_from_metadata(child(0, &ty.children, "key")?)?),
            value: Box::new(type_from_metadata(child(1, &ty.children, "value")?)?),
        }),
        etas_package_metadata::TypeKind::Set => Ok(PackageTypeMetadata::Set {
            element: Box::new(type_from_metadata(child(0, &ty.children, "element")?)?),
        }),
        etas_package_metadata::TypeKind::Range => Ok(PackageTypeMetadata::Range {
            index: Box::new(type_from_metadata(child(0, &ty.children, "index")?)?),
        }),
        etas_package_metadata::TypeKind::Slice => Ok(PackageTypeMetadata::Slice {
            element: Box::new(type_from_metadata(child(0, &ty.children, "element")?)?),
        }),
        etas_package_metadata::TypeKind::Option => Ok(PackageTypeMetadata::Option {
            inner: Box::new(type_from_metadata(child(0, &ty.children, "inner")?)?),
        }),
        etas_package_metadata::TypeKind::Result => Ok(PackageTypeMetadata::Result {
            ok: Box::new(type_from_metadata(child(0, &ty.children, "ok")?)?),
            err: Box::new(type_from_metadata(child(1, &ty.children, "err")?)?),
        }),
        etas_package_metadata::TypeKind::Record => Ok(PackageTypeMetadata::Record {
            fields: ty
                .fields
                .into_iter()
                .map(|field| {
                    Ok(PackageRecordFieldMetadata {
                        name: field.name,
                        ty: type_from_metadata(field.ty)?,
                    })
                })
                .collect::<Result<Vec<_>, PackageError>>()?,
        }),
        etas_package_metadata::TypeKind::Tuple => Ok(PackageTypeMetadata::Tuple {
            elements: ty
                .children
                .into_iter()
                .map(type_from_metadata)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        etas_package_metadata::TypeKind::Function => {
            let mut children = ty.children.into_iter().collect::<Vec<_>>();
            let output = children.pop().ok_or_else(|| {
                missing(
                    Path::new(etas_package_metadata::PACKAGE_METADATA_FILE),
                    "function type is missing output",
                )
            })?;
            Ok(PackageTypeMetadata::Function {
                input: children
                    .into_iter()
                    .map(type_from_metadata)
                    .collect::<Result<Vec<_>, _>>()?,
                output: Box::new(type_from_metadata(output)?),
                effects: ty.effects.map(effect_row_from_metadata).transpose()?,
            })
        }
        etas_package_metadata::TypeKind::Handler => Ok(PackageTypeMetadata::Handler {
            handled: ty
                .effects
                .map(effect_row_from_metadata)
                .transpose()?
                .ok_or_else(|| {
                    missing(
                        Path::new(etas_package_metadata::PACKAGE_METADATA_FILE),
                        "handler type is missing handled effects",
                    )
                })?,
            produced: ty
                .produced_effects
                .map(effect_row_from_metadata)
                .transpose()?,
            result: if let Some(result) = ty.children.into_iter().next() {
                Some(Box::new(type_from_metadata(result)?))
            } else {
                None
            },
        }),
        etas_package_metadata::TypeKind::Prompt => Ok(PackageTypeMetadata::Prompt),
        etas_package_metadata::TypeKind::PromptPart => Ok(PackageTypeMetadata::PromptPart),
        etas_package_metadata::TypeKind::Message => Ok(PackageTypeMetadata::Message {
            inner: Box::new(type_from_metadata(child(0, &ty.children, "inner")?)?),
        }),
        etas_package_metadata::TypeKind::MemorySelection => {
            Ok(PackageTypeMetadata::MemorySelection {
                inner: Box::new(type_from_metadata(child(0, &ty.children, "inner")?)?),
            })
        }
        etas_package_metadata::TypeKind::Store => Ok(PackageTypeMetadata::Store {
            key: Box::new(type_from_metadata(child(0, &ty.children, "key")?)?),
            value: Box::new(type_from_metadata(child(1, &ty.children, "value")?)?),
        }),
        etas_package_metadata::TypeKind::MemoryRegion => Ok(PackageTypeMetadata::MemoryRegion {
            schema: Box::new(type_from_metadata(child(0, &ty.children, "schema")?)?),
        }),
        etas_package_metadata::TypeKind::ResourceHandle => {
            Ok(PackageTypeMetadata::ResourceHandle {
                name: ty.name,
                args: ty
                    .children
                    .into_iter()
                    .map(type_from_metadata)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        etas_package_metadata::TypeKind::Trusted
        | etas_package_metadata::TypeKind::Untrusted
        | etas_package_metadata::TypeKind::Secret
        | etas_package_metadata::TypeKind::Public
        | etas_package_metadata::TypeKind::Sanitized => Ok(PackageTypeMetadata::Trust {
            wrapper: trust_wrapper_name(ty.kind).to_owned(),
            inner: Box::new(type_from_metadata(child(0, &ty.children, "inner")?)?),
        }),
    }
}

fn trust_wrapper_name(kind: etas_package_metadata::TypeKind) -> &'static str {
    match kind {
        etas_package_metadata::TypeKind::Trusted => "Trusted",
        etas_package_metadata::TypeKind::Untrusted => "Untrusted",
        etas_package_metadata::TypeKind::Secret => "Secret",
        etas_package_metadata::TypeKind::Public => "Public",
        etas_package_metadata::TypeKind::Sanitized => "Sanitized",
        _ => unreachable!("trust_wrapper_name is only called for trust wrapper type kinds"),
    }
}

fn effect_row_from_metadata(
    row: etas_package_metadata::EffectRow,
) -> Result<PackageEffectRowMetadata, PackageError> {
    Ok(PackageEffectRowMetadata {
        effects: row
            .effects
            .into_iter()
            .map(effect_ref_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn effect_ref_from_metadata(
    effect: etas_package_metadata::EffectRef,
) -> Result<PackageEffectRefMetadata, PackageError> {
    Ok(PackageEffectRefMetadata {
        path: effect.path,
        args: effect
            .args
            .into_iter()
            .map(effect_arg_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn effect_arg_from_metadata(
    arg: etas_package_metadata::EffectArg,
) -> Result<PackageEffectArgMetadata, PackageError> {
    match arg.kind {
        etas_package_metadata::EffectArgKind::Type => Ok(PackageEffectArgMetadata::Type {
            ty: type_from_metadata(arg.ty.ok_or_else(|| {
                missing(
                    Path::new(".etas/package.etasmeta"),
                    "effect type arg is missing type",
                )
            })?)?,
        }),
        etas_package_metadata::EffectArgKind::Path => {
            Ok(PackageEffectArgMetadata::Path { path: arg.path })
        }
        etas_package_metadata::EffectArgKind::String => {
            Ok(PackageEffectArgMetadata::String { value: arg.value })
        }
        etas_package_metadata::EffectArgKind::Wildcard => Ok(PackageEffectArgMetadata::Wildcard),
    }
}

fn effect_summary_from_metadata(
    summary: etas_package_metadata::EffectSummary,
) -> Result<PackageEffectSummaryMetadata, PackageError> {
    Ok(PackageEffectSummaryMetadata {
        item: summary.item,
        public_effects: effect_row_from_metadata(summary.public_effects)?,
        requested_actions: effect_row_from_metadata(summary.requested_actions)?,
        handled_requested_actions: effect_row_from_metadata(summary.handled_requested_actions)?,
        latent_flows: summary
            .latent_flows
            .into_iter()
            .map(latent_flow_summary_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn latent_flow_summary_from_metadata(
    summary: etas_package_metadata::LatentFlowSummary,
) -> Result<PackageLatentFlowSummaryMetadata, PackageError> {
    Ok(PackageLatentFlowSummaryMetadata {
        declared_bound: effect_row_from_metadata(summary.declared_bound)?,
        inferred_effects: effect_row_from_metadata(summary.inferred_effects)?,
    })
}

fn action_summary_from_metadata(
    summary: etas_package_metadata::ActionSummary,
) -> PackageActionSummaryMetadata {
    PackageActionSummaryMetadata {
        action: summary.action,
        args: summary.args,
    }
}

fn trace_spec_summary_from_metadata(
    summary: etas_package_metadata::TraceSpecSummary,
) -> Result<PackageTraceSpecSummaryMetadata, PackageError> {
    Ok(PackageTraceSpecSummaryMetadata {
        trace_spec: summary.trace_spec,
        clauses: summary
            .clauses
            .into_iter()
            .map(trace_spec_clause_from_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn trace_spec_clause_from_metadata(
    clause: etas_package_metadata::TraceSpecClause,
) -> Result<PackageTraceSpecClauseMetadata, PackageError> {
    Ok(PackageTraceSpecClauseMetadata {
        kind: trace_spec_clause_kind_from_metadata(clause.kind),
        pattern: clause.pattern.map(effect_row_from_metadata).transpose()?,
        guard: clause.guard.map(effect_row_from_metadata).transpose()?,
        target: clause.target.map(effect_row_from_metadata).transpose()?,
        obligation: clause
            .obligation
            .map(effect_row_from_metadata)
            .transpose()?,
    })
}

fn trace_spec_clause_kind_from_metadata(
    kind: etas_package_metadata::TraceSpecClauseKind,
) -> PackageTraceSpecClauseKindMetadata {
    match kind {
        etas_package_metadata::TraceSpecClauseKind::Allow => {
            PackageTraceSpecClauseKindMetadata::Allow
        }
        etas_package_metadata::TraceSpecClauseKind::Deny => {
            PackageTraceSpecClauseKindMetadata::Deny
        }
        etas_package_metadata::TraceSpecClauseKind::RequireBefore => {
            PackageTraceSpecClauseKindMetadata::RequireBefore
        }
        etas_package_metadata::TraceSpecClauseKind::RequireAfter => {
            PackageTraceSpecClauseKindMetadata::RequireAfter
        }
    }
}

fn effect_metadata_from_metadata(
    metadata: etas_package_metadata::EffectMetadata,
) -> PackageEffectMetadata {
    PackageEffectMetadata {
        tags: metadata
            .tags
            .into_iter()
            .map(effect_tag_from_metadata)
            .collect(),
        extensions: metadata
            .extensions
            .into_iter()
            .map(effect_extension_from_metadata)
            .collect(),
    }
}

fn effect_tag_from_metadata(tag: etas_package_metadata::EffectTag) -> PackageEffectTagMetadata {
    PackageEffectTagMetadata {
        path: tag.path,
        runtime_requirement: tag.runtime_requirement,
    }
}

fn effect_extension_from_metadata(
    extension: etas_package_metadata::EffectExtension,
) -> PackageEffectExtensionMetadata {
    PackageEffectExtensionMetadata {
        child: extension.child,
        parent: extension.parent,
    }
}

fn tool_schema_from_metadata(
    schema: etas_package_metadata::ToolSchema,
    path: &Path,
) -> Result<PackageToolSchemaMetadata, PackageError> {
    let value =
        serde_json::from_str(&schema.schema_json).map_err(|source| PackageError::Manifest {
            path: path.to_path_buf(),
            message: format!("tool schema JSON is invalid: {source}"),
        })?;
    Ok(PackageToolSchemaMetadata {
        tool: schema.tool,
        schema: value,
    })
}

fn tool_binding_from_metadata(
    binding: etas_package_metadata::ToolBinding,
) -> PackageToolBindingMetadata {
    PackageToolBindingMetadata {
        tool: binding.tool,
        kind: binding.kind,
        provider: binding.provider,
        effect_row: binding.effect_row,
        action_row: binding.action_row,
    }
}

fn re_export_from_metadata(re_export: etas_package_metadata::ReExport) -> PackageReExportMetadata {
    PackageReExportMetadata {
        from: re_export.from,
        exported: re_export.exported,
    }
}

fn bin_target_from_metadata(bin: etas_package_metadata::BinTarget) -> BinTarget {
    BinTarget {
        name: bin.name,
        module: bin.module,
        flow: bin.flow,
        profile: None,
    }
}

fn visibility_from_metadata(visibility: etas_package_metadata::Visibility) -> String {
    match visibility {
        etas_package_metadata::Visibility::Public => "public",
        etas_package_metadata::Visibility::Private => "private",
    }
    .to_owned()
}

fn missing(path: &Path, message: &str) -> PackageError {
    PackageError::Manifest {
        path: PathBuf::from(path),
        message: message.to_owned(),
    }
}
