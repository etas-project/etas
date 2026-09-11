use super::CommandLifecycle;
use std::io::{self, Write};

pub(crate) struct ProcessOutput {
    lifecycle: CommandLifecycle,
    stderr: bool,
}

impl ProcessOutput {
    pub(crate) fn new(lifecycle: CommandLifecycle, stderr: bool) -> Self {
        Self { lifecycle, stderr }
    }
}

impl Write for ProcessOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let bytes = bytes.to_vec();
        let stderr = self.stderr;
        self.lifecycle.io(
            if stderr { "stderr" } else { "stdout" }.into(),
            false,
            move || {
                let len = bytes.len();
                if stderr {
                    io::stderr().write_all(&bytes)?;
                } else {
                    io::stdout().write_all(&bytes)?;
                }
                Ok(len)
            },
        )
    }
    fn flush(&mut self) -> io::Result<()> {
        let stderr = self.stderr;
        self.lifecycle.io(
            if stderr { "stderr" } else { "stdout" }.into(),
            false,
            move || {
                if stderr {
                    io::stderr().flush()
                } else {
                    io::stdout().flush()
                }
            },
        )
    }
}
