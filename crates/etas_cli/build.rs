use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    // The stamp belongs to this toolchain build, not to an SDK supplied at run
    // time. Rebuild it when workspace code, dependency pins or codegen change.
    println!("cargo:rerun-if-changed=../../crates");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    println!("cargo:rerun-if-changed=../../Cargo.lock");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    println!(
        "cargo:rustc-env=ETAS_EXECUTABLE_BUILD_ID={stamp:032x}-{}-{}",
        std::process::id(),
        env::var("TARGET").unwrap()
    );
}
