/// Toolchain-owned execution ABI. Source declarations and the selected subset
/// of host actions belong to the bundle identity; they cannot change which
/// compiler/host contract set this executable implements.
pub fn fingerprint() -> String {
    etas_std::compiler_contract_identity()
}
