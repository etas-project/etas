mod archive;
mod config;
mod git;
mod git_client;
mod github;
mod github_release;
mod http;
mod local_registry;
mod path;
mod provider;

pub(crate) use archive::pack_etaspkg;
pub use config::{GitHubSourceConfig, PackageSourceConfig, RegistrySourceConfig};
pub use provider::{ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest};

pub fn resolve_dependency_source(
    request: &SourceResolveRequest<'_>,
) -> Result<Option<ResolvedSourcePackage>, Vec<crate::PackageDiagnostic>> {
    let providers: Vec<Box<dyn SourceProvider>> = vec![
        Box::new(path::PathSourceProvider),
        Box::new(local_registry::LocalRegistrySourceProvider),
        Box::new(github::GitHubCloneSourceProvider::<
            git_client::CommandGitClient,
        >::default()),
        Box::new(github_release::GitHubReleaseSourceProvider::default()),
        Box::new(git::GitSourceProvider::<git_client::CommandGitClient>::default()),
    ];
    for provider in providers {
        if let Some(package) = provider.resolve(request)? {
            return Ok(Some(package));
        }
    }
    Ok(None)
}
