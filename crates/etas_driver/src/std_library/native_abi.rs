use std::collections::BTreeMap;

/// Shared by the toolchain build and SDK producer/loader. Runtime artifact
/// selection compares against a build-time ABI; it never constructs legacy std.
pub fn fingerprint(registry: &etas_std::StdRegistry) -> Result<String, serde_json::Error> {
    let bindings = registry
        .symbols()
        // Source owns these binders. Their minimum execution ABI is covered by
        // the compiler contract identity, not a spelling in the query view.
        .filter(|symbol| {
            etas_std::StdSourceRole::from_path(&symbol.qualified_path.join(".")).is_none()
        })
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
        blake3::hash(&serde_json::to_vec(&(
            etas_std::compiler_contract_identity(),
            bindings,
            implementations,
            registry.native_enum_references().collect::<Vec<_>>(),
            registry.native_opaque_references().collect::<Vec<_>>(),
        ))?)
        .to_hex()
    ))
}
