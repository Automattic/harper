use std::io;
use std::process::{Child, Command};

/// Owns the highlighter child process while the Tauri app is running.
///
/// This type exists so child-process cleanup is tied to Rust ownership: keeping it in scope keeps the
/// highlighter alive, and dropping it terminates and reaps the child when the Tauri event loop exits.
pub struct HighlighterProcess {
    child: Child,
}

impl HighlighterProcess {
    pub fn spawn() -> io::Result<Self> {
        let child = Command::new(std::env::current_exe()?)
            .arg("highlighter")
            .spawn()?;

        Ok(Self { child })
    }
}

impl Drop for HighlighterProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
