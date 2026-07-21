use std::{collections::BTreeSet, path::Path};

use crate::{
    PackageDiagnostic, PackageEnvironmentMetadata, PackageError, PackageIdentity,
    PackageToolBindingMetadata, ResolvedDependency, lockfile,
    lockfile::{LockedPackage, Lockfile},
    manifest, metadata, vendor,
};

use super::{
    lock_policy::lockfile_package,
    source::{resolved_source_from_lockfile, spec_source_requires_exact_git_rev},
    version::version_satisfies,
};

pub(super) fn build_environment(
    package_root: &Path,
    manifest: &manifest::Manifest,
    lockfile: &Lockfile,
    index: Option<&metadata::PackageIndex>,
) -> Result<PackageEnvironmentMetadata, PackageError> {
    let mut diagnostics = Vec::new();
    let mut dependencies = Vec::new();

    for (name, spec) in &manifest.dependencies {
        let import_root = spec.import_root(name);
        if metadata::is_builtin_package(name) {
            let package_name = spec.package_name(name);
            let Some(locked) = lockfile_package(lockfile, package_name, &import_root) else {
                diagnostics.push(PackageDiagnostic::new(
                    "package.lockfile.dependency_missing",
                    format!(
                        "dependency `{name}` with import root `{import_root}` is missing from etas.lock"
                    ),
                    Some(package_root.join("etas.lock")),
                ));
                continue;
            };
            if locked.source != "builtin" {
                diagnostics.push(PackageDiagnostic::new(
                    "package.lockfile.invalid_source",
                    format!(
                        "builtin dependency `{name}` must use source `builtin`, not `{}`",
                        locked.source
                    ),
                    Some(package_root.join("etas.lock")),
                ));
                continue;
            }
            validate_version_requirement(
                &mut diagnostics,
                package_root,
                name,
                spec.version(),
                &locked.version,
            );
            dependencies.push(metadata::builtin_std_dependency(
                locked.version.clone(),
                manifest.package.edition.clone(),
            ));
            continue;
        }

        let Some(locked) = lockfile_package_for_spec(lockfile, spec, name, &import_root) else {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.dependency_missing",
                format!(
                    "dependency `{name}` with import root `{import_root}` is missing from etas.lock"
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        };
        if spec_source_requires_exact_git_rev(spec) && !locked.source.contains("rev=") {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.git_rev_missing",
                format!("git dependency `{name}` must be locked to an exact revision in etas.lock"),
                Some(package_root.join("etas.lock")),
            ));
        }
        validate_version_requirement(
            &mut diagnostics,
            package_root,
            name,
            spec.version(),
            &locked.version,
        );

        let Some(index_dependency) =
            index.and_then(|index| dependency_metadata(index, spec, name, &import_root, locked))
        else {
            diagnostics.push(PackageDiagnostic::new(
                "package.resolver.dependency_unresolved",
                format!(
                    "dependency `{name}` with version `{}`, import root `{import_root}`, and lock source `{}` is not resolved in .etas/package-index.json; run `etas pkg update {}` to materialize dependency metadata",
                    locked.version,
                    locked.source,
                    package_root.display()
                ),
                Some(package_root.join("etas.toml")),
            ));
            continue;
        };
        let Some(_source) = resolved_source_from_lockfile(locked) else {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.invalid_source",
                format!(
                    "dependency `{name}` has unsupported lockfile source `{}`",
                    locked.source
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        };
        let expected_source = lock_source_from_resolved_dependency(&index_dependency.source);
        if locked.source != expected_source {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.source_mismatch",
                format!(
                    "dependency `{name}` lockfile source `{}` does not match resolved source `{expected_source}`",
                    locked.source
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        }
        if let Err(diagnostic) =
            validate_resolved_payload(package_root, name, &index_dependency.source)
        {
            diagnostics.push(diagnostic);
            continue;
        }
        validate_locked_dependency_edges(&mut diagnostics, package_root, locked, index_dependency);
        validate_transitive_lock_graph(
            &mut diagnostics,
            package_root,
            lockfile,
            name,
            index_dependency,
            &mut BTreeSet::new(),
        );
        if vendor::dependency_lock_checksum(index_dependency) != locked.checksum {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.checksum_mismatch",
                format!("dependency `{name}` metadata content checksum does not match etas.lock"),
                Some(package_root.join(".etas").join("package-index.json")),
            ));
            continue;
        }
        dependencies.push(index_dependency.clone());
    }

    if !diagnostics.is_empty() {
        return Err(PackageError::Diagnostics(diagnostics));
    }

    let tool_bindings = manifest
        .bindings
        .tools
        .iter()
        .map(|(tool, binding)| PackageToolBindingMetadata {
            tool: tool.clone(),
            kind: binding.kind.clone(),
            provider: binding
                .provider_name()
                .expect("manifest validation checked provider"),
            effect_row: binding.effects.clone(),
            action_row: binding.actions.clone(),
        })
        .chain(
            index
                .into_iter()
                .flat_map(|index| index.tool_bindings.iter().cloned()),
        )
        .chain(
            dependencies
                .iter()
                .flat_map(|dependency| dependency.tool_bindings.iter().cloned()),
        )
        .collect::<Vec<_>>();

    let mut external_modules = index
        .map(|index| index.external_modules.clone())
        .unwrap_or_default();
    for module in &external_modules {
        if module.package.is_none() {
            diagnostics.push(PackageDiagnostic::new(
                "package.metadata.external_module_owner_missing",
                format!(
                    "external module `{}` in .etas/package-index.json does not declare its owning package",
                    module.path.join(".")
                ),
                Some(package_root.join(".etas").join("package-index.json")),
            ));
        }
    }
    for dependency in &dependencies {
        external_modules.extend(
            dependency
                .public_metadata
                .modules
                .iter()
                .cloned()
                .map(|module| external_module_for_dependency(module, dependency)),
        );
    }
    if !diagnostics.is_empty() {
        return Err(PackageError::Diagnostics(diagnostics));
    }
    external_modules.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.id.cmp(&right.id))
    });
    external_modules.dedup_by(|left, right| {
        left.id == right.id && left.path == right.path && left.package == right.package
    });

    let public_metadata = index
        .map(|index| index.public_metadata.clone())
        .unwrap_or_default();
    let mut effect_metadata = index
        .map(|index| index.effect_metadata.clone())
        .unwrap_or_default();
    for dependency in &dependencies {
        effect_metadata
            .tags
            .extend(dependency.effect_metadata.tags.clone());
        effect_metadata
            .extensions
            .extend(dependency.effect_metadata.extensions.clone());
    }

    let metadata_fingerprint = environment_fingerprint(
        manifest,
        lockfile,
        &dependencies,
        &external_modules,
        &tool_bindings,
        &effect_metadata,
    );

    Ok(PackageEnvironmentMetadata {
        current_package: Some(PackageIdentity::current(manifest)),
        dependencies,
        external_modules,
        public_metadata,
        effect_metadata,
        tool_bindings,
        metadata_fingerprint,
    })
}

fn validate_transitive_lock_graph(
    diagnostics: &mut Vec<PackageDiagnostic>,
    package_root: &Path,
    lockfile: &Lockfile,
    root_dependency_name: &str,
    dependency: &ResolvedDependency,
    visited: &mut BTreeSet<String>,
) {
    let key = dependency_lock_ref(dependency);
    if !visited.insert(key) {
        return;
    }
    for child in &dependency.dependencies {
        let Some(locked) = lockfile_package(lockfile, &child.identity.name, &child.import_root)
        else {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.transitive_dependency_missing",
                format!(
                    "dependency `{root_dependency_name}` requires transitive dependency `{}` with import root `{}`, but etas.lock does not contain it",
                    child.identity.name, child.import_root
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        };
        if locked.version != child.identity.version {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.transitive_version_mismatch",
                format!(
                    "transitive dependency `{}` with import root `{}` is locked at version `{}` but metadata requires `{}`",
                    child.identity.name, child.import_root, locked.version, child.identity.version
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        }
        let expected_source = lock_source_from_resolved_dependency(&child.source);
        if locked.source != expected_source {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.transitive_source_mismatch",
                format!(
                    "transitive dependency `{}` with import root `{}` lockfile source `{}` does not match resolved source `{expected_source}`",
                    child.identity.name, child.import_root, locked.source
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        }
        validate_locked_dependency_edges(diagnostics, package_root, locked, child);
        let expected_checksum = vendor::dependency_lock_checksum(child);
        if locked.checksum != expected_checksum {
            diagnostics.push(PackageDiagnostic::new(
                "package.lockfile.transitive_checksum_mismatch",
                format!(
                    "transitive dependency `{}` with import root `{}` metadata content checksum does not match etas.lock",
                    child.identity.name, child.import_root
                ),
                Some(package_root.join("etas.lock")),
            ));
            continue;
        }
        validate_transitive_lock_graph(
            diagnostics,
            package_root,
            lockfile,
            root_dependency_name,
            child,
            visited,
        );
    }
}

fn dependency_lock_ref(dependency: &ResolvedDependency) -> String {
    format!(
        "{}|{}|{}",
        dependency.import_root, dependency.identity.name, dependency.identity.version
    )
}

fn validate_locked_dependency_edges(
    diagnostics: &mut Vec<PackageDiagnostic>,
    package_root: &Path,
    locked: &LockedPackage,
    dependency: &ResolvedDependency,
) {
    let mut expected = dependency
        .dependencies
        .iter()
        .map(dependency_lock_ref)
        .collect::<Vec<_>>();
    expected.sort();
    let mut actual = locked.dependencies.clone();
    actual.sort();
    if actual != expected {
        diagnostics.push(PackageDiagnostic::new(
            "package.lockfile.dependency_edges_mismatch",
            format!(
                "locked dependency `{}` with import root `{}` has dependency edges {:?}, but metadata requires {:?}",
                locked.name, locked.import_root, actual, expected
            ),
            Some(package_root.join("etas.lock")),
        ));
    }
}

fn validate_version_requirement(
    diagnostics: &mut Vec<PackageDiagnostic>,
    package_root: &Path,
    dependency_name: &str,
    requirement: Option<&str>,
    locked_version: &str,
) {
    let Some(requirement) = requirement else {
        return;
    };
    match version_satisfies(locked_version, requirement) {
        Ok(true) => {}
        Ok(false) => diagnostics.push(PackageDiagnostic::new(
            "package.lockfile.version_mismatch",
            format!(
                "locked dependency `{dependency_name}` version `{locked_version}` does not satisfy manifest requirement `{requirement}`"
            ),
            Some(package_root.join("etas.lock")),
        )),
        Err(message) => diagnostics.push(PackageDiagnostic::new(
            "package.manifest.invalid_version_requirement",
            format!(
                "dependency `{dependency_name}` has invalid version requirement `{requirement}`: {message}"
            ),
            Some(package_root.join("etas.toml")),
        )),
    }
}

fn environment_fingerprint(
    manifest: &manifest::Manifest,
    lockfile: &Lockfile,
    dependencies: &[ResolvedDependency],
    external_modules: &[metadata::PackageExternalModuleMetadata],
    tool_bindings: &[PackageToolBindingMetadata],
    effect_metadata: &metadata::PackageEffectMetadata,
) -> String {
    let mut fingerprint_parts = vec![
        "etas-package-env:v2".to_owned(),
        manifest.package.name.clone(),
        manifest.package.version.clone(),
        manifest.package.edition.clone(),
        lockfile
            .manifest_fingerprint
            .clone()
            .unwrap_or_else(|| lockfile::manifest_dependency_fingerprint(manifest)),
    ];
    for dependency in dependencies {
        fingerprint_parts.push(format!(
            "dep:{}:{}:{}",
            dependency.import_root, dependency.identity.name, dependency.identity.version
        ));
        fingerprint_parts.push(format!(
            "dep-source-checksum:{}:{}",
            dependency.import_root,
            dependency_source_checksum(dependency)
        ));
        if let Some(fingerprint) = &dependency.public_metadata.fingerprint {
            fingerprint_parts.push(format!(
                "dep-public-metadata:{}:{fingerprint}",
                dependency.import_root
            ));
        }
    }
    for module in external_modules {
        fingerprint_parts.push(format!("module:{}:{}", module.id, module.path.join(".")));
    }
    for binding in tool_bindings {
        fingerprint_parts.push(format!(
            "tool:{}:{}:{}:{}",
            binding.tool,
            binding.kind,
            binding.provider,
            binding.effect_row.join(",")
        ));
    }
    for tag in &effect_metadata.tags {
        fingerprint_parts.push(format!(
            "effect-tag:{}:{}",
            tag.path.join("."),
            tag.runtime_requirement.clone().unwrap_or_default()
        ));
    }
    for extension in &effect_metadata.extensions {
        fingerprint_parts.push(format!(
            "effect-extension:{}:{}",
            extension.child.join("."),
            extension.parent.join(".")
        ));
    }
    fingerprint_parts.join("|")
}

fn dependency_source_checksum(dependency: &ResolvedDependency) -> &str {
    match &dependency.source {
        metadata::ResolvedDependencySource::Builtin { checksum }
        | metadata::ResolvedDependencySource::Registry { checksum, .. }
        | metadata::ResolvedDependencySource::Git { checksum, .. }
        | metadata::ResolvedDependencySource::GitHubClone { checksum, .. }
        | metadata::ResolvedDependencySource::GitHubRelease {
            payload_checksum: checksum,
            ..
        }
        | metadata::ResolvedDependencySource::Path { checksum, .. }
        | metadata::ResolvedDependencySource::Vendor { checksum, .. } => checksum,
    }
}

fn lock_source_from_resolved_dependency(source: &metadata::ResolvedDependencySource) -> String {
    match source {
        metadata::ResolvedDependencySource::Builtin { .. } => "builtin".to_owned(),
        metadata::ResolvedDependencySource::Registry { registry, .. } => {
            format!("registry+{registry}")
        }
        metadata::ResolvedDependencySource::Git { url, rev, .. } => format!("git+{url}?rev={rev}"),
        metadata::ResolvedDependencySource::GitHubClone { repo, rev, .. } => {
            format!("github+{repo}?rev={rev}")
        }
        metadata::ResolvedDependencySource::GitHubRelease {
            repo,
            release,
            asset,
            ..
        } => format!(
            "github-release+{repo}?release={}&asset={}",
            encode_lock_value(release),
            encode_lock_value(asset)
        ),
        metadata::ResolvedDependencySource::Path { path, .. } => format!("path+{path}"),
        metadata::ResolvedDependencySource::Vendor { path, .. } => format!("vendor+{path}"),
    }
}

fn validate_resolved_payload(
    package_root: &Path,
    dependency_name: &str,
    source: &metadata::ResolvedDependencySource,
) -> Result<(), PackageDiagnostic> {
    match source {
        metadata::ResolvedDependencySource::Vendor {
            path,
            checksum,
            store,
        } => {
            let payload_root = package_root.join(path);
            let actual =
                vendor::with_vendor_lock(package_root, || {
                    vendor::path_package_payload_checksum(&payload_root)
                })
                .map_err(|error| {
                    PackageDiagnostic::new(
                        "package.lockfile.payload_unreadable",
                        format!(
                            "vendored dependency `{dependency_name}` payload at `{}` cannot be checksummed: {error}",
                            payload_root.display()
                        ),
                        Some(payload_root.clone()),
                    )
                })?;
            if actual != *checksum {
                return Err(PackageDiagnostic::new(
                    "package.lockfile.payload_checksum_mismatch",
                    format!(
                        "vendored dependency `{dependency_name}` payload checksum `{actual}` does not match locked checksum `{checksum}`"
                    ),
                    Some(payload_root),
                ));
            }
            if let Some(store_path) = store {
                let store_root = package_root.join(store_path);
                let store_actual =
                    vendor::path_package_payload_checksum(&store_root).map_err(|error| {
                        PackageDiagnostic::new(
                            "package.lockfile.store_payload_unreadable",
                            format!(
                                "stored dependency `{dependency_name}` payload at `{}` cannot be checksummed: {error}",
                                store_root.display()
                            ),
                            Some(store_root.clone()),
                        )
                    })?;
                if store_actual != *checksum {
                    return Err(PackageDiagnostic::new(
                        "package.lockfile.store_payload_checksum_mismatch",
                        format!(
                            "stored dependency `{dependency_name}` payload checksum `{store_actual}` does not match locked checksum `{checksum}`"
                        ),
                        Some(store_root),
                    ));
                }
            }
        }
        metadata::ResolvedDependencySource::Path { path, checksum } => {
            let payload_root = package_root.join(path);
            let actual = vendor::path_package_payload_checksum(&payload_root).map_err(|error| {
                PackageDiagnostic::new(
                    "package.lockfile.payload_unreadable",
                    format!(
                        "path dependency `{dependency_name}` payload at `{}` cannot be checksummed: {error}",
                        payload_root.display()
                    ),
                    Some(payload_root.clone()),
                )
            })?;
            if actual != *checksum {
                return Err(PackageDiagnostic::new(
                    "package.lockfile.payload_checksum_mismatch",
                    format!(
                        "path dependency `{dependency_name}` payload checksum `{actual}` does not match locked checksum `{checksum}`"
                    ),
                    Some(payload_root),
                ));
            }
        }
        metadata::ResolvedDependencySource::Registry {
            checksum,
            store: Some(store_path),
            ..
        }
        | metadata::ResolvedDependencySource::Git {
            checksum,
            store: Some(store_path),
            ..
        }
        | metadata::ResolvedDependencySource::GitHubClone {
            checksum,
            store: Some(store_path),
            ..
        }
        | metadata::ResolvedDependencySource::GitHubRelease {
            payload_checksum: checksum,
            store: Some(store_path),
            ..
        } => {
            validate_store_payload(package_root, dependency_name, checksum, store_path)?;
        }
        metadata::ResolvedDependencySource::Registry { .. }
        | metadata::ResolvedDependencySource::Git { .. }
        | metadata::ResolvedDependencySource::GitHubClone { .. }
        | metadata::ResolvedDependencySource::GitHubRelease { .. } => {
            return Err(PackageDiagnostic::new(
                "package.lockfile.store_missing",
                format!(
                    "dependency `{dependency_name}` must include a materialized content-addressed store path for registry/git/github sources"
                ),
                Some(package_root.join(".etas").join("package-index.json")),
            ));
        }
        metadata::ResolvedDependencySource::Builtin { .. } => {}
    }
    Ok(())
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

fn validate_store_payload(
    package_root: &Path,
    dependency_name: &str,
    checksum: &str,
    store_path: &str,
) -> Result<(), PackageDiagnostic> {
    let expected_store_path =
        crate::store::package_store_relative_path(checksum).map_err(|error| {
            PackageDiagnostic::new(
                "package.lockfile.invalid_store_checksum",
                format!("dependency `{dependency_name}` store checksum is invalid: {error}"),
                Some(package_root.join(".etas").join("package-index.json")),
            )
        })?;
    if store_path != expected_store_path {
        return Err(PackageDiagnostic::new(
            "package.lockfile.store_path_mismatch",
            format!(
                "dependency `{dependency_name}` store path `{store_path}` does not match content-addressed path `{expected_store_path}`"
            ),
            Some(package_root.join(".etas").join("package-index.json")),
        ));
    }
    let store_root = package_root.join(store_path);
    let store_actual = vendor::path_package_payload_checksum(&store_root).map_err(|error| {
        PackageDiagnostic::new(
            "package.lockfile.store_payload_unreadable",
            format!(
                "stored dependency `{dependency_name}` payload at `{}` cannot be checksummed: {error}",
                store_root.display()
            ),
            Some(store_root.clone()),
        )
    })?;
    if store_actual != checksum {
        return Err(PackageDiagnostic::new(
            "package.lockfile.store_payload_checksum_mismatch",
            format!(
                "stored dependency `{dependency_name}` payload checksum `{store_actual}` does not match locked checksum `{checksum}`"
            ),
            Some(store_root),
        ));
    }
    Ok(())
}

fn external_module_for_dependency(
    mut module: metadata::PackageExternalModuleMetadata,
    dependency: &ResolvedDependency,
) -> metadata::PackageExternalModuleMetadata {
    module.package = Some(metadata::PackageExternalModuleOwnerMetadata {
        identity: dependency.identity.clone(),
        import_root: dependency.import_root.clone(),
    });
    module
}

fn dependency_metadata<'a>(
    index: &'a metadata::PackageIndex,
    spec: &manifest::DependencySpec,
    dependency_name: &str,
    import_root: &str,
    locked: &LockedPackage,
) -> Option<&'a ResolvedDependency> {
    let package_name = spec.package_name_constraint(dependency_name);
    index.dependencies.iter().find(|dependency| {
        dependency.import_root == import_root
            && package_name
                .map(|package_name| dependency.identity.name == package_name)
                .unwrap_or(true)
            && dependency.identity.version == locked.version
    })
}

fn lockfile_package_for_spec<'a>(
    lockfile: &'a Lockfile,
    spec: &manifest::DependencySpec,
    dependency_name: &str,
    import_root: &str,
) -> Option<&'a LockedPackage> {
    if let Some(package_name) = spec.package_name_constraint(dependency_name) {
        return lockfile_package(lockfile, package_name, import_root);
    }
    lockfile
        .packages
        .iter()
        .find(|package| package.import_root == import_root)
}
