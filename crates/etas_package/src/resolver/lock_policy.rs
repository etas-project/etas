use std::path::Path;

use crate::{
    PackageDiagnostic, PackageError, lockfile,
    lockfile::{LockedPackage, Lockfile},
    manifest,
};

pub(super) fn resolve_lockfile(
    package_root: &Path,
    manifest: &manifest::Manifest,
) -> Result<Lockfile, PackageError> {
    let expected_fingerprint = lockfile::manifest_dependency_fingerprint(manifest);
    let lockfile = lockfile::read_lockfile(package_root)?;
    match lockfile {
        Some(lockfile) if lockfile.version != 1 => Err(PackageError::diagnostic(
            "package.lockfile.unsupported_version",
            format!("etas.lock version {} is not supported", lockfile.version),
            Some(package_root.join("etas.lock")),
        )),
        Some(lockfile)
            if lockfile.manifest_fingerprint.as_deref() != Some(expected_fingerprint.as_str()) =>
        {
            Err(PackageError::diagnostic(
                "package.lockfile.stale",
                "etas.lock is stale; run `etas pkg lock` after materializing dependencies",
                Some(package_root.join("etas.lock")),
            ))
        }
        Some(lockfile) => validate_lockfile_import_roots(package_root, lockfile),
        None if manifest.dependencies.is_empty() => Ok(Lockfile {
            version: 1,
            manifest_fingerprint: Some(expected_fingerprint),
            packages: Vec::new(),
        }),
        None => Err(PackageError::diagnostic(
            "package.lockfile.missing",
            "etas.lock is required for package compilation; run `etas pkg lock` first",
            Some(package_root.join("etas.lock")),
        )),
    }
}

fn validate_lockfile_import_roots(
    package_root: &Path,
    lockfile: Lockfile,
) -> Result<Lockfile, PackageError> {
    let diagnostics = lockfile
        .packages
        .iter()
        .filter(|package| package.import_root.is_empty())
        .map(|package| {
            PackageDiagnostic::new(
                "package.lockfile.import_root_missing",
                format!(
                    "locked package `{}` version `{}` is missing import_root; regenerate etas.lock with `etas pkg lock`",
                    package.name, package.version
                ),
                Some(package_root.join("etas.lock")),
            )
        })
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        Ok(lockfile)
    } else {
        Err(PackageError::Diagnostics(diagnostics))
    }
}

pub(super) fn lockfile_package<'a>(
    lockfile: &'a Lockfile,
    package_name: &str,
    import_root: &str,
) -> Option<&'a LockedPackage> {
    lockfile
        .packages
        .iter()
        .find(|package| package.name == package_name && package.import_root == import_root)
}
