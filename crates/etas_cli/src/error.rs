use std::{fmt, path::PathBuf};

use crate::exit::CliExit;

#[derive(Debug)]
pub enum CliError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Config {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidUsage(String),
    FrontendSession(String),
    Unsupported {
        command: &'static str,
        reason: &'static str,
    },
}

impl CliError {
    pub fn exit(&self) -> CliExit {
        match self {
            Self::Io { .. } | Self::Config { .. } | Self::InvalidUsage(_) => CliExit::Usage,
            Self::FrontendSession(_) => CliExit::InternalCompilerError,
            Self::Unsupported { .. } => CliExit::InternalCompilerError,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read `{}`: {source}", path.display())
            }
            Self::Config { path, source } => {
                write!(f, "failed to read config `{}`: {source}", path.display())
            }
            Self::InvalidUsage(message) => f.write_str(message),
            Self::FrontendSession(message) => {
                write!(f, "frontend session failed: {message}")
            }
            Self::Unsupported { command, reason } => {
                write!(f, "`etas {command}` is not available yet: {reason}")
            }
        }
    }
}

impl std::error::Error for CliError {}
