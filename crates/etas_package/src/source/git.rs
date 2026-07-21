use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{PackageDiagnostic, manifest, store, vendor};

use super::{
    ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest,
    git_client::{CommandGitClient, GitClient},
};

#[derive(Default)]
pub struct GitSourceProvider<C = CommandGitClient> {
    client: C,
}

impl<C: GitClient> SourceProvider for GitSourceProvider<C> {
    fn resolve(
        &self,
        request: &SourceResolveRequest<'_>,
    ) -> Result<Option<ResolvedSourcePackage>, Vec<PackageDiagnostic>> {
        let manifest::DependencySpec::Detailed(detail) = request.spec else {
            return Ok(None);
        };
        let Some(git_url) = &detail.git else {
            return Ok(None);
        };
        let Some(repo_path) = local_git_path(git_url) else {
            return Err(vec![PackageDiagnostic::new(
                "package.git.unsupported_transport",
                format!(
                    "dependency `{}` uses git source `{git_url}`, but only local file git sources are supported by this backend",
                    request.dependency_name
                ),
                Some(request.package_root.join("etas.toml")),
            )]);
        };
        let selector = detail
            .rev
            .as_deref()
            .or(detail.tag.as_deref())
            .or(detail.branch.as_deref())
            .unwrap_or("HEAD");
        let selector_commit = format!("{selector}^{{commit}}");
        let rev = self
            .client
            .output(&repo_path, &["rev-parse", &selector_commit])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.git.rev_unresolved",
                    format!(
                        "dependency `{}` git selector `{selector}` could not be resolved: {message}",
                        request.dependency_name
                    ),
                    Some(request.package_root.join("etas.toml")),
                )]
            })?;
        let checkout_parent = request
            .package_root
            .join(".etas")
            .join("source")
            .join("git");
        std::fs::create_dir_all(&checkout_parent).map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.git.checkout_failed",
                format!(
                    "dependency `{}` git checkout parent could not be created: {source}",
                    request.dependency_name
                ),
                Some(checkout_parent.clone()),
            )]
        })?;
        let checkout_root =
            checkout_parent.join(unique_checkout_name(request.dependency_name, &rev));
        let repo_arg = repo_path.to_string_lossy().into_owned();
        let checkout_arg = checkout_root.to_string_lossy().into_owned();
        self.client
            .status_no_repo(&[
                "clone",
                "--quiet",
                "--no-checkout",
                &repo_arg,
                &checkout_arg,
            ])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.git.command_failed",
                    format!(
                        "git command for dependency `{}` failed: {message}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        self.client
            .status(&checkout_root, &["checkout", "--quiet", "--detach", &rev])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.git.command_failed",
                    format!(
                        "git command for dependency `{}` failed: {message}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        let git_dir = checkout_root.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|source| {
                vec![PackageDiagnostic::new(
                    "package.git.checkout_failed",
                    format!(
                        "dependency `{}` git checkout metadata could not be removed: {source}",
                        request.dependency_name
                    ),
                    Some(git_dir),
                )]
            })?;
        }
        let artifact_path = vendor::package_metadata_artifact_path(&checkout_root);
        if !artifact_path.is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.git.metadata_missing",
                format!(
                    "git dependency `{}` checkout is missing .etas/package.etasmeta",
                    request.dependency_name
                ),
                Some(artifact_path),
            )]);
        }
        let package_index = vendor::read_package_metadata_artifact_required(&checkout_root)
            .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.git.metadata_unreadable",
                    format!(
                        "git dependency `{}` metadata artifact could not be read: {error}",
                        request.dependency_name
                    ),
                    Some(vendor::package_metadata_artifact_path(&checkout_root)),
                )]
            })?;
        let payload_checksum =
            vendor::path_package_payload_checksum(&checkout_root).map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.git.payload_unreadable",
                    format!(
                        "git dependency `{}` payload could not be checksummed: {error}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        let store_path =
            store::store_package_payload(request.package_root, &checkout_root, &payload_checksum)
                .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.git.store_write_failed",
                    format!(
                        "git dependency `{}` could not be stored: {error}",
                        request.dependency_name
                    ),
                    Some(request.package_root.join(".etas").join("store")),
                )]
            })?;
        Ok(Some(ResolvedSourcePackage {
            package_root: checkout_root,
            package_index,
            source: SourceCandidate::Git {
                url: git_url.clone(),
                rev,
                checksum: payload_checksum,
                store: store_path,
            },
        }))
    }
}

fn local_git_path(git_url: &str) -> Option<PathBuf> {
    if let Some(path) = git_url.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }
    let path = PathBuf::from(git_url);
    path.is_absolute().then_some(path)
}

fn short_rev(rev: &str) -> String {
    rev.chars().take(12).collect()
}

fn unique_checkout_name(dependency_name: &str, rev: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!(
        ".checkout-{}-{}-{}-{stamp}",
        dependency_name,
        short_rev(rev),
        std::process::id()
    )
}
