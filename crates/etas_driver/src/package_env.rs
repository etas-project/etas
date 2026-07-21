use std::collections::BTreeMap;

use etas_frontend::{
    ExternalModuleId, ExternalPackageId, ExternalSymbolId, ModulePath, ProjectEnvironmentInput,
    ProjectExternalActionArgKindInput, ProjectExternalActionSignatureInput,
    ProjectExternalActionSummaryInput, ProjectExternalAgentSignatureInput,
    ProjectExternalCallableSpecSatisfactionInput, ProjectExternalEffectArgInput,
    ProjectExternalEffectRefInput, ProjectExternalEffectRowInput,
    ProjectExternalEffectSummaryInput, ProjectExternalExportInput,
    ProjectExternalFlowSignatureInput, ProjectExternalLatentFlowSummaryInput,
    ProjectExternalModuleInput, ProjectExternalNamedSignatureInput, ProjectExternalPackageInput,
    ProjectExternalPublicMetadataInput, ProjectExternalReExportInput,
    ProjectExternalRecordFieldInput, ProjectExternalSpecBoundInput, ProjectExternalSpecImplInput,
    ProjectExternalSpecKindInput, ProjectExternalSpecMethodInput,
    ProjectExternalSpecSignatureInput, ProjectExternalToolSchemaInput,
    ProjectExternalToolSignatureInput, ProjectExternalTraceSpecClauseInput,
    ProjectExternalTraceSpecClauseKindInput, ProjectExternalTraceSpecConformanceInput,
    ProjectExternalTraceSpecConformanceTargetInput, ProjectExternalTraceSpecSummaryInput,
    ProjectExternalTypeInput, ProjectExternalTypeSpecSatisfactionInput, ProjectToolBindingInput,
};
use etas_std::{StdDecl, StdSymbolKind};

use crate::DriverError;

pub fn frontend_environment(
    metadata: etas_package::PackageEnvironmentMetadata,
) -> Result<ProjectEnvironmentInput, DriverError> {
    let external_packages = external_packages(&metadata)?;
    let effect_metadata = dependency_effect_metadata(&metadata, &external_packages)?;
    let dependency_modules = dependency_external_modules(&metadata.dependencies);
    let external_public_metadata = external_public_metadata(&metadata, &external_packages)?;
    Ok(ProjectEnvironmentInput {
        external_packages: external_packages.clone(),
        external_modules: metadata
            .external_modules
            .into_iter()
            .chain(dependency_modules)
            .filter(|module| !is_builtin_std_module(module))
            .map(|module| external_module_input(module, &external_packages))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold(Vec::new(), dedup_external_module),
        external_public_metadata,
        external_effect_metadata: effect_metadata,
        tool_bindings: metadata
            .tool_bindings
            .into_iter()
            .map(|binding| ProjectToolBindingInput {
                tool: binding.tool,
                provider: binding.provider,
                effect_row: binding.effect_row,
                action_row: binding.action_row,
            })
            .collect(),
        environment_fingerprint: Some(metadata.metadata_fingerprint),
        external_modules_fingerprint: None,
    })
}

fn external_packages(
    metadata: &etas_package::PackageEnvironmentMetadata,
) -> Result<Vec<ProjectExternalPackageInput>, DriverError> {
    let mut seen = BTreeMap::<ExternalPackageId, String>::new();
    let mut packages = Vec::new();
    for dependency in flatten_dependencies(&metadata.dependencies) {
        if is_builtin_dependency(dependency) {
            continue;
        }
        let key = external_package_key(dependency);
        let id = stable_external_package_id(&key);
        if let Some(existing) = seen.insert(id, key.clone()) {
            if existing != key {
                return Err(DriverError::InvalidInput(format!(
                    "external package id collision between `{existing}` and `{key}`"
                )));
            }
            continue;
        }
        packages.push(ProjectExternalPackageInput {
            id,
            name: dependency.identity.name.clone(),
            version: dependency.identity.version.clone(),
            edition: dependency.identity.edition.clone(),
            import_root: dependency.import_root.clone(),
        });
    }
    Ok(packages)
}

fn external_public_metadata(
    metadata: &etas_package::PackageEnvironmentMetadata,
    packages: &[ProjectExternalPackageInput],
) -> Result<Vec<ProjectExternalPublicMetadataInput>, DriverError> {
    let mut seen = BTreeMap::<String, ()>::new();
    flatten_dependencies(&metadata.dependencies)
        .into_iter()
        .filter(|dependency| !is_builtin_dependency(dependency))
        .filter_map(|dependency| {
            let key = external_package_key(dependency);
            if seen.insert(key.clone(), ()).is_some() {
                return None;
            }
            let package = packages
                .iter()
                .find(|package| external_package_key_from_input(package) == key)
                .ok_or_else(|| {
                    DriverError::InvalidInput(format!(
                        "external package `{}` with import root `{}` is missing from frontend package identity set",
                        dependency.identity.name, dependency.import_root
                    ))
                });
            Some(package.map(|package| external_public_metadata_input(
                package,
                packages,
                &dependency.public_metadata,
            )))
        })
        .collect()
}

fn flatten_dependencies(
    dependencies: &[etas_package::ResolvedDependency],
) -> Vec<&etas_package::ResolvedDependency> {
    fn walk<'a>(
        dependency: &'a etas_package::ResolvedDependency,
        output: &mut Vec<&'a etas_package::ResolvedDependency>,
    ) {
        output.push(dependency);
        for child in &dependency.dependencies {
            walk(child, output);
        }
    }
    let mut output = Vec::new();
    for dependency in dependencies {
        walk(dependency, &mut output);
    }
    output
}

fn dependency_external_modules(
    dependencies: &[etas_package::ResolvedDependency],
) -> Vec<etas_package::PackageExternalModuleMetadata> {
    flatten_dependencies(dependencies)
        .into_iter()
        .filter(|dependency| !is_builtin_dependency(dependency))
        .flat_map(|dependency| {
            dependency
                .public_metadata
                .modules
                .iter()
                .cloned()
                .map(move |mut module| {
                    module.package = Some(etas_package::PackageExternalModuleOwnerMetadata {
                        identity: dependency.identity.clone(),
                        import_root: dependency.import_root.clone(),
                    });
                    module
                })
        })
        .collect()
}

fn dedup_external_module(
    mut modules: Vec<ProjectExternalModuleInput>,
    module: ProjectExternalModuleInput,
) -> Vec<ProjectExternalModuleInput> {
    if !modules.iter().any(|existing| {
        existing.package == module.package
            && existing.id == module.id
            && existing.path == module.path
    }) {
        modules.push(module);
    }
    modules
}

fn is_builtin_dependency(dependency: &etas_package::ResolvedDependency) -> bool {
    dependency.identity.name == "std"
        && matches!(
            dependency.source,
            etas_package::ResolvedDependencySource::Builtin { .. }
        )
}

fn is_builtin_std_module(module: &etas_package::PackageExternalModuleMetadata) -> bool {
    matches!(module.path.first().map(String::as_str), Some("std"))
        && module
            .package
            .as_ref()
            .is_none_or(|owner| owner.identity.name == "std")
}

pub(crate) fn external_package_key(dependency: &etas_package::ResolvedDependency) -> String {
    format!(
        "{}|{}|{}|{}",
        dependency.import_root,
        dependency.identity.name,
        dependency.identity.version,
        dependency.identity.edition
    )
}

fn external_package_key_from_input(package: &ProjectExternalPackageInput) -> String {
    format!(
        "{}|{}|{}|{}",
        package.import_root, package.name, package.version, package.edition
    )
}

pub(crate) fn stable_external_package_id(key: &str) -> ExternalPackageId {
    let digest = blake3::hash(key.as_bytes());
    let bytes = digest.as_bytes();
    ExternalPackageId(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn external_public_metadata_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    metadata: &etas_package::PackagePublicMetadata,
) -> ProjectExternalPublicMetadataInput {
    ProjectExternalPublicMetadataInput {
        package: package.id,
        types: metadata
            .types
            .iter()
            .map(|signature| named_signature_input(package, packages, signature))
            .collect(),
        values: metadata
            .values
            .iter()
            .map(|signature| named_signature_input(package, packages, signature))
            .collect(),
        enums: metadata
            .enums
            .iter()
            .map(|signature| named_signature_input(package, packages, signature))
            .collect(),
        flows: metadata
            .flows
            .iter()
            .map(|signature| ProjectExternalFlowSignatureInput {
                path: dependency_path_with_graph(package, packages, &signature.path),
                param_names: signature.param_names.clone(),
                params: signature
                    .params
                    .iter()
                    .map(|param| dependency_type_input_with_graph(package, packages, param))
                    .collect(),
                output: dependency_type_input_with_graph(package, packages, &signature.output),
                effects: signature
                    .effects
                    .as_ref()
                    .map(|row| dependency_effect_row_input(package, packages, row)),
                visibility: signature.visibility.clone(),
            })
            .collect(),
        agents: metadata
            .agents
            .iter()
            .map(|signature| ProjectExternalAgentSignatureInput {
                path: dependency_path_with_graph(package, packages, &signature.path),
                param_names: signature.param_names.clone(),
                input: signature
                    .input
                    .iter()
                    .map(|param| dependency_type_input_with_graph(package, packages, param))
                    .collect(),
                output: dependency_type_input_with_graph(package, packages, &signature.output),
                effects: signature
                    .effects
                    .as_ref()
                    .map(|row| dependency_effect_row_input(package, packages, row)),
                visibility: signature.visibility.clone(),
            })
            .collect(),
        tools: metadata
            .tools
            .iter()
            .map(|signature| ProjectExternalToolSignatureInput {
                path: dependency_path_with_graph(package, packages, &signature.path),
                param_names: signature.param_names.clone(),
                input: signature
                    .input
                    .iter()
                    .map(|param| dependency_type_input_with_graph(package, packages, param))
                    .collect(),
                output: dependency_type_input_with_graph(package, packages, &signature.output),
                effects: signature
                    .effects
                    .as_ref()
                    .map(|row| dependency_effect_row_input(package, packages, row)),
                visibility: signature.visibility.clone(),
            })
            .collect(),
        tool_schemas: metadata
            .tool_schemas
            .iter()
            .map(|schema| ProjectExternalToolSchemaInput {
                path: dependency_path_with_graph(package, packages, &schema.tool),
                schema_json: schema.schema.to_string(),
            })
            .collect(),
        effects: metadata
            .effects
            .iter()
            .map(|signature| named_signature_input(package, packages, signature))
            .collect(),
        trace_specs: metadata
            .trace_specs
            .iter()
            .map(|signature| named_signature_input(package, packages, signature))
            .collect(),
        spec_signatures: metadata
            .spec_signatures
            .iter()
            .map(|signature| spec_signature_input(package, packages, signature))
            .collect(),
        spec_impls: metadata
            .spec_impls
            .iter()
            .map(|implementation| ProjectExternalSpecImplInput {
                self_type: dependency_type_input_with_graph(
                    package,
                    packages,
                    &implementation.self_type,
                ),
                spec: dependency_path_with_graph(package, packages, &implementation.spec),
                args: implementation
                    .args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
                methods: implementation.methods.clone(),
            })
            .collect(),
        type_spec_satisfactions: metadata
            .type_spec_satisfactions
            .iter()
            .map(|fact| ProjectExternalTypeSpecSatisfactionInput {
                self_type: dependency_type_input_with_graph(package, packages, &fact.self_type),
                spec: dependency_path_with_graph(package, packages, &fact.spec),
                args: fact
                    .args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
            })
            .collect(),
        callable_spec_satisfactions: metadata
            .callable_spec_satisfactions
            .iter()
            .map(|fact| ProjectExternalCallableSpecSatisfactionInput {
                item: dependency_path_with_graph(package, packages, &fact.item),
                spec: dependency_path_with_graph(package, packages, &fact.spec),
                args: fact
                    .args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
            })
            .collect(),
        trace_spec_conformances: metadata
            .trace_spec_conformances
            .iter()
            .map(|fact| ProjectExternalTraceSpecConformanceInput {
                item: dependency_path_with_graph(package, packages, &fact.item),
                target: match &fact.target {
                    etas_package::PackageTraceSpecConformanceTargetMetadata::Inline => {
                        ProjectExternalTraceSpecConformanceTargetInput::Inline
                    }
                    etas_package::PackageTraceSpecConformanceTargetMetadata::Named {
                        spec,
                        args,
                    } => ProjectExternalTraceSpecConformanceTargetInput::Named {
                        spec: dependency_path_with_graph(package, packages, spec),
                        args: args
                            .iter()
                            .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                            .collect(),
                    },
                },
            })
            .collect(),
        actions: metadata
            .actions
            .iter()
            .map(|signature| ProjectExternalActionSignatureInput {
                path: dependency_path_with_graph(package, packages, &signature.path),
                params: signature
                    .params
                    .iter()
                    .map(|param| dependency_type_input_with_graph(package, packages, param))
                    .collect(),
                effect_args: signature
                    .effect_args
                    .iter()
                    .map(action_arg_kind_input)
                    .collect(),
                selector_param_names: signature.selector_param_names.clone(),
                selector_defaults: signature
                    .selector_defaults
                    .iter()
                    .map(|arg| {
                        arg.as_ref()
                            .map(|arg| dependency_effect_arg_input(package, packages, arg))
                    })
                    .collect(),
                output: dependency_type_input_with_graph(package, packages, &signature.output),
                returns_never: signature.returns_never,
                visibility: signature.visibility.clone(),
            })
            .collect(),
        effect_summaries: metadata
            .effect_summaries
            .iter()
            .map(|summary| ProjectExternalEffectSummaryInput {
                item: dependency_path_with_graph(package, packages, &summary.item),
                public_effects: dependency_effect_row_input(
                    package,
                    packages,
                    &summary.public_effects,
                ),
                requested_actions: dependency_effect_row_input(
                    package,
                    packages,
                    &summary.requested_actions,
                ),
                handled_requested_actions: dependency_effect_row_input(
                    package,
                    packages,
                    &summary.handled_requested_actions,
                ),
                latent_flows: summary
                    .latent_flows
                    .iter()
                    .map(|latent| ProjectExternalLatentFlowSummaryInput {
                        declared_bound: dependency_effect_row_input(
                            package,
                            packages,
                            &latent.declared_bound,
                        ),
                        inferred_effects: dependency_effect_row_input(
                            package,
                            packages,
                            &latent.inferred_effects,
                        ),
                    })
                    .collect(),
            })
            .collect(),
        action_summaries: metadata
            .action_summaries
            .iter()
            .map(|summary| ProjectExternalActionSummaryInput {
                action: dependency_path_with_graph(package, packages, &summary.action),
                args: summary.args.clone(),
            })
            .collect(),
        trace_spec_summaries: metadata
            .trace_spec_summaries
            .iter()
            .map(|summary| ProjectExternalTraceSpecSummaryInput {
                trace_spec: dependency_path_with_graph(package, packages, &summary.trace_spec),
                clauses: summary
                    .clauses
                    .iter()
                    .map(|clause| dependency_trace_spec_clause_input(package, packages, clause))
                    .collect(),
            })
            .collect(),
        re_exports: metadata
            .re_exports
            .iter()
            .map(|re_export| ProjectExternalReExportInput {
                from: dependency_path_with_graph(package, packages, &re_export.from),
                exported: dependency_path_with_graph(package, packages, &re_export.exported),
            })
            .collect(),
    }
}

fn named_signature_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    signature: &etas_package::PackageNamedSignatureMetadata,
) -> ProjectExternalNamedSignatureInput {
    ProjectExternalNamedSignatureInput {
        path: dependency_path_with_graph(package, packages, &signature.path),
        visibility: signature.visibility.clone(),
        ty: signature
            .ty
            .as_ref()
            .map(|ty| dependency_type_input_with_graph(package, packages, ty)),
    }
}

fn spec_signature_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    signature: &etas_package::PackageSpecSignatureMetadata,
) -> ProjectExternalSpecSignatureInput {
    ProjectExternalSpecSignatureInput {
        path: dependency_path_with_graph(package, packages, &signature.path),
        visibility: signature.visibility.clone(),
        kind: match signature.kind {
            etas_package::PackageSpecKindMetadata::Type => ProjectExternalSpecKindInput::Type,
            etas_package::PackageSpecKindMetadata::Callable => {
                ProjectExternalSpecKindInput::Callable
            }
            etas_package::PackageSpecKindMetadata::Trace => ProjectExternalSpecKindInput::Trace,
        },
        param_names: signature.param_names.clone(),
        callable: signature
            .callable
            .as_ref()
            .map(|callable| ProjectExternalFlowSignatureInput {
                path: dependency_path_with_graph(package, packages, &callable.path),
                param_names: callable.param_names.clone(),
                params: callable
                    .params
                    .iter()
                    .map(|param| dependency_type_input_with_graph(package, packages, param))
                    .collect(),
                output: dependency_type_input_with_graph(package, packages, &callable.output),
                effects: callable
                    .effects
                    .as_ref()
                    .map(|row| dependency_effect_row_input(package, packages, row)),
                visibility: callable.visibility.clone(),
            }),
        methods: signature
            .methods
            .iter()
            .map(|method| ProjectExternalSpecMethodInput {
                name: method.name.clone(),
                path: dependency_path_with_graph(package, packages, &method.path),
                signature: method.signature.as_ref().map(|callable| {
                    ProjectExternalFlowSignatureInput {
                        path: dependency_path_with_graph(package, packages, &callable.path),
                        param_names: callable.param_names.clone(),
                        params: callable
                            .params
                            .iter()
                            .map(|param| dependency_type_input_with_graph(package, packages, param))
                            .collect(),
                        output: dependency_type_input_with_graph(
                            package,
                            packages,
                            &callable.output,
                        ),
                        effects: callable
                            .effects
                            .as_ref()
                            .map(|row| dependency_effect_row_input(package, packages, row)),
                        visibility: callable.visibility.clone(),
                    }
                }),
            })
            .collect(),
        super_specs: signature
            .super_specs
            .iter()
            .map(|bound| ProjectExternalSpecBoundInput {
                spec: dependency_path_with_graph(package, packages, &bound.spec),
                args: bound
                    .args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
            })
            .collect(),
    }
}

fn dependency_path(package: &ProjectExternalPackageInput, path: &[String]) -> Vec<String> {
    let import_root = package
        .import_root
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if import_root.is_empty()
        || path.starts_with(&import_root)
        || matches!(path.first().map(String::as_str), Some("std"))
    {
        return path.to_vec();
    }
    import_root
        .into_iter()
        .chain(path.iter().cloned())
        .collect()
}

fn dependency_path_with_graph(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    path: &[String],
) -> Vec<String> {
    if matches!(path.first().map(String::as_str), Some("std"))
        || path_starts_with_any_import_root(path, packages)
    {
        return path.to_vec();
    }
    dependency_path(package, path)
}

fn path_starts_with_any_import_root(
    path: &[String],
    packages: &[ProjectExternalPackageInput],
) -> bool {
    packages.iter().any(|package| {
        let root = package
            .import_root
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        !root.is_empty()
            && path.len() >= root.len()
            && path
                .iter()
                .zip(root)
                .all(|(segment, root_segment)| segment == root_segment)
    })
}

fn path_segments_from_text(text: &str) -> Vec<String> {
    text.split('.')
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn is_standard_effect_base(base: &str) -> bool {
    if base == "Error" {
        return true;
    }
    let registry = etas_std::standard_registry();
    if registry
        .lookup_qualified(&path_segments_from_text(base))
        .is_some_and(|symbol| {
            matches!(
                symbol.kind,
                StdSymbolKind::Effect | StdSymbolKind::EffectAction
            )
        })
    {
        return true;
    }
    registry.symbols().any(|symbol| match &symbol.decl {
        StdDecl::Effect(effect) => effect.name == base,
        StdDecl::EffectAction(action) => {
            format!("{}.{}", action.owner, action.name) == base
                || symbol.qualified_path.join(".") == base
        }
        _ => false,
    })
}

fn dependency_effect_row_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    row: &etas_package::PackageEffectRowMetadata,
) -> ProjectExternalEffectRowInput {
    ProjectExternalEffectRowInput {
        effects: row
            .effects
            .iter()
            .map(|effect| dependency_effect_ref_input(package, packages, effect))
            .collect(),
    }
}

fn dependency_trace_spec_clause_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    clause: &etas_package::PackageTraceSpecClauseMetadata,
) -> ProjectExternalTraceSpecClauseInput {
    ProjectExternalTraceSpecClauseInput {
        kind: dependency_trace_spec_clause_kind_input(clause.kind.clone()),
        pattern: clause
            .pattern
            .as_ref()
            .map(|row| dependency_effect_row_input(package, packages, row)),
        guard: clause
            .guard
            .as_ref()
            .map(|row| dependency_effect_row_input(package, packages, row)),
        target: clause
            .target
            .as_ref()
            .map(|row| dependency_effect_row_input(package, packages, row)),
        obligation: clause
            .obligation
            .as_ref()
            .map(|row| dependency_effect_row_input(package, packages, row)),
    }
}

fn dependency_trace_spec_clause_kind_input(
    kind: etas_package::PackageTraceSpecClauseKindMetadata,
) -> ProjectExternalTraceSpecClauseKindInput {
    match kind {
        etas_package::PackageTraceSpecClauseKindMetadata::Allow => {
            ProjectExternalTraceSpecClauseKindInput::Allow
        }
        etas_package::PackageTraceSpecClauseKindMetadata::Deny => {
            ProjectExternalTraceSpecClauseKindInput::Deny
        }
        etas_package::PackageTraceSpecClauseKindMetadata::RequireBefore => {
            ProjectExternalTraceSpecClauseKindInput::RequireBefore
        }
        etas_package::PackageTraceSpecClauseKindMetadata::RequireAfter => {
            ProjectExternalTraceSpecClauseKindInput::RequireAfter
        }
    }
}

fn dependency_effect_ref_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    effect: &etas_package::PackageEffectRefMetadata,
) -> ProjectExternalEffectRefInput {
    let path = if is_standard_effect_path(&effect.path) {
        effect.path.clone()
    } else {
        dependency_path_with_graph(package, packages, &effect.path)
    };
    ProjectExternalEffectRefInput {
        path,
        args: effect
            .args
            .iter()
            .map(|arg| dependency_effect_arg_input(package, packages, arg))
            .collect(),
    }
}

fn dependency_effect_arg_input(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    arg: &etas_package::PackageEffectArgMetadata,
) -> ProjectExternalEffectArgInput {
    match arg {
        etas_package::PackageEffectArgMetadata::Type { ty } => ProjectExternalEffectArgInput::Type(
            dependency_type_input_with_graph(package, packages, ty),
        ),
        etas_package::PackageEffectArgMetadata::Path { path } => {
            ProjectExternalEffectArgInput::Path(dependency_path_with_graph(package, packages, path))
        }
        etas_package::PackageEffectArgMetadata::String { value } => {
            ProjectExternalEffectArgInput::String(value.clone())
        }
        etas_package::PackageEffectArgMetadata::Wildcard => ProjectExternalEffectArgInput::Wildcard,
    }
}

fn dependency_effect_arg_ref(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    arg: &etas_package::PackageEffectArgMetadata,
) -> etas_types::EffectArgRef {
    match arg {
        etas_package::PackageEffectArgMetadata::Type { ty } => {
            dependency_type_selector_path(package, packages, ty)
                .map(etas_types::EffectArgRef::Path)
                .unwrap_or(etas_types::EffectArgRef::Wildcard)
        }
        etas_package::PackageEffectArgMetadata::Path { path } => {
            etas_types::EffectArgRef::Path(dependency_path_with_graph(package, packages, path))
        }
        etas_package::PackageEffectArgMetadata::String { value } => {
            etas_types::EffectArgRef::String(value.clone())
        }
        etas_package::PackageEffectArgMetadata::Wildcard => etas_types::EffectArgRef::Wildcard,
    }
}

fn dependency_type_selector_path(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    ty: &etas_package::PackageTypeMetadata,
) -> Option<Vec<String>> {
    match ty {
        etas_package::PackageTypeMetadata::Named { path }
        | etas_package::PackageTypeMetadata::Alias { path, .. }
        | etas_package::PackageTypeMetadata::Nominal { path, .. } => {
            Some(dependency_type_path(package, packages, path))
        }
        _ => None,
    }
}

fn effect_row_input(row: &etas_package::PackageEffectRowMetadata) -> ProjectExternalEffectRowInput {
    ProjectExternalEffectRowInput {
        effects: row.effects.iter().map(effect_ref_input).collect(),
    }
}

fn effect_ref_input(
    effect: &etas_package::PackageEffectRefMetadata,
) -> ProjectExternalEffectRefInput {
    ProjectExternalEffectRefInput {
        path: effect.path.clone(),
        args: effect.args.iter().map(effect_arg_input).collect(),
    }
}

fn effect_arg_input(arg: &etas_package::PackageEffectArgMetadata) -> ProjectExternalEffectArgInput {
    match arg {
        etas_package::PackageEffectArgMetadata::Type { ty } => {
            ProjectExternalEffectArgInput::Type(type_input(ty))
        }
        etas_package::PackageEffectArgMetadata::Path { path } => {
            ProjectExternalEffectArgInput::Path(path.clone())
        }
        etas_package::PackageEffectArgMetadata::String { value } => {
            ProjectExternalEffectArgInput::String(value.clone())
        }
        etas_package::PackageEffectArgMetadata::Wildcard => ProjectExternalEffectArgInput::Wildcard,
    }
}

fn action_arg_kind_input(
    kind: &etas_package::PackageEffectActionArgKindMetadata,
) -> ProjectExternalActionArgKindInput {
    match kind {
        etas_package::PackageEffectActionArgKindMetadata::Type => {
            ProjectExternalActionArgKindInput::Type
        }
        etas_package::PackageEffectActionArgKindMetadata::MemoryPlace => {
            ProjectExternalActionArgKindInput::MemoryPlace
        }
        etas_package::PackageEffectActionArgKindMetadata::StaticResourcePath { ty } => {
            ProjectExternalActionArgKindInput::StaticResourcePath { ty: ty.clone() }
        }
        etas_package::PackageEffectActionArgKindMetadata::StringPattern => {
            ProjectExternalActionArgKindInput::StringPattern
        }
    }
}

#[cfg(test)]
fn dependency_type_input(
    package: &ProjectExternalPackageInput,
    ty: &etas_package::PackageTypeMetadata,
) -> ProjectExternalTypeInput {
    dependency_type_input_with_graph(package, std::slice::from_ref(package), ty)
}

fn dependency_type_input_with_graph(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    ty: &etas_package::PackageTypeMetadata,
) -> ProjectExternalTypeInput {
    match ty {
        etas_package::PackageTypeMetadata::Named { path } => {
            ProjectExternalTypeInput::Named(dependency_type_path(package, packages, path))
        }
        etas_package::PackageTypeMetadata::Applied { path, args } => {
            ProjectExternalTypeInput::Applied {
                path: dependency_type_path(package, packages, path),
                args: args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
            }
        }
        etas_package::PackageTypeMetadata::Alias { path, target } => {
            ProjectExternalTypeInput::Alias {
                path: dependency_type_path(package, packages, path),
                target: Box::new(dependency_type_input_with_graph(package, packages, target)),
            }
        }
        etas_package::PackageTypeMetadata::Nominal {
            path,
            representation,
        } => ProjectExternalTypeInput::Nominal {
            path: dependency_type_path(package, packages, path),
            representation: representation.as_ref().map(|representation| {
                Box::new(dependency_type_input_with_graph(
                    package,
                    packages,
                    representation,
                ))
            }),
        },
        etas_package::PackageTypeMetadata::Array { element } => ProjectExternalTypeInput::Array(
            Box::new(dependency_type_input_with_graph(package, packages, element)),
        ),
        etas_package::PackageTypeMetadata::List { element } => ProjectExternalTypeInput::List(
            Box::new(dependency_type_input_with_graph(package, packages, element)),
        ),
        etas_package::PackageTypeMetadata::Map { key, value } => ProjectExternalTypeInput::Map {
            key: Box::new(dependency_type_input_with_graph(package, packages, key)),
            value: Box::new(dependency_type_input_with_graph(package, packages, value)),
        },
        etas_package::PackageTypeMetadata::Set { element } => ProjectExternalTypeInput::Set(
            Box::new(dependency_type_input_with_graph(package, packages, element)),
        ),
        etas_package::PackageTypeMetadata::Range { index } => ProjectExternalTypeInput::Range(
            Box::new(dependency_type_input_with_graph(package, packages, index)),
        ),
        etas_package::PackageTypeMetadata::Slice { element } => ProjectExternalTypeInput::Slice(
            Box::new(dependency_type_input_with_graph(package, packages, element)),
        ),
        etas_package::PackageTypeMetadata::Option { inner } => ProjectExternalTypeInput::Option(
            Box::new(dependency_type_input_with_graph(package, packages, inner)),
        ),
        etas_package::PackageTypeMetadata::Result { ok, err } => ProjectExternalTypeInput::Result {
            ok: Box::new(dependency_type_input_with_graph(package, packages, ok)),
            err: Box::new(dependency_type_input_with_graph(package, packages, err)),
        },
        etas_package::PackageTypeMetadata::Record { fields } => ProjectExternalTypeInput::Record {
            fields: fields
                .iter()
                .map(|field| ProjectExternalRecordFieldInput {
                    name: field.name.clone(),
                    ty: dependency_type_input_with_graph(package, packages, &field.ty),
                })
                .collect(),
        },
        etas_package::PackageTypeMetadata::Tuple { elements } => ProjectExternalTypeInput::Tuple(
            elements
                .iter()
                .map(|element| dependency_type_input_with_graph(package, packages, element))
                .collect(),
        ),
        etas_package::PackageTypeMetadata::Function {
            input,
            output,
            effects,
        } => ProjectExternalTypeInput::Function {
            input: input
                .iter()
                .map(|input| dependency_type_input_with_graph(package, packages, input))
                .collect(),
            output: Box::new(dependency_type_input_with_graph(package, packages, output)),
            effects: effects
                .as_ref()
                .map(|row| dependency_effect_row_input(package, packages, row)),
        },
        etas_package::PackageTypeMetadata::Handler {
            handled,
            produced,
            result,
        } => ProjectExternalTypeInput::Handler {
            handled: dependency_effect_row_input(package, packages, handled),
            produced: produced
                .as_ref()
                .map(|row| dependency_effect_row_input(package, packages, row)),
            result: result.as_ref().map(|result| {
                Box::new(dependency_type_input_with_graph(package, packages, result))
            }),
        },
        etas_package::PackageTypeMetadata::Trust { wrapper, inner } => {
            ProjectExternalTypeInput::Trust {
                wrapper: wrapper.clone(),
                inner: Box::new(dependency_type_input_with_graph(package, packages, inner)),
            }
        }
        etas_package::PackageTypeMetadata::Message { inner } => ProjectExternalTypeInput::Message(
            Box::new(dependency_type_input_with_graph(package, packages, inner)),
        ),
        etas_package::PackageTypeMetadata::MemorySelection { inner } => {
            ProjectExternalTypeInput::MemorySelection(Box::new(dependency_type_input_with_graph(
                package, packages, inner,
            )))
        }
        etas_package::PackageTypeMetadata::Store { key, value } => {
            ProjectExternalTypeInput::Store {
                key: Box::new(dependency_type_input_with_graph(package, packages, key)),
                value: Box::new(dependency_type_input_with_graph(package, packages, value)),
            }
        }
        etas_package::PackageTypeMetadata::MemoryRegion { schema } => {
            ProjectExternalTypeInput::MemoryRegion(Box::new(dependency_type_input_with_graph(
                package, packages, schema,
            )))
        }
        etas_package::PackageTypeMetadata::ResourceHandle { name, args } => {
            ProjectExternalTypeInput::ResourceHandle {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|arg| dependency_type_input_with_graph(package, packages, arg))
                    .collect(),
            }
        }
        etas_package::PackageTypeMetadata::Primitive { .. }
        | etas_package::PackageTypeMetadata::Var { .. }
        | etas_package::PackageTypeMetadata::Prompt
        | etas_package::PackageTypeMetadata::PromptPart => type_input(ty),
    }
}

fn dependency_type_path(
    package: &ProjectExternalPackageInput,
    packages: &[ProjectExternalPackageInput],
    path: &[String],
) -> Vec<String> {
    if is_standard_type_path(path) {
        return path.to_vec();
    }
    dependency_path_with_graph(package, packages, path)
}

fn is_standard_type_path(path: &[String]) -> bool {
    if matches!(path.first().map(String::as_str), Some("std")) {
        return true;
    }
    let registry = etas_std::standard_registry();
    if registry
        .lookup_qualified(path)
        .is_some_and(|symbol| matches!(symbol.kind, StdSymbolKind::Type))
    {
        return true;
    }
    path.len() == 1
        && registry
            .lookup_prelude(&path[0])
            .is_some_and(|symbol| matches!(symbol.kind, StdSymbolKind::Type))
}

fn type_input(ty: &etas_package::PackageTypeMetadata) -> ProjectExternalTypeInput {
    match ty {
        etas_package::PackageTypeMetadata::Primitive { name } => {
            ProjectExternalTypeInput::Primitive(name.clone())
        }
        etas_package::PackageTypeMetadata::Var { name } => {
            ProjectExternalTypeInput::Var(name.clone())
        }
        etas_package::PackageTypeMetadata::Named { path } => {
            ProjectExternalTypeInput::Named(path.clone())
        }
        etas_package::PackageTypeMetadata::Applied { path, args } => {
            ProjectExternalTypeInput::Applied {
                path: path.clone(),
                args: args.iter().map(type_input).collect(),
            }
        }
        etas_package::PackageTypeMetadata::Alias { path, target } => {
            ProjectExternalTypeInput::Alias {
                path: path.clone(),
                target: Box::new(type_input(target)),
            }
        }
        etas_package::PackageTypeMetadata::Nominal {
            path,
            representation,
        } => ProjectExternalTypeInput::Nominal {
            path: path.clone(),
            representation: representation
                .as_ref()
                .map(|representation| Box::new(type_input(representation))),
        },
        etas_package::PackageTypeMetadata::Array { element } => {
            ProjectExternalTypeInput::Array(Box::new(type_input(element)))
        }
        etas_package::PackageTypeMetadata::List { element } => {
            ProjectExternalTypeInput::List(Box::new(type_input(element)))
        }
        etas_package::PackageTypeMetadata::Map { key, value } => ProjectExternalTypeInput::Map {
            key: Box::new(type_input(key)),
            value: Box::new(type_input(value)),
        },
        etas_package::PackageTypeMetadata::Set { element } => {
            ProjectExternalTypeInput::Set(Box::new(type_input(element)))
        }
        etas_package::PackageTypeMetadata::Range { index } => {
            ProjectExternalTypeInput::Range(Box::new(type_input(index)))
        }
        etas_package::PackageTypeMetadata::Slice { element } => {
            ProjectExternalTypeInput::Slice(Box::new(type_input(element)))
        }
        etas_package::PackageTypeMetadata::Option { inner } => {
            ProjectExternalTypeInput::Option(Box::new(type_input(inner)))
        }
        etas_package::PackageTypeMetadata::Result { ok, err } => ProjectExternalTypeInput::Result {
            ok: Box::new(type_input(ok)),
            err: Box::new(type_input(err)),
        },
        etas_package::PackageTypeMetadata::Record { fields } => ProjectExternalTypeInput::Record {
            fields: fields
                .iter()
                .map(|field| ProjectExternalRecordFieldInput {
                    name: field.name.clone(),
                    ty: type_input(&field.ty),
                })
                .collect(),
        },
        etas_package::PackageTypeMetadata::Tuple { elements } => {
            ProjectExternalTypeInput::Tuple(elements.iter().map(type_input).collect())
        }
        etas_package::PackageTypeMetadata::Function {
            input,
            output,
            effects,
        } => ProjectExternalTypeInput::Function {
            input: input.iter().map(type_input).collect(),
            output: Box::new(type_input(output)),
            effects: effects.as_ref().map(effect_row_input),
        },
        etas_package::PackageTypeMetadata::Handler {
            handled,
            produced,
            result,
        } => ProjectExternalTypeInput::Handler {
            handled: effect_row_input(handled),
            produced: produced.as_ref().map(effect_row_input),
            result: result.as_ref().map(|result| Box::new(type_input(result))),
        },
        etas_package::PackageTypeMetadata::Trust { wrapper, inner } => {
            ProjectExternalTypeInput::Trust {
                wrapper: wrapper.clone(),
                inner: Box::new(type_input(inner)),
            }
        }
        etas_package::PackageTypeMetadata::Prompt => ProjectExternalTypeInput::Prompt,
        etas_package::PackageTypeMetadata::PromptPart => ProjectExternalTypeInput::PromptPart,
        etas_package::PackageTypeMetadata::Message { inner } => {
            ProjectExternalTypeInput::Message(Box::new(type_input(inner)))
        }
        etas_package::PackageTypeMetadata::MemorySelection { inner } => {
            ProjectExternalTypeInput::MemorySelection(Box::new(type_input(inner)))
        }
        etas_package::PackageTypeMetadata::Store { key, value } => {
            ProjectExternalTypeInput::Store {
                key: Box::new(type_input(key)),
                value: Box::new(type_input(value)),
            }
        }
        etas_package::PackageTypeMetadata::MemoryRegion { schema } => {
            ProjectExternalTypeInput::MemoryRegion(Box::new(type_input(schema)))
        }
        etas_package::PackageTypeMetadata::ResourceHandle { name, args } => {
            ProjectExternalTypeInput::ResourceHandle {
                name: name.clone(),
                args: args.iter().map(type_input).collect(),
            }
        }
    }
}

fn external_module_input(
    module: etas_package::PackageExternalModuleMetadata,
    packages: &[ProjectExternalPackageInput],
) -> Result<ProjectExternalModuleInput, DriverError> {
    let package = external_package_for_owner(packages, module.package.as_ref())?;
    let path = package
        .and_then(|package_id| packages.iter().find(|candidate| candidate.id == package_id))
        .map_or_else(
            || module.path.clone(),
            |package| dependency_path_with_graph(package, packages, &module.path),
        );
    Ok(ProjectExternalModuleInput {
        package,
        id: ExternalModuleId(module.id),
        path: ModulePath { segments: path },
        exports: module
            .exports
            .into_iter()
            .map(|export| ProjectExternalExportInput {
                symbol: ExternalSymbolId(export.id),
                name: export.name,
                visibility: if export.visibility == "public" {
                    etas_hir::Visibility::Public
                } else {
                    etas_hir::Visibility::Private
                },
            })
            .collect(),
    })
}

fn external_package_for_owner(
    packages: &[ProjectExternalPackageInput],
    owner: Option<&etas_package::PackageExternalModuleOwnerMetadata>,
) -> Result<Option<ExternalPackageId>, DriverError> {
    let Some(owner) = owner else {
        return Ok(None);
    };
    packages
        .iter()
        .find_map(|package| {
            (package.name == owner.identity.name
                && package.version == owner.identity.version
                && package.edition == owner.identity.edition
                && package.import_root == owner.import_root)
                .then_some(package.id)
        })
        .map(Some)
        .ok_or_else(|| {
            DriverError::InvalidInput(format!(
                "external module owner `{}` {} with import root `{}` is not present in resolved package environment",
                owner.identity.name, owner.identity.version, owner.import_root
            ))
        })
}

fn dependency_effect_metadata(
    metadata: &etas_package::PackageEnvironmentMetadata,
    packages: &[ProjectExternalPackageInput],
) -> Result<etas_effects::DependencyEffectMetadata, DriverError> {
    let mut tags = Vec::new();
    for tag in &metadata.effect_metadata.tags {
        tags.push(etas_effects::DependencyEffectTag {
            path: tag.path.clone(),
            runtime_requirement: tag
                .runtime_requirement
                .as_deref()
                .map(runtime_requirement)
                .transpose()?,
        });
    }
    collect_dependency_effect_tags(&metadata.dependencies, packages, &mut tags)?;
    let mut actions = Vec::new();
    collect_dependency_effect_actions(&metadata.dependencies, packages, &mut actions);
    let mut extensions = metadata
        .effect_metadata
        .extensions
        .iter()
        .map(|extension| etas_effects::DependencyEffectExtension {
            package: None,
            child: extension.child.clone(),
            parent: extension.parent.clone(),
        })
        .collect::<Vec<_>>();
    collect_dependency_effect_extensions(&metadata.dependencies, packages, &mut extensions);
    Ok(etas_effects::DependencyEffectMetadata {
        tags,
        actions,
        extensions,
    })
}

fn collect_dependency_effect_tags(
    dependencies: &[etas_package::ResolvedDependency],
    packages: &[ProjectExternalPackageInput],
    output: &mut Vec<etas_effects::DependencyEffectTag>,
) -> Result<(), DriverError> {
    for dependency in dependencies {
        if is_builtin_dependency(dependency) {
            continue;
        }
        let package = external_package_input_for_dependency(dependency);
        for tag in &dependency.effect_metadata.tags {
            let runtime_requirement = tag
                .runtime_requirement
                .as_deref()
                .map(runtime_requirement)
                .transpose()?;
            output.push(etas_effects::DependencyEffectTag {
                path: dependency_path_with_graph(&package, packages, &tag.path),
                runtime_requirement,
            });
        }
        output.extend(dependency.public_metadata.effects.iter().map(|effect| {
            etas_effects::DependencyEffectTag {
                path: dependency_path_with_graph(&package, packages, &effect.path),
                runtime_requirement: None,
            }
        }));
        collect_dependency_effect_tags(&dependency.dependencies, packages, output)?;
    }
    Ok(())
}

fn collect_dependency_effect_actions(
    dependencies: &[etas_package::ResolvedDependency],
    packages: &[ProjectExternalPackageInput],
    output: &mut Vec<etas_effects::DependencyEffectAction>,
) {
    for dependency in dependencies {
        if is_builtin_dependency(dependency) {
            continue;
        }
        let package = external_package_input_for_dependency(dependency);
        output.extend(dependency.public_metadata.actions.iter().map(|action| {
            etas_effects::DependencyEffectAction {
                path: dependency_path_with_graph(&package, packages, &action.path),
                effect_args: action
                    .effect_args
                    .iter()
                    .map(dependency_action_arg_kind)
                    .collect(),
                selector_param_names: action.selector_param_names.clone(),
                selector_defaults: action
                    .selector_defaults
                    .iter()
                    .map(|default| {
                        default
                            .as_ref()
                            .map(|arg| dependency_effect_arg_ref(&package, packages, arg))
                    })
                    .collect(),
                returns_never: action.returns_never,
                runtime_requirement: None,
            }
        }));
        collect_dependency_effect_actions(&dependency.dependencies, packages, output);
    }
}

fn collect_dependency_effect_extensions(
    dependencies: &[etas_package::ResolvedDependency],
    packages: &[ProjectExternalPackageInput],
    output: &mut Vec<etas_effects::DependencyEffectExtension>,
) {
    for dependency in dependencies {
        if is_builtin_dependency(dependency) {
            continue;
        }
        let package = external_package_input_for_dependency(dependency);
        output.extend(
            dependency
                .effect_metadata
                .extensions
                .iter()
                .map(|extension| etas_effects::DependencyEffectExtension {
                    package: Some(package.id.0),
                    child: dependency_path_with_graph(&package, packages, &extension.child),
                    parent: if is_standard_effect_path(&extension.parent) {
                        extension.parent.clone()
                    } else {
                        dependency_path_with_graph(&package, packages, &extension.parent)
                    },
                }),
        );
        collect_dependency_effect_extensions(&dependency.dependencies, packages, output);
    }
}

fn external_package_input_for_dependency(
    dependency: &etas_package::ResolvedDependency,
) -> ProjectExternalPackageInput {
    let key = external_package_key(dependency);
    ProjectExternalPackageInput {
        id: stable_external_package_id(&key),
        name: dependency.identity.name.clone(),
        version: dependency.identity.version.clone(),
        edition: dependency.identity.edition.clone(),
        import_root: dependency.import_root.clone(),
    }
}

fn is_standard_effect_path(path: &[String]) -> bool {
    is_standard_effect_base(&path.join("."))
}

fn dependency_action_arg_kind(
    kind: &etas_package::PackageEffectActionArgKindMetadata,
) -> etas_effects::EffectActionArgKind {
    match kind {
        etas_package::PackageEffectActionArgKindMetadata::Type => {
            etas_effects::EffectActionArgKind::Type
        }
        etas_package::PackageEffectActionArgKindMetadata::MemoryPlace => {
            etas_effects::EffectActionArgKind::MemoryPlace
        }
        etas_package::PackageEffectActionArgKindMetadata::StaticResourcePath { ty } => {
            etas_effects::EffectActionArgKind::StaticResourcePath { ty: ty.clone() }
        }
        etas_package::PackageEffectActionArgKindMetadata::StringPattern => {
            etas_effects::EffectActionArgKind::StringPattern
        }
    }
}

fn runtime_requirement(value: &str) -> Result<etas_effects::RuntimeRequirementReason, DriverError> {
    match value {
        "Agentic" => Ok(etas_effects::RuntimeRequirementReason::Agentic),
        "ToolCall" => Ok(etas_effects::RuntimeRequirementReason::ToolCall),
        "HostAuthority" => Ok(etas_effects::RuntimeRequirementReason::HostAuthority),
        "DurableMemory" => Ok(etas_effects::RuntimeRequirementReason::DurableMemory),
        "Approval" => Ok(etas_effects::RuntimeRequirementReason::Approval),
        "Checkpoint" => Ok(etas_effects::RuntimeRequirementReason::Checkpoint),
        "Time" => Ok(etas_effects::RuntimeRequirementReason::Time),
        "Network" => Ok(etas_effects::RuntimeRequirementReason::Network),
        "Console" => Ok(etas_effects::RuntimeRequirementReason::Console),
        "FileIO" => Ok(etas_effects::RuntimeRequirementReason::FileIO),
        "Command" => Ok(etas_effects::RuntimeRequirementReason::Command),
        "SecretAccess" => Ok(etas_effects::RuntimeRequirementReason::SecretAccess),
        "RuntimeHandler" => Ok(etas_effects::RuntimeRequirementReason::RuntimeHandler),
        other => Err(DriverError::InvalidInput(format!(
            "unknown package effect runtime requirement `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_type_input_preserves_std_prelude_types_without_package_prefix() {
        let package = ProjectExternalPackageInput {
            id: ExternalPackageId(7),
            name: "edk-algorithm".to_owned(),
            version: "0.1.0".to_owned(),
            edition: "2026".to_owned(),
            import_root: "edk.algorithm".to_owned(),
        };

        let std_error = dependency_type_input(
            &package,
            &etas_package::PackageTypeMetadata::Named {
                path: vec!["IOError".to_owned()],
            },
        );
        assert_eq!(
            std_error,
            ProjectExternalTypeInput::Named(vec!["IOError".to_owned()])
        );

        let package_error = dependency_type_input(
            &package,
            &etas_package::PackageTypeMetadata::Named {
                path: vec!["HttpError".to_owned()],
            },
        );
        assert_eq!(
            package_error,
            ProjectExternalTypeInput::Named(vec![
                "edk".to_owned(),
                "algorithm".to_owned(),
                "HttpError".to_owned()
            ])
        );
    }

    #[test]
    fn frontend_environment_projects_dependency_public_effect_actions() {
        let metadata = etas_package::PackageEnvironmentMetadata {
            dependencies: vec![etas_package::ResolvedDependency {
                identity: etas_package::PackageIdentity {
                    name: "typed-errors".to_owned(),
                    version: "0.1.0".to_owned(),
                    edition: "2026".to_owned(),
                },
                import_root: "errors".to_owned(),
                source: etas_package::ResolvedDependencySource::Vendor {
                    path: ".etas/vendor/errors".to_owned(),
                    checksum: "blake3:00".to_owned(),
                    store: None,
                },
                dependencies: Vec::new(),
                public_metadata: etas_package::PackagePublicMetadata {
                    effects: vec![etas_package::PackageNamedSignatureMetadata {
                        path: vec!["errors".to_owned(), "Net".to_owned()],
                        visibility: "public".to_owned(),
                        ty: None,
                    }],
                    actions: vec![etas_package::PackageEffectActionSignatureMetadata {
                        path: vec!["errors".to_owned(), "Net".to_owned(), "request".to_owned()],
                        params: Vec::new(),
                        effect_args: vec![
                            etas_package::PackageEffectActionArgKindMetadata::StringPattern,
                        ],
                        selector_param_names: vec![String::new()],
                        selector_defaults: vec![None],
                        output: etas_package::PackageTypeMetadata::Primitive {
                            name: "unit".to_owned(),
                        },
                        returns_never: false,
                        visibility: "public".to_owned(),
                    }],
                    ..Default::default()
                },
                effect_metadata: Default::default(),
                tool_bindings: Vec::new(),
            }],
            metadata_fingerprint: "test".to_owned(),
            ..Default::default()
        };

        let environment = frontend_environment(metadata).expect("environment converts");
        assert!(
            environment
                .external_effect_metadata
                .tags
                .iter()
                .any(|tag| tag.path == ["errors", "Net"])
        );
        assert!(
            environment
                .external_effect_metadata
                .actions
                .iter()
                .any(|action| action.path == ["errors", "Net", "request"])
        );
    }

    #[test]
    fn frontend_environment_does_not_project_builtin_std_as_external_package() {
        let std_dependency =
            etas_package::metadata::builtin_std_dependency("0.1.0".to_owned(), "2026".to_owned());
        let metadata = etas_package::PackageEnvironmentMetadata {
            external_modules: std_dependency.public_metadata.modules.clone(),
            dependencies: vec![std_dependency],
            metadata_fingerprint: "test".to_owned(),
            ..Default::default()
        };

        let environment = frontend_environment(metadata).expect("environment converts");
        assert!(
            environment.external_packages.is_empty(),
            "builtin std is compiler-known and must not be exposed as an external package"
        );
        assert!(
            environment.external_modules.is_empty(),
            "builtin std modules must resolve through the frontend std provider"
        );
        assert!(
            environment.external_public_metadata.is_empty(),
            "builtin std public metadata must not create a second external type/effect identity"
        );
        assert!(
            environment.external_effect_metadata.tags.is_empty()
                && environment.external_effect_metadata.actions.is_empty(),
            "builtin std effects/actions are supplied by the effect registry"
        );
    }

    #[test]
    fn frontend_environment_projects_dependency_effect_summary_rows() {
        fn path(segments: &[&str]) -> Vec<String> {
            segments
                .iter()
                .map(|segment| (*segment).to_owned())
                .collect()
        }

        fn effect(
            path: &[&str],
            args: Vec<etas_package::PackageEffectArgMetadata>,
        ) -> etas_package::PackageEffectRefMetadata {
            etas_package::PackageEffectRefMetadata {
                path: path.iter().map(|segment| (*segment).to_owned()).collect(),
                args,
            }
        }

        fn row(
            effects: Vec<etas_package::PackageEffectRefMetadata>,
        ) -> etas_package::PackageEffectRowMetadata {
            etas_package::PackageEffectRowMetadata { effects }
        }

        let metadata = etas_package::PackageEnvironmentMetadata {
            dependencies: vec![etas_package::ResolvedDependency {
                identity: etas_package::PackageIdentity {
                    name: "edk-http".to_owned(),
                    version: "0.1.0".to_owned(),
                    edition: "2026".to_owned(),
                },
                import_root: "edk.http".to_owned(),
                source: etas_package::ResolvedDependencySource::Vendor {
                    path: ".etas/vendor/edk-http".to_owned(),
                    checksum: "blake3:00".to_owned(),
                    store: None,
                },
                dependencies: Vec::new(),
                public_metadata: etas_package::PackagePublicMetadata {
                    effects: vec![etas_package::PackageNamedSignatureMetadata {
                        path: vec!["effects".to_owned(), "EdkHttp".to_owned()],
                        visibility: "public".to_owned(),
                        ty: None,
                    }],
                    actions: vec![etas_package::PackageEffectActionSignatureMetadata {
                        path: vec![
                            "effects".to_owned(),
                            "EdkHttp".to_owned(),
                            "request".to_owned(),
                        ],
                        params: Vec::new(),
                        effect_args: vec![
                            etas_package::PackageEffectActionArgKindMetadata::StringPattern,
                        ],
                        selector_param_names: vec![String::new()],
                        selector_defaults: vec![None],
                        output: etas_package::PackageTypeMetadata::Primitive {
                            name: "unit".to_owned(),
                        },
                        returns_never: false,
                        visibility: "public".to_owned(),
                    }],
                    effect_summaries: vec![etas_package::PackageEffectSummaryMetadata {
                        item: vec!["client".to_owned(), "request".to_owned()],
                        public_effects: row(vec![
                            effect(
                                &["effects", "EdkHttp", "request"],
                                vec![
                                    etas_package::PackageEffectArgMetadata::String {
                                        value: "GET".to_owned(),
                                    },
                                    etas_package::PackageEffectArgMetadata::Path {
                                        path: path(&["ApiHost"]),
                                    },
                                ],
                            ),
                            effect(
                                &["Error"],
                                vec![etas_package::PackageEffectArgMetadata::Path {
                                    path: path(&["HttpError"]),
                                }],
                            ),
                            effect(&["Network"], Vec::new()),
                        ]),
                        requested_actions: row(vec![
                            effect(&["Console", "stdout_write"], Vec::new()),
                            effect(
                                &["effects", "EdkHttp", "request"],
                                vec![
                                    etas_package::PackageEffectArgMetadata::String {
                                        value: "GET".to_owned(),
                                    },
                                    etas_package::PackageEffectArgMetadata::Path {
                                        path: path(&["ApiHost"]),
                                    },
                                ],
                            ),
                        ]),
                        handled_requested_actions: row(vec![effect(
                            &["Console", "stdout_write"],
                            Vec::new(),
                        )]),
                        latent_flows: vec![etas_package::PackageLatentFlowSummaryMetadata {
                            declared_bound: row(vec![effect(
                                &["effects", "EdkHttp", "request"],
                                vec![etas_package::PackageEffectArgMetadata::Path {
                                    path: path(&["ApiHost"]),
                                }],
                            )]),
                            inferred_effects: row(vec![effect(
                                &["Error"],
                                vec![etas_package::PackageEffectArgMetadata::Path {
                                    path: path(&["HttpError"]),
                                }],
                            )]),
                        }],
                    }],
                    ..Default::default()
                },
                effect_metadata: etas_package::PackageEffectMetadata {
                    extensions: vec![etas_package::PackageEffectExtensionMetadata {
                        child: vec!["effects".to_owned(), "EdkHttp".to_owned()],
                        parent: vec!["Network".to_owned()],
                    }],
                    ..Default::default()
                },
                tool_bindings: Vec::new(),
            }],
            metadata_fingerprint: "test".to_owned(),
            ..Default::default()
        };

        let environment = frontend_environment(metadata).expect("environment converts");
        let summary = environment
            .external_public_metadata
            .first()
            .and_then(|metadata| metadata.effect_summaries.first())
            .expect("projected summary exists");
        assert_eq!(summary.item, ["edk", "http", "client", "request"]);
        assert_eq!(
            summary.public_effects.effects[0].path,
            ["edk", "http", "effects", "EdkHttp", "request"]
        );
        assert_eq!(
            summary.public_effects.effects[0].args,
            [
                ProjectExternalEffectArgInput::String("GET".to_owned()),
                ProjectExternalEffectArgInput::Path(path(&["edk", "http", "ApiHost"]))
            ]
        );
        assert_eq!(summary.public_effects.effects[1].path, ["Error"]);
        assert_eq!(
            summary.public_effects.effects[1].args,
            [ProjectExternalEffectArgInput::Path(path(&[
                "edk",
                "http",
                "HttpError"
            ]))]
        );
        assert_eq!(
            summary.requested_actions.effects[0].path,
            ["Console", "stdout_write"]
        );
        assert_eq!(
            summary.requested_actions.effects[1].path,
            ["edk", "http", "effects", "EdkHttp", "request"]
        );
        assert_eq!(
            summary.handled_requested_actions.effects[0].path,
            ["Console", "stdout_write"]
        );
        assert_eq!(
            summary.latent_flows[0].declared_bound.effects[0].args,
            [ProjectExternalEffectArgInput::Path(path(&[
                "edk", "http", "ApiHost"
            ]))]
        );
        assert_eq!(
            summary.latent_flows[0].inferred_effects.effects[0].args,
            [ProjectExternalEffectArgInput::Path(path(&[
                "edk",
                "http",
                "HttpError"
            ]))]
        );
        assert!(
            environment
                .external_effect_metadata
                .tags
                .iter()
                .any(|tag| tag.path == ["edk", "http", "effects", "EdkHttp"])
        );
        assert!(
            environment
                .external_effect_metadata
                .actions
                .iter()
                .any(|action| action.path == ["edk", "http", "effects", "EdkHttp", "request"])
        );
        assert!(
            environment
                .external_effect_metadata
                .extensions
                .iter()
                .any(|extension| {
                    extension.child == ["edk", "http", "effects", "EdkHttp"]
                        && extension.parent == ["Network"]
                })
        );
    }
}
