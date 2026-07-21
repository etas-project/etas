use std::collections::{BTreeMap, BTreeSet};

use etas_std::{
    EffectDecl, FlowDecl, StdDecl, StdPrimitiveType, StdSupportConstraint, StdSymbolKind, StdType,
    ToolDecl, TypeDeclKind, standard_registry,
};

use super::{
    PackageEffectActionArgKindMetadata, PackageEffectActionSignatureMetadata,
    PackageEffectArgMetadata, PackageEffectExtensionMetadata, PackageEffectMetadata,
    PackageEffectRefMetadata, PackageEffectRowMetadata, PackageEffectSummaryMetadata,
    PackageEffectTagMetadata, PackageExternalExportMetadata, PackageExternalModuleMetadata,
    PackageFlowSignatureMetadata, PackageIdentity, PackageNamedSignatureMetadata,
    PackagePublicMetadata, PackageRecordFieldMetadata, PackageToolSignatureMetadata,
    PackageTypeMetadata, ResolvedDependency, ResolvedDependencySource,
};

pub const BUILTIN_STD_VERSION: &str = "0.1.0";

pub fn builtin_std_dependency(version: String, edition: String) -> ResolvedDependency {
    ResolvedDependency {
        identity: PackageIdentity {
            name: "std".to_owned(),
            version,
            edition,
        },
        import_root: "std".to_owned(),
        source: ResolvedDependencySource::Builtin {
            checksum: "builtin:std".to_owned(),
        },
        dependencies: Vec::new(),
        public_metadata: builtin_std_public_metadata(),
        effect_metadata: builtin_std_effect_metadata(),
        tool_bindings: Vec::new(),
    }
}

pub fn is_builtin_package(name: &str) -> bool {
    name == "std"
}

pub fn builtin_std_public_metadata() -> PackagePublicMetadata {
    let registry = standard_registry();
    let mut exports_by_module = BTreeMap::<u32, Vec<PackageExternalExportMetadata>>::new();
    for symbol in registry.symbols() {
        exports_by_module
            .entry(symbol.module.0)
            .or_default()
            .push(PackageExternalExportMetadata {
                id: symbol.id.0,
                name: symbol.name.clone(),
                visibility: public_visibility(),
            });
    }

    let mut modules = registry
        .modules()
        .map(|module| {
            let mut exports = exports_by_module.remove(&module.id.0).unwrap_or_default();
            exports.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
            PackageExternalModuleMetadata {
                package: None,
                id: module.id.0,
                path: module.path.clone(),
                exports,
            }
        })
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut types = Vec::new();
    let mut values = Vec::new();
    let mut enums = Vec::new();
    let mut flows = Vec::new();
    let mut tools = Vec::new();
    let mut effects = Vec::new();
    let mut actions = Vec::new();
    let mut effect_summaries = Vec::new();

    for symbol in registry.symbols() {
        match &symbol.decl {
            StdDecl::Type(decl) => {
                let metadata = named_signature(symbol.qualified_path.clone());
                if decl.kind == TypeDeclKind::Enum {
                    enums.push(metadata);
                } else {
                    types.push(metadata);
                }
            }
            StdDecl::Effect(_) => effects.push(named_signature(symbol.qualified_path.clone())),
            StdDecl::EffectAction(decl) => {
                actions.push(action_signature(symbol.qualified_path.clone(), decl));
            }
            StdDecl::Flow(decl)
                if matches!(
                    symbol.kind,
                    StdSymbolKind::Flow | StdSymbolKind::Constructor
                ) =>
            {
                flows.push(flow_signature(symbol.qualified_path.clone(), decl));
                effect_summaries.push(flow_effect_summary(symbol.qualified_path.clone(), decl));
            }
            StdDecl::Value(decl) => {
                values.push(value_signature(symbol.qualified_path.clone(), decl))
            }
            StdDecl::Tool(decl) => {
                tools.push(tool_signature(symbol.qualified_path.clone(), decl));
            }
            StdDecl::Flow(_) | StdDecl::Impl(_) | StdDecl::Requirement(_) => {}
        }
    }

    sort_named_signatures(&mut types);
    sort_named_signatures(&mut values);
    sort_named_signatures(&mut enums);
    sort_named_signatures(&mut effects);
    flows.sort_by(|left, right| left.path.cmp(&right.path));
    tools.sort_by(|left, right| left.path.cmp(&right.path));
    actions.sort_by(|left, right| left.path.cmp(&right.path));
    effect_summaries.sort_by(|left, right| left.item.cmp(&right.item));

    let mut metadata = PackagePublicMetadata {
        modules,
        types,
        values,
        enums,
        flows,
        tools,
        effects,
        actions,
        effect_summaries,
        fingerprint: None,
        ..PackagePublicMetadata::default()
    };
    metadata.fingerprint = Some(builtin_std_public_metadata_fingerprint(&metadata));
    metadata
}

pub fn builtin_std_effect_metadata() -> PackageEffectMetadata {
    let registry = standard_registry();
    let mut tag_paths = BTreeSet::<Vec<String>>::new();
    let mut tags = Vec::new();
    let mut extension_paths = BTreeSet::<(Vec<String>, Vec<String>)>::new();
    let mut extensions = Vec::new();

    for symbol in registry.symbols() {
        let StdDecl::Effect(decl) = &symbol.decl else {
            continue;
        };
        let path = vec![decl.name.clone()];
        if tag_paths.insert(path.clone()) {
            tags.push(PackageEffectTagMetadata {
                runtime_requirement: runtime_requirement_for_std_effect(decl),
                path,
            });
        }
        for parent in &decl.extends {
            let child = vec![decl.name.clone()];
            let parent = vec![parent.clone()];
            if extension_paths.insert((child.clone(), parent.clone())) {
                extensions.push(PackageEffectExtensionMetadata { child, parent });
            }
        }
    }
    tags.sort_by(|left, right| left.path.cmp(&right.path));
    extensions.sort_by(|left, right| {
        left.child
            .cmp(&right.child)
            .then_with(|| left.parent.cmp(&right.parent))
    });

    PackageEffectMetadata { tags, extensions }
}

fn public_visibility() -> String {
    "public".to_owned()
}

fn named_signature(path: Vec<String>) -> PackageNamedSignatureMetadata {
    PackageNamedSignatureMetadata {
        path,
        visibility: public_visibility(),
        ty: None,
    }
}

fn value_signature(path: Vec<String>, decl: &etas_std::ValueDecl) -> PackageNamedSignatureMetadata {
    PackageNamedSignatureMetadata {
        path,
        visibility: public_visibility(),
        ty: Some(package_type_from_std_type(&decl.ty)),
    }
}

fn sort_named_signatures(signatures: &mut [PackageNamedSignatureMetadata]) {
    signatures.sort_by(|left, right| left.path.cmp(&right.path));
}

fn flow_signature(path: Vec<String>, decl: &FlowDecl) -> PackageFlowSignatureMetadata {
    PackageFlowSignatureMetadata {
        path,
        param_names: Vec::new(),
        params: decl.params.iter().map(package_type_from_std_type).collect(),
        output: package_type_from_std_type(&decl.output),
        effects: Some(effect_row_from_texts(&decl.public_effects)),
        visibility: public_visibility(),
    }
}

fn tool_signature(path: Vec<String>, decl: &ToolDecl) -> PackageToolSignatureMetadata {
    PackageToolSignatureMetadata {
        path,
        param_names: Vec::new(),
        input: decl.params.iter().map(package_type_from_std_type).collect(),
        output: package_type_from_std_type(&decl.output),
        effects: Some(effect_row_from_texts(&decl.effects)),
        visibility: public_visibility(),
    }
}

fn action_signature(
    path: Vec<String>,
    decl: &etas_std::EffectActionDecl,
) -> PackageEffectActionSignatureMetadata {
    let effect_args = decl
        .effect_args
        .iter()
        .map(package_action_arg_kind_from_std)
        .collect::<Vec<_>>();
    let selector_len = effect_args.len();
    PackageEffectActionSignatureMetadata {
        path,
        params: decl.params.iter().map(package_type_from_std_type).collect(),
        effect_args,
        selector_param_names: vec![String::new(); selector_len],
        selector_defaults: vec![None; selector_len],
        output: package_type_from_std_type(&decl.output),
        returns_never: matches!(&decl.output, StdType::Primitive(StdPrimitiveType::Never)),
        visibility: public_visibility(),
    }
}

fn package_action_arg_kind_from_std(
    kind: &etas_std::EffectActionArgKind,
) -> PackageEffectActionArgKindMetadata {
    match kind {
        etas_std::EffectActionArgKind::Type => PackageEffectActionArgKindMetadata::Type,
        etas_std::EffectActionArgKind::MemoryPlace => {
            PackageEffectActionArgKindMetadata::MemoryPlace
        }
        etas_std::EffectActionArgKind::StaticResourcePath { ty } => {
            PackageEffectActionArgKindMetadata::StaticResourcePath {
                ty: (*ty).to_owned(),
            }
        }
        etas_std::EffectActionArgKind::StringPattern => {
            PackageEffectActionArgKindMetadata::StringPattern
        }
    }
}

fn flow_effect_summary(path: Vec<String>, decl: &FlowDecl) -> PackageEffectSummaryMetadata {
    PackageEffectSummaryMetadata {
        item: path,
        public_effects: effect_row_from_texts(&decl.public_effects),
        requested_actions: effect_row_from_texts(&decl.requested_actions),
        handled_requested_actions: effect_row_from_texts(&decl.requested_actions),
        latent_flows: Vec::new(),
    }
}

fn effect_row_from_texts(effects: &[String]) -> PackageEffectRowMetadata {
    PackageEffectRowMetadata {
        effects: effects
            .iter()
            .map(|effect| effect_ref_from_text(effect))
            .collect(),
    }
}

fn effect_ref_from_text(text: &str) -> PackageEffectRefMetadata {
    let text = text.trim();
    let (path, args) = match text.split_once('[') {
        Some((head, rest)) => {
            let args = rest
                .strip_suffix(']')
                .map(split_top_level_commas)
                .unwrap_or_default()
                .into_iter()
                .map(|arg| effect_arg_from_text(head, arg))
                .collect();
            (path_from_text(head), args)
        }
        None => (path_from_text(text), Vec::new()),
    };
    PackageEffectRefMetadata { path, args }
}

fn effect_arg_from_text(owner: &str, text: &str) -> PackageEffectArgMetadata {
    let text = text.trim();
    if text == "_" {
        return PackageEffectArgMetadata::Wildcard;
    }
    if let Some(value) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return PackageEffectArgMetadata::String {
            value: value.to_owned(),
        };
    }
    if owner.trim() == "Error" {
        return PackageEffectArgMetadata::Type {
            ty: package_type_from_std_type(&StdType::parse(text)),
        };
    }
    PackageEffectArgMetadata::Path {
        path: path_from_text(text),
    }
}

fn path_from_text(text: &str) -> Vec<String> {
    text.split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut parts = Vec::new();
    for (index, ch) in text.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

fn package_type_from_std_type(ty: &StdType) -> PackageTypeMetadata {
    match ty {
        StdType::Primitive(primitive) => PackageTypeMetadata::Primitive {
            name: primitive_type_name(*primitive).to_owned(),
        },
        StdType::Var(name) => PackageTypeMetadata::Var { name: name.clone() },
        StdType::Support(constraint) => PackageTypeMetadata::Named {
            path: vec![support_constraint_name(*constraint).to_owned()],
        },
        StdType::Array(element) => PackageTypeMetadata::Array {
            element: Box::new(package_type_from_std_type(element)),
        },
        StdType::List(element) => PackageTypeMetadata::List {
            element: Box::new(package_type_from_std_type(element)),
        },
        StdType::Map { key, value } => PackageTypeMetadata::Map {
            key: Box::new(package_type_from_std_type(key)),
            value: Box::new(package_type_from_std_type(value)),
        },
        StdType::Set(element) => PackageTypeMetadata::Set {
            element: Box::new(package_type_from_std_type(element)),
        },
        StdType::Range(index) => PackageTypeMetadata::Range {
            index: Box::new(package_type_from_std_type(index)),
        },
        StdType::Slice(element) => PackageTypeMetadata::Slice {
            element: Box::new(package_type_from_std_type(element)),
        },
        StdType::Option(inner) => PackageTypeMetadata::Option {
            inner: Box::new(package_type_from_std_type(inner)),
        },
        StdType::Result { ok, err } => PackageTypeMetadata::Result {
            ok: Box::new(package_type_from_std_type(ok)),
            err: Box::new(package_type_from_std_type(err)),
        },
        StdType::Schema(inner) => PackageTypeMetadata::ResourceHandle {
            name: "Schema".to_owned(),
            args: vec![package_type_from_std_type(inner)],
        },
        StdType::Trust { wrapper, inner } => PackageTypeMetadata::Trust {
            wrapper: match wrapper {
                etas_std::StdTrustWrapper::Trusted => "Trusted",
                etas_std::StdTrustWrapper::Untrusted => "Untrusted",
                etas_std::StdTrustWrapper::Secret => "Secret",
                etas_std::StdTrustWrapper::Public => "Public",
                etas_std::StdTrustWrapper::Sanitized => "Sanitized",
            }
            .to_owned(),
            inner: Box::new(package_type_from_std_type(inner)),
        },
        StdType::Prompt => PackageTypeMetadata::Prompt,
        StdType::PromptPart => PackageTypeMetadata::PromptPart,
        StdType::Message(inner) => PackageTypeMetadata::Message {
            inner: Box::new(package_type_from_std_type(inner)),
        },
        StdType::MemorySelection(inner) => PackageTypeMetadata::MemorySelection {
            inner: Box::new(package_type_from_std_type(inner)),
        },
        StdType::Store { key, value } => PackageTypeMetadata::Store {
            key: Box::new(package_type_from_std_type(key)),
            value: Box::new(package_type_from_std_type(value)),
        },
        StdType::MemoryRegion(schema) => PackageTypeMetadata::MemoryRegion {
            schema: Box::new(package_type_from_std_type(schema)),
        },
        StdType::ResourceHandleMemoryRegion(schema) => PackageTypeMetadata::ResourceHandle {
            name: "MemoryRegion".to_owned(),
            args: vec![package_type_from_std_type(schema)],
        },
        StdType::Named(name) => PackageTypeMetadata::Named {
            path: path_from_text(name),
        },
        StdType::NamedApplied { name, args } => PackageTypeMetadata::Applied {
            path: path_from_text(name),
            args: args.iter().map(package_type_from_std_type).collect(),
        },
        StdType::Record(fields) => PackageTypeMetadata::Record {
            fields: fields
                .iter()
                .map(|field| PackageRecordFieldMetadata {
                    name: field.name.clone(),
                    ty: package_type_from_std_type(&field.ty),
                })
                .collect(),
        },
        StdType::Tuple(elements) => PackageTypeMetadata::Tuple {
            elements: elements.iter().map(package_type_from_std_type).collect(),
        },
    }
}

fn primitive_type_name(primitive: StdPrimitiveType) -> &'static str {
    match primitive {
        StdPrimitiveType::Bool => "bool",
        StdPrimitiveType::I8 => "i8",
        StdPrimitiveType::I16 => "i16",
        StdPrimitiveType::I32 => "i32",
        StdPrimitiveType::I64 => "i64",
        StdPrimitiveType::I128 => "i128",
        StdPrimitiveType::ISize => "isize",
        StdPrimitiveType::U8 => "u8",
        StdPrimitiveType::U16 => "u16",
        StdPrimitiveType::U32 => "u32",
        StdPrimitiveType::U64 => "u64",
        StdPrimitiveType::U128 => "u128",
        StdPrimitiveType::USize => "usize",
        StdPrimitiveType::F32 => "f32",
        StdPrimitiveType::F64 => "f64",
        StdPrimitiveType::Char => "char",
        StdPrimitiveType::String => "string",
        StdPrimitiveType::Bytes => "bytes",
        StdPrimitiveType::Unit => "unit",
        StdPrimitiveType::Never => "never",
    }
}

fn support_constraint_name(constraint: StdSupportConstraint) -> &'static str {
    match constraint {
        StdSupportConstraint::Index => "Index",
        StdSupportConstraint::LengthInput => "LengthInput",
        StdSupportConstraint::EmptinessInput => "EmptinessInput",
    }
}

fn runtime_requirement_for_std_effect(decl: &EffectDecl) -> Option<String> {
    let requirement = match decl.name.as_str() {
        "Agentic" => "Agentic",
        "Network" | "Web" | "Db" | "Vector" | "Browser" | "Email" | "Calendar" | "Queue"
        | "ObjectStore" | "Payment" | "Package" | "Deploy" => "Network",
        "FileIO" | "Workspace" | "File" => "FileIO",
        "Command" => "Command",
        "Memory" => "DurableMemory",
        "Secret" => "SecretAccess",
        "Time" | "Clock" => "Time",
        "Console" => "Console",
        "Approval" => "Approval",
        _ => return None,
    };
    Some(requirement.to_owned())
}

fn builtin_std_public_metadata_fingerprint(metadata: &PackagePublicMetadata) -> String {
    let bytes =
        serde_json::to_vec(metadata).expect("builtin std public metadata must be serializable");
    format!("builtin:std:{}", blake3::hash(&bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_std_public_metadata_is_projected_from_registry() {
        let metadata = builtin_std_public_metadata();

        assert!(
            metadata
                .modules
                .iter()
                .any(|module| module.path == ["std", "io"]),
            "std.io module should be projected from etas_std registry"
        );
        assert!(
            metadata.flows.iter().any(|flow| {
                flow.path == ["std", "io", "println"]
                    && flow.effects.as_ref().is_some_and(|row| {
                        row.effects.iter().any(|effect| {
                            effect.path == ["Error"]
                                && matches!(
                                    effect.args.first(),
                                    Some(PackageEffectArgMetadata::Type { .. })
                                )
                        })
                    })
            }),
            "std.io.println should expose its typed public Error row"
        );
        assert!(
            metadata.effect_summaries.iter().any(|summary| {
                summary.item == ["std", "io", "println"]
                    && summary
                        .requested_actions
                        .effects
                        .iter()
                        .any(|action| action.path == ["Console", "stdout_write"])
            }),
            "std.io.println should expose requested Console action metadata"
        );
        assert!(
            metadata
                .actions
                .iter()
                .any(|action| action.path == ["std", "effects", "actions", "Command", "run"]),
            "std Command.run action signature should be projected"
        );
    }

    #[test]
    fn builtin_std_effect_metadata_is_projected_from_registry() {
        let metadata = builtin_std_effect_metadata();

        assert!(
            metadata.tags.iter().any(|tag| tag.path == ["Console"]
                && tag.runtime_requirement.as_deref() == Some("Console")),
            "Console tag should carry console runtime requirement"
        );
        assert!(
            metadata
                .extensions
                .iter()
                .any(|extension| extension.child == ["Console"] && extension.parent == ["FileIO"]),
            "Console extends FileIO should come from registry effect declaration"
        );
        assert!(
            metadata.tags.iter().any(|tag| tag.path == ["Network"]
                && tag.runtime_requirement.as_deref() == Some("Network")),
            "core and standard effect tags should not be limited to old Console/FileIO hardcode"
        );
    }
}
