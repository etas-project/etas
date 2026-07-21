use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::PackageError;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageSourceConfig {
    #[serde(default)]
    pub registries: BTreeMap<String, RegistrySourceConfig>,
    #[serde(default)]
    pub github: GitHubSourceConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegistrySourceConfig {
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GitHubSourceConfig {
    #[serde(default)]
    pub clone_base_url: Option<String>,
    #[serde(default)]
    pub release_base_url: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

impl PackageSourceConfig {
    pub fn read_project(package_root: &std::path::Path) -> Result<Self, PackageError> {
        let path = package_root.join(".etas").join("source-config.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|source| PackageError::Manifest {
            path,
            message: format!("package source config is not valid JSON: {source}"),
        })
    }
}
