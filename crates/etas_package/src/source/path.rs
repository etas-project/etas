use crate::{PackageDiagnostic, manifest, vendor};

use super::{ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest};

pub struct PathSourceProvider;

impl SourceProvider for PathSourceProvider {
    fn resolve(
        &self,
        request: &SourceResolveRequest<'_>,
    ) -> Result<Option<ResolvedSourcePackage>, Vec<PackageDiagnostic>> {
        let manifest::DependencySpec::Detailed(detail) = request.spec else {
            return Ok(None);
        };
        let Some(path) = &detail.path else {
            return Ok(None);
        };
        let absolute = if path.is_absolute() {
            path.clone()
        } else {
            request.package_root.join(path)
        };
        let package_root = match absolute.canonicalize() {
            Ok(package_root) => package_root,
            Err(source) => {
                return Err(vec![PackageDiagnostic::new(
                    "package.materialize.path_missing",
                    format!(
                        "path dependency `{}` points to `{}`, which cannot be read: {source}",
                        request.dependency_name,
                        absolute.display()
                    ),
                    Some(absolute),
                )]);
            }
        };
        let artifact_path = vendor::package_metadata_artifact_path(&package_root);
        if !artifact_path.is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.materialize.metadata_missing",
                format!(
                    "path dependency `{}` is missing .etas/package.etasmeta at `{}`",
                    request.dependency_name,
                    package_root.display()
                ),
                Some(artifact_path),
            )]);
        }
        let package_index = vendor::read_package_metadata_artifact_required(&package_root)
            .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.materialize.metadata_unreadable",
                    format!(
                        "path dependency `{}` metadata artifact could not be read: {error}",
                        request.dependency_name
                    ),
                    Some(vendor::package_metadata_artifact_path(&package_root)),
                )]
            })?;
        Ok(Some(ResolvedSourcePackage {
            package_root,
            package_index,
            source: SourceCandidate::Path,
        }))
    }
}
