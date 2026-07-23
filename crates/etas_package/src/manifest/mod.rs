use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{PackageDiagnostic, PackageError};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub package: PackageSection,
    #[serde(default)]
    pub source: SourceSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default, rename = "bin")]
    pub bins: Vec<BinTarget>,
    #[serde(default)]
    pub bindings: BindingsSection,
    #[serde(default)]
    pub runtime: RuntimeSection,
}

impl Manifest {
    pub fn validate(&self, manifest_path: &Path) -> Result<(), PackageError> {
        let mut diagnostics = Vec::new();
        if self.package.name.trim().is_empty() {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.missing_name",
                "`[package].name` must not be empty",
                Some(manifest_path.to_path_buf()),
            ));
        }
        if self.package.version.trim().is_empty() {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.missing_version",
                "`[package].version` must not be empty",
                Some(manifest_path.to_path_buf()),
            ));
        } else if let Err(source) = semver::Version::parse(&self.package.version) {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.invalid_package_version",
                format!(
                    "`[package].version` must be a SemVer version, got `{}`: {source}",
                    self.package.version
                ),
                Some(manifest_path.to_path_buf()),
            ));
        }
        if self.source.root.is_absolute() {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.absolute_source_root",
                "`[source].root` must be relative to the package root",
                Some(manifest_path.to_path_buf()),
            ));
        }
        let mut import_roots = BTreeMap::<String, String>::new();
        for (name, dependency) in &self.dependencies {
            let import_root = dependency.import_root(name);
            if !is_valid_import_root(&import_root) {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_import_root",
                    format!(
                        "dependency `{name}` import root `{import_root}` must be dot-separated Etas identifiers"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if let Some(previous) = import_roots.insert(import_root.clone(), name.clone()) {
                diagnostics.push(PackageDiagnostic::new(
                    "package.resolver.import_root_conflict",
                    format!(
                        "dependencies `{previous}` and `{name}` both use import root `{import_root}`"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            diagnostics.extend(dependency.validate(name, manifest_path));
        }
        let mut bin_names = BTreeSet::new();
        for bin in &self.bins {
            if !bin_names.insert(bin.name.clone()) {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.duplicate_bin",
                    format!("duplicate bin target `{}`", bin.name),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if bin.module.trim().is_empty() || bin.flow.trim().is_empty() {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_bin",
                    "bin target requires non-empty `module` and `flow`",
                    Some(manifest_path.to_path_buf()),
                ));
            }
        }
        for (tool, binding) in &self.bindings.tools {
            if binding.kind.trim().is_empty() {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_tool_binding",
                    format!("tool binding `{tool}` requires non-empty `kind`"),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if binding.provider_name().is_none() {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_tool_binding",
                    format!(
                        "tool binding `{tool}` requires either `provider` or both `server` and `tool`"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if binding.effects.is_empty() && binding.actions.is_empty() {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_tool_binding",
                    format!("tool binding `{tool}` must declare `effects` or `actions`"),
                    Some(manifest_path.to_path_buf()),
                ));
            }
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(PackageError::Diagnostics(diagnostics))
        }
    }

    pub fn default_entry(&self) -> Option<&BinTarget> {
        self.bins.first()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceSection {
    #[serde(default = "default_source_root")]
    pub root: PathBuf,
}

impl Default for SourceSection {
    fn default() -> Self {
        Self {
            root: default_source_root(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BinTarget {
    pub name: String,
    pub module: String,
    pub flow: String,
    #[serde(default)]
    pub profile: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSection {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub execution: RuntimeExecutionProfile,
    #[serde(default)]
    pub profiles: BTreeMap<String, RuntimeProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeExecutionProfile {
    #[serde(default)]
    pub max_call_depth: Option<u32>,
    #[serde(default)]
    pub max_steps: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfile {
    #[serde(default)]
    pub model: Option<RuntimeModelProfile>,
    #[serde(default)]
    pub memory: Option<RuntimeBackendProfile>,
    #[serde(default)]
    pub session: Option<RuntimeBackendProfile>,
    #[serde(default)]
    pub policy: Option<RuntimePolicyProfile>,
    #[serde(default)]
    pub approval: Option<RuntimeModeProfile>,
    #[serde(default)]
    pub network: Option<RuntimeNetworkProfile>,
    #[serde(default)]
    pub filesystem: Option<RuntimeFilesystemProfile>,
    #[serde(default)]
    pub tools: Option<RuntimeToolsProfile>,
    #[serde(default)]
    pub secret: Option<RuntimeModeProfile>,
    #[serde(default)]
    pub command: Option<RuntimeCommandProfile>,
    #[serde(default)]
    pub boundary_policy: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeModelProfile {
    #[serde(default)]
    pub adapter: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub allow_private: Option<bool>,
    #[serde(default)]
    pub retry: Option<RuntimeRetryProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRetryProfile {
    #[serde(default)]
    pub attempts: Option<u8>,
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeBackendProfile {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimePolicyProfile {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub token_env: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub allow_private: Option<bool>,
    #[serde(default)]
    pub retry: Option<RuntimeRetryProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeModeProfile {
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeNetworkProfile {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFilesystemProfile {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub workspace_root: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeToolsProfile {
    #[serde(default)]
    pub http: BTreeMap<String, String>,
    #[serde(default)]
    pub mcp: BTreeMap<String, String>,
    #[serde(default)]
    pub process: BTreeMap<String, String>,
    #[serde(default)]
    pub allow_private: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCommandProfile {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DependencySpec {
    Version(String),
    Detailed(Box<DependencyDetail>),
}

impl DependencySpec {
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::Version(version) => Some(version),
            Self::Detailed(detail) => detail.version.as_deref(),
        }
    }

    pub fn import_root(&self, dependency_name: &str) -> String {
        match self {
            Self::Version(_) => dependency_name.to_owned(),
            Self::Detailed(detail) => detail
                .import
                .clone()
                .unwrap_or_else(|| dependency_name.to_owned()),
        }
    }

    pub fn package_name<'a>(&'a self, dependency_name: &'a str) -> &'a str {
        match self {
            Self::Version(_) => dependency_name,
            Self::Detailed(detail) => detail.package.as_deref().unwrap_or(dependency_name),
        }
    }

    pub fn package_name_constraint<'a>(&'a self, dependency_name: &'a str) -> Option<&'a str> {
        match self {
            Self::Version(_) => Some(dependency_name),
            Self::Detailed(detail) => {
                if let Some(package) = detail.package.as_deref() {
                    return Some(package);
                }
                if detail.path.is_some() || detail.git.is_some() || detail.github.is_some() {
                    None
                } else {
                    Some(dependency_name)
                }
            }
        }
    }

    pub fn lock_fingerprint(&self) -> String {
        match self {
            Self::Version(version) => format!("version:{version}"),
            Self::Detailed(detail) => detail.lock_fingerprint(),
        }
    }

    fn validate(&self, dependency_name: &str, manifest_path: &Path) -> Vec<PackageDiagnostic> {
        match self {
            Self::Version(version) => {
                if version.trim().is_empty() {
                    vec![PackageDiagnostic::new(
                        "package.manifest.invalid_dependency",
                        format!("dependency `{dependency_name}` has an empty version requirement"),
                        Some(manifest_path.to_path_buf()),
                    )]
                } else if let Err(source) = semver::VersionReq::parse(version) {
                    vec![PackageDiagnostic::new(
                        "package.manifest.invalid_version_requirement",
                        format!(
                            "dependency `{dependency_name}` has invalid version requirement `{version}`: {source}"
                        ),
                        Some(manifest_path.to_path_buf()),
                    )]
                } else {
                    Vec::new()
                }
            }
            Self::Detailed(detail) => detail.validate(dependency_name, manifest_path),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DependencyDetail {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub import: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub github: Option<String>,
    #[serde(default)]
    pub rev: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub registry: Option<String>,
}

impl DependencyDetail {
    pub fn lock_fingerprint(&self) -> String {
        let mut parts = Vec::new();
        if let Some(package) = &self.package {
            parts.push(format!("package={package}"));
        }
        if let Some(version) = &self.version {
            parts.push(format!("version={version}"));
        }
        if let Some(import) = &self.import {
            parts.push(format!("import={import}"));
        }
        if let Some(path) = &self.path {
            parts.push(format!("path={}", path.display()));
        }
        if let Some(git) = &self.git {
            parts.push(format!("git={git}"));
        }
        if let Some(github) = &self.github {
            parts.push(format!("github={github}"));
        }
        if let Some(rev) = &self.rev {
            parts.push(format!("rev={rev}"));
        }
        if let Some(tag) = &self.tag {
            parts.push(format!("tag={tag}"));
        }
        if let Some(branch) = &self.branch {
            parts.push(format!("branch={branch}"));
        }
        if let Some(release) = &self.release {
            parts.push(format!("release={release}"));
        }
        if let Some(asset) = &self.asset {
            parts.push(format!("asset={asset}"));
        }
        if let Some(checksum) = &self.checksum {
            parts.push(format!("checksum={checksum}"));
        }
        if let Some(registry) = &self.registry {
            parts.push(format!("registry={registry}"));
        }
        parts.join(",")
    }

    fn validate(&self, dependency_name: &str, manifest_path: &Path) -> Vec<PackageDiagnostic> {
        let mut diagnostics = Vec::new();
        let source_count = usize::from(self.path.is_some())
            + usize::from(self.git.is_some())
            + usize::from(self.github.is_some())
            + usize::from(self.registry.is_some());
        if source_count > 1 {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.dependency_source_conflict",
                format!("dependency `{dependency_name}` must use only one of `path`, `git`, `github`, or `registry`"),
                Some(manifest_path.to_path_buf()),
            ));
        }
        let git_selector_count = usize::from(self.rev.is_some())
            + usize::from(self.tag.is_some())
            + usize::from(self.branch.is_some());
        if self.git.is_none() && self.github.is_none() && git_selector_count > 0 {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.git_selector_without_git",
                format!("dependency `{dependency_name}` declares `rev`, `tag`, or `branch` without `git` or `github`"),
                Some(manifest_path.to_path_buf()),
            ));
        }
        if git_selector_count > 1 {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.git_selector_conflict",
                format!(
                    "dependency `{dependency_name}` must use only one of `rev`, `tag`, or `branch`"
                ),
                Some(manifest_path.to_path_buf()),
            ));
        }
        if self.path.is_none()
            && self.git.is_none()
            && self.github.is_none()
            && self
                .version
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.registry_dependency_without_version",
                format!("registry dependency `{dependency_name}` requires a version constraint"),
                Some(manifest_path.to_path_buf()),
            ));
        }
        if let Some(github) = &self.github {
            if !is_valid_github_repo(github) {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.invalid_github_repo",
                    format!(
                        "dependency `{dependency_name}` github source must be `owner/repo`, got `{github}`"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if self.release.is_some() && git_selector_count > 0 {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.github_release_selector_conflict",
                    format!(
                        "dependency `{dependency_name}` cannot combine `release` with `rev`, `tag`, or `branch`"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
            if self.release.is_some() {
                if self.asset.as_deref().unwrap_or_default().trim().is_empty() {
                    diagnostics.push(PackageDiagnostic::new(
                        "package.manifest.github_release_asset_missing",
                        format!("GitHub release dependency `{dependency_name}` requires `asset`"),
                        Some(manifest_path.to_path_buf()),
                    ));
                }
                match self.checksum.as_deref() {
                    Some(checksum) if is_blake3_checksum(checksum) => {}
                    Some(checksum) => diagnostics.push(PackageDiagnostic::new(
                        "package.manifest.github_release_checksum_invalid",
                        format!(
                            "GitHub release dependency `{dependency_name}` checksum must be lowercase `blake3:<hex>`, got `{checksum}`"
                        ),
                        Some(manifest_path.to_path_buf()),
                    )),
                    None => diagnostics.push(PackageDiagnostic::new(
                        "package.manifest.github_release_checksum_missing",
                        format!(
                            "GitHub release dependency `{dependency_name}` requires `checksum`"
                        ),
                        Some(manifest_path.to_path_buf()),
                    )),
                }
            } else if self.asset.is_some() || self.checksum.is_some() {
                diagnostics.push(PackageDiagnostic::new(
                    "package.manifest.github_release_field_without_release",
                    format!(
                        "dependency `{dependency_name}` declares `asset` or `checksum` without `release`"
                    ),
                    Some(manifest_path.to_path_buf()),
                ));
            }
        } else if self.release.is_some() || self.asset.is_some() || self.checksum.is_some() {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.github_release_without_github",
                format!("dependency `{dependency_name}` declares `release`, `asset`, or `checksum` without `github`"),
                Some(manifest_path.to_path_buf()),
            ));
        }
        if let Some(version) = &self.version
            && let Err(source) = semver::VersionReq::parse(version)
        {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.invalid_version_requirement",
                format!(
                    "dependency `{dependency_name}` has invalid version requirement `{version}`: {source}"
                ),
                Some(manifest_path.to_path_buf()),
            ));
        }
        diagnostics
    }
}

fn is_valid_github_repo(value: &str) -> bool {
    let Some((owner, repo)) = value.split_once('/') else {
        return false;
    };
    !owner.is_empty()
        && !repo.is_empty()
        && !repo.contains('/')
        && owner.chars().all(is_github_name_char)
        && repo.chars().all(is_github_name_char)
}

fn is_github_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

fn is_blake3_checksum(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, 'a'..='f'))
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BindingsSection {
    #[serde(default)]
    pub tools: BTreeMap<String, ToolBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolBinding {
    pub kind: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub actions: Vec<String>,
}

impl ToolBinding {
    pub fn provider_name(&self) -> Option<String> {
        if let Some(provider) = &self.provider {
            if !provider.trim().is_empty() {
                return Some(provider.clone());
            }
        }
        match (&self.server, &self.tool) {
            (Some(server), Some(tool)) if !server.trim().is_empty() && !tool.trim().is_empty() => {
                Some(format!("{server}.{tool}"))
            }
            _ => None,
        }
    }
}

pub fn discover_manifest(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = current.join("etas.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn read_manifest(package_root: &Path) -> Result<Manifest, PackageError> {
    let path = package_root.join("etas.toml");
    let text = std::fs::read_to_string(&path).map_err(|source| PackageError::Io {
        path: path.clone(),
        source,
    })?;
    let manifest = toml::from_str::<Manifest>(&text).map_err(|source| PackageError::Manifest {
        path: path.clone(),
        message: source.to_string(),
    })?;
    manifest.validate(&path)?;
    Ok(manifest)
}

fn default_edition() -> String {
    "2026".to_owned()
}

fn default_source_root() -> PathBuf {
    PathBuf::from("src")
}

fn is_valid_import_root(import_root: &str) -> bool {
    !import_root.is_empty()
        && import_root.split('.').all(|segment| {
            let mut chars = segment.chars();
            matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
                && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_accepts_runtime_profiles() {
        let manifest = toml::from_str::<Manifest>(
            r#"
[package]
name = "runtime-profile-example"
version = "0.1.0"

[runtime]
default_profile = "local-omlx"

[runtime.execution]
max_call_depth = 96
max_steps = 1000000

[runtime.profiles.local-omlx.model]
adapter = "omlx-openai"
model = "Qwen3.5-0.8B-MLX-4bit"
base_url = "http://127.0.0.1:8848/v1"
api_key_env = "ETAS_HOST_OMLX_API_KEY"
allow_private = true

[runtime.profiles.local-omlx.memory]
backend = "memory"

[runtime.profiles.local-omlx.policy]
mode = "local-static"
rules = ["model=allow", "console=allow"]

[runtime.profiles.local-omlx.approval]
mode = "auto"

[runtime.profiles.local-omlx.network]
allow = ["127.0.0.1:8848"]

[runtime.profiles.local-omlx.tools]
allow_private = true

[runtime.profiles.local-omlx.tools.http]
"app.tools.EvidenceLookup" = "http://127.0.0.1:9000"

[[bin]]
name = "smoke"
module = "app"
flow = "main"
profile = "local-omlx"
"#,
        )
        .expect("runtime profile manifest should parse");

        assert_eq!(
            manifest.runtime.default_profile.as_deref(),
            Some("local-omlx")
        );
        assert_eq!(manifest.runtime.execution.max_call_depth, Some(96));
        assert_eq!(manifest.runtime.execution.max_steps, Some(1_000_000));
        let profile = manifest
            .runtime
            .profiles
            .get("local-omlx")
            .expect("profile exists");
        assert_eq!(
            profile
                .model
                .as_ref()
                .and_then(|model| model.adapter.as_deref()),
            Some("omlx-openai")
        );
        assert_eq!(
            profile.model.as_ref().and_then(|model| model.allow_private),
            Some(true)
        );
        assert_eq!(
            profile
                .tools
                .as_ref()
                .and_then(|tools| tools.http.get("app.tools.EvidenceLookup"))
                .map(String::as_str),
            Some("http://127.0.0.1:9000")
        );
        assert_eq!(
            profile.tools.as_ref().and_then(|tools| tools.allow_private),
            Some(true)
        );
        assert_eq!(manifest.bins[0].profile.as_deref(), Some("local-omlx"));
    }

    #[test]
    fn manifest_rejects_plaintext_runtime_secret() {
        let error = toml::from_str::<Manifest>(
            r#"
[package]
name = "runtime-profile-secret"
version = "0.1.0"

[runtime.profiles.local.model]
adapter = "omlx-openai"
model = "Qwen3.5-0.8B-MLX-4bit"
api_key = "do-not-commit"
"#,
        )
        .expect_err("runtime model profile must reject plaintext api_key");
        assert!(error.to_string().contains("api_key"), "{error}");
    }

    #[test]
    fn runtime_profile_does_not_affect_dependency_lock_fingerprint() {
        let base = toml::from_str::<Manifest>(
            r#"
[package]
name = "fingerprint"
version = "0.1.0"

[dependencies]
dep = { path = "../dep", import = "dep" }
"#,
        )
        .expect("base manifest parses");
        let with_runtime = toml::from_str::<Manifest>(
            r#"
[package]
name = "fingerprint"
version = "0.1.0"

[dependencies]
dep = { path = "../dep", import = "dep" }

[runtime]
default_profile = "local"

[runtime.execution]
max_call_depth = 96

[runtime.profiles.local.model]
adapter = "omlx-openai"
model = "Qwen3.5-0.8B-MLX-4bit"
"#,
        )
        .expect("runtime manifest parses");

        assert_eq!(
            base.dependencies["dep"].lock_fingerprint(),
            with_runtime.dependencies["dep"].lock_fingerprint()
        );
    }
}
