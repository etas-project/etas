use super::{AuthConfig, CliModelConfig, RuntimeEnvOverrides};

pub(super) const ENV_KEYS: &[&str] = &["ETAS_HOST_MODEL_API_KEY", "ETAS_HOST_OMLX_API_KEY"];

pub(super) fn resolve(
    env: &RuntimeEnvOverrides,
    profile_key_env: Option<&str>,
    current: Option<&CliModelConfig>,
) -> AuthConfig {
    if let Some(key) = ENV_KEYS.iter().find_map(|name| env.optional(name)) {
        return AuthConfig::BearerToken(key);
    }
    match profile_key_env {
        // An explicit source replaces the previous credential, even if unset.
        Some(name) => env
            .optional(name)
            .map_or(AuthConfig::None, AuthConfig::BearerToken),
        None => current.map_or(AuthConfig::None, |model| model.auth.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    const SECRET: &str = "profile-credential-not-for-output";

    fn profile() -> RuntimeProfile {
        RuntimeProfile {
            model: Some(RuntimeModelProfile {
                adapter: Some("omlx-openai".into()),
                model: Some("profile-model".into()),
                base_url: Some("http://127.0.0.1:8848/v1".into()),
                api_key_env: Some("PROFILE_KEY".into()),
                typed_output_encoding: Some("json-schema".into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn env() -> RuntimeEnvOverrides {
        RuntimeEnvOverrides {
            values: BTreeMap::from([("PROFILE_KEY".into(), SECRET.into())]),
        }
    }

    fn configured(local: Option<&RuntimeProfile>, env: &RuntimeEnvOverrides) -> CliHostConfig {
        CliHostConfig::from_runtime_profile("mock".into(), Some(&profile()), local, env, &[])
            .unwrap()
    }

    fn assert_key(config: &CliHostConfig, key: &str) {
        let model = config.model.as_ref().unwrap();
        assert_eq!(
            model.auth.headers(),
            vec![("Authorization".into(), format!("Bearer {key}"))]
        );
        assert!(!config.runtime_profile_json().to_string().contains(key));
    }

    #[test]
    fn non_credential_model_overrides_preserve_resolved_auth() {
        for (name, value) in [
            ("ETAS_HOST_MODEL_ADAPTER", "omlx-openai"),
            ("ETAS_HOST_MODEL_NAME", "override-model"),
            ("ETAS_HOST_MODEL_BASE_URL", "http://127.0.0.1:9999/v1"),
            ("ETAS_HOST_MODEL_ALLOW_PRIVATE", "true"),
            ("ETAS_HOST_MODEL_CONNECT_TIMEOUT_MS", "1200"),
            ("ETAS_HOST_MODEL_REQUEST_DEADLINE_MS", "30000"),
            ("ETAS_HOST_MODEL_RETRY_ATTEMPTS", "2"),
            ("ETAS_HOST_MODEL_RETRY_DELAY_MS", "20"),
            (model_output::ENV_KEY, "json-schema"),
            (model_output::ENV_KEY, "output-tool"),
        ] {
            let mut overrides = env();
            overrides.values.insert(name.into(), value.into());
            assert_key(&configured(None, &overrides), SECRET);

            // Incremental configuration patches need not repeat the secret.
            let mut config = configured(None, &env());
            config
                .apply_env_overrides(&RuntimeEnvOverrides {
                    values: BTreeMap::from([(name.into(), value.into())]),
                })
                .unwrap();
            assert_key(&config, SECRET);
        }
    }

    #[test]
    fn partial_local_model_profile_preserves_or_explicitly_replaces_auth() {
        let mut local = RuntimeProfile {
            model: Some(RuntimeModelProfile {
                timeout: Some(RuntimeModelTimeoutProfile {
                    request_deadline_ms: Some(30000),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let config = configured(Some(&local), &env());
        assert_key(&config, SECRET);
        assert_eq!(
            config.runtime_profile_json()["model"]["timeout"]["request_deadline_ms"],
            30000
        );

        local.model.as_mut().unwrap().api_key_env = Some("LOCAL_KEY".into());
        let mut overrides = env();
        overrides
            .values
            .insert("LOCAL_KEY".into(), "local-credential".into());
        assert_key(&configured(Some(&local), &overrides), "local-credential");
        let missing = configured(Some(&local), &env());
        assert!(
            missing.model.as_ref().unwrap().auth.headers().is_empty(),
            "an absent explicitly selected key must not reuse the old identity"
        );
    }

    #[test]
    fn credential_only_overrides_are_applied_without_enabling_an_adapter() {
        for name in model_auth::ENV_KEYS {
            let overrides = RuntimeEnvOverrides {
                values: BTreeMap::from([((*name).into(), "explicit-credential".into())]),
            };
            let mut config = configured(None, &env());
            config.apply_env_overrides(&overrides).unwrap();
            assert_key(&config, "explicit-credential");
            let unconfigured =
                CliHostConfig::from_runtime_profile("none".into(), None, None, &overrides, &[])
                    .unwrap();
            assert!(unconfigured.model.is_none());
            assert!(unconfigured.adapter_transport_allowlist.is_empty());
            assert!(unconfigured.program_network_allowlist.is_empty());
        }
    }

    #[test]
    fn credential_rotation_is_not_persisted_or_part_of_the_profile_fingerprint() {
        let first = configured(None, &env());
        let mut overrides = env();
        overrides
            .values
            .insert("PROFILE_KEY".into(), "rotated-credential".into());
        let second = configured(None, &overrides);
        assert_key(&first, SECRET);
        assert_key(&second, "rotated-credential");
        assert_eq!(first.runtime_profile_json(), second.runtime_profile_json());

        let mut invalid = profile();
        invalid.model.as_mut().unwrap().typed_output_encoding = Some("invalid".into());
        let error =
            CliHostConfig::from_runtime_profile("mock".into(), Some(&invalid), None, &env(), &[])
                .err()
                .unwrap();
        assert!(error.to_string().contains("typed output encoding"));
        assert!(!error.to_string().contains(SECRET));
    }
}
