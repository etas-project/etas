//! Installed std producer. The public package projection and checked private
//! body graph share the normal ETASMETA container.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use etas_frontend::{PackageMetadataBuildInput, library_hir::LibraryHirImage};
use etas_package_metadata::{EncodedMetadataSection, MetadataSectionKind};
use serde::{Deserialize, Serialize};

use super::{StdLibraryCheckRequest, check_standard_library_source};
use crate::{DriverError, DriverOptions};

mod load;
pub use load::{LoadedStdSdk, load_standard_library_sdk};

pub const STD_SDK_SCHEMA: u32 = 1;
pub const STD_TOOLCHAIN_ID: &str = env!("ETAS_TOOLCHAIN_ID");
const ARTIFACT_PATH: &str = "artifacts/std.etasmeta";

#[derive(Clone, Debug)]
pub struct StdLibraryBuildRequest {
    pub manifest_path: PathBuf,
    /// The std directory inside a sysroot, e.g. /tmp/sdk/std.
    pub out_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct StdLibraryBuildOutput {
    pub out_dir: PathBuf,
    pub manifest: StdSdkManifest,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StdSdkManifest {
    pub schema: u32,
    pub package: String,
    pub version: String,
    pub edition: String,
    pub compiler_version: String,
    pub body_schema: u32,
    pub metadata_schema: u32,
    pub toolchain_id: String,
    pub native_binding_abi: String,
    pub std_contract: String,
    pub target: String,
    pub artifact: PathBuf,
    pub artifact_hash: String,
    pub source_payload_hash: String,
    pub manifest_hash: String,
    pub dependency_lock_hash: String,
    /// Paths are relative to the installed std directory.
    pub sources: BTreeMap<PathBuf, String>,
}

pub fn build_standard_library(
    options: &DriverOptions,
    request: StdLibraryBuildRequest,
) -> Result<StdLibraryBuildOutput, DriverError> {
    let output = check_standard_library_source(
        options,
        StdLibraryCheckRequest {
            manifest_path: request.manifest_path,
        },
    )?;
    let checked = output.response.output.checked.as_ref().ok_or_else(|| {
        invalid(format!(
            "std source did not pass checking: {}",
            output
                .response
                .output
                .diagnostics
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ))
    })?;
    let image =
        LibraryHirImage::from_checked(checked, output.definition_origin, &output.package_root)
            .map_err(invalid)?;
    let prepared = etas_package::prepare_package(etas_package::PreparePackageOptions {
        package_root: output.package_root.clone(),
        selected_bin: None,
    })?;
    let source_payload_hash = etas_package::package_source_payload_hash(&output.package_root)?;
    let manifest_hash = etas_package::package_manifest_hash(&output.package_root)?;
    let dependency_lock_hash = etas_package::package_dependency_lock_hash(&output.package_root)?;
    let public = etas_frontend::build_package_metadata_artifact_from_checked(
        PackageMetadataBuildInput {
            definition_origin: Some(output.definition_origin),
            package_id: output.manifest.package.name.clone(),
            package_version: output.manifest.package.version.clone(),
            package_edition: output.manifest.package.edition.clone(),
            compiler_version: image.compiler_version.clone(),
            source_payload_hash: source_payload_hash.clone(),
            manifest_hash: manifest_hash.clone(),
            dependency_lock_hash: dependency_lock_hash.clone(),
            bins: vec![],
            dependencies: prepared
                .environment
                .dependencies
                .iter()
                .map(etas_package::resolved_dependency_metadata)
                .collect::<Result<Vec<_>, _>>()?,
            tool_bindings: output
                .manifest
                .bindings
                .tools
                .iter()
                .map(
                    |(tool, binding)| etas_frontend::PackageMetadataToolBindingInput {
                        tool: tool.clone(),
                        kind: binding.kind.clone(),
                        provider: binding.provider_name().unwrap_or_default(),
                        effect_row: binding.effects.clone(),
                        action_row: binding.actions.clone(),
                    },
                )
                .collect(),
        },
        checked,
    )
    .map_err(invalid)?;
    let decoded =
        etas_package_metadata::decode_metadata_artifact(Path::new(ARTIFACT_PATH), &public.bytes)
            .map_err(invalid)?;
    let mut sections = decoded
        .sections
        .into_iter()
        .map(|(kind, payload)| EncodedMetadataSection { kind, payload })
        .collect::<Vec<_>>();
    sections.push(EncodedMetadataSection {
        kind: MetadataSectionKind::LibraryBodies,
        payload: image.encode().map_err(invalid)?,
    });
    let bytes = etas_package_metadata::encode_metadata_artifact(&decoded.header, sections)
        .map_err(invalid)?;
    let manifest = StdSdkManifest {
        schema: STD_SDK_SCHEMA,
        package: output.manifest.package.name,
        version: output.manifest.package.version,
        edition: output.manifest.package.edition,
        compiler_version: image.compiler_version.clone(),
        body_schema: image.schema,
        metadata_schema: decoded.header.artifact_schema_version,
        native_binding_abi: super::native_abi::fingerprint(&checked.std_registry)
            .map_err(invalid)?,
        std_contract: image.std_contract.clone(),
        target: env!("ETAS_STD_TARGET").into(),
        toolchain_id: STD_TOOLCHAIN_ID.into(),
        artifact: ARTIFACT_PATH.into(),
        artifact_hash: etas_package_metadata::blake3_hash(&bytes),
        source_payload_hash,
        manifest_hash,
        dependency_lock_hash,
        sources: image
            .sources
            .iter()
            .map(|source| {
                (
                    source.relative_path.clone(),
                    etas_package_metadata::blake3_hash(source.text.as_bytes()),
                )
            })
            .collect(),
    };
    let out_dir = request.out_dir;
    create_dir(&out_dir)?;
    let out_dir = out_dir.canonicalize().map_err(|source| DriverError::Io {
        path: out_dir,
        source,
    })?;
    let source_root = output.package_root.join(&output.manifest.source.root);
    if out_dir.starts_with(&source_root) || source_root.starts_with(&out_dir) {
        return Err(invalid(
            "std output directory must be separate from its maintained source tree",
        ));
    }
    let installed_sources = out_dir.join(&output.manifest.source.root);
    if installed_sources.exists() {
        fs::remove_dir_all(&installed_sources).map_err(|source| DriverError::Io {
            path: installed_sources.clone(),
            source,
        })?;
    }
    copy_tree(&source_root, &installed_sources)?;
    let package_manifest = output.package_root.join("etas.toml");
    write_file(&out_dir.join("etas.toml"), &read_file(&package_manifest)?)?;
    let lock_path = output.package_root.join("etas.lock");
    if lock_path.exists() {
        write_file(&out_dir.join("etas.lock"), &read_file(&lock_path)?)?;
    } else if out_dir.join("etas.lock").exists() {
        fs::remove_file(out_dir.join("etas.lock")).map_err(|source| DriverError::Io {
            path: out_dir.join("etas.lock"),
            source,
        })?;
    }
    write_file(&out_dir.join(ARTIFACT_PATH), &bytes)?;
    write_file(
        &out_dir.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest).map_err(invalid)?,
    )?;
    Ok(StdLibraryBuildOutput { out_dir, manifest })
}

fn invalid(error: impl std::fmt::Display) -> DriverError {
    DriverError::InvalidInput(error.to_string())
}

fn create_dir(path: &Path) -> Result<(), DriverError> {
    fs::create_dir_all(path).map_err(|source| DriverError::Io {
        path: path.to_owned(),
        source,
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), DriverError> {
    create_dir(path.parent().expect("installed file has a parent"))?;
    fs::write(path, bytes).map_err(|source| DriverError::Io {
        path: path.to_owned(),
        source,
    })
}

fn read_file(path: &Path) -> Result<Vec<u8>, DriverError> {
    fs::read(path).map_err(|source| DriverError::Io {
        path: path.to_owned(),
        source,
    })
}

fn copy_tree(source_root: &Path, destination: &Path) -> Result<(), DriverError> {
    create_dir(destination)?;
    let entries = fs::read_dir(source_root).map_err(|source| DriverError::Io {
        path: source_root.to_owned(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DriverError::Io {
            path: source_root.to_owned(),
            source,
        })?;
        let source = entry.path();
        let target = destination.join(entry.file_name());
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            write_file(&target, &read_file(&source)?)?;
        }
    }
    Ok(())
}
