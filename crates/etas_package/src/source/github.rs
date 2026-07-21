use std::time::{SystemTime, UNIX_EPOCH};

use crate::{PackageDiagnostic, manifest, store, vendor};

use super::{
    ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest,
    git_client::{CommandGitClient, GitClient},
};

#[derive(Default)]
pub struct GitHubCloneSourceProvider<C = CommandGitClient> {
    client: C,
}

impl<C: GitClient> SourceProvider for GitHubCloneSourceProvider<C> {
    fn resolve(
        &self,
        request: &SourceResolveRequest<'_>,
    ) -> Result<Option<ResolvedSourcePackage>, Vec<PackageDiagnostic>> {
        let manifest::DependencySpec::Detailed(detail) = request.spec else {
            return Ok(None);
        };
        let Some(repo) = &detail.github else {
            return Ok(None);
        };
        if detail.release.is_some() {
            return Ok(None);
        }
        let clone_url =
            github_clone_url(request.source_config.github.clone_base_url.as_deref(), repo);
        let checkout_parent = request
            .package_root
            .join(".etas")
            .join("source")
            .join("github");
        std::fs::create_dir_all(&checkout_parent).map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.github.checkout_failed",
                format!(
                    "dependency `{}` GitHub checkout parent could not be created: {source}",
                    request.dependency_name
                ),
                Some(checkout_parent.clone()),
            )]
        })?;
        let checkout_root = checkout_parent.join(unique_checkout_name(request.dependency_name));
        let checkout_arg = checkout_root.to_string_lossy().into_owned();
        self.client
            .status_no_repo(&[
                "clone",
                "--quiet",
                "--no-checkout",
                &clone_url,
                &checkout_arg,
            ])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.github.clone_failed",
                    format!(
                        "GitHub dependency `{}` could not be cloned from `{clone_url}`: {message}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        let selector = detail
            .rev
            .as_deref()
            .or(detail.tag.as_deref())
            .or(detail.branch.as_deref())
            .unwrap_or("HEAD");
        let selector_commit = format!("{selector}^{{commit}}");
        let rev = self
            .client
            .output(&checkout_root, &["rev-parse", &selector_commit])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.github.rev_unresolved",
                    format!(
                        "GitHub dependency `{}` selector `{selector}` could not be resolved: {message}",
                        request.dependency_name
                    ),
                    Some(request.package_root.join("etas.toml")),
                )]
            })?;
        self.client
            .status(&checkout_root, &["checkout", "--quiet", "--detach", &rev])
            .map_err(|message| {
                vec![PackageDiagnostic::new(
                    "package.github.checkout_failed",
                    format!(
                        "GitHub dependency `{}` exact revision `{rev}` could not be checked out: {message}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        let git_dir = checkout_root.join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).map_err(|source| {
                vec![PackageDiagnostic::new(
                    "package.github.checkout_failed",
                    format!(
                        "GitHub dependency `{}` checkout metadata could not be removed: {source}",
                        request.dependency_name
                    ),
                    Some(git_dir),
                )]
            })?;
        }
        let artifact_path = vendor::package_metadata_artifact_path(&checkout_root);
        if !artifact_path.is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.github.metadata_missing",
                format!(
                    "GitHub dependency `{}` checkout is missing .etas/package.etasmeta",
                    request.dependency_name
                ),
                Some(artifact_path),
            )]);
        }
        let package_index = vendor::read_package_metadata_artifact_required(&checkout_root)
            .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.github.metadata_unreadable",
                    format!(
                        "GitHub dependency `{}` metadata artifact could not be read: {error}",
                        request.dependency_name
                    ),
                    Some(vendor::package_metadata_artifact_path(&checkout_root)),
                )]
            })?;
        let payload_checksum =
            vendor::path_package_payload_checksum(&checkout_root).map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.github.payload_unreadable",
                    format!(
                        "GitHub dependency `{}` payload could not be checksummed: {error}",
                        request.dependency_name
                    ),
                    Some(checkout_root.clone()),
                )]
            })?;
        let store_path =
            store::store_package_payload(request.package_root, &checkout_root, &payload_checksum)
                .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.github.store_write_failed",
                    format!(
                        "GitHub dependency `{}` could not be stored: {error}",
                        request.dependency_name
                    ),
                    Some(request.package_root.join(".etas").join("store")),
                )]
            })?;
        Ok(Some(ResolvedSourcePackage {
            package_root: checkout_root,
            package_index,
            source: SourceCandidate::GitHubClone {
                repo: repo.clone(),
                rev,
                checksum: payload_checksum,
                store: store_path,
            },
        }))
    }
}

fn github_clone_url(base: Option<&str>, repo: &str) -> String {
    let base = base.unwrap_or("https://github.com").trim_end_matches('/');
    format!("{base}/{repo}.git")
}

fn unique_checkout_name(dependency_name: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!(
        ".checkout-{}-{}-{stamp}",
        dependency_name,
        std::process::id()
    )
}
