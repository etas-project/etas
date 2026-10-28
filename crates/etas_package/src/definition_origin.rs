use crate::{PackageError, PackageIdentity};
use etas_code::{CodeFingerprint, PackageKey};
use std::path::Path;

/// Capture a local package's original owner before vendoring or compilation.
/// Import aliases, payload revisions and cache/store paths are not its identity.
pub fn local_definition_package_key(
    root: &Path,
    identity: &PackageIdentity,
) -> Result<PackageKey, PackageError> {
    if [&identity.name, &identity.version, &identity.edition]
        .iter()
        .any(|part| part.is_empty() || part.contains('\0'))
    {
        return Err(PackageError::Manifest {
            path: root.join("etas.toml"),
            message: "invalid definition package identity".into(),
        });
    }
    let root = root.canonicalize().map_err(|source| PackageError::Io {
        path: root.to_owned(),
        source,
    })?;
    Ok(PackageKey::from_canonical_fingerprint(
        CodeFingerprint::of_parts(
            "etas.code.local-package-origin.v1",
            [
                root.as_os_str().as_encoded_bytes(),
                identity.name.as_bytes(),
                identity.version.as_bytes(),
                identity.edition.as_bytes(),
            ],
        ),
    ))
}
