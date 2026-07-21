use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{
    PackageError,
    file_lock::PackageFileLock,
    manifest,
    metadata::{PackageIndex, ResolvedDependency, ResolvedDependencySource},
    metadata_artifact, store,
};

pub fn package_index_path(package_root: &Path) -> PathBuf {
    package_root.join(".etas").join("package-index.json")
}

pub fn package_metadata_artifact_path(package_root: &Path) -> PathBuf {
    metadata_artifact::package_metadata_artifact_path(package_root)
}

pub fn vendor_root(package_root: &Path) -> PathBuf {
    package_root.join(".etas").join("vendor")
}

pub fn vendored_dependency_root(package_root: &Path, import_root: &str) -> PathBuf {
    import_root
        .split('.')
        .filter(|segment| !segment.is_empty())
        .fold(vendor_root(package_root), |path, segment| {
            path.join(segment)
        })
}

pub fn vendored_dependency_relative_path(import_root: &str) -> String {
    let mut path = PathBuf::from(".etas").join("vendor");
    for segment in import_root.split('.').filter(|segment| !segment.is_empty()) {
        path = path.join(segment);
    }
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendoredDependencyPayload {
    pub relative_path: String,
    pub checksum: String,
    pub store_path: String,
}

pub fn write_vendored_path_dependency(
    package_root: &Path,
    import_root: &str,
    source_package_root: &Path,
) -> Result<VendoredDependencyPayload, PackageError> {
    let destination = vendored_dependency_root(package_root, import_root);
    let source_manifest = manifest::read_manifest(source_package_root)?;
    let source_root = source_package_root.join(&source_manifest.source.root);
    if !source_root.is_dir() {
        return Err(PackageError::Manifest {
            path: source_root,
            message: "source root for vendored dependency does not exist".to_owned(),
        });
    }
    if destination.starts_with(source_package_root) {
        return Err(PackageError::Manifest {
            path: destination,
            message: "vendored dependency destination cannot be inside the source package"
                .to_owned(),
        });
    }

    let vendor_root = vendor_root(package_root);
    let _lock = acquire_vendor_lock(package_root)?;
    let temp = unique_temp_path(&vendor_root, "vendor-write");
    if temp.exists() {
        fs::remove_dir_all(&temp).map_err(|source| PackageError::Io {
            path: temp.clone(),
            source,
        })?;
    }
    fs::create_dir_all(&temp).map_err(|source| PackageError::Io {
        path: temp.clone(),
        source,
    })?;

    copy_file(
        &source_package_root.join("etas.toml"),
        &temp.join("etas.toml"),
    )?;
    copy_directory(&source_root, &temp.join(&source_manifest.source.root))?;
    let source_index = package_index_path(source_package_root);
    if source_index.is_file() {
        copy_file(&source_index, &package_index_path(&temp))?;
    }
    let source_artifact = package_metadata_artifact_path(source_package_root);
    if source_artifact.is_file() {
        copy_file(&source_artifact, &package_metadata_artifact_path(&temp))?;
    }

    let source_checksum = path_package_payload_checksum(source_package_root)?;
    let vendored_checksum = path_package_payload_checksum(&temp)?;
    if source_checksum != vendored_checksum {
        let _ = fs::remove_dir_all(&temp);
        return Err(PackageError::Manifest {
            path: temp,
            message: format!(
                "vendored dependency checksum mismatch: source payload is `{source_checksum}` but vendor payload is `{vendored_checksum}`"
            ),
        });
    }
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|source| PackageError::Io {
            path: destination.clone(),
            source,
        })?;
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source| PackageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::rename(&temp, &destination).map_err(|source| PackageError::Io {
        path: destination.clone(),
        source,
    })?;

    Ok(VendoredDependencyPayload {
        relative_path: vendored_dependency_relative_path(import_root),
        store_path: store::store_package_payload(package_root, &destination, &vendored_checksum)?,
        checksum: vendored_checksum,
    })
}

pub(crate) fn with_vendor_lock<T>(
    package_root: &Path,
    f: impl FnOnce() -> Result<T, PackageError>,
) -> Result<T, PackageError> {
    let _lock = acquire_vendor_lock(package_root)?;
    f()
}

fn acquire_vendor_lock(package_root: &Path) -> Result<PackageFileLock, PackageError> {
    PackageFileLock::acquire(vendor_root(package_root).join(".lock"))
}

fn unique_temp_path(parent: &Path, prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    parent.join(format!(".{prefix}-{}-{stamp}", std::process::id()))
}

pub fn read_package_index(package_root: &Path) -> Result<Option<PackageIndex>, PackageError> {
    let path = package_index_path(package_root);
    if !path.exists() {
        return Ok(None);
    }
    let _lock = PackageFileLock::acquire(package_root.join(".etas").join("package-index.lock"))?;
    let text = std::fs::read_to_string(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|source| PackageError::Manifest {
            path,
            message: format!("package index is not valid JSON: {source}"),
        })
}

pub fn read_package_metadata_artifact_required(
    package_root: &Path,
) -> Result<PackageIndex, PackageError> {
    metadata_artifact::read_package_metadata_artifact(package_root)?.ok_or_else(|| {
        PackageError::Manifest {
            path: package_metadata_artifact_path(package_root),
            message: "package metadata artifact `.etas/package.etasmeta` is missing".to_owned(),
        }
    })
}

pub fn write_package_index(package_root: &Path, index: &PackageIndex) -> Result<(), PackageError> {
    let path = package_index_path(package_root);
    let _lock = PackageFileLock::acquire(package_root.join(".etas").join("package-index.lock"))?;
    let parent = path.parent().ok_or_else(|| PackageError::Manifest {
        path: path.clone(),
        message: "package index path has no parent".to_owned(),
    })?;
    std::fs::create_dir_all(parent).map_err(|source| PackageError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let text = serde_json::to_string_pretty(index).expect("package index serializes");
    let temp = unique_temp_path(parent, "package-index-write").with_extension("json.tmp");
    let mut file = File::create(&temp).map_err(|source| PackageError::Io {
        path: temp.clone(),
        source,
    })?;
    file.write_all(text.as_bytes())
        .map_err(|source| PackageError::Io {
            path: temp.clone(),
            source,
        })?;
    file.sync_all().map_err(|source| PackageError::Io {
        path: temp.clone(),
        source,
    })?;
    drop(file);
    fs::rename(&temp, &path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    sync_directory(parent)?;
    Ok(())
}

pub fn dependency_content_checksum(dependency: &ResolvedDependency) -> String {
    let document = DependencyChecksumDocument {
        identity: &dependency.identity,
        import_root: &dependency.import_root,
        source: DependencyChecksumSource::from(&dependency.source),
        dependencies: &dependency.dependencies,
        public_metadata: &dependency.public_metadata,
        effect_metadata: &dependency.effect_metadata,
        tool_bindings: &dependency.tool_bindings,
    };
    let bytes = serde_json::to_vec(&document).expect("dependency checksum document serializes");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

pub fn dependency_lock_checksum(dependency: &ResolvedDependency) -> String {
    match &dependency.source {
        ResolvedDependencySource::Builtin { checksum } => checksum.clone(),
        _ => dependency_content_checksum(dependency),
    }
}

pub fn path_package_payload_checksum(package_root: &Path) -> Result<String, PackageError> {
    let manifest = manifest::read_manifest(package_root)?;
    let mut files = vec![package_root.join("etas.toml")];
    let package_index = package_index_path(package_root);
    if package_index.is_file() {
        files.push(package_index);
    }
    let metadata_artifact = package_metadata_artifact_path(package_root);
    if metadata_artifact.is_file() {
        files.push(metadata_artifact);
    }
    collect_payload_files(&package_root.join(&manifest.source.root), &mut files)?;
    files.sort();

    let mut hasher = blake3::Hasher::new();
    for file in files {
        let relative = file.strip_prefix(package_root).unwrap_or(&file);
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(&[0]);
        let bytes = std::fs::read(&file).map_err(|source| PackageError::Io {
            path: file.clone(),
            source,
        })?;
        hasher.update(&bytes);
        hasher.update(&[0]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn sync_directory(path: &Path) -> Result<(), PackageError> {
    let directory = File::open(path).map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    directory.sync_all().map_err(|source| PackageError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn collect_payload_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), PackageError> {
    if !root.exists() {
        return Ok(());
    }
    let entries = std::fs::read_dir(root).map_err(|source| PackageError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| PackageError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.is_dir() {
            collect_payload_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), PackageError> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|source| PackageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::copy(source, destination).map_err(|source_error| PackageError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), PackageError> {
    std::fs::create_dir_all(destination).map_err(|source_error| PackageError::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    let entries = std::fs::read_dir(source).map_err(|source_error| PackageError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source_error| PackageError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let path = entry.path();
        let target = destination.join(entry.file_name());
        let metadata = entry.metadata().map_err(|source_error| PackageError::Io {
            path: path.clone(),
            source: source_error,
        })?;
        if metadata.is_dir() {
            copy_directory(&path, &target)?;
        } else if metadata.is_file() {
            copy_file(&path, &target)?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct DependencyChecksumDocument<'a> {
    identity: &'a crate::PackageIdentity,
    import_root: &'a str,
    source: DependencyChecksumSource<'a>,
    dependencies: &'a [ResolvedDependency],
    public_metadata: &'a crate::PackagePublicMetadata,
    effect_metadata: &'a crate::PackageEffectMetadata,
    tool_bindings: &'a [crate::PackageToolBindingMetadata],
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DependencyChecksumSource<'a> {
    Builtin {
        payload_checksum: &'a str,
    },
    Registry {
        registry: &'a str,
        payload_checksum: &'a str,
        store: Option<&'a str>,
    },
    Git {
        url: &'a str,
        rev: &'a str,
        payload_checksum: &'a str,
        store: Option<&'a str>,
    },
    GitHubClone {
        repo: &'a str,
        rev: &'a str,
        payload_checksum: &'a str,
        store: Option<&'a str>,
    },
    GitHubRelease {
        repo: &'a str,
        release: &'a str,
        asset: &'a str,
        asset_checksum: &'a str,
        payload_checksum: &'a str,
        store: Option<&'a str>,
    },
    Path {
        path: &'a str,
        payload_checksum: &'a str,
    },
    Vendor {
        path: &'a str,
        payload_checksum: &'a str,
        store: Option<&'a str>,
    },
}

impl<'a> From<&'a ResolvedDependencySource> for DependencyChecksumSource<'a> {
    fn from(source: &'a ResolvedDependencySource) -> Self {
        match source {
            ResolvedDependencySource::Builtin { checksum } => Self::Builtin {
                payload_checksum: checksum,
            },
            ResolvedDependencySource::Registry {
                registry,
                checksum,
                store,
            } => Self::Registry {
                registry,
                payload_checksum: checksum,
                store: store.as_deref(),
            },
            ResolvedDependencySource::Git {
                url,
                rev,
                checksum,
                store,
            } => Self::Git {
                url,
                rev,
                payload_checksum: checksum,
                store: store.as_deref(),
            },
            ResolvedDependencySource::GitHubClone {
                repo,
                rev,
                checksum,
                store,
            } => Self::GitHubClone {
                repo,
                rev,
                payload_checksum: checksum,
                store: store.as_deref(),
            },
            ResolvedDependencySource::GitHubRelease {
                repo,
                release,
                asset,
                asset_checksum,
                payload_checksum,
                store,
            } => Self::GitHubRelease {
                repo,
                release,
                asset,
                asset_checksum,
                payload_checksum,
                store: store.as_deref(),
            },
            ResolvedDependencySource::Path { path, checksum } => Self::Path {
                path,
                payload_checksum: checksum,
            },
            ResolvedDependencySource::Vendor {
                path,
                checksum,
                store,
            } => Self::Vendor {
                path,
                payload_checksum: checksum,
                store: store.as_deref(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn write_vendored_path_dependency_is_serialized_under_concurrent_writes() {
        let root = temp_dir("vendor-concurrent-root");
        let dependency = temp_dir("vendor-concurrent-dep");
        write_payload_package(&dependency, "payload");

        let mut handles = Vec::new();
        for _ in 0..6 {
            let root = root.clone();
            let dependency = dependency.clone();
            handles.push(thread::spawn(move || {
                write_vendored_path_dependency(&root, "payload", &dependency)
            }));
        }

        let mut payloads = Vec::new();
        for handle in handles {
            payloads.push(handle.join().unwrap().unwrap());
        }
        assert!(
            payloads
                .iter()
                .all(|payload| payload.relative_path == payloads[0].relative_path
                    && payload.checksum == payloads[0].checksum
                    && payload.store_path == payloads[0].store_path)
        );
        let vendored = vendored_dependency_root(&root, "payload");
        assert_eq!(
            path_package_payload_checksum(&vendored).unwrap(),
            payloads[0].checksum
        );
        assert_eq!(
            path_package_payload_checksum(&root.join(&payloads[0].store_path)).unwrap(),
            payloads[0].checksum
        );
    }

    fn write_payload_package(root: &Path, module: &str) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "{module}"
version = "0.1.0"
edition = "2026"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("src").join("lib.es"),
            format!("module {module};\n"),
        )
        .unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("etas-package-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
