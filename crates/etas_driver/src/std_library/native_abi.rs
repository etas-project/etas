use std::collections::BTreeMap;

/// Shared by the toolchain build and SDK producer/loader. Runtime artifact
/// selection compares against a build-time ABI; it never constructs legacy std.
pub fn fingerprint(registry: &etas_std::StdRegistry) -> Result<String, serde_json::Error> {
    let bindings = registry
        .symbols()
        .filter_map(|symbol| symbol.intrinsic.as_ref())
        .map(|binding| (binding.id, binding))
        .collect::<BTreeMap<_, _>>();
    Ok(format!(
        "blake3:{}",
        blake3::hash(&serde_json::to_vec(&bindings)?).to_hex()
    ))
}
