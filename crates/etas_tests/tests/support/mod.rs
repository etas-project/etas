#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use etas_core::{SourceFile, SourceId};

pub fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}

pub fn fixtures_under(relative: &str) -> Vec<PathBuf> {
    let root = fixture(relative);
    let mut files = Vec::new();
    collect_etas_files(&root, &mut files);
    files.sort();
    files
}

pub fn read_source(path: &Path) -> SourceFile {
    SourceFile::new(
        SourceId(1),
        Some(path.to_path_buf()),
        std::fs::read_to_string(path).unwrap(),
    )
}

pub fn run_cli<const N: usize>(args: [&str; N]) -> (i32, String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = etas_cli::run_with(args, &mut stdout, &mut stderr);

    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

pub fn run_cli_with_env<const N: usize, const M: usize>(
    args: [&str; N],
    env: [(&str, &str); M],
) -> (i32, String, String) {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let previous = env
        .iter()
        .map(|(key, _)| (*key, std::env::var_os(key)))
        .collect::<Vec<_>>();

    for (key, value) in env {
        // Tests run the in-process CLI. Scope the env mutation with a mutex so
        // host-profile fixtures can exercise CLI config without cross-test bleed.
        unsafe { std::env::set_var(key, value) };
    }

    let result = run_cli(args);

    for (key, value) in previous {
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }

    result
}

pub fn path_str(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn collect_etas_files(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_etas_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "es") {
            files.push(path);
        }
    }
}
