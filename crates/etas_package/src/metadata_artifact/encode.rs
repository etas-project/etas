use crate::{
    PackageError,
    metadata::{
        PackageActionSummaryMetadata, PackageAnnotationArgMetadata, PackageAnnotationFieldMetadata,
        PackageAnnotationMetadata, PackageAnnotationValueKindMetadata,
        PackageAnnotationValueMetadata, PackageCallableSpecSatisfactionMetadata,
        PackageEffectActionArgKindMetadata, PackageEffectActionSignatureMetadata,
        PackageEffectArgMetadata, PackageEffectExtensionMetadata, PackageEffectMetadata,
        PackageEffectRefMetadata, PackageEffectRowMetadata, PackageEffectSummaryMetadata,
        PackageEffectTagMetadata, PackageExternalExportMetadata, PackageExternalModuleMetadata,
        PackageExternalModuleOwnerMetadata, PackageIdentity, PackageLatentFlowSummaryMetadata,
        PackageNamedSignatureMetadata, PackagePublicMetadata, PackageReExportMetadata,
        PackageSpecBoundMetadata, PackageSpecImplMetadata, PackageSpecKindMetadata,
        PackageSpecMethodMetadata, PackageSpecSignatureMetadata, PackageToolBindingMetadata,
        PackageToolSchemaMetadata, PackageTraceSpecClauseKindMetadata,
        PackageTraceSpecClauseMetadata, PackageTraceSpecConformanceMetadata,
        PackageTraceSpecConformanceTargetMetadata, PackageTraceSpecSummaryMetadata,
        PackageTypeMetadata, PackageTypeSpecSatisfactionMetadata,
    },
};

#[cfg(any(test, feature = "test-support"))]
pub(super) fn package_index_to_metadata(
    index: &crate::metadata::PackageIndex,
) -> Result<etas_package_metadata::PackageMetadata, PackageError> {
    Ok(etas_package_metadata::PackageMetadata {
        version: index.version,
        package: package_identity_to_metadata(&index.package),
        dependencies: index
            .dependencies
            .iter()
            .map(resolved_dependency_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        external_modules: index
            .external_modules
            .iter()
            .map(external_module_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        public_metadata: public_metadata_to_metadata(&index.public_metadata)?,
        effect_metadata: effect_metadata_to_metadata(&index.effect_metadata),
        tool_bindings: index
            .tool_bindings
            .iter()
            .map(tool_binding_to_metadata)
            .collect(),
        bins: index.bins.iter().map(bin_target_to_metadata).collect(),
    })
}

fn package_identity_to_metadata(
    identity: &PackageIdentity,
) -> etas_package_metadata::PackageIdentity {
    etas_package_metadata::PackageIdentity {
        name: identity.name.clone(),
        version: identity.version.clone(),
        edition: identity.edition.clone(),
    }
}

pub fn resolved_dependency_metadata(
    dependency: &crate::metadata::ResolvedDependency,
) -> Result<etas_package_metadata::ResolvedDependency, PackageError> {
    resolved_dependency_to_metadata(dependency)
}

fn resolved_dependency_to_metadata(
    dependency: &crate::metadata::ResolvedDependency,
) -> Result<etas_package_metadata::ResolvedDependency, PackageError> {
    Ok(etas_package_metadata::ResolvedDependency {
        identity: package_identity_to_metadata(&dependency.identity),
        import_root: dependency.import_root.clone(),
        source: resolved_source_to_metadata(&dependency.source),
        dependencies: dependency
            .dependencies
            .iter()
            .map(resolved_dependency_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        public_metadata: public_metadata_to_metadata(&dependency.public_metadata)?,
        effect_metadata: effect_metadata_to_metadata(&dependency.effect_metadata),
        tool_bindings: dependency
            .tool_bindings
            .iter()
            .map(tool_binding_to_metadata)
            .collect(),
    })
}

fn resolved_source_to_metadata(
    source: &crate::metadata::ResolvedDependencySource,
) -> etas_package_metadata::ResolvedDependencySource {
    match source {
        crate::metadata::ResolvedDependencySource::Builtin { checksum } => {
            etas_package_metadata::ResolvedDependencySource::Builtin {
                checksum: checksum.clone(),
            }
        }
        crate::metadata::ResolvedDependencySource::Registry {
            registry,
            checksum,
            store,
        } => etas_package_metadata::ResolvedDependencySource::Registry {
            registry: registry.clone(),
            checksum: checksum.clone(),
            store: store.clone(),
        },
        crate::metadata::ResolvedDependencySource::Git {
            url,
            rev,
            checksum,
            store,
        } => etas_package_metadata::ResolvedDependencySource::Git {
            url: url.clone(),
            rev: rev.clone(),
            checksum: checksum.clone(),
            store: store.clone(),
        },
        crate::metadata::ResolvedDependencySource::GitHubClone {
            repo,
            rev,
            checksum,
            store,
        } => etas_package_metadata::ResolvedDependencySource::GitHubClone {
            repo: repo.clone(),
            rev: rev.clone(),
            checksum: checksum.clone(),
            store: store.clone(),
        },
        crate::metadata::ResolvedDependencySource::GitHubRelease {
            repo,
            release,
            asset,
            asset_checksum,
            payload_checksum,
            store,
        } => etas_package_metadata::ResolvedDependencySource::GitHubRelease {
            repo: repo.clone(),
            release: release.clone(),
            asset: asset.clone(),
            asset_checksum: asset_checksum.clone(),
            payload_checksum: payload_checksum.clone(),
            store: store.clone(),
        },
        crate::metadata::ResolvedDependencySource::Path { path, checksum } => {
            etas_package_metadata::ResolvedDependencySource::Path {
                path: path.clone(),
                checksum: checksum.clone(),
            }
        }
        crate::metadata::ResolvedDependencySource::Vendor {
            path,
            checksum,
            store,
        } => etas_package_metadata::ResolvedDependencySource::Vendor {
            path: path.clone(),
            checksum: checksum.clone(),
            store: store.clone(),
        },
    }
}

fn public_metadata_to_metadata(
    metadata: &PackagePublicMetadata,
) -> Result<etas_package_metadata::PublicMetadata, PackageError> {
    Ok(etas_package_metadata::PublicMetadata {
        modules: metadata
            .modules
            .iter()
            .map(external_module_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        types: metadata
            .types
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        values: metadata
            .values
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        enums: metadata
            .enums
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        flows: metadata
            .flows
            .iter()
            .map(flow_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        agents: metadata
            .agents
            .iter()
            .map(agent_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        tools: metadata
            .tools
            .iter()
            .map(tool_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        effects: metadata
            .effects
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        actions: metadata
            .actions
            .iter()
            .map(action_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        trace_specs: metadata
            .trace_specs
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        spec_signatures: metadata
            .spec_signatures
            .iter()
            .map(spec_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        spec_impls: metadata
            .spec_impls
            .iter()
            .map(spec_impl_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        type_spec_satisfactions: metadata
            .type_spec_satisfactions
            .iter()
            .map(type_spec_satisfaction_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        callable_spec_satisfactions: metadata
            .callable_spec_satisfactions
            .iter()
            .map(callable_spec_satisfaction_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        trace_spec_conformances: metadata
            .trace_spec_conformances
            .iter()
            .map(trace_spec_conformance_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        protocols: metadata
            .protocols
            .iter()
            .map(named_signature_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        effect_summaries: metadata
            .effect_summaries
            .iter()
            .map(effect_summary_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        action_summaries: metadata
            .action_summaries
            .iter()
            .map(action_summary_to_metadata)
            .collect(),
        tool_schemas: metadata
            .tool_schemas
            .iter()
            .map(tool_schema_to_metadata)
            .collect(),
        trace_spec_summaries: metadata
            .trace_spec_summaries
            .iter()
            .map(trace_spec_summary_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        re_exports: metadata
            .re_exports
            .iter()
            .map(re_export_to_metadata)
            .collect(),
        annotations: metadata
            .annotations
            .iter()
            .map(annotation_to_metadata)
            .collect(),
        fingerprint: metadata.fingerprint.clone(),
    })
}

fn annotation_to_metadata(
    annotation: &PackageAnnotationMetadata,
) -> etas_package_metadata::AnnotationMetadata {
    etas_package_metadata::AnnotationMetadata {
        item: annotation.item.clone(),
        path: annotation.path.clone(),
        args: annotation
            .args
            .iter()
            .map(annotation_arg_to_metadata)
            .collect(),
    }
}

fn annotation_arg_to_metadata(
    arg: &PackageAnnotationArgMetadata,
) -> etas_package_metadata::AnnotationArgMetadata {
    etas_package_metadata::AnnotationArgMetadata {
        name: arg.name.clone(),
        value: annotation_value_to_metadata(&arg.value),
    }
}

fn annotation_field_to_metadata(
    field: &PackageAnnotationFieldMetadata,
) -> etas_package_metadata::AnnotationFieldMetadata {
    etas_package_metadata::AnnotationFieldMetadata {
        name: field.name.clone(),
        value: annotation_value_to_metadata(&field.value),
    }
}

fn annotation_value_to_metadata(
    value: &PackageAnnotationValueMetadata,
) -> etas_package_metadata::AnnotationValueMetadata {
    etas_package_metadata::AnnotationValueMetadata {
        kind: annotation_value_kind_to_metadata(&value.kind),
        value: value.value.clone(),
        path: value.path.clone(),
        elements: value
            .elements
            .iter()
            .map(annotation_value_to_metadata)
            .collect(),
        fields: value
            .fields
            .iter()
            .map(annotation_field_to_metadata)
            .collect(),
    }
}

fn annotation_value_kind_to_metadata(
    kind: &PackageAnnotationValueKindMetadata,
) -> etas_package_metadata::AnnotationValueKind {
    match kind {
        PackageAnnotationValueKindMetadata::Unit => {
            etas_package_metadata::AnnotationValueKind::Unit
        }
        PackageAnnotationValueKindMetadata::Bool => {
            etas_package_metadata::AnnotationValueKind::Bool
        }
        PackageAnnotationValueKindMetadata::Int => etas_package_metadata::AnnotationValueKind::Int,
        PackageAnnotationValueKindMetadata::Float => {
            etas_package_metadata::AnnotationValueKind::Float
        }
        PackageAnnotationValueKindMetadata::String => {
            etas_package_metadata::AnnotationValueKind::String
        }
        PackageAnnotationValueKindMetadata::Char => {
            etas_package_metadata::AnnotationValueKind::Char
        }
        PackageAnnotationValueKindMetadata::Path => {
            etas_package_metadata::AnnotationValueKind::Path
        }
        PackageAnnotationValueKindMetadata::Array => {
            etas_package_metadata::AnnotationValueKind::Array
        }
        PackageAnnotationValueKindMetadata::List => {
            etas_package_metadata::AnnotationValueKind::List
        }
        PackageAnnotationValueKindMetadata::Set => etas_package_metadata::AnnotationValueKind::Set,
        PackageAnnotationValueKindMetadata::Tuple => {
            etas_package_metadata::AnnotationValueKind::Tuple
        }
        PackageAnnotationValueKindMetadata::Record => {
            etas_package_metadata::AnnotationValueKind::Record
        }
        PackageAnnotationValueKindMetadata::Constructor => {
            etas_package_metadata::AnnotationValueKind::Constructor
        }
        PackageAnnotationValueKindMetadata::Limit => {
            etas_package_metadata::AnnotationValueKind::Limit
        }
    }
}

fn external_module_to_metadata(
    module: &PackageExternalModuleMetadata,
) -> Result<etas_package_metadata::ExternalModule, PackageError> {
    Ok(etas_package_metadata::ExternalModule {
        package: module.package.as_ref().map(module_owner_to_metadata),
        id: module.id,
        path: module.path.clone(),
        exports: module
            .exports
            .iter()
            .map(external_export_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn module_owner_to_metadata(
    owner: &PackageExternalModuleOwnerMetadata,
) -> etas_package_metadata::ExternalModuleOwner {
    etas_package_metadata::ExternalModuleOwner {
        identity: package_identity_to_metadata(&owner.identity),
        import_root: owner.import_root.clone(),
    }
}

fn external_export_to_metadata(
    export: &PackageExternalExportMetadata,
) -> Result<etas_package_metadata::ExternalExport, PackageError> {
    Ok(etas_package_metadata::ExternalExport {
        id: export.id,
        name: export.name.clone(),
        visibility: visibility_to_metadata(&export.visibility)?,
    })
}

fn named_signature_to_metadata(
    signature: &PackageNamedSignatureMetadata,
) -> Result<etas_package_metadata::NamedSignature, PackageError> {
    Ok(etas_package_metadata::NamedSignature {
        path: signature.path.clone(),
        visibility: visibility_to_metadata(&signature.visibility)?,
        ty: signature.ty.as_ref().map(type_to_metadata).transpose()?,
    })
}

fn flow_signature_to_metadata(
    signature: &crate::metadata::PackageFlowSignatureMetadata,
) -> Result<etas_package_metadata::CallableSignature, PackageError> {
    Ok(etas_package_metadata::CallableSignature {
        path: signature.path.clone(),
        param_names: signature.param_names.clone(),
        input: signature
            .params
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: Some(type_to_metadata(&signature.output)?),
        effects: signature
            .effects
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        visibility: visibility_to_metadata(&signature.visibility)?,
    })
}

fn agent_signature_to_metadata(
    signature: &crate::metadata::PackageAgentSignatureMetadata,
) -> Result<etas_package_metadata::CallableSignature, PackageError> {
    Ok(etas_package_metadata::CallableSignature {
        path: signature.path.clone(),
        param_names: signature.param_names.clone(),
        input: signature
            .input
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: Some(type_to_metadata(&signature.output)?),
        effects: signature
            .effects
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        visibility: visibility_to_metadata(&signature.visibility)?,
    })
}

fn tool_signature_to_metadata(
    signature: &crate::metadata::PackageToolSignatureMetadata,
) -> Result<etas_package_metadata::CallableSignature, PackageError> {
    Ok(etas_package_metadata::CallableSignature {
        path: signature.path.clone(),
        param_names: signature.param_names.clone(),
        input: signature
            .input
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        output: Some(type_to_metadata(&signature.output)?),
        effects: signature
            .effects
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        visibility: visibility_to_metadata(&signature.visibility)?,
    })
}

fn spec_signature_to_metadata(
    signature: &PackageSpecSignatureMetadata,
) -> Result<etas_package_metadata::SpecSignature, PackageError> {
    Ok(etas_package_metadata::SpecSignature {
        path: signature.path.clone(),
        visibility: visibility_to_metadata(&signature.visibility)?,
        kind: spec_kind_to_metadata(signature.kind),
        param_names: signature.param_names.clone(),
        callable: signature
            .callable
            .as_ref()
            .map(flow_signature_to_metadata)
            .transpose()?,
        methods: signature
            .methods
            .iter()
            .map(spec_method_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        super_specs: signature
            .super_specs
            .iter()
            .map(spec_bound_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn spec_method_to_metadata(
    method: &PackageSpecMethodMetadata,
) -> Result<etas_package_metadata::SpecMethod, PackageError> {
    Ok(etas_package_metadata::SpecMethod {
        name: method.name.clone(),
        path: method.path.clone(),
        signature: method
            .signature
            .as_ref()
            .map(flow_signature_to_metadata)
            .transpose()?,
    })
}

fn spec_bound_to_metadata(
    bound: &PackageSpecBoundMetadata,
) -> Result<etas_package_metadata::SpecBound, PackageError> {
    Ok(etas_package_metadata::SpecBound {
        spec: bound.spec.clone(),
        args: bound
            .args
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn spec_impl_to_metadata(
    implementation: &PackageSpecImplMetadata,
) -> Result<etas_package_metadata::SpecImpl, PackageError> {
    Ok(etas_package_metadata::SpecImpl {
        self_type: type_to_metadata(&implementation.self_type)?,
        spec: implementation.spec.clone(),
        args: implementation
            .args
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        methods: implementation.methods.clone(),
    })
}

fn type_spec_satisfaction_to_metadata(
    fact: &PackageTypeSpecSatisfactionMetadata,
) -> Result<etas_package_metadata::TypeSpecSatisfaction, PackageError> {
    Ok(etas_package_metadata::TypeSpecSatisfaction {
        self_type: type_to_metadata(&fact.self_type)?,
        spec: fact.spec.clone(),
        args: fact
            .args
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn callable_spec_satisfaction_to_metadata(
    fact: &PackageCallableSpecSatisfactionMetadata,
) -> Result<etas_package_metadata::CallableSpecSatisfaction, PackageError> {
    Ok(etas_package_metadata::CallableSpecSatisfaction {
        item: fact.item.clone(),
        spec: fact.spec.clone(),
        args: fact
            .args
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn trace_spec_conformance_to_metadata(
    fact: &PackageTraceSpecConformanceMetadata,
) -> Result<etas_package_metadata::TraceSpecConformance, PackageError> {
    Ok(etas_package_metadata::TraceSpecConformance {
        item: fact.item.clone(),
        target: match &fact.target {
            PackageTraceSpecConformanceTargetMetadata::Inline => {
                etas_package_metadata::TraceSpecConformanceTarget::Inline
            }
            PackageTraceSpecConformanceTargetMetadata::Named { spec, args } => {
                etas_package_metadata::TraceSpecConformanceTarget::Named {
                    spec: spec.clone(),
                    args: args
                        .iter()
                        .map(type_to_metadata)
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
        },
    })
}

fn spec_kind_to_metadata(kind: PackageSpecKindMetadata) -> etas_package_metadata::SpecKind {
    match kind {
        PackageSpecKindMetadata::Type => etas_package_metadata::SpecKind::Type,
        PackageSpecKindMetadata::Callable => etas_package_metadata::SpecKind::Callable,
        PackageSpecKindMetadata::Trace => etas_package_metadata::SpecKind::Trace,
    }
}

fn action_signature_to_metadata(
    signature: &PackageEffectActionSignatureMetadata,
) -> Result<etas_package_metadata::ActionSignature, PackageError> {
    validate_action_selector_metadata(signature)?;
    Ok(etas_package_metadata::ActionSignature {
        path: signature.path.clone(),
        params: signature
            .params
            .iter()
            .map(type_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
        effect_args: signature
            .effect_args
            .iter()
            .map(action_arg_kind_to_metadata)
            .collect(),
        selector_param_names: signature.selector_param_names.clone(),
        selector_defaults: signature
            .selector_defaults
            .iter()
            .map(|arg| arg.as_ref().map(effect_arg_to_metadata).transpose())
            .collect::<Result<Vec<_>, _>>()?,
        output: Some(type_to_metadata(&signature.output)?),
        returns_never: signature.returns_never,
        visibility: visibility_to_metadata(&signature.visibility)?,
    })
}

fn validate_action_selector_metadata(
    signature: &PackageEffectActionSignatureMetadata,
) -> Result<(), PackageError> {
    let path = std::path::PathBuf::from(".etas/package-index.json");
    if signature.selector_param_names.len() != signature.effect_args.len() {
        return Err(PackageError::Manifest {
            path,
            message: format!(
                "action signature `{}` selector_param_names length {} does not match effect_args length {}",
                signature.path.join("."),
                signature.selector_param_names.len(),
                signature.effect_args.len()
            ),
        });
    }
    if signature.selector_defaults.len() != signature.effect_args.len() {
        return Err(PackageError::Manifest {
            path,
            message: format!(
                "action signature `{}` selector_defaults length {} does not match effect_args length {}",
                signature.path.join("."),
                signature.selector_defaults.len(),
                signature.effect_args.len()
            ),
        });
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
            return Err(PackageError::Manifest {
                path,
                message: format!(
                    "action signature `{}` selector default at index {index} does not match selector kind",
                    signature.path.join(".")
                ),
            });
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

fn action_arg_kind_to_metadata(
    kind: &PackageEffectActionArgKindMetadata,
) -> etas_package_metadata::ActionArgKind {
    match kind {
        PackageEffectActionArgKindMetadata::Type => etas_package_metadata::ActionArgKind::Type,
        PackageEffectActionArgKindMetadata::MemoryPlace => {
            etas_package_metadata::ActionArgKind::MemoryPlace
        }
        PackageEffectActionArgKindMetadata::StaticResourcePath { ty } => {
            etas_package_metadata::ActionArgKind::StaticResourcePath { ty: ty.clone() }
        }
        PackageEffectActionArgKindMetadata::StringPattern => {
            etas_package_metadata::ActionArgKind::StringPattern
        }
    }
}

fn type_to_metadata(ty: &PackageTypeMetadata) -> Result<etas_package_metadata::Type, PackageError> {
    let mut metadata = etas_package_metadata::Type::default();
    match ty {
        PackageTypeMetadata::Primitive { name } => {
            metadata.kind = etas_package_metadata::TypeKind::Primitive;
            metadata.name = name.clone();
        }
        PackageTypeMetadata::Var { name } => {
            metadata.kind = etas_package_metadata::TypeKind::Var;
            metadata.name = name.clone();
        }
        PackageTypeMetadata::Named { path } => {
            metadata.kind = etas_package_metadata::TypeKind::Named;
            metadata.path = path.clone();
        }
        PackageTypeMetadata::Applied { path, args } => {
            metadata.kind = etas_package_metadata::TypeKind::Applied;
            metadata.path = path.clone();
            metadata.children = args
                .iter()
                .map(type_to_metadata)
                .collect::<Result<Vec<_>, _>>()?;
        }
        PackageTypeMetadata::Alias { path, target } => {
            metadata.kind = etas_package_metadata::TypeKind::Alias;
            metadata.path = path.clone();
            metadata.children.push(type_to_metadata(target)?);
        }
        PackageTypeMetadata::Nominal {
            path,
            representation,
        } => {
            metadata.kind = etas_package_metadata::TypeKind::Nominal;
            metadata.path = path.clone();
            if let Some(representation) = representation {
                metadata.children.push(type_to_metadata(representation)?);
            }
        }
        PackageTypeMetadata::Array { element } => unary_type(
            etas_package_metadata::TypeKind::Array,
            element,
            &mut metadata,
        )?,
        PackageTypeMetadata::List { element } => unary_type(
            etas_package_metadata::TypeKind::List,
            element,
            &mut metadata,
        )?,
        PackageTypeMetadata::Map { key, value } => {
            metadata.kind = etas_package_metadata::TypeKind::Map;
            metadata.children.push(type_to_metadata(key)?);
            metadata.children.push(type_to_metadata(value)?);
        }
        PackageTypeMetadata::Set { element } => {
            unary_type(etas_package_metadata::TypeKind::Set, element, &mut metadata)?
        }
        PackageTypeMetadata::Range { index } => {
            unary_type(etas_package_metadata::TypeKind::Range, index, &mut metadata)?
        }
        PackageTypeMetadata::Slice { element } => unary_type(
            etas_package_metadata::TypeKind::Slice,
            element,
            &mut metadata,
        )?,
        PackageTypeMetadata::Option { inner } => unary_type(
            etas_package_metadata::TypeKind::Option,
            inner,
            &mut metadata,
        )?,
        PackageTypeMetadata::Result { ok, err } => {
            metadata.kind = etas_package_metadata::TypeKind::Result;
            metadata.children.push(type_to_metadata(ok)?);
            metadata.children.push(type_to_metadata(err)?);
        }
        PackageTypeMetadata::Record { fields } => {
            metadata.kind = etas_package_metadata::TypeKind::Record;
            metadata.fields = fields
                .iter()
                .map(|field| {
                    Ok(etas_package_metadata::TypeField {
                        name: field.name.clone(),
                        ty: type_to_metadata(&field.ty)?,
                    })
                })
                .collect::<Result<Vec<_>, PackageError>>()?;
        }
        PackageTypeMetadata::Tuple { elements } => {
            metadata.kind = etas_package_metadata::TypeKind::Tuple;
            metadata.children = elements
                .iter()
                .map(type_to_metadata)
                .collect::<Result<Vec<_>, _>>()?;
        }
        PackageTypeMetadata::Function {
            input,
            output,
            effects,
        } => {
            metadata.kind = etas_package_metadata::TypeKind::Function;
            metadata.children = input
                .iter()
                .map(type_to_metadata)
                .collect::<Result<Vec<_>, _>>()?;
            metadata.children.push(type_to_metadata(output)?);
            metadata.effects = effects.as_ref().map(effect_row_to_metadata).transpose()?;
        }
        PackageTypeMetadata::Handler {
            handled,
            produced,
            result,
        } => {
            metadata.kind = etas_package_metadata::TypeKind::Handler;
            metadata.effects = Some(effect_row_to_metadata(handled)?);
            metadata.produced_effects =
                produced.as_ref().map(effect_row_to_metadata).transpose()?;
            if let Some(result) = result {
                metadata.children.push(type_to_metadata(result)?);
            }
        }
        PackageTypeMetadata::Trust { wrapper, inner } => {
            unary_type(trust_wrapper_kind(wrapper)?, inner, &mut metadata)?
        }
        PackageTypeMetadata::Prompt => metadata.kind = etas_package_metadata::TypeKind::Prompt,
        PackageTypeMetadata::PromptPart => {
            metadata.kind = etas_package_metadata::TypeKind::PromptPart
        }
        PackageTypeMetadata::Message { inner } => unary_type(
            etas_package_metadata::TypeKind::Message,
            inner,
            &mut metadata,
        )?,
        PackageTypeMetadata::MemorySelection { inner } => unary_type(
            etas_package_metadata::TypeKind::MemorySelection,
            inner,
            &mut metadata,
        )?,
        PackageTypeMetadata::Store { key, value } => {
            metadata.kind = etas_package_metadata::TypeKind::Store;
            metadata.children.push(type_to_metadata(key)?);
            metadata.children.push(type_to_metadata(value)?);
        }
        PackageTypeMetadata::MemoryRegion { schema } => unary_type(
            etas_package_metadata::TypeKind::MemoryRegion,
            schema,
            &mut metadata,
        )?,
        PackageTypeMetadata::ResourceHandle { name, args } => {
            metadata.kind = etas_package_metadata::TypeKind::ResourceHandle;
            metadata.name = name.clone();
            metadata.children = args
                .iter()
                .map(type_to_metadata)
                .collect::<Result<Vec<_>, _>>()?;
        }
    }
    Ok(metadata)
}

fn trust_wrapper_kind(wrapper: &str) -> Result<etas_package_metadata::TypeKind, PackageError> {
    match wrapper {
        "Trusted" => Ok(etas_package_metadata::TypeKind::Trusted),
        "Untrusted" => Ok(etas_package_metadata::TypeKind::Untrusted),
        "Secret" => Ok(etas_package_metadata::TypeKind::Secret),
        "Public" => Ok(etas_package_metadata::TypeKind::Public),
        "Sanitized" => Ok(etas_package_metadata::TypeKind::Sanitized),
        other => Err(PackageError::Manifest {
            path: std::path::PathBuf::from(".etas/package-index.json"),
            message: format!("trust wrapper `{other}` is not supported by package metadata"),
        }),
    }
}

fn unary_type(
    kind: etas_package_metadata::TypeKind,
    inner: &PackageTypeMetadata,
    metadata: &mut etas_package_metadata::Type,
) -> Result<(), PackageError> {
    metadata.kind = kind;
    metadata.children.push(type_to_metadata(inner)?);
    Ok(())
}

fn effect_row_to_metadata(
    row: &PackageEffectRowMetadata,
) -> Result<etas_package_metadata::EffectRow, PackageError> {
    Ok(etas_package_metadata::EffectRow {
        effects: row
            .effects
            .iter()
            .map(effect_ref_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn effect_ref_to_metadata(
    effect: &PackageEffectRefMetadata,
) -> Result<etas_package_metadata::EffectRef, PackageError> {
    Ok(etas_package_metadata::EffectRef {
        path: effect.path.clone(),
        args: effect
            .args
            .iter()
            .map(effect_arg_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn effect_arg_to_metadata(
    arg: &PackageEffectArgMetadata,
) -> Result<etas_package_metadata::EffectArg, PackageError> {
    match arg {
        PackageEffectArgMetadata::Type { ty } => Ok(etas_package_metadata::EffectArg {
            kind: etas_package_metadata::EffectArgKind::Type,
            ty: Some(type_to_metadata(ty)?),
            path: Vec::new(),
            value: String::new(),
        }),
        PackageEffectArgMetadata::Path { path } => Ok(etas_package_metadata::EffectArg {
            kind: etas_package_metadata::EffectArgKind::Path,
            ty: None,
            path: path.clone(),
            value: String::new(),
        }),
        PackageEffectArgMetadata::String { value } => Ok(etas_package_metadata::EffectArg {
            kind: etas_package_metadata::EffectArgKind::String,
            ty: None,
            path: Vec::new(),
            value: value.clone(),
        }),
        PackageEffectArgMetadata::Wildcard => Ok(etas_package_metadata::EffectArg {
            kind: etas_package_metadata::EffectArgKind::Wildcard,
            ty: None,
            path: Vec::new(),
            value: String::new(),
        }),
    }
}

fn effect_summary_to_metadata(
    summary: &PackageEffectSummaryMetadata,
) -> Result<etas_package_metadata::EffectSummary, PackageError> {
    Ok(etas_package_metadata::EffectSummary {
        item: summary.item.clone(),
        public_effects: effect_row_to_metadata(&summary.public_effects)?,
        requested_actions: effect_row_to_metadata(&summary.requested_actions)?,
        handled_requested_actions: effect_row_to_metadata(&summary.handled_requested_actions)?,
        latent_flows: summary
            .latent_flows
            .iter()
            .map(latent_flow_summary_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn latent_flow_summary_to_metadata(
    summary: &PackageLatentFlowSummaryMetadata,
) -> Result<etas_package_metadata::LatentFlowSummary, PackageError> {
    Ok(etas_package_metadata::LatentFlowSummary {
        declared_bound: effect_row_to_metadata(&summary.declared_bound)?,
        inferred_effects: effect_row_to_metadata(&summary.inferred_effects)?,
    })
}

fn action_summary_to_metadata(
    summary: &PackageActionSummaryMetadata,
) -> etas_package_metadata::ActionSummary {
    etas_package_metadata::ActionSummary {
        action: summary.action.clone(),
        args: summary.args.clone(),
    }
}

fn trace_spec_summary_to_metadata(
    summary: &PackageTraceSpecSummaryMetadata,
) -> Result<etas_package_metadata::TraceSpecSummary, PackageError> {
    Ok(etas_package_metadata::TraceSpecSummary {
        trace_spec: summary.trace_spec.clone(),
        clauses: summary
            .clauses
            .iter()
            .map(trace_spec_clause_to_metadata)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn trace_spec_clause_to_metadata(
    clause: &PackageTraceSpecClauseMetadata,
) -> Result<etas_package_metadata::TraceSpecClause, PackageError> {
    Ok(etas_package_metadata::TraceSpecClause {
        kind: trace_spec_clause_kind_to_metadata(clause.kind.clone()),
        pattern: clause
            .pattern
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        guard: clause
            .guard
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        target: clause
            .target
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
        obligation: clause
            .obligation
            .as_ref()
            .map(effect_row_to_metadata)
            .transpose()?,
    })
}

fn trace_spec_clause_kind_to_metadata(
    kind: PackageTraceSpecClauseKindMetadata,
) -> etas_package_metadata::TraceSpecClauseKind {
    match kind {
        PackageTraceSpecClauseKindMetadata::Allow => {
            etas_package_metadata::TraceSpecClauseKind::Allow
        }
        PackageTraceSpecClauseKindMetadata::Deny => {
            etas_package_metadata::TraceSpecClauseKind::Deny
        }
        PackageTraceSpecClauseKindMetadata::RequireBefore => {
            etas_package_metadata::TraceSpecClauseKind::RequireBefore
        }
        PackageTraceSpecClauseKindMetadata::RequireAfter => {
            etas_package_metadata::TraceSpecClauseKind::RequireAfter
        }
    }
}

fn effect_metadata_to_metadata(
    metadata: &PackageEffectMetadata,
) -> etas_package_metadata::EffectMetadata {
    etas_package_metadata::EffectMetadata {
        tags: metadata.tags.iter().map(effect_tag_to_metadata).collect(),
        extensions: metadata
            .extensions
            .iter()
            .map(effect_extension_to_metadata)
            .collect(),
    }
}

fn effect_tag_to_metadata(tag: &PackageEffectTagMetadata) -> etas_package_metadata::EffectTag {
    etas_package_metadata::EffectTag {
        path: tag.path.clone(),
        runtime_requirement: tag.runtime_requirement.clone(),
    }
}

fn effect_extension_to_metadata(
    extension: &PackageEffectExtensionMetadata,
) -> etas_package_metadata::EffectExtension {
    etas_package_metadata::EffectExtension {
        child: extension.child.clone(),
        parent: extension.parent.clone(),
    }
}

fn tool_schema_to_metadata(
    schema: &PackageToolSchemaMetadata,
) -> etas_package_metadata::ToolSchema {
    etas_package_metadata::ToolSchema {
        tool: schema.tool.clone(),
        schema_json: schema.schema.to_string(),
    }
}

fn tool_binding_to_metadata(
    binding: &PackageToolBindingMetadata,
) -> etas_package_metadata::ToolBinding {
    etas_package_metadata::ToolBinding {
        tool: binding.tool.clone(),
        kind: binding.kind.clone(),
        provider: binding.provider.clone(),
        effect_row: binding.effect_row.clone(),
        action_row: binding.action_row.clone(),
    }
}

fn re_export_to_metadata(re_export: &PackageReExportMetadata) -> etas_package_metadata::ReExport {
    etas_package_metadata::ReExport {
        from: re_export.from.clone(),
        exported: re_export.exported.clone(),
    }
}

#[cfg(any(test, feature = "test-support"))]
fn bin_target_to_metadata(bin: &crate::BinTarget) -> etas_package_metadata::BinTarget {
    etas_package_metadata::BinTarget {
        name: bin.name.clone(),
        module: bin.module.clone(),
        flow: bin.flow.clone(),
    }
}

fn visibility_to_metadata(value: &str) -> Result<etas_package_metadata::Visibility, PackageError> {
    match value {
        "public" => Ok(etas_package_metadata::Visibility::Public),
        "private" => Ok(etas_package_metadata::Visibility::Private),
        other => Err(PackageError::Manifest {
            path: std::path::PathBuf::from(".etas/package.etasmeta"),
            message: format!("package metadata visibility `{other}` is not supported"),
        }),
    }
}
