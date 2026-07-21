pub mod diagnostics;
mod file_lock;
pub mod lockfile;
pub mod manifest;
pub mod metadata;
mod metadata_artifact;
mod pack;
mod resolver;
mod source;
mod store;
mod vendor;

pub use diagnostics::{PackageDiagnostic, PackageError};
pub use manifest::{
    BinTarget, BindingsSection, DependencySpec, Manifest, PackageSection, RuntimeBackendProfile,
    RuntimeCommandProfile, RuntimeExecutionProfile, RuntimeFilesystemProfile, RuntimeModeProfile,
    RuntimeModelProfile, RuntimeNetworkProfile, RuntimePolicyProfile, RuntimeProfile,
    RuntimeRetryProfile, RuntimeSection, RuntimeToolsProfile, SourceSection, ToolBinding,
    discover_manifest, read_manifest,
};
pub use metadata::{
    PackageActionSummaryMetadata, PackageAgentSignatureMetadata,
    PackageCallableSpecSatisfactionMetadata, PackageEffectActionArgKindMetadata,
    PackageEffectActionSignatureMetadata, PackageEffectArgMetadata, PackageEffectExtensionMetadata,
    PackageEffectMetadata, PackageEffectRefMetadata, PackageEffectRowMetadata,
    PackageEffectSummaryMetadata, PackageEffectTagMetadata, PackageEnvironmentMetadata,
    PackageExternalExportMetadata, PackageExternalModuleMetadata,
    PackageExternalModuleOwnerMetadata, PackageFlowSignatureMetadata, PackageIdentity,
    PackageIndex, PackageLatentFlowSummaryMetadata, PackageNamedSignatureMetadata,
    PackagePublicMetadata, PackageReExportMetadata, PackageRecordFieldMetadata,
    PackageSpecBoundMetadata, PackageSpecImplMetadata, PackageSpecKindMetadata,
    PackageSpecMethodMetadata, PackageSpecSignatureMetadata, PackageToolBindingMetadata,
    PackageToolSchemaMetadata, PackageToolSignatureMetadata, PackageTraceSpecClauseKindMetadata,
    PackageTraceSpecClauseMetadata, PackageTraceSpecConformanceMetadata,
    PackageTraceSpecConformanceTargetMetadata, PackageTraceSpecSummaryMetadata,
    PackageTypeMetadata, PackageTypeSpecSatisfactionMetadata, ResolvedDependency,
    ResolvedDependencySource,
};
#[cfg(feature = "test-support")]
pub use metadata_artifact::write_fixture_package_metadata_artifact;
pub use metadata_artifact::{
    MetadataArtifactHeader, MetadataArtifactInfo, MetadataSectionKind, metadata_artifact_hash,
    package_dependency_lock_hash, package_manifest_hash, package_metadata_artifact_path,
    package_source_payload_hash, read_package_metadata_artifact,
    read_package_metadata_artifact_info, resolved_dependency_metadata,
};
pub use pack::{PackPackageOptions, PackedPackageResult, pack_package};
pub use resolver::{
    LockPackageOptions, LockedPackageResult, MaterializePackageOptions, MaterializedPackageResult,
    PreparePackageOptions, PreparedPackage, lock_package, materialize_package, prepare_package,
};
pub use source::{GitHubSourceConfig, PackageSourceConfig, RegistrySourceConfig};
