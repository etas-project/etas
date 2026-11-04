use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Component, Path, PathBuf},
};

use etas_frontend::{ProjectSourceLoader, SourceLoader};
use etas_host::{HostError, HostErrorCode};

use super::*;

/// Builds a producer project exclusively from a verified source snapshot and
/// its already locked dependency graph. No path is reopened by the loader.
pub(crate) fn load_locked_source_project(
    snapshot: &etas_package::LockedPackageSourceSnapshot,
    owner: &etas_package::ResolvedDependency,
    checkpoint: &dyn Fn() -> Result<(), HostError>,
) -> Result<LoadedDriverProject, HostError> {
    checkpoint()?;
    let environment = etas_package::environment_from_locked_snapshot(snapshot, owner)
        .map_err(|error| invalid(error.to_string()))?;
    checkpoint()?;
    let root = snapshot.package_root().to_owned();
    let source_root = root.join(snapshot.source_root());
    let manifest = snapshot.manifest().clone();
    let prepared = etas_package::PreparedPackage {
        package_root: root.clone(),
        source_root: source_root.clone(),
        selected_bin: manifest.default_entry().cloned(),
        manifest,
        environment: environment.clone(),
    };
    let entry = entry_from_package(&prepared, None, EntryPolicy::Optional)
        .map_err(|error| invalid(error.to_string()))?;
    let reflection = reflection_options(&prepared.manifest.reflection);
    let frontend_environment =
        frontend_environment(environment.clone()).map_err(|error| invalid(error.to_string()))?;
    let loader = SnapshotSourceLoader::new(snapshot, checkpoint)?;
    let loaded = ProjectSourceLoader::new(loader)
        .load(ProjectSourceLoadOptions {
            project_root: root.clone(),
            roots: vec![source_root.clone()],
            entry: entry.clone(),
            scope: ProjectSourceLoadScope::FullSourceTree,
            source_kind: Some(SourceKind::SourceProjectFile),
            import_search_roots: vec![root.clone()],
            external_module_roots: standard_external_module_roots(),
        })
        .map_err(|error| {
            checkpoint()
                .err()
                .unwrap_or_else(|| invalid(format!("locked source project is invalid: {error}")))
        })?;
    checkpoint()?;
    let local = LocalSourceProject {
        project_root: root.clone(),
        package_root: Some(root),
        roots: vec![source_root],
        entry,
        environment: frontend_environment,
        package_environment: Some(environment),
        mode: DriverProjectMode::Package,
        source_dependencies: Vec::new(),
        reflection,
    };
    let options = DriverOptions {
        cache_mode: crate::DriverCacheMode::Off,
        ..Default::default()
    };
    let loaded = finish_loaded_project(&options, loaded, local, EntryPolicy::Optional)
        .map_err(|error| invalid(error.to_string()))?;
    checkpoint()?;
    Ok(loaded)
}

fn invalid(message: String) -> HostError {
    HostError::new(HostErrorCode::InvalidRequest, message)
}

struct SnapshotSourceLoader<'a> {
    snapshot: &'a etas_package::LockedPackageSourceSnapshot,
    checkpoint: &'a dyn Fn() -> Result<(), HostError>,
    directories: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

impl<'a> SnapshotSourceLoader<'a> {
    fn new(
        snapshot: &'a etas_package::LockedPackageSourceSnapshot,
        checkpoint: &'a dyn Fn() -> Result<(), HostError>,
    ) -> Result<Self, HostError> {
        let mut directories = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
        directories.entry(PathBuf::new()).or_default();
        for (path, _) in snapshot.files() {
            checkpoint()?;
            let mut parent = PathBuf::new();
            for component in path.components() {
                let Component::Normal(name) = component else {
                    return Err(invalid("locked snapshot has a non-normalized path".into()));
                };
                let child = parent.join(name);
                directories.entry(parent).or_default().insert(child.clone());
                parent = child;
            }
        }
        Ok(Self {
            snapshot,
            checkpoint,
            directories,
        })
    }

    fn relative<'b>(&self, path: &'b Path) -> io::Result<&'b Path> {
        let relative = path
            .strip_prefix(self.snapshot.package_root())
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "path is outside the locked snapshot",
                )
            })?;
        if relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not normalized",
            ));
        }
        Ok(relative)
    }

    fn check(&self) -> io::Result<()> {
        (self.checkpoint)().map_err(|error| io::Error::other(error.to_string()))
    }
}

impl SourceLoader for SnapshotSourceLoader<'_> {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.check()?;
        let relative = self.relative(path)?;
        if self.snapshot.file(relative).is_none() && !self.directories.contains_key(relative) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path is absent from the locked snapshot",
            ));
        }
        Ok(self.snapshot.package_root().join(relative))
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.check()?;
        let bytes = self.snapshot.file(self.relative(path)?).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "file is absent from the locked snapshot",
            )
        })?;
        let text = std::str::from_utf8(bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let mut output = String::with_capacity(text.len());
        let mut start = 0;
        while start < text.len() {
            self.check()?;
            let mut end = start.saturating_add(64 * 1024).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            output.push_str(&text[start..end]);
            start = end;
        }
        self.check()?;
        Ok(output)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.relative(path)
            .ok()
            .is_some_and(|path| self.snapshot.file(path).is_some())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.relative(path)
            .ok()
            .is_some_and(|path| self.directories.contains_key(path))
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.check()?;
        let entries = self.directories.get(self.relative(path)?).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "directory is absent from the locked snapshot",
            )
        })?;
        entries
            .iter()
            .map(|path| {
                self.check()?;
                Ok(self.snapshot.package_root().join(path))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "etas-locked-project-{}-{nonce}",
                std::process::id()
            ));
            std::fs::create_dir_all(root.join("src/library")).unwrap();
            std::fs::write(root.join("etas.toml"), "[package]\nname = 'library'\nversion = '1.0.0'\nedition = '2026'\n[reflection]\nexport_bodies = ['library.api.identity']\n").unwrap();
            std::fs::write(
                root.join("src/library/api.es"),
                "module library.api; public flow identity<T>(value: T) -> T ![] { return value; }",
            )
            .unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn locked_project_matches_producer_without_reading_changed_files() {
        let fixture = Fixture::new();
        let options = DriverOptions {
            cache_mode: crate::DriverCacheMode::Off,
            ..Default::default()
        };
        let original = load_project(
            &options,
            LoadProjectRequest {
                paths: vec![fixture.0.clone()],
                all: false,
                entry_policy: EntryPolicy::Optional,
                flow: None,
                bin: None,
                source_dependency_mode: SourceDependencyMode::MetadataOnly,
            },
        )
        .unwrap();
        let checksum = etas_package::package_payload_checksum(&fixture.0).unwrap();
        let snapshot = etas_package::read_locked_package_sources(
            &fixture.0,
            &checksum,
            Default::default(),
            || Ok::<_, std::convert::Infallible>(()),
        )
        .unwrap();
        let owner = etas_package::ResolvedDependency {
            identity: etas_package::PackageIdentity::current(snapshot.manifest()),
            std_compatibility: etas_std::standard_registry()
                .version()
                .compatibility_identity(),
            import_root: "dependency".into(),
            source: etas_package::ResolvedDependencySource::Path {
                path: "library".into(),
                checksum,
            },
            dependencies: vec![],
            public_metadata: Default::default(),
            effect_metadata: Default::default(),
            tool_bindings: vec![],
        };
        std::fs::write(
            fixture.0.join("src/library/api.es"),
            "changed after verification",
        )
        .unwrap();
        std::fs::write(fixture.0.join("etas.toml"), "invalid after verification").unwrap();
        let restored = load_locked_source_project(&snapshot, &owner, &|| Ok(())).unwrap();
        assert_eq!(
            restored.loaded.input.sources.len(),
            original.loaded.input.sources.len()
        );
        for (actual, expected) in restored
            .loaded
            .input
            .sources
            .iter()
            .zip(&original.loaded.input.sources)
        {
            assert_eq!(actual.path, expected.path);
            assert_eq!(actual.text, expected.text);
        }
        assert_eq!(
            restored
                .loaded
                .input
                .environment
                .canonical_environment_fingerprint(),
            original
                .loaded
                .input
                .environment
                .canonical_environment_fingerprint()
        );
        assert_eq!(
            restored.loaded.input.environment.stable_source_keys,
            original.loaded.input.environment.stable_source_keys
        );
        let error = load_locked_source_project(&snapshot, &owner, &|| {
            Err(HostError::new(HostErrorCode::Cancelled, "cancelled"))
        })
        .unwrap_err();
        assert_eq!(error.code, HostErrorCode::Cancelled);
    }
}
