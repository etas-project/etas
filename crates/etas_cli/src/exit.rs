#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliExit {
    Success,
    Diagnostics,
    Usage,
    RuntimeFailure,
    InternalCompilerError,
    Interrupted,
    Terminated,
}

impl CliExit {
    pub fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Diagnostics => 1,
            Self::Usage => 2,
            Self::RuntimeFailure => 3,
            Self::InternalCompilerError => 4,
            Self::Interrupted => 130,
            Self::Terminated => 143,
        }
    }
}
