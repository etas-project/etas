use crate::{PackageError, PackageIdentity};
use etas_code::PackageKey;
use std::path::Path;

/// Capture a local package's original owner before vendoring or compilation.
/// Import aliases, payload revisions and cache/store paths are not its identity.
pub fn local_definition_package_key(
    root: &Path,
    identity: &PackageIdentity,
) -> Result<PackageKey, PackageError> {
    PackageKey::from_name_and_version(&identity.name, &identity.version).map_err(|_| {
        PackageError::Manifest {
            path: root.join("etas.toml"),
            message: "invalid definition package identity".into(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(name: &str, version: &str, edition: &str) -> PackageIdentity {
        PackageIdentity {
            name: name.into(),
            version: version.into(),
            edition: edition.into(),
        }
    }

    #[test]
    fn local_identity_depends_only_on_name_and_version() {
        let first = local_definition_package_key(
            Path::new("/checkout/one"),
            &identity("app", "1.0.0", "2025"),
        )
        .unwrap();
        let second = local_definition_package_key(
            Path::new("/checkout/two"),
            &identity("app", "1.0.0", "2026"),
        )
        .unwrap();
        assert_eq!(first, second);
        for other in [
            identity("other", "1.0.0", "2025"),
            identity("app", "1.0.1", "2025"),
        ] {
            assert_ne!(
                first,
                local_definition_package_key(Path::new("/checkout/one"), &other).unwrap()
            );
        }
    }

    #[test]
    fn local_identity_rejects_invalid_name_or_version() {
        for invalid in [
            identity("", "1.0.0", "2026"),
            identity("app", "", "2026"),
            identity("app\0other", "1.0.0", "2026"),
        ] {
            assert!(local_definition_package_key(Path::new("/checkout"), &invalid).is_err());
        }
    }
}
