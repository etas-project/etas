use std::path::Path;

use crate::PackageDiagnostic;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReflectionSection {
    #[serde(default)]
    pub retain_bodies: Vec<String>,
    #[serde(default)]
    pub export_bodies: Vec<String>,
}

impl ReflectionSection {
    pub(super) fn validate(&self, manifest: &Path) -> Vec<PackageDiagnostic> {
        let mut diagnostics = Vec::new();
        for (setting, entries) in [
            ("retain_bodies", &self.retain_bodies),
            ("export_bodies", &self.export_bodies),
        ] {
            let mut seen = std::collections::BTreeSet::new();
            for entry in entries {
                if !super::is_valid_import_root(entry) || !entry.contains('.') {
                    diagnostics.push(PackageDiagnostic::new(
                        "package.manifest.invalid_reflection_item",
                        format!("`[reflection].{setting}` entry `{entry}` must be a fully qualified item name, not a file path"),
                        Some(manifest.to_path_buf()),
                    ));
                }
                if !seen.insert(entry) {
                    diagnostics.push(PackageDiagnostic::new(
                        "package.manifest.duplicate_reflection_item",
                        format!("duplicate `[reflection].{setting}` entry `{entry}`"),
                        Some(manifest.to_path_buf()),
                    ));
                }
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(extra: &str) -> super::super::Manifest {
        toml::from_str(&format!(
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\n{extra}"
        ))
        .unwrap()
    }

    #[test]
    fn reflection_defaults_to_no_body_retention_or_export() {
        assert_eq!(parse("").reflection, ReflectionSection::default());
    }

    #[test]
    fn reflection_configuration_does_not_change_dependency_resolution() {
        let original = parse("");
        let reflected = parse(
            "[reflection]\nretain_bodies = [\"app.Worker\"]\nexport_bodies = [\"app.Normalize\"]\n",
        );
        reflected.validate(Path::new("etas.toml")).unwrap();
        assert_eq!(reflected.reflection.retain_bodies, ["app.Worker"]);
        assert_eq!(
            crate::lockfile::manifest_dependency_fingerprint(&original),
            crate::lockfile::manifest_dependency_fingerprint(&reflected)
        );
    }

    #[test]
    fn reflection_rejects_paths_unknown_options_and_duplicate_entries() {
        for entry in ["src/app.es", "Worker", "app..Worker", "app.*"] {
            let manifest = parse(&format!("[reflection]\nretain_bodies = [{entry:?}]\n"));
            assert!(
                manifest.validate(Path::new("etas.toml")).is_err(),
                "{entry}"
            );
        }
        assert!(
            parse("[reflection]\nretain_bodies = [\"app.Worker\", \"app.Worker\"]")
                .validate(Path::new("etas.toml"))
                .is_err()
        );
        assert!(
            toml::from_str::<super::super::Manifest>(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[reflection]\nretain_all = true\n"
            )
            .is_err()
        );
    }
}
