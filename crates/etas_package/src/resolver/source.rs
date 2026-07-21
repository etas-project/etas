use crate::{lockfile::LockedPackage, manifest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum LockedSourceRef {
    Builtin,
    Registry {
        registry: String,
    },
    Path {
        path: String,
    },
    Vendor {
        path: String,
    },
    Git {
        url: String,
        rev: String,
    },
    GitHubClone {
        repo: String,
        rev: String,
    },
    GitHubRelease {
        repo: String,
        release: String,
        asset: String,
    },
}

pub(super) fn spec_source_requires_exact_git_rev(spec: &manifest::DependencySpec) -> bool {
    matches!(spec, manifest::DependencySpec::Detailed(detail) if detail.git.is_some() || (detail.github.is_some() && detail.release.is_none()))
}

pub(super) fn resolved_source_from_lockfile(locked: &LockedPackage) -> Option<LockedSourceRef> {
    if locked.source == "builtin" {
        return Some(LockedSourceRef::Builtin);
    }
    if let Some(registry) = locked.source.strip_prefix("registry+") {
        return Some(LockedSourceRef::Registry {
            registry: registry.to_owned(),
        });
    }
    if let Some(path) = locked.source.strip_prefix("path+") {
        return Some(LockedSourceRef::Path {
            path: path.to_owned(),
        });
    }
    if let Some(path) = locked.source.strip_prefix("vendor+") {
        return Some(LockedSourceRef::Vendor {
            path: path.to_owned(),
        });
    }
    if let Some(rest) = locked.source.strip_prefix("git+") {
        let (url, query) = rest.split_once('?').unwrap_or((rest, ""));
        let rev = query
            .split('&')
            .find_map(|part| part.strip_prefix("rev="))
            .filter(|rev| !rev.is_empty())?;
        return Some(LockedSourceRef::Git {
            url: url.to_owned(),
            rev: rev.to_owned(),
        });
    }
    if let Some(rest) = locked.source.strip_prefix("github+") {
        let (repo, query) = rest.split_once('?').unwrap_or((rest, ""));
        let rev = query
            .split('&')
            .find_map(|part| part.strip_prefix("rev="))
            .filter(|rev| !rev.is_empty())?;
        return Some(LockedSourceRef::GitHubClone {
            repo: repo.to_owned(),
            rev: rev.to_owned(),
        });
    }
    if let Some(rest) = locked.source.strip_prefix("github-release+") {
        let (repo, query) = rest.split_once('?').unwrap_or((rest, ""));
        let release = query
            .split('&')
            .find_map(|part| part.strip_prefix("release="))
            .and_then(decode_lock_value)
            .filter(|release| !release.is_empty())?;
        let asset = query
            .split('&')
            .find_map(|part| part.strip_prefix("asset="))
            .and_then(decode_lock_value)
            .filter(|asset| !asset.is_empty())?;
        return Some(LockedSourceRef::GitHubRelease {
            repo: repo.to_owned(),
            release,
            asset,
        });
    }
    None
}

fn decode_lock_value(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            output.push(from_hex(high)? << 4 | from_hex(low)?);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
