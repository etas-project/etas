use serde::{Deserialize, Serialize};

use crate::{BinTarget, Manifest};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageIdentity {
    pub name: String,
    pub version: String,
    pub edition: String,
}

impl PackageIdentity {
    pub fn current(manifest: &Manifest) -> Self {
        Self {
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            edition: manifest.package.edition.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEnvironmentMetadata {
    pub current_package: Option<PackageIdentity>,
    pub dependencies: Vec<ResolvedDependency>,
    pub external_modules: Vec<PackageExternalModuleMetadata>,
    pub public_metadata: PackagePublicMetadata,
    pub effect_metadata: PackageEffectMetadata,
    pub tool_bindings: Vec<PackageToolBindingMetadata>,
    pub metadata_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResolvedDependency {
    pub identity: PackageIdentity,
    pub import_root: String,
    pub source: ResolvedDependencySource,
    #[serde(default)]
    pub dependencies: Vec<ResolvedDependency>,
    #[serde(default, skip_serializing_if = "PackagePublicMetadata::is_empty")]
    pub public_metadata: PackagePublicMetadata,
    #[serde(default, skip_serializing_if = "PackageEffectMetadata::is_empty")]
    pub effect_metadata: PackageEffectMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_bindings: Vec<PackageToolBindingMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolvedDependencySource {
    Builtin {
        checksum: String,
    },
    Registry {
        registry: String,
        checksum: String,
        #[serde(default)]
        store: Option<String>,
    },
    Git {
        url: String,
        rev: String,
        checksum: String,
        #[serde(default)]
        store: Option<String>,
    },
    GitHubClone {
        repo: String,
        rev: String,
        checksum: String,
        #[serde(default)]
        store: Option<String>,
    },
    GitHubRelease {
        repo: String,
        release: String,
        asset: String,
        asset_checksum: String,
        payload_checksum: String,
        #[serde(default)]
        store: Option<String>,
    },
    Path {
        path: String,
        checksum: String,
    },
    Vendor {
        path: String,
        checksum: String,
        #[serde(default)]
        store: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageExternalModuleMetadata {
    #[serde(default)]
    pub package: Option<PackageExternalModuleOwnerMetadata>,
    pub id: u32,
    pub path: Vec<String>,
    pub exports: Vec<PackageExternalExportMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageExternalModuleOwnerMetadata {
    pub identity: PackageIdentity,
    pub import_root: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageExternalExportMetadata {
    pub id: u32,
    pub name: String,
    pub visibility: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageToolBindingMetadata {
    pub tool: String,
    pub kind: String,
    pub provider: String,
    pub effect_row: Vec<String>,
    #[serde(default)]
    pub action_row: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageIndex {
    pub version: u32,
    pub package: PackageIdentity,
    #[serde(default)]
    pub dependencies: Vec<ResolvedDependency>,
    #[serde(default)]
    pub external_modules: Vec<PackageExternalModuleMetadata>,
    #[serde(default)]
    pub public_metadata: PackagePublicMetadata,
    #[serde(default)]
    pub effect_metadata: PackageEffectMetadata,
    #[serde(default)]
    pub tool_bindings: Vec<PackageToolBindingMetadata>,
    #[serde(default)]
    pub bins: Vec<BinTarget>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackagePublicMetadata {
    #[serde(default)]
    pub modules: Vec<PackageExternalModuleMetadata>,
    #[serde(default)]
    pub types: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub values: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub enums: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub flows: Vec<PackageFlowSignatureMetadata>,
    #[serde(default)]
    pub agents: Vec<PackageAgentSignatureMetadata>,
    #[serde(default)]
    pub tools: Vec<PackageToolSignatureMetadata>,
    #[serde(default)]
    pub effects: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub actions: Vec<PackageEffectActionSignatureMetadata>,
    #[serde(default)]
    pub trace_specs: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub spec_signatures: Vec<PackageSpecSignatureMetadata>,
    #[serde(default)]
    pub spec_impls: Vec<PackageSpecImplMetadata>,
    #[serde(default)]
    pub type_spec_satisfactions: Vec<PackageTypeSpecSatisfactionMetadata>,
    #[serde(default)]
    pub callable_spec_satisfactions: Vec<PackageCallableSpecSatisfactionMetadata>,
    #[serde(default)]
    pub trace_spec_conformances: Vec<PackageTraceSpecConformanceMetadata>,
    #[serde(default)]
    pub protocols: Vec<PackageNamedSignatureMetadata>,
    #[serde(default)]
    pub effect_summaries: Vec<PackageEffectSummaryMetadata>,
    #[serde(default)]
    pub action_summaries: Vec<PackageActionSummaryMetadata>,
    #[serde(default)]
    pub tool_schemas: Vec<PackageToolSchemaMetadata>,
    #[serde(default)]
    pub trace_spec_summaries: Vec<PackageTraceSpecSummaryMetadata>,
    #[serde(default)]
    pub re_exports: Vec<PackageReExportMetadata>,
    #[serde(default)]
    pub annotations: Vec<PackageAnnotationMetadata>,
    #[serde(default)]
    pub fingerprint: Option<String>,
}

impl PackagePublicMetadata {
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
            && self.types.is_empty()
            && self.values.is_empty()
            && self.enums.is_empty()
            && self.flows.is_empty()
            && self.agents.is_empty()
            && self.tools.is_empty()
            && self.effects.is_empty()
            && self.actions.is_empty()
            && self.trace_specs.is_empty()
            && self.spec_signatures.is_empty()
            && self.spec_impls.is_empty()
            && self.type_spec_satisfactions.is_empty()
            && self.callable_spec_satisfactions.is_empty()
            && self.trace_spec_conformances.is_empty()
            && self.protocols.is_empty()
            && self.effect_summaries.is_empty()
            && self.action_summaries.is_empty()
            && self.tool_schemas.is_empty()
            && self.trace_spec_summaries.is_empty()
            && self.re_exports.is_empty()
            && self.annotations.is_empty()
            && self.fingerprint.is_none()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageAnnotationMetadata {
    pub item: Vec<String>,
    pub path: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageAnnotationArgMetadata>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageAnnotationArgMetadata {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    pub value: PackageAnnotationValueMetadata,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageAnnotationFieldMetadata {
    pub name: String,
    pub value: PackageAnnotationValueMetadata,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageAnnotationValueMetadata {
    pub kind: PackageAnnotationValueKindMetadata,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<PackageAnnotationValueMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<PackageAnnotationFieldMetadata>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageAnnotationValueKindMetadata {
    #[default]
    Unit,
    Bool,
    Int,
    Float,
    String,
    Char,
    Path,
    Array,
    List,
    Set,
    Tuple,
    Record,
    Constructor,
    Limit,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageNamedSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub visibility: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<PackageTypeMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageFlowSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub param_names: Vec<String>,
    #[serde(default)]
    pub params: Vec<PackageTypeMetadata>,
    pub output: PackageTypeMetadata,
    #[serde(default)]
    pub effects: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub visibility: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageAgentSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub param_names: Vec<String>,
    #[serde(default)]
    pub input: Vec<PackageTypeMetadata>,
    pub output: PackageTypeMetadata,
    #[serde(default)]
    pub effects: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub visibility: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageToolSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub param_names: Vec<String>,
    #[serde(default)]
    pub input: Vec<PackageTypeMetadata>,
    pub output: PackageTypeMetadata,
    #[serde(default)]
    pub effects: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub visibility: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectActionSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub params: Vec<PackageTypeMetadata>,
    #[serde(default)]
    pub effect_args: Vec<PackageEffectActionArgKindMetadata>,
    #[serde(default)]
    pub selector_param_names: Vec<String>,
    #[serde(default)]
    pub selector_defaults: Vec<Option<PackageEffectArgMetadata>>,
    pub output: PackageTypeMetadata,
    #[serde(default)]
    pub returns_never: bool,
    #[serde(default)]
    pub visibility: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageEffectActionArgKindMetadata {
    Type,
    MemoryPlace,
    StaticResourcePath { ty: String },
    StringPattern,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectRowMetadata {
    #[serde(default)]
    pub effects: Vec<PackageEffectRefMetadata>,
}

impl PackageEffectRowMetadata {
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectRefMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageEffectArgMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageEffectArgMetadata {
    Type { ty: PackageTypeMetadata },
    Path { path: Vec<String> },
    String { value: String },
    Wildcard,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageTypeMetadata {
    Primitive {
        name: String,
    },
    Var {
        name: String,
    },
    Named {
        path: Vec<String>,
    },
    Applied {
        path: Vec<String>,
        args: Vec<PackageTypeMetadata>,
    },
    Alias {
        path: Vec<String>,
        target: Box<PackageTypeMetadata>,
    },
    Nominal {
        path: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        representation: Option<Box<PackageTypeMetadata>>,
    },
    Array {
        element: Box<PackageTypeMetadata>,
    },
    List {
        element: Box<PackageTypeMetadata>,
    },
    Map {
        key: Box<PackageTypeMetadata>,
        value: Box<PackageTypeMetadata>,
    },
    Set {
        element: Box<PackageTypeMetadata>,
    },
    Range {
        index: Box<PackageTypeMetadata>,
    },
    Slice {
        element: Box<PackageTypeMetadata>,
    },
    Option {
        inner: Box<PackageTypeMetadata>,
    },
    Result {
        ok: Box<PackageTypeMetadata>,
        err: Box<PackageTypeMetadata>,
    },
    Record {
        fields: Vec<PackageRecordFieldMetadata>,
    },
    Tuple {
        elements: Vec<PackageTypeMetadata>,
    },
    Function {
        input: Vec<PackageTypeMetadata>,
        output: Box<PackageTypeMetadata>,
        #[serde(default)]
        effects: Option<PackageEffectRowMetadata>,
    },
    Handler {
        handled: PackageEffectRowMetadata,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        produced: Option<PackageEffectRowMetadata>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<Box<PackageTypeMetadata>>,
    },
    Trust {
        wrapper: String,
        inner: Box<PackageTypeMetadata>,
    },
    Prompt,
    PromptPart,
    Message {
        inner: Box<PackageTypeMetadata>,
    },
    MemorySelection {
        inner: Box<PackageTypeMetadata>,
    },
    Store {
        key: Box<PackageTypeMetadata>,
        value: Box<PackageTypeMetadata>,
    },
    MemoryRegion {
        schema: Box<PackageTypeMetadata>,
    },
    ResourceHandle {
        name: String,
        #[serde(default)]
        args: Vec<PackageTypeMetadata>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageRecordFieldMetadata {
    pub name: String,
    pub ty: PackageTypeMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectSummaryMetadata {
    pub item: Vec<String>,
    #[serde(default)]
    pub public_effects: PackageEffectRowMetadata,
    #[serde(default)]
    pub requested_actions: PackageEffectRowMetadata,
    #[serde(default, skip_serializing_if = "PackageEffectRowMetadata::is_empty")]
    pub handled_requested_actions: PackageEffectRowMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub latent_flows: Vec<PackageLatentFlowSummaryMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageLatentFlowSummaryMetadata {
    #[serde(default)]
    pub declared_bound: PackageEffectRowMetadata,
    #[serde(default)]
    pub inferred_effects: PackageEffectRowMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageActionSummaryMetadata {
    pub action: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageSpecSignatureMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub visibility: String,
    pub kind: PackageSpecKindMetadata,
    #[serde(default)]
    pub param_names: Vec<String>,
    #[serde(default)]
    pub callable: Option<PackageFlowSignatureMetadata>,
    #[serde(default)]
    pub methods: Vec<PackageSpecMethodMetadata>,
    #[serde(default)]
    pub super_specs: Vec<PackageSpecBoundMetadata>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageSpecKindMetadata {
    Type,
    Callable,
    Trace,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageSpecMethodMetadata {
    pub name: String,
    pub path: Vec<String>,
    #[serde(default)]
    pub signature: Option<PackageFlowSignatureMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageSpecBoundMetadata {
    pub spec: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageTypeMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageSpecImplMetadata {
    pub self_type: PackageTypeMetadata,
    pub spec: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageTypeMetadata>,
    #[serde(default)]
    pub methods: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageTypeSpecSatisfactionMetadata {
    pub self_type: PackageTypeMetadata,
    pub spec: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageTypeMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageCallableSpecSatisfactionMetadata {
    pub item: Vec<String>,
    pub spec: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackageTypeMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageTraceSpecConformanceMetadata {
    pub item: Vec<String>,
    pub target: PackageTraceSpecConformanceTargetMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageTraceSpecConformanceTargetMetadata {
    Inline,
    Named {
        spec: Vec<String>,
        #[serde(default)]
        args: Vec<PackageTypeMetadata>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageToolSchemaMetadata {
    pub tool: Vec<String>,
    pub schema: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageTraceSpecSummaryMetadata {
    pub trace_spec: Vec<String>,
    #[serde(default)]
    pub clauses: Vec<PackageTraceSpecClauseMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageTraceSpecClauseMetadata {
    pub kind: PackageTraceSpecClauseKindMetadata,
    #[serde(default)]
    pub pattern: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub guard: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub target: Option<PackageEffectRowMetadata>,
    #[serde(default)]
    pub obligation: Option<PackageEffectRowMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageTraceSpecClauseKindMetadata {
    Allow,
    Deny,
    RequireBefore,
    RequireAfter,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageReExportMetadata {
    pub from: Vec<String>,
    pub exported: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectMetadata {
    #[serde(default)]
    pub tags: Vec<PackageEffectTagMetadata>,
    #[serde(default)]
    pub extensions: Vec<PackageEffectExtensionMetadata>,
}

impl PackageEffectMetadata {
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty() && self.extensions.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectTagMetadata {
    pub path: Vec<String>,
    #[serde(default)]
    pub runtime_requirement: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageEffectExtensionMetadata {
    pub child: Vec<String>,
    pub parent: Vec<String>,
}
