use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCompilerProfile {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub allow_check: Option<bool>,
    #[serde(default)]
    pub allow_compile: Option<bool>,
    #[serde(default)]
    pub max_active_workers: Option<usize>,
    #[serde(default)]
    pub max_retained_candidates: Option<usize>,
    #[serde(default)]
    pub max_retained_candidate_bytes: Option<u64>,
    #[serde(default)]
    pub max_worker_cpu_ms: Option<u64>,
    #[serde(default)]
    pub max_process_peak_resident_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_runtime_configuration_is_data_not_a_dependency_input() {
        let original = "[package]\nname = 'app'\nversion = '0.1.0'\n[dependencies]\nlib = { path = '../lib' }\n";
        let before: crate::Manifest = toml::from_str(original).unwrap();
        let after: crate::Manifest = toml::from_str(&format!(
            "{original}\n[runtime.profiles.local.compiler]\nbackend = 'native'\nallow_check = true\nallow_compile = false\nmax_active_workers = 2\nmax_worker_cpu_ms = 250\nmax_process_peak_resident_bytes = 1073741824\n"
        )).unwrap();
        let config = after.runtime.profiles["local"].compiler.as_ref().unwrap();
        assert_eq!(config.backend.as_deref(), Some("native"));
        assert_eq!(config.allow_check, Some(true));
        assert_eq!(config.allow_compile, Some(false));
        assert_eq!(config.max_active_workers, Some(2));
        assert_eq!(
            before.dependencies["lib"].lock_fingerprint(),
            after.dependencies["lib"].lock_fingerprint()
        );
    }

    #[test]
    fn compiler_configuration_rejects_unknown_authority_fields() {
        assert!(
            toml::from_str::<RuntimeCompilerProfile>("backend = 'native'\nallow_all = true")
                .is_err()
        );
    }
}
