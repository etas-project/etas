use crate::{PackageError, manifest};

pub(super) fn select_bin(
    manifest: &manifest::Manifest,
    selected: Option<&str>,
) -> Result<Option<manifest::BinTarget>, PackageError> {
    let Some(selected) = selected else {
        return Ok(manifest.default_entry().cloned());
    };
    manifest
        .bins
        .iter()
        .find(|bin| bin.name == selected || bin.flow == selected)
        .cloned()
        .map(Some)
        .ok_or_else(|| {
            PackageError::diagnostic(
                "driver.entry_missing",
                format!("selected bin or flow `{selected}` is not declared in etas.toml"),
                None,
            )
        })
}
