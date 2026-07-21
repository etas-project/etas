#[cfg(any(test, feature = "test-support"))]
pub(super) const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(super) const CREATED_TARGET: &str = "etas-frontend";

pub use etas_package_metadata::{
    ARTIFACT_SCHEMA_VERSION, MetadataArtifactHeader, MetadataArtifactInfo, MetadataSectionKind,
};
