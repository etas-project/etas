use std::path::{Path, PathBuf};

mod entry;
mod environment;
mod lock;
mod lock_policy;
mod materialize;
mod source;
mod version;

use entry::select_bin;
use environment::build_environment;
use lock_policy::resolve_lockfile;

pub use lock::{LockPackageOptions, LockedPackageResult, lock_package};
pub use materialize::{MaterializePackageOptions, MaterializedPackageResult, materialize_package};

use crate::{PackageEnvironmentMetadata, PackageError, manifest, vendor};

#[derive(Clone, Debug)]
pub struct PreparePackageOptions {
    pub package_root: PathBuf,
    pub selected_bin: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PreparedPackage {
    pub package_root: PathBuf,
    pub source_root: PathBuf,
    pub manifest: manifest::Manifest,
    pub selected_bin: Option<manifest::BinTarget>,
    pub environment: PackageEnvironmentMetadata,
}

pub fn prepare_package(options: PreparePackageOptions) -> Result<PreparedPackage, PackageError> {
    let package_root = canonical_package_root(&options.package_root)?;
    let manifest = manifest::read_manifest(&package_root)?;
    let package_index = vendor::read_package_index(&package_root)?;
    let lockfile = resolve_lockfile(&package_root, &manifest)?;
    let environment =
        build_environment(&package_root, &manifest, &lockfile, package_index.as_ref())?;
    let selected_bin = select_bin(&manifest, options.selected_bin.as_deref())?;
    let source_root = package_root.join(&manifest.source.root);
    if !source_root.is_dir() {
        return Err(PackageError::diagnostic(
            "driver.source_root_missing",
            format!("source root `{}` does not exist", source_root.display()),
            Some(source_root),
        ));
    }
    Ok(PreparedPackage {
        package_root,
        source_root,
        manifest,
        selected_bin,
        environment,
    })
}

fn canonical_package_root(package_root: &Path) -> Result<PathBuf, PackageError> {
    package_root
        .canonicalize()
        .map_err(|source| PackageError::Io {
            path: package_root.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::{lockfile, metadata, metadata_artifact, store};

    #[test]
    fn prepare_package_accepts_manifest_with_builtin_std_only() {
        let root = temp_package("std-only");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "std-only"
version = "0.1.0"
edition = "2026"

[source]
root = "src"

[dependencies]
std = { version = "0.1" }

[[bin]]
name = "std-only"
module = "app.main"
flow = "main"
"#,
        )
        .unwrap();
        write_builtin_std_lockfile(&root);

        let prepared = prepare_package(PreparePackageOptions {
            package_root: root.clone(),
            selected_bin: None,
        })
        .unwrap();

        assert_eq!(prepared.manifest.package.name, "std-only");
        assert_eq!(
            prepared.source_root,
            root.join("src").canonicalize().unwrap()
        );
        assert_eq!(prepared.environment.dependencies.len(), 1);
        assert_eq!(prepared.environment.dependencies[0].import_root, "std");
        assert_eq!(
            prepared
                .selected_bin
                .as_ref()
                .map(|bin| bin.module.as_str()),
            Some("app.main")
        );
    }

    #[test]
    fn prepare_package_rejects_missing_lockfile_for_non_std_dependency() {
        let root = temp_package("missing-lock");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "unresolved-dep"
version = "0.1.0"
edition = "2026"

[dependencies]
std = { version = "0.1" }
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "package.lockfile.missing" })
        );
    }

    #[test]
    fn manifest_rejects_github_release_without_checksum() {
        let root = temp_package("github-release-no-checksum");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-release-no-checksum"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools.etaspkg", import = "tools" }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.manifest.github_release_checksum_missing"
        }));
    }

    #[test]
    fn manifest_rejects_uppercase_github_release_checksum() {
        let root = temp_package("github-release-uppercase-checksum");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-release-uppercase-checksum"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools.etaspkg", checksum = "blake3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", import = "tools" }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.manifest.github_release_checksum_invalid"
        }));
    }

    #[test]
    fn manifest_rejects_unsupported_github_repo_format() {
        let root = temp_package("github-repo-format");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-repo-format"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "https://github.com/owner/agent-tools", tag = "v0.1.0", import = "tools" }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "package.manifest.invalid_github_repo" })
        );
    }

    #[test]
    fn manifest_rejects_checksum_outside_github_release() {
        let root = temp_package("github-checksum-without-release");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-checksum-without-release"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", path = "../agent-tools", checksum = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", import = "tools" }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.manifest.github_release_without_github"
        }));
    }

    #[test]
    fn prepare_package_uses_dependency_key_as_default_import_root() {
        let root = temp_package("default-import-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        let (store_path, payload_checksum) =
            write_stored_package_payload(&root, "company-agents", "1.2.3", "company_agents");
        let manifest_text = r#"
[package]
name = "default-import-root"
version = "0.1.0"
edition = "2026"

[dependencies]
company_agents = { package = "company-agents", version = "1.2" }
"#;
        fs::write(root.join("etas.toml"), manifest_text).unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "company-agents"
version = "1.2.3"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "company_agents"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            format!(
                r#"{{
  "version": 1,
  "package": {{ "name": "default-import-root", "version": "0.1.0", "edition": "2026" }},
  "dependencies": [
    {{
      "identity": {{ "name": "company-agents", "version": "1.2.3", "edition": "2026" }},
      "import_root": "company_agents",
      "source": {{ "kind": "registry", "registry": "https://packages.etas.dev", "checksum": "{payload_checksum}", "store": "{store_path}" }}
    }}
  ]
}}
"#
            ),
        )
        .unwrap();
        write_lockfile_from_package_index(
            &root,
            "company-agents",
            "1.2.3",
            "registry+https://packages.etas.dev",
            "company_agents",
        );

        let prepared = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();

        assert_eq!(
            prepared.environment.dependencies[0].import_root,
            "company_agents"
        );
    }

    #[test]
    fn manifest_rejects_old_tool_binding_table() {
        let root = temp_package("old-tool-binding");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "old-tool-binding"
version = "0.1.0"
edition = "2026"

[tool.bindings]
"app.main.search" = { kind = "mcp", provider = "browser.search", effects = ["Network"] }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        assert!(matches!(error, PackageError::Manifest { .. }));
    }

    #[test]
    fn manifest_rejects_path_like_import_root() {
        let root = temp_package("invalid-import-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "invalid-import-root"
version = "0.1.0"
edition = "2026"

[dependencies]
dep = { package = "dep", version = "0.1", import = "../dep" }
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.manifest.invalid_import_root")
        );
    }

    #[test]
    fn manifest_rejects_non_semver_package_version() {
        let root = temp_package("invalid-package-version");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "invalid-package-version"
version = "latest"
edition = "2026"
"#,
        )
        .unwrap();

        let error = manifest::read_manifest(&root).unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "package.manifest.invalid_package_version"
            })
        );
    }

    #[test]
    fn prepare_package_reads_spec_tool_binding_table() {
        let root = temp_package("tool-binding");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "tool-binding"
version = "0.1.0"
edition = "2026"

[bindings.tools]
"app.main.search" = { kind = "mcp", server = "browser", tool = "search", effects = ["Network"] }
"#,
        )
        .unwrap();

        let prepared = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();

        assert_eq!(prepared.environment.tool_bindings.len(), 1);
        assert_eq!(
            prepared.environment.tool_bindings[0].provider,
            "browser.search"
        );
    }

    #[test]
    fn prepare_package_rejects_unresolved_locked_dependency_without_metadata() {
        let root = temp_package("unresolved-dep");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "unresolved-dep"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "agent-tools"
version = "0.1.0"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "tools"
"#
            ),
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.resolver.dependency_unresolved"
                && diagnostic.message.contains("tools")
        }));
    }

    #[test]
    fn lock_package_reports_unmaterialized_path_dependency_without_metadata_placeholder() {
        let root = temp_package("lock-unmaterialized-path-dep");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "lock-unmaterialized-path-dep"
version = "0.1.0"
edition = "2026"

[dependencies]
edk_http = { package = "edk-http", version = "0.1", import = "edk.http", path = "../edk-http" }
"#,
        )
        .unwrap();

        let error = lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "package.resolver.dependency_unresolved")
            .expect("expected unresolved dependency diagnostic");
        assert!(diagnostic.message.contains("dependency `edk_http`"));
        assert!(diagnostic.message.contains("import root `edk.http`"));
        assert!(diagnostic.message.contains("path `../edk-http`"));
        assert!(diagnostic.message.contains("etas pkg update"));
        assert!(!diagnostic.message.contains("<metadata package>"));
    }

    #[test]
    fn prepare_package_rejects_missing_lockfile_for_builtin_dependency() {
        let root = temp_package("missing-builtin-lock");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "missing-builtin-lock"
version = "0.1.0"
edition = "2026"

[dependencies]
std = { version = "0.1" }
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root.clone(),
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.lockfile.missing")
        );
        assert!(
            !root.join("etas.lock").exists(),
            "compile prepare must not write etas.lock"
        );
    }

    #[test]
    fn prepare_package_rejects_stale_lockfile_for_non_builtin_dependencies() {
        let root = temp_package("stale-lock");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "stale-lock"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.2", import = "tools" }
"#,
        )
        .unwrap();
        fs::write(
            root.join("etas.lock"),
            r#"
version = 1
manifest_fingerprint = "old"

[[package]]
name = "agent-tools"
version = "0.1.0"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "tools"
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.lockfile.stale")
        );
    }

    #[test]
    fn prepare_package_rejects_locked_version_outside_manifest_requirement() {
        let root = temp_package("version-mismatch");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "version-mismatch"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = ">=0.2,<0.3", import = "tools" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "agent-tools"
version = "0.1.9"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "tools"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "version-mismatch", "version": "0.1.0", "edition": "2026" },
  "dependencies": [
    {
      "identity": { "name": "agent-tools", "version": "0.1.9", "edition": "2026" },
      "import_root": "tools",
      "source": { "kind": "registry", "registry": "https://packages.etas.dev", "checksum": "sha256:test" }
    }
  ]
}
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.lockfile.version_mismatch")
        );
    }

    #[test]
    fn prepare_package_rejects_package_index_import_root_mismatch() {
        let root = temp_package("metadata-import-mismatch");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "metadata-import-mismatch"
version = "0.1.0"
edition = "2026"

[dependencies]
company_agents = { package = "company-agents", version = "1.2" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "company-agents"
version = "1.2.3"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "company_agents"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "metadata-import-mismatch", "version": "0.1.0", "edition": "2026" },
  "dependencies": [
    {
      "identity": { "name": "company-agents", "version": "1.2.3", "edition": "2026" },
      "import_root": "company-agents",
      "source": { "kind": "registry", "registry": "https://packages.etas.dev", "checksum": "sha256:test" }
    }
  ]
}
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.resolver.dependency_unresolved")
        );
    }

    #[test]
    fn prepare_package_stamps_dependency_external_module_owner() {
        let root = temp_package("module-owner");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        let (store_path, payload_checksum) =
            write_stored_package_payload(&root, "company-agents", "1.2.3", "company_agents");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "module-owner"
version = "0.1.0"
edition = "2026"

[dependencies]
company_agents = { package = "company-agents", version = "1.2" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "company-agents"
version = "1.2.3"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
import_root = "company_agents"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            format!(
                r#"{{
  "version": 1,
  "package": {{ "name": "module-owner", "version": "0.1.0", "edition": "2026" }},
  "dependencies": [
    {{
      "identity": {{ "name": "company-agents", "version": "1.2.3", "edition": "2026" }},
      "import_root": "company_agents",
      "source": {{ "kind": "registry", "registry": "https://packages.etas.dev", "checksum": "{payload_checksum}", "store": "{store_path}" }},
      "public_metadata": {{
        "modules": [
          {{
            "id": 7,
            "path": ["company_agents", "writer"],
            "exports": []
          }}
        ]
      }}
    }}
  ]
}}
"#
            ),
        )
        .unwrap();
        write_lockfile_from_package_index(
            &root,
            "company-agents",
            "1.2.3",
            "registry+https://packages.etas.dev",
            "company_agents",
        );

        let prepared = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();

        let owner = prepared.environment.external_modules[0]
            .package
            .as_ref()
            .expect("dependency module owner should be materialized");
        assert_eq!(owner.identity.name, "company-agents");
        assert_eq!(owner.identity.version, "1.2.3");
        assert_eq!(owner.import_root, "company_agents");
    }

    #[test]
    fn prepare_package_rejects_ownerless_package_index_external_module() {
        let root = temp_package("ownerless-external-module");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "ownerless-external-module"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "ownerless-external-module", "version": "0.1.0", "edition": "2026" },
  "external_modules": [
    {
      "id": 1,
      "path": ["company_agents", "writer"],
      "exports": []
    }
  ]
}
"#,
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.metadata.external_module_owner_missing"
        }));
    }

    #[test]
    fn prepare_package_rejects_package_index_checksum_mismatch() {
        let root = temp_package("checksum-mismatch");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        let (store_path, payload_checksum) =
            write_stored_package_payload(&root, "agent-tools", "0.1.0", "tools");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "checksum-mismatch"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "agent-tools"
version = "0.1.0"
source = "registry+https://packages.etas.dev"
checksum = "sha256:lock"
import_root = "tools"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            format!(
                r#"{{
  "version": 1,
  "package": {{ "name": "checksum-mismatch", "version": "0.1.0", "edition": "2026" }},
  "dependencies": [
    {{
      "identity": {{ "name": "agent-tools", "version": "0.1.0", "edition": "2026" }},
      "import_root": "tools",
      "source": {{ "kind": "registry", "registry": "https://packages.etas.dev", "checksum": "{payload_checksum}", "store": "{store_path}" }}
    }}
  ]
}}
"#
            ),
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.lockfile.checksum_mismatch")
        );
    }

    #[test]
    fn prepare_package_rejects_ambiguous_lockfile_package_without_import_root() {
        let root = temp_package("missing-lock-import-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "missing-lock-import-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
more_tools = { package = "agent-tools", version = "0.1", import = "more_tools" }
"#,
        )
        .unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "agent-tools"
version = "0.1.0"
source = "registry+https://packages.etas.dev"
checksum = "sha256:test"
"#
            ),
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected diagnostics for ambiguous missing import_root, got {error:?}");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "package.lockfile.import_root_missing" })
        );
    }

    #[test]
    fn lock_package_writes_builtin_std_lockfile_explicitly() {
        let root = temp_package("lock-builtin-std");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "lock-builtin-std"
version = "0.1.0"
edition = "2026"

[dependencies]
std = { version = "0.1" }
"#,
        )
        .unwrap();

        let result = lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();

        assert_eq!(result.lockfile.packages.len(), 1);
        assert_eq!(result.lockfile.packages[0].name, "std");
        assert_eq!(result.lockfile.packages[0].source, "builtin");
        assert!(root.join("etas.lock").is_file());
    }

    #[test]
    fn lock_package_rejects_unmaterialized_dependency() {
        let root = temp_package("lock-unmaterialized-dep");
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "lock-unmaterialized-dep"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();

        let error = lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap_err();

        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "package.resolver.dependency_unresolved" })
        );
        assert!(!root.join("etas.lock").exists());
    }

    #[test]
    fn materialize_path_dependency_preserves_tool_bindings() {
        let dependency = temp_package("materialize-tool-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("tools.es"), "module tools;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" },
  "tool_bindings": [
    {
      "tool": "tools.search",
      "kind": "mcp",
      "provider": "browser.search",
      "effect_row": ["Network"],
      "action_row": []
    }
  ]
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("materialize-tool-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "materialize-tool-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let prepared = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();

        assert!(
            prepared
                .environment
                .tool_bindings
                .iter()
                .any(|binding| binding.tool == "tools.search"
                    && binding.provider == "browser.search")
        );
    }

    #[test]
    fn materialize_path_dependency_accepts_authoritative_metadata_artifact_without_json_mirror() {
        let dependency = temp_package("materialize-artifact-only-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("tools.es"), "module tools;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" },
  "tool_bindings": [
    {
      "tool": "tools.search",
      "kind": "mcp",
      "provider": "browser.search",
      "effect_row": ["Network"],
      "action_row": []
    }
  ]
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);
        fs::remove_file(dependency.join(".etas").join("package-index.json")).unwrap();

        let root = temp_package("materialize-artifact-only-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "materialize-artifact-only-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let index = vendor::read_package_index(&root).unwrap().unwrap();

        assert_eq!(index.dependencies.len(), 1);
        assert_eq!(index.dependencies[0].identity.name, "agent-tools");
        assert!(
            index.dependencies[0]
                .tool_bindings
                .iter()
                .any(|binding| binding.tool == "tools.search")
        );
    }

    #[test]
    fn path_dependency_without_package_field_uses_artifact_identity() {
        let dependency = temp_package("materialize-alias-package-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("tools.es"), "module tools;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" }
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("materialize-alias-package-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "materialize-alias-package-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools_alias = {{ version = "0.1", import = "tools", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let prepared = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();

        assert_eq!(prepared.environment.dependencies.len(), 1);
        assert_eq!(
            prepared.environment.dependencies[0].identity.name,
            "agent-tools"
        );
        assert_eq!(prepared.environment.dependencies[0].import_root, "tools");
    }

    #[test]
    fn materialize_path_dependency_payload_checksum_changes_with_source() {
        let dependency = temp_package("materialize-payload-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "payload-dep"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("lib.es"), "module payload;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "payload-dep", "version": "0.1.0", "edition": "2026" }
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("materialize-payload-root");
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "materialize-payload-root"
version = "0.1.0"
edition = "2026"

[dependencies]
payload = {{ package = "payload-dep", version = "0.1", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let first_checksum = materialized_dependency_payload_checksum(&root, "payload");

        fs::write(
            dependency.join("src").join("lib.es"),
            "module payload;\nflow changed() -> unit { return; }\n",
        )
        .unwrap();
        seal_package_metadata(&dependency);
        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let second_checksum = materialized_dependency_payload_checksum(&root, "payload");

        assert_ne!(first_checksum, second_checksum);
    }

    #[test]
    fn prepare_package_rejects_tampered_vendored_payload() {
        let dependency = temp_package("vendor-tamper-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "vendor-tamper-dep"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("lib.es"), "module payload;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "vendor-tamper-dep", "version": "0.1.0", "edition": "2026" }
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("vendor-tamper-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "vendor-tamper-root"
version = "0.1.0"
edition = "2026"

[dependencies]
payload = {{ package = "vendor-tamper-dep", version = "0.1", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        fs::write(
            root.join(".etas")
                .join("vendor")
                .join("payload")
                .join("src")
                .join("lib.es"),
            "module payload;\nflow tampered() -> unit { return; }\n",
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "package.lockfile.payload_checksum_mismatch"
            })
        );
    }

    #[test]
    fn prepare_package_rejects_tampered_store_payload() {
        let dependency = temp_package("store-tamper-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "store-tamper-dep"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("lib.es"), "module payload;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "store-tamper-dep", "version": "0.1.0", "edition": "2026" }
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("store-tamper-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "store-tamper-root"
version = "0.1.0"
edition = "2026"

[dependencies]
payload = {{ package = "store-tamper-dep", version = "0.1", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let store_path = materialized_dependency_store_path(&root, "payload");
        fs::write(
            root.join(store_path).join("src").join("lib.es"),
            "module payload;\nflow tampered() -> unit { return; }\n",
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.lockfile.store_payload_checksum_mismatch"
        }));
    }

    #[test]
    fn prepare_package_rejects_lock_source_mismatch() {
        let dependency = temp_package("source-mismatch-dep");
        fs::create_dir_all(dependency.join("src")).unwrap();
        fs::write(
            dependency.join("etas.toml"),
            r#"
[package]
name = "source-mismatch-dep"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(dependency.join("src").join("lib.es"), "module payload;\n").unwrap();
        fs::create_dir_all(dependency.join(".etas")).unwrap();
        fs::write(
            dependency.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "source-mismatch-dep", "version": "0.1.0", "edition": "2026" }
}
"#,
        )
        .unwrap();
        seal_package_metadata(&dependency);

        let root = temp_package("source-mismatch-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "source-mismatch-root"
version = "0.1.0"
edition = "2026"

[dependencies]
payload = {{ package = "source-mismatch-dep", version = "0.1", path = "{}" }}
"#,
                dependency.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        fs::write(
            root.join("etas.lock"),
            lockfile.replace(
                "source = \"vendor+.etas/vendor/payload\"",
                "source = \"path+../dep\"",
            ),
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.lockfile.source_mismatch")
        );
    }

    #[test]
    fn materialize_package_resolves_local_registry_dependency() {
        let root = temp_package("registry-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        write_local_registry_index(&registry, "agent-tools", "0.1.0", &checksum);

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let index = vendor::read_package_index(&root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == "tools")
            .expect("registry dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::Registry {
                registry,
                checksum,
                store: Some(store),
            } => {
                assert!(registry.starts_with("file://"), "{registry}");
                assert!(checksum.starts_with("blake3:"), "{checksum}");
                assert!(store.starts_with(".etas/store/packages/blake3/"), "{store}");
            }
            other => panic!("expected registry dependency source, got {other:?}"),
        }

        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        assert!(
            lockfile.contains("source = \"registry+file://"),
            "{lockfile}"
        );

        prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();
    }

    #[test]
    fn prepare_package_rejects_tampered_local_registry_store_payload() {
        let root = temp_package("registry-store-tamper-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        write_local_registry_index(&registry, "agent-tools", "0.1.0", &checksum);

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-store-tamper-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let store_path = materialized_dependency_store_path(&root, "tools");
        fs::write(
            root.join(store_path).join("src").join("lib.es"),
            "module tools;\nflow tampered() -> unit { return; }\n",
        )
        .unwrap();

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.lockfile.store_payload_checksum_mismatch"
        }));
    }

    #[test]
    fn materialize_package_rejects_invalid_local_registry_version() {
        let root = temp_package("registry-invalid-version-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        fs::write(
            registry.join("index").join("agent-tools.json"),
            format!(
                r#"{{
  "versions": [
    {{
      "version": "latest",
      "path": "packages/agent-tools/0.1.0",
      "checksum": "{checksum}"
    }}
  ]
}}
"#
            ),
        )
        .unwrap();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-invalid-version-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.registry.invalid_version")
        );
    }

    #[test]
    fn materialize_package_rejects_local_registry_version_without_checksum() {
        let root = temp_package("registry-missing-checksum-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        fs::write(
            registry.join("index").join("agent-tools.json"),
            r#"{
  "versions": [
    {
      "version": "0.1.0",
      "path": "packages/agent-tools/0.1.0"
    }
  ]
}
"#,
        )
        .unwrap();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-missing-checksum-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.registry.index_unreadable")
        );
    }

    #[test]
    fn materialize_package_rejects_local_registry_stale_metadata_artifact() {
        let root = temp_package("registry-metadata-checksum-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        write_local_registry_index(&registry, "agent-tools", "0.1.0", &checksum);
        fs::write(
            package_root.join("src").join("lib.es"),
            "module tools;\nflow changed() -> unit { return; }\n",
        )
        .unwrap();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-metadata-checksum-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code
            == "package.registry.metadata_unreadable"
            && diagnostic.message.contains("source hash")));
    }

    #[test]
    fn materialize_package_rejects_corrupted_content_addressed_store_entry() {
        let root = temp_package("registry-corrupt-store-root");
        let registry = root.join("fixtures").join("registry");
        init_local_registry_fixture(&registry);
        write_source_config(&root, "local", "fixtures/registry");
        let package_root = registry.join("packages").join("agent-tools").join("0.1.0");
        write_registry_package_payload(&package_root, "agent-tools", "0.1.0", "tools");
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        write_local_registry_index(&registry, "agent-tools", "0.1.0", &checksum);

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "registry-corrupt-store-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", registry = "local", import = "tools" }
"#,
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let store_path = materialized_dependency_store_path(&root, "tools");
        fs::write(
            root.join(store_path).join("src").join("lib.es"),
            "module tools;\nflow corrupted() -> unit { return; }\n",
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "package.registry.store_write_failed")
        );
    }

    #[test]
    fn lock_package_records_transitive_dependency_graph() {
        let root = temp_package("lock-transitive-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "lock-transitive-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            r#"
{
  "version": 1,
  "package": { "name": "lock-transitive-root", "version": "0.1.0", "edition": "2026" },
  "dependencies": [
    {
      "identity": { "name": "agent-tools", "version": "0.1.0", "edition": "2026" },
      "import_root": "tools",
      "source": { "kind": "builtin", "checksum": "builtin:tools" },
      "dependencies": [
        {
          "identity": { "name": "helper-lib", "version": "0.2.0", "edition": "2026" },
          "import_root": "helper",
          "source": { "kind": "builtin", "checksum": "builtin:helper" }
        }
      ]
    }
  ]
}
"#,
        )
        .unwrap();

        let result = lock_package(LockPackageOptions { package_root: root }).unwrap();

        assert_eq!(result.lockfile.packages.len(), 2);
        let tools = result
            .lockfile
            .packages
            .iter()
            .find(|package| package.import_root == "tools")
            .expect("root dependency should be locked");
        assert_eq!(
            tools.dependencies,
            vec!["helper|helper-lib|0.2.0".to_owned()]
        );
        assert!(
            result
                .lockfile
                .packages
                .iter()
                .any(|package| package.import_root == "helper")
        );
    }

    #[test]
    fn prepare_package_rejects_missing_transitive_lock_entry() {
        let root = temp_package("missing-transitive-lock-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        let parent_root = root.join(".etas").join("payloads").join("agent-tools");
        write_registry_package_payload(&parent_root, "agent-tools", "0.1.0", "tools");
        let parent_checksum = vendor::path_package_payload_checksum(&parent_root).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "missing-transitive-lock-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();
        fs::write(
            root.join(".etas").join("package-index.json"),
            format!(
                r#"{{
  "version": 1,
  "package": {{ "name": "missing-transitive-lock-root", "version": "0.1.0", "edition": "2026" }},
  "dependencies": [
    {{
      "identity": {{ "name": "agent-tools", "version": "0.1.0", "edition": "2026" }},
      "import_root": "tools",
      "source": {{ "kind": "path", "path": ".etas/payloads/agent-tools", "checksum": "{parent_checksum}" }},
      "dependencies": [
        {{
          "identity": {{ "name": "helper-lib", "version": "0.2.0", "edition": "2026" }},
          "import_root": "helper",
          "source": {{ "kind": "builtin", "checksum": "builtin:helper" }}
        }}
      ]
    }}
  ]
}}
"#
            ),
        )
        .unwrap();
        write_lockfile_from_package_index(
            &root,
            "agent-tools",
            "0.1.0",
            "path+.etas/payloads/agent-tools",
            "tools",
        );

        let error = prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "package.lockfile.transitive_dependency_missing"
        }));
    }

    #[test]
    fn prepare_package_accepts_metadata_only_transitive_dependency_without_root_payload() {
        let root = temp_package("metadata-only-transitive-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas").join("payloads")).unwrap();
        let parent_root = root.join(".etas").join("payloads").join("agent-tools");
        write_registry_package_payload(&parent_root, "agent-tools", "0.1.0", "tools");
        let parent_checksum = vendor::path_package_payload_checksum(&parent_root).unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "metadata-only-transitive-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", import = "tools" }
"#,
        )
        .unwrap();
        let helper_dependency = metadata::ResolvedDependency {
            identity: metadata::PackageIdentity {
                name: "helper-lib".to_owned(),
                version: "0.2.0".to_owned(),
                edition: "2026".to_owned(),
            },
            import_root: "helper".to_owned(),
            source: metadata::ResolvedDependencySource::Registry {
                registry: "https://packages.etas.dev".to_owned(),
                checksum: "blake3:1111111111111111111111111111111111111111111111111111111111111111"
                    .to_owned(),
                store: Some(
                    ".etas/store/packages/blake3/1111111111111111111111111111111111111111111111111111111111111111"
                        .to_owned(),
                ),
            },
            dependencies: Vec::new(),
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
        };
        let helper_checksum = vendor::dependency_content_checksum(&helper_dependency);
        let parent_dependency = metadata::ResolvedDependency {
            identity: metadata::PackageIdentity {
                name: "agent-tools".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            import_root: "tools".to_owned(),
            source: metadata::ResolvedDependencySource::Path {
                path: ".etas/payloads/agent-tools".to_owned(),
                checksum: parent_checksum.clone(),
            },
            dependencies: vec![helper_dependency],
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
        };
        let parent_checksum_document = vendor::dependency_content_checksum(&parent_dependency);
        let index = metadata::PackageIndex {
            version: 1,
            package: metadata::PackageIdentity {
                name: "metadata-only-transitive-root".to_owned(),
                version: "0.1.0".to_owned(),
                edition: "2026".to_owned(),
            },
            dependencies: vec![parent_dependency],
            external_modules: Vec::new(),
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: Vec::new(),
            bins: Vec::new(),
        };
        vendor::write_package_index(&root, &index).unwrap();
        let manifest = manifest::read_manifest(&root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "agent-tools"
version = "0.1.0"
source = "path+.etas/payloads/agent-tools"
checksum = "{parent_checksum_document}"
dependencies = ["helper|helper-lib|0.2.0"]
import_root = "tools"

[[package]]
name = "helper-lib"
version = "0.2.0"
source = "registry+https://packages.etas.dev"
checksum = "{helper_checksum}"
dependencies = []
import_root = "helper"
"#
            ),
        )
        .unwrap();

        prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();
    }

    #[test]
    fn materialize_package_resolves_local_git_dependency_to_exact_rev() {
        let dependency_root = temp_package("git-agent-tools");
        write_registry_package_payload(&dependency_root, "agent-tools", "0.1.0", "tools");
        git(&dependency_root, &["init"]);
        git(&dependency_root, &["add", "."]);
        git(
            &dependency_root,
            &[
                "-c",
                "user.name=Etas",
                "-c",
                "user.email=etas@example.invalid",
                "commit",
                "-m",
                "initial",
            ],
        );
        git(&dependency_root, &["tag", "v0.1.0"]);
        let exact_rev = git_output(&dependency_root, &["rev-parse", "HEAD"]);

        let root = temp_package("git-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "git-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", git = "file://{}", tag = "v0.1.0", import = "tools" }}
"#,
                dependency_root.display()
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let index = vendor::read_package_index(&root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == "tools")
            .expect("git dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::Git {
                url,
                rev,
                checksum,
                store: Some(store),
            } => {
                assert!(url.starts_with("file://"), "{url}");
                assert_eq!(rev, &exact_rev);
                assert!(checksum.starts_with("blake3:"), "{checksum}");
                assert!(store.starts_with(".etas/store/packages/blake3/"), "{store}");
            }
            other => panic!("expected git dependency source, got {other:?}"),
        }

        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        assert!(
            lockfile.contains(&format!(
                "source = \"git+file://{}?rev={exact_rev}\"",
                dependency_root.display()
            )),
            "{lockfile}"
        );

        prepare_package(PreparePackageOptions {
            package_root: root,
            selected_bin: None,
        })
        .unwrap();
    }

    #[test]
    fn materialize_package_resolves_github_clone_dependency_to_exact_rev() {
        let github_root = temp_package("github-clone-base");
        let dependency_root = github_root.join("owner").join("agent-tools.git");
        fs::create_dir_all(dependency_root.parent().unwrap()).unwrap();
        write_registry_package_payload(&dependency_root, "agent-tools", "0.1.0", "tools");
        git(&dependency_root, &["init"]);
        git(&dependency_root, &["add", "."]);
        git(
            &dependency_root,
            &[
                "-c",
                "user.name=Etas",
                "-c",
                "user.email=etas@example.invalid",
                "commit",
                "-m",
                "initial",
            ],
        );
        git(&dependency_root, &["tag", "v0.1.0"]);
        let exact_rev = git_output(&dependency_root, &["rev-parse", "HEAD"]);

        let root = temp_package("github-clone-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "clone_base_url": "file://{}" }}
}}
"#,
                github_root.display()
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-clone-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", tag = "v0.1.0", import = "tools" }
"#,
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let index = vendor::read_package_index(&root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == "tools")
            .expect("GitHub dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::GitHubClone {
                repo,
                rev,
                checksum,
                store: Some(store),
            } => {
                assert_eq!(repo, "owner/agent-tools");
                assert_eq!(rev, &exact_rev);
                assert!(checksum.starts_with("blake3:"), "{checksum}");
                assert!(store.starts_with(".etas/store/packages/blake3/"), "{store}");
            }
            other => panic!("expected GitHub clone dependency source, got {other:?}"),
        }

        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        assert!(
            lockfile.contains(&format!(
                "source = \"github+owner/agent-tools?rev={exact_rev}\""
            )),
            "{lockfile}"
        );
        assert!(!lockfile.contains("tag = \"v0.1.0\""), "{lockfile}");
    }

    #[test]
    fn materialize_package_resolves_github_branch_to_exact_rev() {
        let github_root = temp_package("github-branch-base");
        let dependency_root = github_root.join("owner").join("agent-tools.git");
        fs::create_dir_all(dependency_root.parent().unwrap()).unwrap();
        write_registry_package_payload(&dependency_root, "agent-tools", "0.1.0", "tools");
        git(&dependency_root, &["init"]);
        git(&dependency_root, &["checkout", "-b", "main"]);
        git(&dependency_root, &["add", "."]);
        git(
            &dependency_root,
            &[
                "-c",
                "user.name=Etas",
                "-c",
                "user.email=etas@example.invalid",
                "commit",
                "-m",
                "initial",
            ],
        );
        let exact_rev = git_output(&dependency_root, &["rev-parse", "HEAD"]);

        let root = temp_package("github-branch-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "clone_base_url": "file://{}" }}
}}
"#,
                github_root.display()
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-branch-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", branch = "main", import = "tools" }
"#,
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();

        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        assert!(
            lockfile.contains(&format!(
                "source = \"github+owner/agent-tools?rev={exact_rev}\""
            )),
            "{lockfile}"
        );
        assert!(
            !lockfile.contains("source = \"github+owner/agent-tools?branch=main\""),
            "{lockfile}"
        );
    }

    #[test]
    fn materialize_package_downloads_github_release_asset() {
        let dependency_root = temp_package("github-release-dep");
        write_registry_package_payload(&dependency_root, "agent-tools", "0.1.0", "tools");
        let asset_name = "agent tools & 0.1.0.etaspkg";
        let asset_path = dependency_root.join("target").join(asset_name);
        let checksum = crate::pack_package(crate::PackPackageOptions {
            package_root: dependency_root,
            output: asset_path.clone(),
        })
        .unwrap()
        .checksum;
        let asset_bytes = fs::read(&asset_path).unwrap();
        let base_url = serve_once(
            "/owner/agent-tools/releases/download/v0.1.0/agent%20tools%20%26%200.1.0.etaspkg",
            asset_bytes,
        );

        let root = temp_package("github-release-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "release_base_url": "{base_url}" }}
}}
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "github-release-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "{asset_name}", checksum = "{checksum}", import = "tools" }}
"#
            ),
        )
        .unwrap();

        materialize_package(MaterializePackageOptions {
            package_root: root.clone(),
            source_config: None,
        })
        .unwrap();
        let index = vendor::read_package_index(&root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == "tools")
            .expect("GitHub release dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::GitHubRelease {
                repo,
                release,
                asset,
                asset_checksum,
                payload_checksum,
                store: Some(store),
            } => {
                assert_eq!(repo, "owner/agent-tools");
                assert_eq!(release, "v0.1.0");
                assert_eq!(asset, asset_name);
                assert_eq!(asset_checksum, &checksum);
                assert!(
                    payload_checksum.starts_with("blake3:"),
                    "{payload_checksum}"
                );
                assert!(store.starts_with(".etas/store/packages/blake3/"), "{store}");
            }
            other => panic!("expected GitHub release dependency source, got {other:?}"),
        }

        lock_package(LockPackageOptions {
            package_root: root.clone(),
        })
        .unwrap();
        let lockfile = fs::read_to_string(root.join("etas.lock")).unwrap();
        assert!(
            lockfile.contains("source = \"github-release+owner/agent-tools?release=v0.1.0&asset=agent%20tools%20%26%200.1.0.etaspkg\""),
            "{lockfile}"
        );
    }

    #[test]
    fn materialize_package_rejects_github_release_missing_metadata_artifact() {
        let dependency_root = temp_package("github-release-missing-meta-dep");
        fs::create_dir_all(dependency_root.join("src")).unwrap();
        fs::write(
            dependency_root.join("etas.toml"),
            r#"
[package]
name = "agent-tools"
version = "0.1.0"
edition = "2026"
"#,
        )
        .unwrap();
        fs::write(
            dependency_root.join("src").join("lib.es"),
            "module tools;\n",
        )
        .unwrap();
        let asset_path = dependency_root
            .join("target")
            .join("agent-tools-0.1.0.etaspkg");
        write_unchecked_etaspkg(&dependency_root, &asset_path);
        let checksum = file_blake3_checksum(&asset_path);
        let base_url = serve_once(
            "/owner/agent-tools/releases/download/v0.1.0/agent-tools-0.1.0.etaspkg",
            fs::read(&asset_path).unwrap(),
        );

        let root = temp_package("github-release-missing-meta-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "release_base_url": "{base_url}" }}
}}
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "github-release-missing-meta-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools-0.1.0.etaspkg", checksum = "{checksum}", import = "tools" }}
"#
            ),
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "package.github_release.metadata_missing" })
        );
    }

    #[test]
    fn materialize_package_rejects_github_release_stale_metadata_artifact() {
        let dependency_root = temp_package("github-release-stale-meta-dep");
        write_registry_package_payload(&dependency_root, "agent-tools", "0.1.0", "tools");
        fs::write(
            dependency_root.join("src").join("changed.es"),
            "module tools.changed;\n",
        )
        .unwrap();
        let asset_path = dependency_root
            .join("target")
            .join("agent-tools-0.1.0.etaspkg");
        write_unchecked_etaspkg(&dependency_root, &asset_path);
        let checksum = file_blake3_checksum(&asset_path);
        let base_url = serve_once(
            "/owner/agent-tools/releases/download/v0.1.0/agent-tools-0.1.0.etaspkg",
            fs::read(&asset_path).unwrap(),
        );

        let root = temp_package("github-release-stale-meta-root");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "release_base_url": "{base_url}" }}
}}
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            format!(
                r#"
[package]
name = "github-release-stale-meta-root"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = {{ package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools-0.1.0.etaspkg", checksum = "{checksum}", import = "tools" }}
"#
            ),
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let text = error.to_string();
        assert!(text.contains("source hash"), "{text}");
    }

    #[test]
    fn materialize_package_rejects_github_release_checksum_mismatch() {
        let base_url = serve_once(
            "/owner/agent-tools/releases/download/v0.1.0/agent-tools-0.1.0.etaspkg",
            b"not the package".to_vec(),
        );

        let root = temp_package("github-release-checksum-mismatch");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "release_base_url": "{base_url}" }}
}}
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-release-checksum-mismatch"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools-0.1.0.etaspkg", checksum = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", import = "tools" }
"#,
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let PackageError::Diagnostics(diagnostics) = error else {
            panic!("expected package diagnostics");
        };
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "package.github_release.checksum_mismatch"
            })
        );
    }

    #[test]
    fn pack_package_rejects_stale_metadata() {
        let root = temp_package("pack-stale-metadata");
        write_registry_package_payload(&root, "pack-stale-metadata", "0.1.0", "stale");
        fs::write(
            root.join("src").join("changed.es"),
            "module stale.changed;\n",
        )
        .unwrap();

        let error = crate::pack_package(crate::PackPackageOptions {
            package_root: root.clone(),
            output: root.join("target").join("stale.etaspkg"),
        })
        .unwrap_err();

        assert!(error.to_string().contains("source hash"), "{error}");
    }

    #[test]
    fn github_release_diagnostic_does_not_leak_token() {
        let base_url = serve_once_status(
            "/owner/agent-tools/releases/download/v0.1.0/agent-tools-0.1.0.etaspkg",
            500,
            b"nope".to_vec(),
        );
        let root = temp_package("github-release-token-redaction");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "github": {{ "release_base_url": "{base_url}", "token": "super-secret-token" }}
}}
"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("etas.toml"),
            r#"
[package]
name = "github-release-token-redaction"
version = "0.1.0"
edition = "2026"

[dependencies]
tools = { package = "agent-tools", version = "0.1", github = "owner/agent-tools", release = "v0.1.0", asset = "agent-tools-0.1.0.etaspkg", checksum = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", import = "tools" }
"#,
        )
        .unwrap();

        let error = materialize_package(MaterializePackageOptions {
            package_root: root,
            source_config: None,
        })
        .unwrap_err();
        let text = error.to_string();
        assert!(!text.contains("super-secret-token"), "{text}");
    }

    fn temp_package(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("etas-package-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn seal_package_metadata(package_root: &Path) {
        let text =
            fs::read_to_string(package_root.join(".etas").join("package-index.json")).unwrap();
        let index = serde_json::from_str::<metadata::PackageIndex>(&text).unwrap();
        metadata_artifact::write_fixture_package_metadata_artifact(package_root, &index).unwrap();
    }

    fn init_local_registry_fixture(root: &Path) {
        fs::create_dir_all(root.join("index")).unwrap();
        fs::create_dir_all(root.join("packages")).unwrap();
    }

    fn write_source_config(root: &Path, registry_name: &str, registry_path: &str) {
        fs::create_dir_all(root.join(".etas")).unwrap();
        fs::write(
            root.join(".etas").join("source-config.json"),
            format!(
                r#"{{
  "registries": {{
    "{registry_name}": {{ "path": "{registry_path}" }}
  }}
}}
"#
            ),
        )
        .unwrap();
    }

    fn write_registry_package_payload(
        package_root: &Path,
        package_name: &str,
        version: &str,
        module: &str,
    ) {
        fs::create_dir_all(package_root.join("src")).unwrap();
        fs::create_dir_all(package_root.join(".etas")).unwrap();
        fs::write(
            package_root.join("etas.toml"),
            format!(
                r#"
[package]
name = "{package_name}"
version = "{version}"
edition = "2026"
"#
            ),
        )
        .unwrap();
        fs::write(
            package_root.join("src").join("lib.es"),
            format!("module {module};\npublic flow answer() -> i32 {{ return 42; }}\n"),
        )
        .unwrap();
        fs::write(
            package_root.join(".etas").join("package-index.json"),
            format!(
                r#"{{
  "version": 1,
  "package": {{ "name": "{package_name}", "version": "{version}", "edition": "2026" }},
  "public_metadata": {{
    "modules": [
      {{ "id": 1, "path": ["{module}"], "exports": [] }}
    ]
  }}
}}
"#
            ),
        )
        .unwrap();
        seal_package_metadata(package_root);
    }

    fn write_stored_package_payload(
        root: &Path,
        package_name: &str,
        version: &str,
        module: &str,
    ) -> (String, String) {
        let package_root = root
            .join("fixtures")
            .join("payloads")
            .join(format!("{package_name}-{version}"));
        write_registry_package_payload(&package_root, package_name, version, module);
        let checksum = vendor::path_package_payload_checksum(&package_root).unwrap();
        let store_path = store::store_package_payload(root, &package_root, &checksum).unwrap();
        (store_path, checksum)
    }

    fn write_local_registry_index(
        registry: &Path,
        package_name: &str,
        version: &str,
        checksum: &str,
    ) {
        fs::write(
            registry.join("index").join(format!("{package_name}.json")),
            format!(
                r#"{{
  "versions": [
    {{
      "version": "{version}",
      "path": "packages/{package_name}/{version}",
      "checksum": "{checksum}"
    }}
  ]
}}
"#
            ),
        )
        .unwrap();
    }

    fn git(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .unwrap();
        assert!(
            status.success(),
            "git -C {} {} exited with {status}",
            root.display(),
            args.join(" ")
        );
    }

    fn git_output(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git -C {} {} failed: {}",
            root.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    fn write_builtin_std_lockfile(root: &Path) {
        let manifest = manifest::read_manifest(root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "std"
version = "0.1.0"
source = "builtin"
checksum = "builtin:std"
import_root = "std"
"#
            ),
        )
        .unwrap();
    }

    fn serve_once(path: &'static str, body: Vec<u8>) -> String {
        serve_once_status(path, 200, body)
    }

    fn serve_once_status(path: &'static str, status: u16, body: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 4096];
            let read = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with(&format!("GET {path} ")), "{request}");
            let header = format!(
                "HTTP/1.1 {status} TEST\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
        });
        format!("http://{addr}")
    }

    fn write_unchecked_etaspkg(package_root: &Path, output: &Path) {
        let mut files = Vec::new();
        files.push(package_root.join("etas.toml"));
        collect_test_files(&package_root.join("src"), &mut files);
        let metadata = package_root.join(".etas").join("package.etasmeta");
        if metadata.is_file() {
            files.push(metadata);
        }
        files.sort();

        let mut tar = Vec::new();
        for file in files {
            let relative = file.strip_prefix(package_root).unwrap();
            append_test_tar_file(&mut tar, relative, &file);
        }
        tar.extend([0u8; 512]);
        tar.extend([0u8; 512]);
        let compressed = zstd::stream::encode_all(tar.as_slice(), 19).unwrap();
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, compressed).unwrap();
    }

    fn collect_test_files(root: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_test_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }

    fn append_test_tar_file(tar: &mut Vec<u8>, relative: &Path, file: &Path) {
        let bytes = fs::read(file).unwrap();
        let path = relative.to_string_lossy().replace('\\', "/");
        let mut header = [0u8; 512];
        write_test_tar_bytes(&mut header[0..100], path.as_bytes());
        write_test_tar_octal(&mut header[100..108], 0o644);
        write_test_tar_octal(&mut header[108..116], 0);
        write_test_tar_octal(&mut header[116..124], 0);
        write_test_tar_octal(&mut header[124..136], bytes.len() as u64);
        write_test_tar_octal(&mut header[136..148], 0);
        for byte in &mut header[148..156] {
            *byte = b' ';
        }
        header[156] = b'0';
        write_test_tar_bytes(&mut header[257..263], b"ustar\0");
        write_test_tar_bytes(&mut header[263..265], b"00");
        let checksum = header.iter().map(|byte| u32::from(*byte)).sum::<u32>();
        write_test_tar_bytes(
            &mut header[148..156],
            format!("{checksum:06o}\0 ").as_bytes(),
        );
        tar.extend(header);
        tar.extend(&bytes);
        tar.extend(std::iter::repeat_n(0u8, (512 - (bytes.len() % 512)) % 512));
    }

    fn write_test_tar_bytes(target: &mut [u8], bytes: &[u8]) {
        let len = bytes.len().min(target.len());
        target[..len].copy_from_slice(&bytes[..len]);
    }

    fn write_test_tar_octal(target: &mut [u8], value: u64) {
        write_test_tar_bytes(
            target,
            format!("{value:0width$o}\0", width = target.len() - 1).as_bytes(),
        );
    }

    fn file_blake3_checksum(path: &Path) -> String {
        format!("blake3:{}", blake3::hash(&fs::read(path).unwrap()).to_hex())
    }

    fn write_lockfile_from_package_index(
        root: &Path,
        package_name: &str,
        package_version: &str,
        source: &str,
        import_root: &str,
    ) {
        let manifest = manifest::read_manifest(root).unwrap();
        let fingerprint = lockfile::manifest_dependency_fingerprint(&manifest);
        let index = vendor::read_package_index(root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| {
                dependency.identity.name == package_name
                    && dependency.identity.version == package_version
                    && dependency.import_root == import_root
            })
            .expect("test package index should contain dependency");
        let checksum = vendor::dependency_content_checksum(dependency);
        fs::write(
            root.join("etas.lock"),
            format!(
                r#"
version = 1
manifest_fingerprint = "{fingerprint}"

[[package]]
name = "{package_name}"
version = "{package_version}"
source = "{source}"
checksum = "{checksum}"
import_root = "{import_root}"
"#
            ),
        )
        .unwrap();
    }

    fn materialized_dependency_payload_checksum(root: &Path, import_root: &str) -> String {
        let index = vendor::read_package_index(root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == import_root)
            .expect("test dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::Vendor { checksum, .. } => checksum.clone(),
            other => panic!("expected vendored dependency, got {other:?}"),
        }
    }

    fn materialized_dependency_store_path(root: &Path, import_root: &str) -> String {
        let index = vendor::read_package_index(root).unwrap().unwrap();
        let dependency = index
            .dependencies
            .iter()
            .find(|dependency| dependency.import_root == import_root)
            .expect("test dependency should be materialized");
        match &dependency.source {
            metadata::ResolvedDependencySource::Vendor {
                store: Some(store), ..
            }
            | metadata::ResolvedDependencySource::Registry {
                store: Some(store), ..
            } => store.clone(),
            other => panic!("expected materialized dependency with store, got {other:?}"),
        }
    }
}
