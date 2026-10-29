mod definition_origin;
pub mod diagnostics;
mod file_lock;
pub mod lockfile;
pub mod manifest;
pub mod metadata;
mod metadata_artifact;
mod pack;
mod resolver;
mod source;
mod source_snapshot;
mod store;
mod vendor;

pub use definition_origin::local_definition_package_key;
pub use diagnostics::{PackageDiagnostic, PackageError};
pub use manifest::{
    BinTarget, BindingsSection, DependencySpec, Manifest, PackageSection, ReflectionSection,
    RuntimeBackendProfile, RuntimeCommandProfile, RuntimeCompilerProfile, RuntimeExecutionProfile,
    RuntimeFilesystemProfile, RuntimeFilesystemRegionProfile, RuntimeModeProfile,
    RuntimeModelProfile, RuntimeModelTimeoutProfile, RuntimeNetworkProfile, RuntimePolicyProfile,
    RuntimeProfile, RuntimeRetryProfile, RuntimeSection, RuntimeToolsProfile, SourceSection,
    ToolBinding, discover_manifest, read_manifest,
};
pub use metadata::{
    PackageActionRequestMetadata, PackageActionRequestSignatureMetadata,
    PackageActionSummaryMetadata, PackageActionTraceEventSourceMetadata,
    PackageActionTraceMetadata, PackageAgentSignatureMetadata,
    PackageCallableSpecSatisfactionMetadata, PackageDefaultHandlerContributionMetadata,
    PackageDefaultHandlerEffectsMetadata, PackageEffectActionArgKindMetadata,
    PackageEffectActionSignatureMetadata, PackageEffectArgMetadata, PackageEffectExtensionMetadata,
    PackageEffectMetadata, PackageEffectRefMetadata, PackageEffectRowMetadata,
    PackageEffectSummaryMetadata, PackageEffectTagMetadata, PackageEnvironmentMetadata,
    PackageExternalExportMetadata, PackageExternalModuleMetadata,
    PackageExternalModuleOwnerMetadata, PackageFlowSignatureMetadata, PackageIdentity,
    PackageIndex, PackageLatentFlowSummaryMetadata, PackageNamedSignatureMetadata,
    PackageParameterRequestMetadata, PackagePublicMetadata, PackageReExportMetadata,
    PackageRecordFieldMetadata, PackageRequestInstantiationMetadata, PackageSpecBoundMetadata,
    PackageSpecImplMetadata, PackageSpecKindMetadata, PackageSpecMethodMetadata,
    PackageSpecSignatureMetadata, PackageToolBindingMetadata, PackageToolSchemaMetadata,
    PackageToolSignatureMetadata, PackageTraceSpecClauseKindMetadata,
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
pub use source_snapshot::{
    LockedPackageSourceSnapshot, SourceSnapshotBudget, SourceSnapshotError, SourceSnapshotLimits,
    SourceSnapshotResource, read_locked_package_sources,
};
#[cfg(feature = "test-support")]
pub use vendor::dependency_lock_checksum as dependency_lock_checksum_for_test;
pub use vendor::path_package_payload_checksum as package_payload_checksum;
