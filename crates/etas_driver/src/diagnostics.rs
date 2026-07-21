use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum DriverError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Package(etas_package::PackageError),
    FrontendSession(String),
    InvalidInput(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read `{}`: {source}", path.display()),
            Self::Package(error) => write!(f, "{error}"),
            Self::FrontendSession(message) => write!(f, "frontend session failed: {message}"),
            Self::InvalidInput(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<etas_package::PackageError> for DriverError {
    fn from(error: etas_package::PackageError) -> Self {
        Self::Package(error)
    }
}
