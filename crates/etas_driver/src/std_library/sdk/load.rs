use std::path::{Component, Path, PathBuf};

use etas_frontend::library_hir::LibraryHirImage;
use etas_package_metadata::{
    ARTIFACT_SCHEMA_VERSION, MetadataArtifactError, MetadataSectionKind,
    package_metadata_from_package_graph_payload, validate_artifact_schema,
};

use super::{
    EFFECT_ANALYSIS_PATH, PREPARED_IMAGE_PATH, STD_SDK_SCHEMA, STD_TOOLCHAIN_ID, StdSdkManifest,
    invalid, read_file,
};
use crate::DriverError;

#[derive(Clone, Debug)]
pub struct LoadedStdSdk {
    pub root: PathBuf,
    pub manifest: StdSdkManifest,
    pub public: etas_package_metadata::PackageMetadata,
    pub image: LibraryHirImage,
}

impl LoadedStdSdk {
    pub(crate) fn load_effect_analysis(&self, expected_hash: &str) -> Result<Vec<u8>, DriverError> {
        let path = self.root.join(EFFECT_ANALYSIS_PATH);
        let bytes = read_file(&path)?;
        matches(
            &path,
            "Effect analysis digest",
            etas_package_metadata::blake3_hash(&bytes).as_str(),
            expected_hash,
        )?;
        let mut archive =
            etas_package_metadata::decode_metadata_artifact(&path, &bytes).map_err(invalid)?;
        archive
            .sections
            .remove(&MetadataSectionKind::LibraryBodies)
            .ok_or_else(|| {
                invalid(format!(
                    "{}: installed SDK has no solved Effect analysis",
                    path.display()
                ))
            })
    }
}

/// Load a complete installed std with no handwritten registry construction or
/// per-symbol fallback. Checked-program publication follows in the frontend.
pub fn load_standard_library_sdk(sysroot: &Path) -> Result<LoadedStdSdk, DriverError> {
    let root = sysroot.join("std");
    let manifest_path = root.join("manifest.json");
    let manifest: StdSdkManifest = serde_json::from_slice(&read_file(&manifest_path)?)
        .map_err(|error| invalid(format!("{}: {error}", manifest_path.display())))?;
    matches(
        &manifest_path,
        "SDK schema",
        &manifest.schema,
        &STD_SDK_SCHEMA,
    )?;
    matches(
        &manifest_path,
        "metadata schema",
        &manifest.metadata_schema,
        &ARTIFACT_SCHEMA_VERSION,
    )?;
    matches(
        &manifest_path,
        "compiler",
        manifest.compiler_version.as_str(),
        etas_frontend::FRONTEND_COMPILER_VERSION,
    )?;
    matches(
        &manifest_path,
        "toolchain",
        manifest.toolchain_id.as_str(),
        STD_TOOLCHAIN_ID,
    )?;
    matches(
        &manifest_path,
        "target",
        manifest.target.as_str(),
        env!("ETAS_STD_TARGET"),
    )?;
    matches(
        &manifest_path,
        "binding ABI",
        manifest.native_binding_abi.as_str(),
        env!("ETAS_STD_NATIVE_ABI"),
    )?;
    matches(
        &manifest_path,
        "std contract",
        manifest.std_contract.as_str(),
        env!("ETAS_STD_CONTRACT"),
    )?;
    matches(&manifest_path, "package", manifest.package.as_str(), "std")?;
    let package = etas_package::read_manifest(&root)?;
    matches(
        &manifest_path,
        "package version",
        manifest.version.as_str(),
        package.package.version.as_str(),
    )?;
    matches(
        &manifest_path,
        "edition",
        manifest.edition.as_str(),
        package.package.edition.as_str(),
    )?;
    matches(
        &manifest_path,
        "source digest",
        &manifest.source_payload_hash,
        &etas_package::package_source_payload_hash(&root)?,
    )?;
    matches(
        &manifest_path,
        "manifest digest",
        &manifest.manifest_hash,
        &etas_package::package_manifest_hash(&root)?,
    )?;
    matches(
        &manifest_path,
        "lock digest",
        &manifest.dependency_lock_hash,
        &etas_package::package_dependency_lock_hash(&root)?,
    )?;
    let artifact_path = relative_file(&root, &manifest.artifact)?;
    let bytes = read_file(&artifact_path)?;
    matches(
        &artifact_path,
        "artifact digest",
        &etas_package_metadata::blake3_hash(&bytes),
        &manifest.artifact_hash,
    )?;
    let prepared;
    let archive = if let Some(expected_hash) = &manifest.prepared_image_hash {
        let path = root.join(PREPARED_IMAGE_PATH);
        prepared = read_file(&path)?;
        matches(
            &path,
            "prepared image digest",
            &etas_package_metadata::blake3_hash(&prepared),
            expected_hash,
        )?;
        &prepared
    } else {
        &bytes
    };
    let decoded = etas_package_metadata::decode_metadata_artifact(&artifact_path, archive)
        .map_err(invalid)?;
    let header = &decoded.header;
    matches(
        &artifact_path,
        "package",
        header.package_id.as_str(),
        manifest.package.as_str(),
    )?;
    matches(
        &artifact_path,
        "version",
        header.package_version.as_str(),
        manifest.version.as_str(),
    )?;
    matches(
        &artifact_path,
        "compiler",
        header.compiler_version.as_str(),
        manifest.compiler_version.as_str(),
    )?;
    matches(
        &artifact_path,
        "source digest",
        &header.source_payload_hash,
        &manifest.source_payload_hash,
    )?;
    matches(
        &artifact_path,
        "manifest digest",
        &header.manifest_hash,
        &manifest.manifest_hash,
    )?;
    matches(
        &artifact_path,
        "lock digest",
        &header.dependency_lock_hash,
        &manifest.dependency_lock_hash,
    )?;
    let body = decoded
        .sections
        .get(&MetadataSectionKind::LibraryBodies)
        .ok_or_else(|| {
            invalid(format!(
                "{}: std artifact is missing checked library bodies",
                artifact_path.display()
            ))
        })?;
    let image = if manifest.prepared_image_hash.is_some() {
        LibraryHirImage::decode_prepared(body)
    } else {
        LibraryHirImage::decode(body)
    }
    .map_err(invalid)?;
    matches(
        &artifact_path,
        "body schema",
        &image.schema,
        &manifest.body_schema,
    )?;
    let owner = etas_code::PackageKey::from_name_and_version(&manifest.package, &manifest.version)
        .map_err(invalid)?;
    if image.package != owner {
        return Err(invalid(format!(
            "{}: checked bodies have a different package owner",
            artifact_path.display()
        )));
    }
    matches(
        &artifact_path,
        "body std contract",
        &image.std_contract,
        &manifest.std_contract,
    )?;
    image
        .native_registry
        .clone()
        .into_registry()
        .map_err(invalid)?;
    let body_sources = image
        .sources
        .iter()
        .map(|source| {
            (
                source.relative_path.clone(),
                etas_package_metadata::blake3_hash(source.text.as_bytes()),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    if body_sources != manifest.sources {
        return Err(invalid(format!(
            "{}: checked body sources differ from the SDK source index",
            artifact_path.display()
        )));
    }
    for (relative, expected) in &manifest.sources {
        let path = relative_file(&root, relative)?;
        matches(
            &path,
            "source digest",
            &etas_package_metadata::blake3_hash(&read_file(&path)?),
            expected,
        )?;
    }
    validate_artifact_schema(header).map_err(invalid)?;
    let graph = decoded
        .sections
        .get(&MetadataSectionKind::PackageGraph)
        .ok_or_else(|| {
            MetadataArtifactError::invalid(
                &artifact_path,
                "package metadata artifact is missing package_graph section",
            )
        })
        .map_err(invalid)?;
    let public = package_metadata_from_package_graph_payload(graph).map_err(invalid)?;
    Ok(LoadedStdSdk {
        root,
        manifest,
        public,
        image,
    })
}

fn relative_file(root: &Path, path: &Path) -> Result<PathBuf, DriverError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(format!(
            "{}: SDK file must be relative to {}",
            path.display(),
            root.display()
        )));
    }
    Ok(root.join(path))
}

fn matches<T: PartialEq + std::fmt::Display + ?Sized>(
    path: &Path,
    field: &str,
    actual: &T,
    expected: &T,
) -> Result<(), DriverError> {
    if actual == expected {
        return Ok(());
    }
    Err(invalid(format!(
        "{}: incompatible std {field}: expected `{expected}`, got `{actual}`",
        path.display()
    )))
}
