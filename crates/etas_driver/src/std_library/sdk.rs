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

mod behaviors;
mod delta;
mod load;
pub use load::{LoadedStdSdk, load_standard_library_sdk};

pub const STD_SDK_SCHEMA: u32 = 1;
pub const STD_TOOLCHAIN_ID: &str = env!("ETAS_TOOLCHAIN_ID");
const ARTIFACT_PATH: &str = "artifacts/std.etasmeta";
const EFFECT_ANALYSIS_PATH: &str = "artifacts/std.effects.etasmeta";
const PREPARED_QUERIES_PATH: &str = "artifacts/std.queries";
const INTERPRETER_LAYOUT_PATH: &str = "artifacts/std.runtime";
const PREPARED_IMAGE_PATH: &str = "artifacts/std.prepared.etasmeta";

pub fn installed_sysroot(prefix: &Path) -> PathBuf {
    prefix.join("lib/etas").join(STD_TOOLCHAIN_ID)
}

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
    pub installed_program: Option<etas_frontend::CheckedProgram>,
    pub installed_image: Option<LibraryHirImage>,
    pub execution_slices: Vec<etas_frontend::library_hir::LibrarySlicePlan>,
    pub bundles: Vec<StdModuleBundle>,
    bundle_store: delta::BundleStore,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_analysis_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_image_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpreter_layout_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_queries_hash: Option<String>,
    pub source_payload_hash: String,
    pub manifest_hash: String,
    pub dependency_lock_hash: String,
    /// Paths are relative to the installed std directory.
    pub sources: BTreeMap<PathBuf, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PreparedStdQueries {
    pub registry: Option<etas_std::StdRegistryImage>,
    pub interface: std::sync::Arc<etas_frontend::StandardLibraryInterface>,
    pub signatures: StdBundleFile,
    pub behaviors: Vec<behaviors::BehaviorEntry>,
    pub bootstrap_identity: String,
    #[serde(with = "callable_index")]
    pub callables: BTreeMap<etas_code::CallableId, u32>,
    pub bundles: Vec<StdModuleIndexEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StdModuleIndexEntry {
    pub modules: std::collections::BTreeSet<Vec<String>>,
    pub callables: std::collections::BTreeSet<u32>,
    pub image: StdBundleFile,
    pub publication: StdBundleFile,
    pub layouts: Option<StdBundleFile>,
    pub image_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StdBundleFile {
    pub path: PathBuf,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<Box<StdBundleFile>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StdModuleBundle {
    pub modules: std::collections::BTreeSet<Vec<String>>,
    pub callables: std::collections::BTreeSet<etas_code::CallableId>,
    pub image: StdBundleFile,
    pub publication: StdBundleFile,
    pub layouts: Option<StdBundleFile>,
    pub image_size: usize,
}

pub(crate) fn decode_queries(bytes: &[u8]) -> Result<PreparedStdQueries, DriverError> {
    let payload = bytes
        .strip_prefix(b"ETASQRY6")
        .ok_or_else(|| invalid("incompatible installed std queries"))?;
    serde_json::from_slice(payload).map_err(invalid)
}

mod callable_index {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        index: &BTreeMap<etas_code::CallableId, u32>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        index.iter().collect::<Vec<_>>().serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<etas_code::CallableId, u32>, D::Error> {
        Ok(
            Vec::<(etas_code::CallableId, u32)>::deserialize(deserializer)?
                .into_iter()
                .collect(),
        )
    }
}

impl StdLibraryBuildOutput {
    pub fn publish_queries(
        &mut self,
        provider: &super::provider::ArtifactStdProvider,
    ) -> Result<(), DriverError> {
        provider
            .interface
            .validate_for_install(&provider.registry)
            .map_err(invalid)?;
        let mut callables = BTreeMap::new();
        let bundles = self
            .bundles
            .iter()
            .map(|bundle| {
                let indices = bundle
                    .callables
                    .iter()
                    .map(|callable| {
                        let next = callables.len() as u32;
                        *callables.entry(callable.clone()).or_insert(next)
                    })
                    .collect();
                StdModuleIndexEntry {
                    modules: bundle.modules.clone(),
                    callables: indices,
                    image: bundle.image.clone(),
                    publication: bundle.publication.clone(),
                    layouts: bundle.layouts.clone(),
                    image_size: bundle.image_size,
                }
            })
            .collect();
        let (interface, signatures, behaviors) = behaviors::publish(self, provider)?;
        let queries = PreparedStdQueries {
            signatures,
            behaviors,
            registry: Some(
                etas_std::StdRegistryImage::capture(&provider.registry).map_err(invalid)?,
            ),
            interface,
            bootstrap_identity: provider.bootstrap_identity.clone(),
            callables,
            bundles,
        };
        let mut payload = b"ETASQRY6".to_vec();
        serde_json::to_writer(&mut payload, &queries).map_err(invalid)?;
        decode_queries(&payload)?;
        write_file(&self.out_dir.join(PREPARED_QUERIES_PATH), &payload)?;
        self.manifest.prepared_queries_hash = Some(etas_package_metadata::blake3_hash(&payload));
        let bytes = serde_json::to_vec_pretty(&self.manifest).map_err(invalid)?;
        write_file(&self.out_dir.join("manifest.json"), &bytes)
    }

    pub fn publish_bundle(
        &mut self,
        modules: std::collections::BTreeSet<Vec<String>>,
        image: &LibraryHirImage,
        publication: &[u8],
        layouts: Option<&[u8]>,
    ) -> Result<(), DriverError> {
        let base = PathBuf::from(format!("artifacts/bundles/{}", self.bundles.len()));
        let payload = image.encode_prepared().map_err(invalid)?;
        let mut write = |extension: &str, bytes: &[u8]| {
            self.bundle_store.write(
                &self.out_dir,
                base.with_extension(extension),
                extension,
                bytes,
            )
        };
        let image_file = write("body", &payload)?;
        let publication = write("publication", publication)?;
        let layouts = layouts.map(|bytes| write("runtime", bytes)).transpose()?;
        let callables = image
            .declarations
            .iter()
            .filter(|(_, item)| match &image.program.items[**item] {
                etas_hir::HirItem::Flow(flow) => flow.body.as_block().is_some() || matches!(flow.body, etas_hir::HirFlowBody::Runtime { .. }),
                etas_hir::HirItem::Tool(tool) => matches!(&tool.body, etas_hir::HirToolBody::Source(body) if body.as_block().is_some()),
                _ => false,
            })
            .map(|(key, _)| etas_code::CallableId::declaration(key.clone()).map_err(invalid))
            .chain(
                image
                    .methods
                    .iter()
                    .filter(|(_, item)| matches!(&image.program.items[**item], etas_hir::HirItem::Flow(flow) if flow.body.as_block().is_some() || matches!(flow.body, etas_hir::HirFlowBody::Runtime { .. })))
                    .map(|(method, _)| Ok(etas_code::CallableId::method(method.clone()))),
            )
            .collect::<Result<_, DriverError>>()?;
        self.bundles.push(StdModuleBundle {
            modules,
            callables,
            image: image_file,
            publication,
            layouts,
            image_size: payload.len(),
        });
        Ok(())
    }

    pub fn publish_runtime_layouts(&mut self, payload: &[u8]) -> Result<(), DriverError> {
        write_file(&self.out_dir.join(INTERPRETER_LAYOUT_PATH), payload)?;
        self.manifest.interpreter_layout_hash = Some(etas_package_metadata::blake3_hash(payload));
        let bytes = serde_json::to_vec_pretty(&self.manifest).map_err(invalid)?;
        write_file(&self.out_dir.join("manifest.json"), &bytes)
    }
}

impl StdSdkManifest {
    pub fn identity(&self) -> Result<String, DriverError> {
        let bytes = serde_json::to_vec(self).map_err(invalid)?;
        Ok(etas_package_metadata::blake3_hash(&bytes))
    }
}

pub fn build_standard_library(
    options: &DriverOptions,
    request: StdLibraryBuildRequest,
) -> Result<StdLibraryBuildOutput, DriverError> {
    build_sdk(options, request, false)
}

pub fn build_installed_standard_library(
    options: &DriverOptions,
    request: StdLibraryBuildRequest,
) -> Result<StdLibraryBuildOutput, DriverError> {
    build_sdk(options, request, cfg!(target_os = "linux"))
}

fn build_sdk(
    options: &DriverOptions,
    request: StdLibraryBuildRequest,
    capture_effect_analysis: bool,
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
                .map(|d| {
                    let source = output.response.output.sources.as_ref().and_then(|sources| {
                        sources
                            .files
                            .iter()
                            .find(|source| source.id == d.primary.span.source)
                    });
                    match source {
                        Some(source) => {
                            let position = source.line_index.line_col(d.primary.span.range.start);
                            format!(
                                "{}:{}:{}: {}",
                                source
                                    .path
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_else(|| "<std source>".into()),
                                position.line + 1,
                                position.col + 1,
                                d.message
                            )
                        }
                        None => d.message.clone(),
                    }
                })
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
    let source_root = output.package_root.join(&output.manifest.source.root);
    let mut assets = BTreeMap::new();
    read_source_assets(&output.package_root, &source_root, &mut assets)?;
    // The body and source index must describe the bytes that actually passed
    // checking, including when the maintained tree changes during the build.
    let source_payload_hash = etas_package_metadata::source_payload_checksum_for_entries(
        image
            .sources
            .iter()
            .map(|source| (source.relative_path.as_path(), source.text.as_bytes()))
            .chain(
                assets
                    .iter()
                    .map(|(path, bytes)| (path.as_path(), bytes.as_slice())),
            ),
    );
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
    let prepared_image = if capture_effect_analysis {
        let payload = image.encode_prepared().map_err(invalid)?;
        // Validate the native codec while preparing the installation.
        LibraryHirImage::decode_prepared(&payload).map_err(invalid)?;
        let mut prepared = sections.clone();
        prepared.push(EncodedMetadataSection {
            kind: MetadataSectionKind::LibraryBodies,
            payload,
        });
        Some(
            etas_package_metadata::encode_metadata_artifact(&decoded.header, prepared)
                .map_err(invalid)?,
        )
    } else {
        None
    };
    sections.push(EncodedMetadataSection {
        kind: MetadataSectionKind::LibraryBodies,
        payload: image.encode().map_err(invalid)?,
    });
    let bytes = etas_package_metadata::encode_metadata_artifact(&decoded.header, sections)
        .map_err(invalid)?;
    let effect_analysis = if capture_effect_analysis {
        let _capture = options
            .profile
            .span("std.prepare_installed_publication", "std");
        let payload = image
            .prepare_installed_publication(checked, &output.package_root, &options.profile)
            .map_err(invalid)?;
        Some(
            etas_package_metadata::encode_metadata_artifact(
                &decoded.header,
                vec![EncodedMetadataSection {
                    kind: MetadataSectionKind::LibraryBodies,
                    payload,
                }],
            )
            .map_err(invalid)?,
        )
    } else {
        None
    };
    let manifest = StdSdkManifest {
        schema: STD_SDK_SCHEMA,
        package: output.manifest.package.name,
        version: output.manifest.package.version,
        edition: output.manifest.package.edition,
        compiler_version: image.compiler_version.clone(),
        body_schema: image.schema,
        metadata_schema: decoded.header.artifact_schema_version,
        native_binding_abi: super::native_abi::fingerprint(),
        std_contract: image.std_contract.clone(),
        target: env!("ETAS_STD_TARGET").into(),
        toolchain_id: STD_TOOLCHAIN_ID.into(),
        artifact: ARTIFACT_PATH.into(),
        artifact_hash: etas_package_metadata::blake3_hash(&bytes),
        effect_analysis_hash: effect_analysis
            .as_deref()
            .map(etas_package_metadata::blake3_hash),
        prepared_image_hash: prepared_image
            .as_deref()
            .map(etas_package_metadata::blake3_hash),
        interpreter_layout_hash: None,
        prepared_queries_hash: None,
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
    for source in &image.sources {
        write_file(&out_dir.join(&source.relative_path), source.text.as_bytes())?;
    }
    for (path, bytes) in assets {
        write_file(&out_dir.join(path), &bytes)?;
    }
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
    if let Some(prepared_image) = prepared_image {
        write_file(&out_dir.join(PREPARED_IMAGE_PATH), &prepared_image)?;
    }
    if let Some(effect_analysis) = effect_analysis {
        write_file(&out_dir.join(EFFECT_ANALYSIS_PATH), &effect_analysis)?;
    }
    write_file(
        &out_dir.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest).map_err(invalid)?,
    )?;
    Ok(StdLibraryBuildOutput {
        out_dir,
        manifest,
        installed_program: capture_effect_analysis.then(|| checked.clone()),
        installed_image: capture_effect_analysis.then(|| image.clone()),
        execution_slices: if capture_effect_analysis {
            image.execution_slices(checked)
        } else {
            Vec::new()
        },
        bundles: Vec::new(),
        bundle_store: delta::BundleStore::default(),
    })
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

fn read_source_assets(
    package: &Path,
    directory: &Path,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), DriverError> {
    let io = |source| DriverError::Io {
        path: directory.to_owned(),
        source,
    };
    for entry in fs::read_dir(directory).map_err(io)? {
        let entry = entry.map_err(io)?;
        let kind = entry.file_type().map_err(io)?;
        let path = entry.path();
        if kind.is_dir() {
            read_source_assets(package, &path, files)?;
        } else if kind.is_file() && path.extension() != Some(std::ffi::OsStr::new("es")) {
            let relative = path
                .strip_prefix(package)
                .expect("source asset belongs to package");
            files.insert(relative.to_owned(), read_file(&path)?);
        }
    }
    Ok(())
}
