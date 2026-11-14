use std::collections::BTreeMap;

/// Shared by the toolchain build and SDK producer/loader. Runtime artifact
/// selection compares against a build-time ABI; it never constructs legacy std.
pub fn fingerprint(registry: &etas_std::StdRegistry) -> Result<String, serde_json::Error> {
    let bindings = registry
        .symbols()
        .map(|symbol| {
            (
                symbol.qualified_path.join("."),
                (
                    symbol.kind,
                    &symbol.decl,
                    &symbol.intrinsic,
                    symbol.enum_owner.map(|owner| {
                        &registry
                            .symbol(owner)
                            .expect("native enum owner")
                            .qualified_path
                    }),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let implementations = registry.spec_impls().collect::<Vec<_>>();
    Ok(format!(
        "blake3:{}",
        blake3::hash(&serde_json::to_vec(&(bindings, implementations))?).to_hex()
    ))
}
