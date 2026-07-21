use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{PackageDiagnostic, manifest, store, vendor};

use super::{
    ResolvedSourcePackage, SourceCandidate, SourceProvider, SourceResolveRequest, archive,
    http::{HttpDownload, github_token},
};

#[derive(Default)]
pub struct GitHubReleaseSourceProvider;

impl SourceProvider for GitHubReleaseSourceProvider {
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
        let Some(release) = &detail.release else {
            return Ok(None);
        };
        let asset = detail
            .asset
            .as_deref()
            .expect("manifest validation checked asset");
        let expected_asset_checksum = detail
            .checksum
            .as_deref()
            .expect("manifest validation checked checksum");
        if !asset.ends_with(".etaspkg") {
            return Err(vec![PackageDiagnostic::new(
                "package.github_release.unsupported_asset",
                format!(
                    "GitHub release dependency `{}` asset `{asset}` must end with .etaspkg",
                    request.dependency_name
                ),
                Some(request.package_root.join("etas.toml")),
            )]);
        }

        let download_parent = request
            .package_root
            .join(".etas")
            .join("source")
            .join("github-release");
        fs::create_dir_all(&download_parent).map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.github_release.download_failed",
                format!(
                    "dependency `{}` download parent could not be created: {source}",
                    request.dependency_name
                ),
                Some(download_parent.clone()),
            )]
        })?;
        let temp_name = unique_name(request.dependency_name);
        let archive_path = download_parent.join(format!("{temp_name}.etaspkg"));
        let unpack_root = download_parent.join(format!("{temp_name}-unpacked"));
        let url = github_release_url(
            request.source_config.github.release_base_url.as_deref(),
            repo,
            release,
            asset,
        );
        HttpDownload::github(
            url,
            github_token(request.source_config.github.token.as_deref()),
        )
        .download_to(&archive_path)
        .map_err(|diagnostic| vec![diagnostic])?;

        let actual_asset_checksum =
            file_checksum(&archive_path).map_err(|diagnostic| vec![diagnostic])?;
        if actual_asset_checksum != expected_asset_checksum {
            return Err(vec![PackageDiagnostic::new(
                "package.github_release.checksum_mismatch",
                format!(
                    "GitHub release dependency `{}` asset checksum `{actual_asset_checksum}` does not match expected `{expected_asset_checksum}`",
                    request.dependency_name
                ),
                Some(archive_path),
            )]);
        }
        if unpack_root.exists() {
            fs::remove_dir_all(&unpack_root).map_err(|source| {
                vec![PackageDiagnostic::new(
                    "package.github_release.unpack_failed",
                    format!(
                        "dependency `{}` stale unpack directory could not be removed: {source}",
                        request.dependency_name
                    ),
                    Some(unpack_root.clone()),
                )]
            })?;
        }
        fs::create_dir_all(&unpack_root).map_err(|source| {
            vec![PackageDiagnostic::new(
                "package.github_release.unpack_failed",
                format!(
                    "dependency `{}` unpack directory could not be created: {source}",
                    request.dependency_name
                ),
                Some(unpack_root.clone()),
            )]
        })?;
        archive::unpack_etaspkg(&archive_path, &unpack_root).map_err(|error| {
            vec![PackageDiagnostic::new(
                "package.github_release.unpack_failed",
                format!(
                    "GitHub release dependency `{}` could not be unpacked: {error}",
                    request.dependency_name
                ),
                Some(archive_path.clone()),
            )]
        })?;
        if !unpack_root.join("etas.toml").is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.github_release.manifest_missing",
                format!(
                    "GitHub release dependency `{}` asset is missing etas.toml",
                    request.dependency_name
                ),
                Some(unpack_root.join("etas.toml")),
            )]);
        }
        let artifact_path = vendor::package_metadata_artifact_path(&unpack_root);
        if !artifact_path.is_file() {
            return Err(vec![PackageDiagnostic::new(
                "package.github_release.metadata_missing",
                format!(
                    "GitHub release dependency `{}` asset is missing .etas/package.etasmeta",
                    request.dependency_name
                ),
                Some(artifact_path),
            )]);
        }
        let package_index = vendor::read_package_metadata_artifact_required(&unpack_root)
            .map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.github_release.metadata_unreadable",
                    format!(
                        "GitHub release dependency `{}` metadata artifact could not be read: {error}",
                        request.dependency_name
                    ),
                    Some(vendor::package_metadata_artifact_path(&unpack_root)),
                )]
            })?;
        let payload_checksum =
            vendor::path_package_payload_checksum(&unpack_root).map_err(|error| {
                vec![PackageDiagnostic::new(
                    "package.github_release.payload_unreadable",
                    format!(
                        "GitHub release dependency `{}` payload could not be checksummed: {error}",
                        request.dependency_name
                    ),
                    Some(unpack_root.clone()),
                )]
            })?;
        let store_path =
            store::store_package_payload(request.package_root, &unpack_root, &payload_checksum)
                .map_err(|error| {
                    vec![PackageDiagnostic::new(
                        "package.github_release.store_write_failed",
                        format!(
                            "GitHub release dependency `{}` could not be stored: {error}",
                            request.dependency_name
                        ),
                        Some(request.package_root.join(".etas").join("store")),
                    )]
                })?;
        Ok(Some(ResolvedSourcePackage {
            package_root: unpack_root,
            package_index,
            source: SourceCandidate::GitHubRelease {
                repo: repo.clone(),
                release: release.clone(),
                asset: asset.to_owned(),
                asset_checksum: expected_asset_checksum.to_owned(),
                payload_checksum,
                store: store_path,
            },
        }))
    }
}

fn github_release_url(base: Option<&str>, repo: &str, release: &str, asset: &str) -> String {
    let base = base.unwrap_or("https://github.com").trim_end_matches('/');
    format!(
        "{base}/{repo}/releases/download/{}/{}",
        encode_path_segment(release),
        encode_path_segment(asset)
    )
}

fn encode_path_segment(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                output.push(char::from(byte));
            }
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

fn unique_name(dependency_name: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!(
        ".download-{}-{}-{stamp}",
        dependency_name,
        std::process::id()
    )
}

fn file_checksum(path: &PathBuf) -> Result<String, PackageDiagnostic> {
    let bytes = fs::read(path).map_err(|source| {
        PackageDiagnostic::new(
            "package.github_release.checksum_unreadable",
            format!("downloaded release asset could not be read: {source}"),
            Some(path.clone()),
        )
    })?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}
