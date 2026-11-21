//! Canonical SDK object storage and optional, rebuildable object-pack caches.
//! Cache bytes have no independent execution authority: every object is checked
//! against the immutable SDK manifest before the compiler links it.
use super::{LoadedStdSdk, StdBundleFile, invalid, write_file};
use crate::DriverError;
use etas_frontend::{
    ModuleId,
    library_hir::{
        LibraryHirImage,
        objects::{LibraryObjectArchive, LibraryObjectIndex},
    },
};
use etas_hir::HirItemId;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckedObjectFiles {
    pub index: StdBundleFile,
    pub shared: StdBundleFile,
    pub modules: BTreeMap<ModuleId, StdBundleFile>,
}

fn publish_object(root: &Path, bytes: &[u8]) -> Result<StdBundleFile, DriverError> {
    let hash = etas_package_metadata::blake3_hash(bytes);
    let path = PathBuf::from(format!(
        "artifacts/objects/{}.bin",
        hash.strip_prefix("blake3:").expect("object digest")
    ));
    write_file(&root.join(&path), bytes)?;
    Ok(StdBundleFile {
        path,
        hash,
        base: None,
    })
}

pub(super) fn publish(
    root: &Path,
    archive: LibraryObjectArchive,
) -> Result<CheckedObjectFiles, DriverError> {
    let index = publish_object(root, &archive.index.encode().map_err(invalid)?)?;
    let shared = publish_object(root, &archive.shared)?;
    let modules = archive
        .modules
        .into_iter()
        .map(|(id, bytes)| Ok((id, publish_object(root, &bytes)?)))
        .collect::<Result<_, DriverError>>()?;
    Ok(CheckedObjectFiles {
        index,
        shared,
        modules,
    })
}

impl LoadedStdSdk {
    pub(crate) fn object_index(&self) -> Result<&Arc<LibraryObjectIndex>, DriverError> {
        self.object_index
            .get_or_init(|| {
                let bytes = self
                    .load_bundle_file(&self.manifest.checked_objects.index)
                    .map_err(|e| e.to_string())?;
                LibraryObjectIndex::decode(&bytes)
                    .map(Arc::new)
                    .map_err(|e| e.to_string())
            })
            .as_ref()
            .map_err(invalid)
    }

    pub(crate) fn assemble_objects(
        &self,
        definitions: &BTreeSet<HirItemId>,
        profile: &etas_utils::ProfileHandle,
        roots: Option<&BTreeSet<HirItemId>>,
    ) -> Result<LibraryHirImage, DriverError> {
        let _load = profile.span("std.load_module_objects", "std");
        let index = self.object_index()?;
        let selected = index.selected_modules(definitions);
        let objects = &self.manifest.checked_objects;
        let files = std::iter::once(&objects.shared)
            .chain(selected.iter().map(|id| &objects.modules[id]))
            .collect::<Vec<_>>();
        let cache = roots.and_then(|roots| cache_path(self, roots));
        let cached = cache.as_deref().and_then(|path| read_pack(path, &files));
        profile.counter("std.object_pack_cache_hit", cached.is_some() as u64);
        let bytes = match cached {
            Some(bytes) => bytes,
            None => {
                let bytes = files
                    .iter()
                    .map(|file| self.load_bundle_file(file))
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(path) = cache {
                    write_pack(&path, &bytes);
                }
                bytes
            }
        };
        profile.counter("std.loaded_body_modules", selected.len() as u64);
        profile.counter(
            "std.loaded_body_bytes",
            bytes.iter().map(|bytes| bytes.len() as u64).sum(),
        );
        profile.counter("std.loaded_canonical_objects", bytes.len() as u64);
        let mut bytes = bytes.into_iter();
        let shared = bytes.next().expect("shared checked SDK object");
        let _decode = profile.span("std.assemble_checked_objects", "std");
        LibraryHirImage::from_checked_objects(&shared, index, bytes, definitions, profile)
            .map_err(invalid)
    }
}

fn cache_path(sdk: &LoadedStdSdk, definitions: &BTreeSet<HirItemId>) -> Option<PathBuf> {
    let root = std::env::var_os("ETAS_STD_CACHE_ROOT")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
                .map(|root| root.join("etas/std-objects"))
        })?;
    let mut key = blake3::Hasher::new_derive_key("etas.sdk.object-pack.v1");
    key.update(sdk.manifest.identity().expect("SDK identity").as_bytes());
    key.update(sdk.manifest.compiler_version.as_bytes());
    key.update(sdk.manifest.native_binding_abi.as_bytes());
    key.update(sdk.manifest.target.as_bytes());
    // The manifest also binds the interpreter layout digest and object schema.
    for definition in definitions {
        key.update(&definition.0.to_le_bytes());
    }
    Some(root.join(format!("{}.pack", key.finalize().to_hex())))
}

fn read_pack(path: &Path, expected: &[&StdBundleFile]) -> Option<Vec<Vec<u8>>> {
    let bytes = fs::read(path).ok()?;
    let mut input = Cursor::new(bytes.as_slice());
    let mut magic = [0; 8];
    input.read_exact(&mut magic).ok()?;
    if &magic != b"ETASPK01" {
        return None;
    }
    let mut number = [0; 8];
    input.read_exact(&mut number).ok()?;
    if u64::from_le_bytes(number) as usize != expected.len() {
        return None;
    }
    let mut objects = Vec::with_capacity(expected.len());
    for file in expected {
        input.read_exact(&mut number).ok()?;
        let start = input.position() as usize;
        let end = start.checked_add(u64::from_le_bytes(number) as usize)?;
        let bytes = bytes.get(start..end)?;
        if etas_package_metadata::blake3_hash(bytes) != file.hash {
            return None;
        }
        objects.push(bytes.to_vec());
        input.set_position(end as u64);
    }
    (input.position() as usize == bytes.len()).then_some(objects)
}

fn write_pack(path: &Path, objects: &[Vec<u8>]) {
    // Derived caches are optional; write permission or an absent cache never
    // changes which checked objects are selected or whether a program can run.
    let store = || -> std::io::Result<()> {
        fs::create_dir_all(path.parent().expect("object cache directory"))?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        let mut file = fs::File::create(&temporary)?;
        file.write_all(b"ETASPK01")?;
        file.write_all(&(objects.len() as u64).to_le_bytes())?;
        for object in objects {
            file.write_all(&(object.len() as u64).to_le_bytes())?;
            file.write_all(object)?;
        }
        drop(file);
        fs::rename(temporary, path)
    };
    let _ = store();
}
