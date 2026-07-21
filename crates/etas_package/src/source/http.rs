use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::PackageDiagnostic;

#[derive(Clone, Debug)]
pub struct HttpDownload {
    pub url: String,
    pub token: Option<String>,
    pub user_agent: String,
    pub timeout: Duration,
}

impl HttpDownload {
    pub fn github(url: String, token: Option<String>) -> Self {
        Self {
            url,
            token,
            user_agent: "etas-package/0.1".to_owned(),
            timeout: Duration::from_secs(60),
        }
    }

    pub fn download_to(&self, destination: &Path) -> Result<(), PackageDiagnostic> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent(&self.user_agent)
            .build()
            .map_err(|source| {
                PackageDiagnostic::new(
                    "package.http.client_failed",
                    format!("HTTP client could not be initialized: {source}"),
                    Some(destination.to_path_buf()),
                )
            })?;
        let mut request = client.get(&self.url);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        let response = request.send().map_err(|source| {
            PackageDiagnostic::new(
                "package.http.download_failed",
                format!(
                    "download from `{}` failed: {}",
                    sanitize_url(&self.url, self.token.as_deref()),
                    sanitize_message(&source.to_string(), self.token.as_deref())
                ),
                Some(destination.to_path_buf()),
            )
        })?;
        if !response.status().is_success() {
            return Err(PackageDiagnostic::new(
                "package.http.status_failed",
                format!(
                    "download from `{}` returned HTTP {}",
                    sanitize_url(&self.url, self.token.as_deref()),
                    response.status()
                ),
                Some(destination.to_path_buf()),
            ));
        }
        let bytes = response.bytes().map_err(|source| {
            PackageDiagnostic::new(
                "package.http.body_failed",
                format!(
                    "download body from `{}` could not be read: {}",
                    sanitize_url(&self.url, self.token.as_deref()),
                    sanitize_message(&source.to_string(), self.token.as_deref())
                ),
                Some(destination.to_path_buf()),
            )
        })?;
        atomic_write(destination, &bytes).map_err(|source| {
            PackageDiagnostic::new(
                "package.http.write_failed",
                format!("download destination could not be written: {source}"),
                Some(destination.to_path_buf()),
            )
        })
    }
}

fn atomic_write(destination: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = temp_path(destination);
    let mut file = fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, destination)?;
    Ok(())
}

fn temp_path(destination: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");
    destination.with_file_name(format!(".{file_name}.tmp-{}-{stamp}", std::process::id()))
}

pub fn github_token(config_token: Option<&str>) -> Option<String> {
    config_token
        .map(str::to_owned)
        .or_else(|| std::env::var("ETAS_GITHUB_TOKEN").ok())
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .filter(|token| !token.trim().is_empty())
}

fn sanitize_url(url: &str, token: Option<&str>) -> String {
    sanitize_message(url, token)
}

fn sanitize_message(message: &str, token: Option<&str>) -> String {
    let Some(token) = token else {
        return message.to_owned();
    };
    message.replace(token, "<redacted>")
}
