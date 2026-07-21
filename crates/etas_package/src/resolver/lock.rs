use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    PackageDiagnostic, PackageError, ResolvedDependency, ResolvedDependencySource, lockfile,
    lockfile::{LockedPackage, Lockfile},
    manifest, metadata, vendor,
};

use super::version::version_satisfies;

#[derive(Clone, Debug)]
pub struct LockPackageOptions {
    pub package_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LockedPackageResult {
    pub package_root: PathBuf,
    pub lockfile: Lockfile,
}

pub fn lock_package(options: LockPackageOptions) -> Result<LockedPackageResult, PackageError> {
    let package_root = canonical_package_root(&options.package_root)?;
    let manifest = manifest::read_manifest(&package_root)?;
    let package_index = vendor::read_package_index(&package_root)?;
    let mut diagnostics = Vec::new();
    let mut packages = BTreeMap::<String, LockedPackage>::new();

    for (dependency_name, spec) in &manifest.dependencies {
        let import_root = spec.import_root(dependency_name);
        if metadata::is_builtin_package(dependency_name) {
            let package_name = spec.package_name(dependency_name);
            match locked_builtin_dependency(&package_root, dependency_name, spec, package_name) {
                Ok(locked) => insert_locked_package(&mut packages, locked, &mut diagnostics),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        if let Some(requirement) = spec.version()
            && let Err(message) = version_satisfies("0.0.0", requirement)
        {
            diagnostics.push(PackageDiagnostic::new(
                "package.manifest.invalid_version_requirement",
                format!(
                    "dependency `{dependency_name}` has invalid version requirement `{requirement}`: {message}"
                ),
                Some(package_root.join("etas.toml")),
            ));
            continue;
        }

        let Some(index_dependency) = package_index
            .as_ref()
            .and_then(|index| index_dependency(index, dependency_name, &import_root, spec))
        else {
            diagnostics.push(PackageDiagnostic::new(
                "package.resolver.dependency_unresolved",
                format!(
                    "dependency `{dependency_name}` with import root `{import_root}` ({}) is not resolved in .etas/package-index.json; run `etas pkg update {}` before locking",
                    dependency_source_description(spec),
                    package_root.display()
                ),
                Some(package_root.join("etas.toml")),
            ));
            continue;
        };

        insert_index_dependency(&mut packages, index_dependency, &mut diagnostics);
    }

    if !diagnostics.is_empty() {
        return Err(PackageError::Diagnostics(diagnostics));
    }

    let mut packages = packages.into_values().collect::<Vec<_>>();
    packages.sort_by(|left, right| {
        left.import_root
            .cmp(&right.import_root)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.version.cmp(&right.version))
    });

    let lockfile = Lockfile {
        version: 1,
        manifest_fingerprint: Some(lockfile::manifest_dependency_fingerprint(&manifest)),
        packages,
    };
    lockfile::write_lockfile(&package_root, &lockfile)?;

    Ok(LockedPackageResult {
        package_root,
        lockfile,
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

fn dependency_source_description(spec: &manifest::DependencySpec) -> String {
    match spec {
        manifest::DependencySpec::Version(version) => {
            format!("registry version requirement `{version}`")
        }
        manifest::DependencySpec::Detailed(detail) => {
            if let Some(path) = &detail.path {
                format!("path `{}`", path.display())
            } else if let Some(github) = &detail.github {
                if let Some(release) = &detail.release {
                    format!("GitHub release `{github}` `{release}`")
                } else {
                    format!("GitHub repository `{github}`")
                }
            } else if let Some(git) = &detail.git {
                format!("git repository `{git}`")
            } else if let Some(registry) = &detail.registry {
                format!("registry `{registry}`")
            } else if let Some(version) = &detail.version {
                format!("registry version requirement `{version}`")
            } else {
                "unspecified source".to_owned()
            }
        }
    }
}

fn locked_builtin_dependency(
    package_root: &Path,
    dependency_name: &str,
    spec: &manifest::DependencySpec,
    package_name: &str,
) -> Result<LockedPackage, PackageDiagnostic> {
    let version = metadata::BUILTIN_STD_VERSION.to_owned();
    if let Some(requirement) = spec.version() {
        match version_satisfies(&version, requirement) {
            Ok(true) => {}
            Ok(false) => {
                return Err(PackageDiagnostic::new(
                    "package.lockfile.version_mismatch",
                    format!(
                        "builtin dependency `{dependency_name}` version `{version}` does not satisfy manifest requirement `{requirement}`"
                    ),
                    Some(package_root.join("etas.toml")),
                ));
            }
            Err(message) => {
                return Err(PackageDiagnostic::new(
                    "package.manifest.invalid_version_requirement",
                    format!(
                        "dependency `{dependency_name}` has invalid version requirement `{requirement}`: {message}"
                    ),
                    Some(package_root.join("etas.toml")),
                ));
            }
        }
    }
    Ok(LockedPackage {
        name: package_name.to_owned(),
        version,
        source: "builtin".to_owned(),
        checksum: "builtin:std".to_owned(),
        dependencies: Vec::new(),
        import_root: spec.import_root(dependency_name),
        metadata_fingerprint: Some("builtin:std".to_owned()),
    })
}

fn index_dependency<'a>(
    index: &'a metadata::PackageIndex,
    dependency_name: &str,
    import_root: &str,
    spec: &manifest::DependencySpec,
) -> Option<&'a ResolvedDependency> {
    let package_name = spec.package_name_constraint(dependency_name);
    index.dependencies.iter().find(|dependency| {
        dependency.import_root == import_root
            && package_name
                .map(|package_name| dependency.identity.name == package_name)
                .unwrap_or(true)
            && spec
                .version()
                .map(|requirement| {
                    version_satisfies(&dependency.identity.version, requirement).unwrap_or(false)
                })
                .unwrap_or(true)
    })
}

fn locked_index_dependency(dependency: &ResolvedDependency) -> LockedPackage {
    let metadata_fingerprint = match &dependency.source {
        ResolvedDependencySource::Builtin { .. } => Some("builtin:std".to_owned()),
        _ => dependency.public_metadata.fingerprint.clone(),
    };
    LockedPackage {
        name: dependency.identity.name.clone(),
        version: dependency.identity.version.clone(),
        source: lock_source(&dependency.source),
        checksum: vendor::dependency_lock_checksum(dependency),
        dependencies: dependency
            .dependencies
            .iter()
            .map(dependency_lock_ref)
            .collect(),
        import_root: dependency.import_root.clone(),
        metadata_fingerprint,
    }
}

fn insert_index_dependency(
    packages: &mut BTreeMap<String, LockedPackage>,
    dependency: &ResolvedDependency,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    insert_locked_package(packages, locked_index_dependency(dependency), diagnostics);
    for child in &dependency.dependencies {
        insert_index_dependency(packages, child, diagnostics);
    }
}

fn insert_locked_package(
    packages: &mut BTreeMap<String, LockedPackage>,
    locked: LockedPackage,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    let key = locked_package_key(&locked);
    if let Some(existing) = packages.get(&key) {
        if existing != &locked {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.dependency_conflict",
                format!(
                    "resolved package `{}` appears with conflicting source/checksum metadata",
                    locked_dependency_ref(&locked)
                ),
                None,
            ));
        }
        return;
    }
    packages.insert(key, locked);
}

fn locked_package_key(locked: &LockedPackage) -> String {
    format!("{}|{}|{}", locked.import_root, locked.name, locked.version)
}

fn dependency_lock_ref(dependency: &ResolvedDependency) -> String {
    format!(
        "{}|{}|{}",
        dependency.import_root, dependency.identity.name, dependency.identity.version
    )
}

fn locked_dependency_ref(locked: &LockedPackage) -> String {
    format!("{}|{}|{}", locked.import_root, locked.name, locked.version)
}

fn lock_source(source: &ResolvedDependencySource) -> String {
    match source {
        ResolvedDependencySource::Builtin { .. } => "builtin".to_owned(),
        ResolvedDependencySource::Registry { registry, .. } => format!("registry+{registry}"),
        ResolvedDependencySource::Git { url, rev, .. } => format!("git+{url}?rev={rev}"),
        ResolvedDependencySource::GitHubClone { repo, rev, .. } => {
            format!("github+{repo}?rev={rev}")
        }
        ResolvedDependencySource::GitHubRelease {
            repo,
            release,
            asset,
            ..
        } => format!(
            "github-release+{repo}?release={}&asset={}",
            encode_lock_value(release),
            encode_lock_value(asset)
        ),
        ResolvedDependencySource::Path { path, .. } => format!("path+{path}"),
        ResolvedDependencySource::Vendor { path, .. } => format!("vendor+{path}"),
    }
}

fn encode_lock_value(value: &str) -> String {
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
