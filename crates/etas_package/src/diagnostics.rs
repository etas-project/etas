use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub path: Option<PathBuf>,
}

impl PackageDiagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            code,
            message: message.into(),
            path,
        }
    }
}

#[derive(Debug)]
pub enum PackageError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Manifest {
        path: PathBuf,
        message: String,
    },
    Lockfile {
        path: PathBuf,
        message: String,
    },
    Diagnostics(Vec<PackageDiagnostic>),
}

impl PackageError {
    pub fn diagnostic(
        code: &'static str,
        message: impl Into<String>,
        path: Option<PathBuf>,
    ) -> Self {
        Self::Diagnostics(vec![PackageDiagnostic::new(code, message, path)])
    }
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read `{}`: {source}", path.display()),
            Self::Manifest { path, message } => {
                write!(f, "invalid manifest `{}`: {message}", path.display())
            }
            Self::Lockfile { path, message } => {
                write!(f, "invalid lockfile `{}`: {message}", path.display())
            }
            Self::Diagnostics(diagnostics) => {
                for (index, diagnostic) in diagnostics.iter().enumerate() {
                    if index > 0 {
                        f.write_str("; ")?;
                    }
                    match &diagnostic.path {
                        Some(path) => {
                            write!(
                                f,
                                "{}: {} ({})",
                                path.display(),
                                diagnostic.message,
                                diagnostic.code
                            )?;
                        }
                        None => write!(f, "{} ({})", diagnostic.message, diagnostic.code)?,
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PackageError {}
