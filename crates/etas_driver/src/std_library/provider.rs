use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, OnceLock},
};

use etas_code::{CallableId, DefinitionKey, descriptor::GenericOwner};
use etas_package_metadata as metadata;
use etas_std::{StdRegistry, StdRegistryBuilder, StdType};

use super::sdk::{LoadedStdSdk, StdSdkManifest, installed_sysroot, load_standard_library_sdk};
use crate::DriverError;

mod effects;
mod specs;
mod types;

#[derive(Clone, Debug, Default)]
pub enum StdProvider {
    #[default]
    Legacy,
    /// Internal std producer phase: compiler/host contracts without a legacy library.
    SourceBootstrap(Arc<StdRegistry>),
    Artifact(Arc<ArtifactStdProvider>),
}

#[derive(Clone, Debug)]
pub struct ArtifactStdProvider {
    pub sdk: LoadedStdSdk,
    pub registry: Arc<StdRegistry>,
    pub interface: Arc<etas_frontend::StandardLibraryInterface>,
    library: OnceLock<Result<Arc<etas_frontend::StandardLibrary>, String>>,
    pub identity: String,
    pub bootstrap_identity: String,
}

impl ArtifactStdProvider {
    pub fn executable_library(
        &self,
        profile: &etas_utils::ProfileHandle,
    ) -> Result<Arc<etas_frontend::StandardLibrary>, DriverError> {
        self.library
            .get_or_init(|| {
                let _restore = profile.span("std.load_executable_library", "std");
                let analysis = self
                    .installed_effect_analysis(profile)
                    .map_err(|error| error.to_string())?;
                let installed_layouts = if analysis.is_some() {
                    self.sdk
                        .load_runtime_layouts()
                        .map_err(|error| error.to_string())?
                } else {
                    None
                };
                let image = {
                    let _decode = profile.span("std.decode_executable_image", "std");
                    self.sdk.image().map_err(|error| error.to_string())?.clone()
                };
                let program = match analysis {
                    Some(payload) => image.into_checked_from_installed_analysis(
                        &self.sdk.root,
                        &payload,
                        profile,
                    ),
                    None => image.into_checked_from_image_profiled(&self.sdk.root, profile),
                }
                .map_err(|error| error.to_string())?;
                Ok(Arc::new(etas_frontend::StandardLibrary {
                    profile: profile.clone(),
                    installed_layouts,
                    program,
                    interface: self.interface.clone(),
                }))
            })
            .clone()
            .map_err(invalid)
    }

    pub fn executable_library_for(
        &self,
        consumer: &etas_frontend::CheckedProgramData,
        profile: &etas_utils::ProfileHandle,
    ) -> Result<Arc<etas_frontend::StandardLibrary>, DriverError> {
        let Some(_) = &self.sdk.prepared_queries else {
            return self.executable_library(profile);
        };
        // Reflection and caller-performed actions can introduce execution roots
        // beyond ordinary imported callables. They retain the complete SDK.
        if consumer.requires_definition_catalog()
            || !consumer.types.definition_operations.is_empty()
            || !consumer.types.action_calls.is_empty()
        {
            return self.executable_library(profile);
        }
        let mut required = std::collections::BTreeSet::new();
        required.extend(
            consumer
                .types
                .library_flow_calls
                .values()
                .map(|call| call.callable.clone()),
        );
        for symbol in consumer.referenced_standard_symbols() {
            if consumer
                .std_registry
                .symbol(symbol)
                .is_some_and(|symbol| matches!(symbol.decl, etas_std::StdDecl::Value(_)))
            {
                return self.executable_library(profile);
            }
            if let Some(binding) = consumer.std_registry.body_binding(symbol) {
                required.insert(binding.callable.clone());
            }
        }
        required.extend(
            consumer
                .referenced_standard_methods()
                .into_iter()
                .map(etas_code::CallableId::method),
        );
        required.extend(consumer.types.value_equality_dependencies(&consumer.type_store)
            .into_iter().map(etas_code::CallableId::method));
        required.extend(
            consumer
                .types
                .generic_method_dependencies(&consumer.type_store)
                .map_err(invalid)?
                .into_iter()
                .map(etas_code::CallableId::method),
        );
        let index = self.sdk.object_index()?;
        let Some(roots) = required
            .iter()
            .map(|callable| index.callable(callable))
            .collect::<Option<Vec<_>>>()
        else {
            return self.executable_library(profile);
        };
        let roots = roots.into_iter().collect::<std::collections::BTreeSet<_>>();
        let definitions = index.closure(roots.iter().copied());
        let image = self
            .sdk
            .assemble_objects(&definitions, profile, Some(&roots))?;
        let publication = self
            .installed_effect_analysis(profile)?
            .expect("installed SDK publication");
        let installed_layouts = self.sdk.load_runtime_layouts()?;
        let program = image
            .into_checked_from_installed_analysis(&self.sdk.root, &publication, profile)
            .map_err(invalid)?;
        Ok(Arc::new(etas_frontend::StandardLibrary {
            profile: profile.clone(),
            installed_layouts,
            program,
            interface: self.interface.clone(),
        }))
    }

    fn installed_effect_analysis(
        &self,
        profile: &etas_utils::ProfileHandle,
    ) -> Result<Option<Vec<u8>>, DriverError> {
        let Some(expected_hash) = self.sdk.manifest.effect_analysis_hash.as_deref() else {
            return Ok(None);
        };
        let _load = profile.span("std.load_effect_analysis", "std");
        if self.sdk.prepared_queries.is_none()
            && !super::install_receipt::installed_artifact_is_trusted(
                &self.sdk.manifest.identity()?,
            )
        {
            return Ok(None);
        }
        self.sdk.load_effect_analysis(expected_hash).map(Some)
    }
}

impl StdProvider {
    pub(crate) fn lock_identity(&self) -> etas_package::StdLockIdentity {
        match self {
            Self::Legacy | Self::SourceBootstrap(_) => etas_package::StdLockIdentity::legacy(),
            Self::Artifact(provider) => etas_package::StdLockIdentity {
                version: provider.sdk.manifest.version.clone(),
                checksum: format!(
                    "builtin:std:artifact:{}:{}",
                    provider.sdk.manifest.version, provider.sdk.manifest.native_binding_abi,
                ),
            },
        }
    }

    pub(crate) fn package_metadata_identity(&self) -> Option<String> {
        match self {
            Self::Legacy => None,
            Self::SourceBootstrap(_) => Some(etas_std::compiler_contract_identity()),
            Self::Artifact(provider) => Some(provider.identity.clone()),
        }
    }

    fn validate_std_dependencies(
        &self,
        metadata: &etas_package::PackageEnvironmentMetadata,
    ) -> Result<(), DriverError> {
        if matches!(self, Self::SourceBootstrap(_)) {
            return Ok(());
        }
        let selected = self.lock_identity();
        let upgrade = match self {
            Self::Artifact(provider) => format!(
                "etas pkg lock --std-provider artifact --sysroot \"{}\" <PACKAGE_ROOT>",
                provider
                    .sdk
                    .root
                    .parent()
                    .expect("installed std has a sysroot")
                    .display(),
            ),
            Self::Legacy => "etas pkg lock --std-provider legacy <PACKAGE_ROOT>".to_owned(),
            Self::SourceBootstrap(_) => unreachable!(),
        };
        let mut pending = metadata.dependencies.iter().collect::<Vec<_>>();
        while let Some(dependency) = pending.pop() {
            if dependency.identity.name == "std"
                && let etas_package::ResolvedDependencySource::Builtin { checksum } =
                    &dependency.source
                && (dependency.identity.version != selected.version
                    || *checksum != selected.checksum)
            {
                return Err(invalid(format!(
                    "selected std version `{}` with identity `{}` differs from etas.lock version `{}` with identity `{checksum}`; explicitly upgrade the builtin std lock with `{upgrade}`",
                    selected.version, selected.checksum, dependency.identity.version,
                )));
            }
            pending.extend(&dependency.dependencies);
        }
        Ok(())
    }

    pub(crate) fn source_bootstrap() -> Self {
        Self::SourceBootstrap(Arc::new(etas_std::compiler_contract_registry()))
    }

    pub fn artifact(sysroot: Option<&Path>) -> Result<Self, DriverError> {
        Self::artifact_profiled(sysroot, &etas_utils::ProfileHandle::disabled())
    }

    pub fn artifact_profiled(
        sysroot: Option<&Path>,
        profile: &etas_utils::ProfileHandle,
    ) -> Result<Self, DriverError> {
        let _load = profile.span("std.load_artifact", "std");
        let sysroot = match sysroot {
            Some(path) => path.to_owned(),
            None => {
                let binary = std::env::current_exe().map_err(|source| DriverError::Io {
                    path: "<current executable>".into(),
                    source,
                })?;
                let prefix = binary
                    .parent()
                    .and_then(Path::parent)
                    .ok_or_else(|| invalid("toolchain executable has no installation prefix"))?;
                installed_sysroot(prefix)
            }
        };
        let mut sdk = {
            let _decode = profile.span("std.decode_artifact", "std");
            load_standard_library_sdk(&sysroot)?
        };
        let prepared_signatures = {
            let _signatures = profile.span("std.load_imported_signatures", "std");
            sdk.prepared_queries
                .as_ref()
                .map(|_| sdk.load_signatures())
                .transpose()?
        };
        if let Some(queries) = &mut sdk.prepared_queries {
            let registry = Arc::new(
                queries
                    .registry
                    .take()
                    .expect("decoded SDK owns its registry")
                    .into_registry()
                    .map_err(invalid)?,
            );
            let mut interface = queries.interface.as_ref().clone();
            if let Some((imported, fallback)) = prepared_signatures {
                interface.prepared_external = Some(imported);
                interface.signature_fallback = Some(fallback);
            }
            let interface = Arc::new(interface);
            interface.restore_installed_validation(&registry);
            let bootstrap_identity = queries.bootstrap_identity.clone();
            let identity = semantic_identity(&sdk.manifest)?;
            return Ok(Self::Artifact(Arc::new(ArtifactStdProvider {
                sdk,
                registry,
                interface,
                library: OnceLock::new(),
                identity,
                bootstrap_identity,
            })));
        }
        let registry = {
            let _query = profile.span("std.build_query_registry", "std");
            Arc::new(query_registry(&sdk)?)
        };
        let package = etas_frontend::ProjectExternalPackageInput {
            id: etas_frontend::ExternalPackageId(0),
            name: sdk.manifest.package.clone(),
            version: sdk.manifest.version.clone(),
            edition: sdk.manifest.edition.clone(),
            import_root: "std".into(),
            std_compatibility: sdk.manifest.std_contract.clone(),
        };
        let index = etas_package::package_index_from_metadata(
            sdk.public()?.clone(),
            &sdk.root.join(&sdk.manifest.artifact),
        )?;
        let public_metadata = crate::package_env::external_public_metadata_input(
            &package,
            std::slice::from_ref(&package),
            &index.public_metadata,
            &registry,
        );
        let interface = Arc::new(etas_frontend::StandardLibraryInterface {
            prepared_signatures: None,
            prepared_external: None,
            signature_fallback: None,
            validation: Default::default(),
            package,
            metadata: public_metadata,
            modules: sdk
                .public()?
                .public_metadata
                .modules
                .iter()
                .map(|module| etas_frontend::ProjectExternalModuleInput {
                    package: Some(etas_frontend::ExternalPackageId(0)),
                    id: etas_frontend::ExternalModuleId(module.id),
                    path: etas_frontend::ModulePath {
                        segments: module.path.clone(),
                    },
                    exports: module
                        .exports
                        .iter()
                        .map(|export| etas_frontend::ProjectExternalExportInput {
                            symbol: etas_frontend::ExternalSymbolId(export.id),
                            name: export.name.clone(),
                            visibility: match export.visibility {
                                metadata::Visibility::Public => etas_hir::Visibility::Public,
                                metadata::Visibility::Private => etas_hir::Visibility::Private,
                            },
                        })
                        .collect(),
                })
                .collect(),
            origins: sdk
                .public()?
                .public_metadata
                .definition_origins
                .clone()
                .ok_or_else(|| invalid("std artifact has no canonical declaration origins"))?,
            descriptors: Arc::new(
                sdk.public()?
                    .public_metadata
                    .definition_descriptors
                    .clone()
                    .ok_or_else(|| {
                        invalid("std artifact has no canonical public definition descriptors")
                    })?,
            ),
        });
        // An installation's optional solver cache is unrelated to application
        // and checkpoint compatibility with the underlying library.
        let identity = StdSdkManifest {
            effect_analysis_hash: None,
            interpreter_layout_hash: None,
            prepared_queries_hash: None,
            ..sdk.manifest.clone()
        }
        .identity()?;
        let bootstrap_identity = metadata::blake3_hash(
            &serde_json::to_vec(&sdk.image()?.native_registry).map_err(invalid)?,
        );
        Ok(Self::Artifact(Arc::new(ArtifactStdProvider {
            sdk,
            registry,
            interface,
            library: OnceLock::new(),
            identity,
            bootstrap_identity,
        })))
    }

    pub fn frontend_environment(
        &self,
        metadata: etas_package::PackageEnvironmentMetadata,
    ) -> Result<etas_frontend::ProjectEnvironmentInput, DriverError> {
        self.validate_std_dependencies(&metadata)?;
        match self {
            Self::Legacy => crate::package_env::frontend_environment(metadata),
            Self::SourceBootstrap(registry) => {
                crate::package_env::frontend_environment_with_std_registry(metadata, registry)
            }
            Self::Artifact(provider) => crate::package_env::frontend_environment_with_std_registry(
                metadata,
                &provider.registry,
            ),
        }
    }

    pub(crate) fn configure(&self, input: &mut etas_frontend::ProjectInput) {
        if let Self::SourceBootstrap(_) = self {
            input.options.standard_library_mode =
                etas_frontend::StandardLibraryMode::SourceBootstrap;
            input.options.standard_library_artifact = Some(etas_std::compiler_contract_identity());
            return;
        }
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

fn semantic_identity(manifest: &StdSdkManifest) -> Result<String, DriverError> {
    StdSdkManifest {
        effect_analysis_hash: None,
        interpreter_layout_hash: None,
        prepared_queries_hash: None,
        ..manifest.clone()
    }
    .identity()
}

/// Project structured checked package interfaces into the existing std query
/// API. Source flows keep their canonical bundle callables; compiler limit
/// bindings retain their semantic tags and primitive execution contracts.
fn query_registry(sdk: &LoadedStdSdk) -> Result<StdRegistry, DriverError> {
    let image = sdk.image()?;
    let native = image
        .native_registry
        .clone()
        .into_registry()
        .map_err(invalid)?;
    let mut bindings = image
        .types
        .source_type_roles
        .values()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    // Language scalar types are compiler contracts even when no std source
    // declaration binds them. Source prelude exports may refer to these types.
    bindings.extend(native.symbols().filter_map(|symbol| {
        matches!(&symbol.decl, etas_std::StdDecl::Type(declaration)
            if declaration.kind == etas_std::TypeDeclKind::Primitive)
        .then_some(symbol.id)
    }));
    bindings.extend(
        image
            .program
            .standard_action_targets
            .iter()
            .filter_map(|(symbol, target)| {
                matches!(
                    image.program.symbols.get(*symbol).map(|symbol| &symbol.def),
                    Some(etas_hir::SymbolDef::EffectAction { .. })
                )
                .then_some(*target)
            }),
    );
    for (_, item) in image.program.items.iter() {
        if let etas_hir::HirItem::Flow(etas_hir::HirFlowDecl {
            body: etas_hir::HirFlowBody::Runtime { role, .. },
            ..
        }) = item
            && let Some(path) = role.native_contract_path()
            && let Some(symbol) = native.lookup_qualified(&path.split('.').collect::<Vec<_>>())
        {
            bindings.insert(symbol.id);
        }
    }
    // The producer's bootstrap query is not the installed public library.
    // Only source-verified roles/primitives seed this consumer's query view.
    let native = native.for_source_bindings(&bindings);

    let mut modules = native
        .modules()
        .map(|m| (m.path.clone(), m.id))
        .collect::<BTreeMap<_, _>>();
    let mut symbols = native
        .symbols()
        .map(|s| (s.qualified_path.clone(), s.id))
        .collect::<BTreeMap<_, _>>();
    let requirements = native
        .symbols()
        .filter(|symbol| matches!(symbol.decl, etas_std::StdDecl::Requirement(_)))
        .map(|symbol| (symbol.qualified_path.clone(), symbol.id))
        .collect::<BTreeMap<_, _>>();
    let primitive_bindings = native
        .symbols()
        .filter(|symbol| {
            matches!(symbol.decl, etas_std::StdDecl::Flow(_)) && symbol.intrinsic.is_some()
        })
        .map(|symbol| (symbol.qualified_path.clone(), symbol.id))
        .collect::<BTreeMap<_, _>>();
    let mut builder = StdRegistryBuilder::from_registry(native);
    let descriptors = sdk
        .public()?
        .public_metadata
        .definition_descriptors
        .as_ref()
        .ok_or_else(|| invalid("std artifact has no canonical definition descriptors"))?;
    let headers = descriptors
        .headers
        .as_ref()
        .ok_or_else(|| invalid("std artifact has no callable headers"))?;
    for header in &headers.callables {
        let Some(etas_code::descriptor::TypeInfo::Function(signature)) =
            descriptors.graph.types.get(header.signature.0 as usize)
        else {
            return Err(invalid("std callable header has no function signature"));
        };
        // A spec implementation can keep different lexical parameter names in
        // its body. Its calling interface inherits the spec's parameter names.
        let interface = header
            .id
            .as_method()
            .and_then(|method| image.methods.get(method))
            .and_then(|item| image.program.items.get(*item))
            .and_then(etas_hir::HirItem::symbol)
            .and_then(|symbol| {
                image.types.methods.entries().find_map(|entry| {
                    let source = match &entry.target {
                        etas_types::method::MethodTarget::Source { symbol, .. }
                        | etas_types::method::MethodTarget::Agent { symbol, .. } => *symbol,
                        _ => return None,
                    };
                    (source == symbol)
                        .then_some(entry.signature.parameter_interface.as_deref())
                        .flatten()
                })
            });
        if interface.is_some_and(|parameters| parameters.len() != signature.parameters.len()) {
            return Err(invalid(
                "std method calling interface has a different parameter count",
            ));
        }
        builder.source_parameter_interface(
            header.id.clone(),
            signature
                .parameters
                .iter()
                .enumerate()
                .map(|(slot, parameter)| etas_std::StdParameterInterface {
                    name: interface.map_or_else(
                        || parameter.name.clone(),
                        |parameters| parameters[slot].name.clone(),
                    ),
                    default: parameter.default.clone(),
                })
                .collect(),
        );
    }
    for (key, item) in &image.declarations {
        if let etas_hir::HirItem::Flow(etas_hir::HirFlowDecl {
            body: etas_hir::HirFlowBody::Runtime { role, .. },
            ..
        }) = &image.program.items[*item]
        {
            builder.source_execution_role(
                CallableId::declaration(key.clone()).map_err(invalid)?,
                *role,
            );
        }
    }
    for (identity, item) in &image.methods {
        if let etas_hir::HirItem::Flow(etas_hir::HirFlowDecl {
            body: etas_hir::HirFlowBody::Runtime { role, .. },
            ..
        }) = &image.program.items[*item]
        {
            builder.source_execution_role(CallableId::method(identity.clone()), *role);
        }
    }
    let declarations = image
        .declarations
        .keys()
        .map(|key| {
            let mut path = key.module().to_vec();
            path.extend_from_slice(key.declaration());
            (path, key)
        })
        .collect::<BTreeMap<_, _>>();
    let exports = &sdk.public()?.public_metadata.re_exports;
    let aliases = exports
        .iter()
        .map(|export| (export.exported.clone(), export.from.clone()))
        .collect();
    effects::publish(
        sdk,
        &mut builder,
        &mut modules,
        &mut symbols,
        &declarations,
        &aliases,
    )?;
    specs::publish(
        sdk,
        &mut builder,
        &mut modules,
        &mut symbols,
        &declarations,
        &aliases,
    )?;
    types::publish(
        sdk,
        &mut builder,
        &mut modules,
        &mut symbols,
        &declarations,
        &aliases,
    )?;
    let mut values = std::collections::BTreeSet::new();
    for value in &sdk.public()?.public_metadata.values {
        if value.visibility != metadata::Visibility::Public {
            continue;
        }
        let canonical = canonical_export(&value.path, &aliases)?;
        if !values.insert(canonical.clone()) {
            continue;
        }
        let key = *declarations
            .get(&canonical)
            .ok_or_else(|| invalid("std value has no checked source declaration"))?;
        let (name, parent) = canonical
            .split_last()
            .ok_or_else(|| invalid("std value has no public path"))?;
        let module = *modules.entry(parent.to_vec()).or_insert_with(|| {
            builder.module(&parent.iter().map(String::as_str).collect::<Vec<_>>(), "")
        });
        let projector = TypeProjector {
            owner: key,
            params: &value.generic_params,
        };
        let symbol = builder.symbol(
            module,
            name,
            etas_std::StdSymbolKind::Value,
            etas_std::StdDecl::Value(etas_std::ValueDecl {
                name: name.clone(),
                ty: projector.ty(value
                    .ty
                    .as_ref()
                    .ok_or_else(|| invalid("std value has no checked type"))?)?,
                runtime_role: None,
            }),
            "",
        );
        builder.bind_declaration(symbol, key.clone());
        symbols.insert(canonical, symbol);
    }
    let mut flows = sdk
        .public()?
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
    let mut registered = BTreeMap::new();
    let mut definitions = std::collections::BTreeSet::new();
    for (flow, canonical) in flows {
        if flow.visibility != metadata::Visibility::Public {
            continue;
        }
        // Re-exports share the original symbol and checked body identity.
        let key = flow
            .definition
            .as_ref()
            .or_else(|| declarations.get(&canonical).copied());
        if key.is_none() && symbols.contains_key(&canonical) {
            continue;
        }
        let key = key.ok_or_else(|| {
            invalid(format!(
                "std source {} has no canonical checked body",
                flow.path.join(".")
            ))
        })?;
        if !image.declarations.contains_key(key) {
            return Err(invalid("std callable metadata has no checked declaration"));
        }
        if !definitions.insert(key.clone()) {
            continue;
        }
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
            .public()?
            .public_metadata
            .effect_summaries
            .iter()
            .find(|s| s.definition.as_ref() == Some(key))
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
        if let Some(native) = source_native_binding(image, key, &requirements) {
            let symbol = *symbols.get(&canonical).expect("bootstrap limit contract");
            if symbol != native {
                return Err(invalid(
                    "source limit binding differs from its canonical ABI",
                ));
            }
            builder.source_requirement(symbol, declaration.params, key.clone());
            if flow.path[..flow.path.len() - 1] == ["std", "prelude"] {
                builder.prelude(name, symbol);
            }
            symbols.insert(flow.path.clone(), symbol);
            symbols.insert(canonical, symbol);
            continue;
        }
        // Runtime primitives retain their execution ABI while the checked
        // source declaration owns the public path and signature.
        if let Some(native) = source_native_binding(image, key, &primitive_bindings) {
            builder.source_primitive_flow(native, module, declaration, key.clone());
            symbols.retain(|_, symbol| *symbol != native);
            symbols.insert(flow.path.clone(), native);
            symbols.insert(canonical, native);
            continue;
        }
        let callable = CallableId::declaration(key.clone()).map_err(invalid)?;
        let symbol = if let Some(symbol) = registered.get(&canonical).copied() {
            builder
                .source_flow_overload(symbol, declaration, callable)
                .map_err(invalid)?;
            symbol
        } else {
            let symbol = builder.source_flow(module, name, declaration, callable, "");
            registered.insert(canonical.clone(), symbol);
            symbol
        };
        if flow.path[..flow.path.len() - 1] == ["std", "prelude"] {
            builder.prelude(name, symbol);
        }
        symbols.insert(flow.path.clone(), symbol);
        symbols.insert(canonical, symbol);
    }
    for export in &sdk.public()?.public_metadata.re_exports {
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
    let registry = builder.try_finish().map_err(invalid)?;
    registry
        .validate_resolved_native_type_references()
        .map_err(invalid)?;
    registry
        .validate_resolved_native_enum_references()
        .map_err(invalid)?;
    registry
        .validate_resolved_native_opaque_references()
        .map_err(invalid)?;
    registry
        .validate_resolved_native_record_references()
        .map_err(invalid)?;
    registry
        .validate_resolved_native_projection_references()
        .map_err(invalid)?;
    Ok(registry)
}

/// Only a checked canonical binding retains the primitive execution identity.
fn source_native_binding(
    image: &etas_frontend::library_hir::LibraryHirImage,
    definition: &DefinitionKey,
    bindings: &BTreeMap<Vec<String>, etas_std::StdSymbolId>,
) -> Option<etas_std::StdSymbolId> {
    let item = *image.declarations.get(definition)?;
    if let etas_hir::HirItem::Flow(etas_hir::HirFlowDecl {
        body: etas_hir::HirFlowBody::Runtime { role, .. },
        ..
    }) = &image.program.items[item]
        && let Some(path) = role.native_contract_path()
    {
        return bindings
            .get(&path.split('.').map(str::to_owned).collect::<Vec<_>>())
            .copied();
    }
    image
        .program
        .symbols
        .iter()
        .filter(|symbol| symbol.defining_item == Some(item))
        .find_map(|symbol| {
            let etas_hir::SymbolDef::ImportAlias {
                path,
                origin: etas_hir::ImportAliasOrigin::NativeBinding,
            } = &symbol.def
            else {
                return None;
            };
            let native = bindings.get(path)?;
            path.iter()
                .eq(definition.module().iter().chain(definition.declaration()))
                .then_some(*native)
        })
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
            K::Handler => StdType::Handler {
                handled: self.row(ty.effects.as_ref())?,
                produced: ty
                    .produced_effects
                    .as_ref()
                    .map(|row| self.row(Some(row)))
                    .transpose()?,
                result: ty
                    .children
                    .first()
                    .map(|ty| self.ty(ty).map(Box::new))
                    .transpose()?,
            },
            K::Existential => StdType::Existential {
                bounds: ty
                    .bounds
                    .iter()
                    .map(|bound| {
                        Ok(etas_std::StdSpecRef {
                            path: bound.spec.clone(),
                            args: bound
                                .args
                                .iter()
                                .map(|ty| self.ty(ty))
                                .collect::<Result<_, DriverError>>()?,
                        })
                    })
                    .collect::<Result<_, DriverError>>()?,
            },
            K::Prompt => StdType::Prompt,
            K::PromptPart => StdType::PromptPart,
            K::Message => StdType::Message(child(0)?),
            K::MemorySelection => StdType::MemorySelection(child(0)?),
            K::MemoryRegion => StdType::MemoryRegion(child(0)?),
            K::ResourceHandle if ty.name == "MemoryRegion" => {
                StdType::ResourceHandleMemoryRegion(child(0)?)
            }
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
            K::Var | K::ResourceHandle | K::AgentContext => {
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
