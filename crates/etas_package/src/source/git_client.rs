use std::{path::Path, process::Command};

pub trait GitClient {
    fn output(&self, repo: &Path, args: &[&str]) -> Result<String, String>;
    fn status(&self, repo: &Path, args: &[&str]) -> Result<(), String>;
    fn status_no_repo(&self, args: &[&str]) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CommandGitClient;

impl GitClient for CommandGitClient {
    fn output(&self, repo: &Path, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    fn status(&self, repo: &Path, args: &[&str]) -> Result<(), String> {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git exited with {status}"))
        }
    }

    fn status_no_repo(&self, args: &[&str]) -> Result<(), String> {
        let status = Command::new("git")
            .args(args)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git exited with {status}"))
        }
    }
}
