use crate::communication::Server;
use crate::config::Config;
use std::io;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

/// Represents a highlighter process.
pub struct HighlighterProcess {
    child: Child,
}

impl HighlighterProcess {
    /// Must be called from within a Tokio runtime
    pub fn spawn() -> io::Result<Self> {
        let child = Command::new(std::env::current_exe()?)
            .arg("highlighter")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self { child })
    }

    /// Builds the Tauri-side protocol server from the highlighter process stdio handles.
    ///
    /// This consumes the child stdio handles and can only succeed once per process.
    pub fn create_server(
        &mut self,
        config: Arc<Mutex<Config>>,
    ) -> io::Result<Server<ChildStdout, ChildStdin>> {
        let stdout = self.child.stdout.take().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "highlighter stdout is unavailable",
            )
        })?;
        let stdin = self.child.stdin.take().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "highlighter stdin is unavailable",
            )
        })?;

        Ok(Server::new(stdout, stdin, config))
    }

    pub async fn terminate(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

impl Drop for HighlighterProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
