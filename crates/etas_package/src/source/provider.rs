use std::path::{Path, PathBuf};

use crate::{PackageDiagnostic, manifest, metadata};

use super::PackageSourceConfig;

pub trait SourceProvider {
    fn resolve(
        &self,
        request: &SourceResolveRequest<'_>,
    ) -> Result<Option<ResolvedSourcePackage>, Vec<PackageDiagnostic>>;
}

pub struct SourceResolveRequest<'a> {
    pub package_root: &'a Path,
    pub source_config: &'a PackageSourceConfig,
    pub dependency_name: &'a str,
    pub spec: &'a manifest::DependencySpec,
}

#[derive(Clone, Debug)]
pub struct ResolvedSourcePackage {
    pub package_root: PathBuf,
    pub package_index: metadata::PackageIndex,
    pub source: SourceCandidate,
}

#[derive(Clone, Debug)]
pub enum SourceCandidate {
    Path,
    Registry {
        registry: String,
        checksum: String,
        store: String,
    },
    Git {
        url: String,
        rev: String,
        checksum: String,
        store: String,
    },
    GitHubClone {
        repo: String,
        rev: String,
        checksum: String,
        store: String,
    },
    GitHubRelease {
        repo: String,
        release: String,
        asset: String,
        asset_checksum: String,
        payload_checksum: String,
        store: String,
    },
}
