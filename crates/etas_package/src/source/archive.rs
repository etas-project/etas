use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::{PackageError, manifest, vendor};

const BLOCK: usize = 512;

pub fn pack_etaspkg(package_root: &Path, output: &Path) -> Result<String, PackageError> {
    let manifest = manifest::read_manifest(package_root)?;
    let metadata = vendor::package_metadata_artifact_path(package_root);
    if !metadata.is_file() {
        return Err(PackageError::Manifest {
            path: metadata,
            message:
                "package metadata artifact `.etas/package.etasmeta` is required before packing"
                    .to_owned(),
        });
    }
    vendor::read_package_metadata_artifact_required(package_root)?;

    let mut files = vec![package_root.join("etas.toml"), metadata];
    collect_files(&package_root.join(&manifest.source.root), &mut files)?;
    files.sort();

    let mut tar = Vec::new();
    for file in files {
        let relative = file.strip_prefix(package_root).unwrap_or(&file);
        append_file(&mut tar, relative, &file)?;
    }
    tar.extend([0u8; BLOCK]);
    tar.extend([0u8; BLOCK]);

    let compressed =
        zstd::stream::encode_all(tar.as_slice(), 19).map_err(|source| PackageError::Manifest {
            path: output.to_path_buf(),
            message: format!("failed to compress .etaspkg archive: {source}"),
        })?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|source| PackageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let temp = output.with_extension(format!(
        "{}.tmp-{}",
        output
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("etaspkg"),
        std::process::id()
    ));
    fs::write(&temp, &compressed).map_err(|source| PackageError::Io {
        path: temp.clone(),
        source,
    })?;
    fs::rename(&temp, output).map_err(|source| PackageError::Io {
        path: output.to_path_buf(),
        source,
    })?;
    Ok(format!("blake3:{}", blake3::hash(&compressed).to_hex()))
}

pub fn unpack_etaspkg(archive: &Path, destination: &Path) -> Result<(), PackageError> {
    let bytes = fs::read(archive).map_err(|source| PackageError::Io {
        path: archive.to_path_buf(),
        source,
    })?;
    let decoded =
        zstd::stream::decode_all(bytes.as_slice()).map_err(|source| PackageError::Manifest {
            path: archive.to_path_buf(),
            message: format!("failed to decompress .etaspkg archive: {source}"),
        })?;
    unpack_tar_bytes(archive, &decoded, destination)
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), PackageError> {
    let entries = fs::read_dir(root).map_err(|source| PackageError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| PackageError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.is_dir() {
            collect_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn append_file(tar: &mut Vec<u8>, relative: &Path, file: &Path) -> Result<(), PackageError> {
    let path = relative.to_string_lossy().replace('\\', "/");
    if path.len() > 100 {
        return Err(PackageError::Manifest {
            path: file.to_path_buf(),
            message: "archive path is too long for etaspkg v1".to_owned(),
        });
    }
    let bytes = fs::read(file).map_err(|source| PackageError::Io {
        path: file.to_path_buf(),
        source,
    })?;
    let mut header = [0u8; BLOCK];
    write_bytes(&mut header[0..100], path.as_bytes());
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], bytes.len() as u64);
    write_octal(&mut header[136..148], 0);
    for byte in &mut header[148..156] {
        *byte = b' ';
    }
    header[156] = b'0';
    write_bytes(&mut header[257..263], b"ustar\0");
    write_bytes(&mut header[263..265], b"00");
    let checksum = header.iter().map(|byte| u32::from(*byte)).sum::<u32>();
    write_checksum(&mut header[148..156], checksum);
    tar.extend(header);
    tar.extend(&bytes);
    let padding = (BLOCK - (bytes.len() % BLOCK)) % BLOCK;
    tar.extend(std::iter::repeat_n(0u8, padding));
    Ok(())
}

fn unpack_tar_bytes(archive: &Path, bytes: &[u8], destination: &Path) -> Result<(), PackageError> {
    let mut offset = 0usize;
    while offset + BLOCK <= bytes.len() {
        let header = &bytes[offset..offset + BLOCK];
        offset += BLOCK;
        if header.iter().all(|byte| *byte == 0) {
            return Ok(());
        }
        let name = read_c_string(&header[0..100]);
        let entry_type = header[156];
        let size = read_octal(&header[124..136]).ok_or_else(|| PackageError::Manifest {
            path: archive.to_path_buf(),
            message: format!("archive entry `{name}` has invalid size"),
        })?;
        let relative = safe_archive_path(archive, &name)?;
        let target = destination.join(relative);
        match entry_type {
            0 | b'0' => {
                let size = usize::try_from(size).map_err(|_| PackageError::Manifest {
                    path: archive.to_path_buf(),
                    message: format!("archive entry `{name}` is too large"),
                })?;
                let end = offset
                    .checked_add(size)
                    .ok_or_else(|| PackageError::Manifest {
                        path: archive.to_path_buf(),
                        message: format!("archive entry `{name}` size overflows"),
                    })?;
                let content = bytes
                    .get(offset..end)
                    .ok_or_else(|| PackageError::Manifest {
                        path: archive.to_path_buf(),
                        message: format!("archive entry `{name}` exceeds archive length"),
                    })?;
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|source| PackageError::Io {
                        path: parent.to_path_buf(),
                        source,
                    })?;
                }
                fs::write(&target, content).map_err(|source| PackageError::Io {
                    path: target.clone(),
                    source,
                })?;
                offset = end + ((BLOCK - (size % BLOCK)) % BLOCK);
            }
            b'5' => {
                fs::create_dir_all(&target).map_err(|source| PackageError::Io {
                    path: target.clone(),
                    source,
                })?;
            }
            b'1' | b'2' => {
                return Err(PackageError::Manifest {
                    path: archive.to_path_buf(),
                    message: format!(
                        "archive entry `{name}` is a link; .etaspkg links are not allowed"
                    ),
                });
            }
            other => {
                return Err(PackageError::Manifest {
                    path: archive.to_path_buf(),
                    message: format!("archive entry `{name}` has unsupported type `{other}`"),
                });
            }
        }
    }
    Err(PackageError::Manifest {
        path: archive.to_path_buf(),
        message: "archive ended before tar terminator".to_owned(),
    })
}

fn safe_archive_path(archive: &Path, name: &str) -> Result<PathBuf, PackageError> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(PackageError::Manifest {
            path: archive.to_path_buf(),
            message: format!("archive entry `{name}` uses an absolute path"),
        });
    }
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => output.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(PackageError::Manifest {
                    path: archive.to_path_buf(),
                    message: format!("archive entry `{name}` escapes the archive root"),
                });
            }
        }
    }
    if output.as_os_str().is_empty() {
        return Err(PackageError::Manifest {
            path: archive.to_path_buf(),
            message: "archive entry has an empty path".to_owned(),
        });
    }
    Ok(output)
}

fn write_bytes(target: &mut [u8], bytes: &[u8]) {
    let len = bytes.len().min(target.len());
    target[..len].copy_from_slice(&bytes[..len]);
}

fn write_octal(target: &mut [u8], value: u64) {
    let text = format!("{value:0width$o}\0", width = target.len() - 1);
    write_bytes(target, text.as_bytes());
}

fn write_checksum(target: &mut [u8], value: u32) {
    let text = format!("{value:06o}\0 ",);
    write_bytes(target, text.as_bytes());
}

fn read_c_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn read_octal(bytes: &[u8]) -> Option<u64> {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim_matches(char::from(0)).trim();
    if trimmed.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(trimmed, 8).ok()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn unpack_rejects_path_traversal() {
        let root = temp_dir("archive-traversal");
        let file = root.join("payload.txt");
        fs::write(&file, "bad").unwrap();
        let mut tar = Vec::new();
        append_file(&mut tar, Path::new("../payload.txt"), &file).unwrap();
        tar.extend([0u8; BLOCK]);
        tar.extend([0u8; BLOCK]);
        let compressed = zstd::stream::encode_all(tar.as_slice(), 1).unwrap();
        let archive = root.join("bad.etaspkg");
        fs::write(&archive, compressed).unwrap();
        let output = root.join("out");

        let error = unpack_etaspkg(&archive, &output).unwrap_err();

        assert!(
            error.to_string().contains("escapes the archive root"),
            "{error}"
        );
        assert!(!root.join("payload.txt.tmp").exists());
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("etas-package-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
