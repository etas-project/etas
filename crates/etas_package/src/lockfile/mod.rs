use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{PackageError, manifest::Manifest};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Lockfile {
    pub version: u32,
    #[serde(default)]
    pub manifest_fingerprint: Option<String>,
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub import_root: String,
    #[serde(default)]
    pub metadata_fingerprint: Option<String>,
}

pub fn read_lockfile(package_root: &Path) -> Result<Option<Lockfile>, PackageError> {
    let path = package_root.join("etas.lock");
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    toml::from_str(&text)
        .map(Some)
        .map_err(|source| PackageError::Lockfile {
            path,
            message: source.to_string(),
        })
}

pub fn write_lockfile(package_root: &Path, lockfile: &Lockfile) -> Result<(), PackageError> {
    let path = package_root.join("etas.lock");
    let text = toml::to_string_pretty(lockfile).map_err(|source| PackageError::Lockfile {
        path: path.clone(),
        message: source.to_string(),
    })?;
    std::fs::write(&path, text).map_err(|source| PackageError::Io { path, source })
}

pub fn manifest_dependency_fingerprint(manifest: &Manifest) -> String {
    let mut parts = vec![
        "etas-lock-input:v1".to_owned(),
        format!("package={}", manifest.package.name),
        format!("version={}", manifest.package.version),
        format!("edition={}", manifest.package.edition),
    ];
    for (name, dependency) in &manifest.dependencies {
        parts.push(format!(
            "dependency:{name}:{}",
            dependency.lock_fingerprint()
        ));
    }
    parts.join("|")
}
