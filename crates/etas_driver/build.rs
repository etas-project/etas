#[path = "src/std_library/native_abi.rs"]
mod native_abi;

fn main() {
    println!("cargo:rerun-if-changed=src/std_library/native_abi.rs");
    println!("cargo:rerun-if-env-changed=ETAS_TOOLCHAIN_ID");
    let target = std::env::var("TARGET").unwrap();
    let registry = etas_std::standard_registry().native_bootstrap();
    let abi = native_abi::fingerprint(&registry).unwrap();
    let toolchain = std::env::var("ETAS_TOOLCHAIN_ID").unwrap_or_else(|_| {
        format!(
            "etas-{}-{target}-{}",
            env!("CARGO_PKG_VERSION"),
            &abi[7..19]
        )
    });
    println!("cargo:rustc-env=ETAS_STD_TARGET={target}");
    println!("cargo:rustc-env=ETAS_STD_NATIVE_ABI={abi}");
    println!(
        "cargo:rustc-env=ETAS_STD_CONTRACT={}",
        registry.version().compatibility_identity()
    );
    println!("cargo:rustc-env=ETAS_TOOLCHAIN_ID={toolchain}");
}
