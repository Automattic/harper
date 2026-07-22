use thiserror::Error;
use winit::error::{EventLoopError, ExternalError, OsError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("highlighter event loop failed: {0}")]
    EventLoop(#[from] EventLoopError),
    #[error("highlighter window operation failed: {0}")]
    External(#[from] ExternalError),
    #[error("highlighter window creation failed: {0}")]
    Window(#[from] OsError),
    #[error("highlighter renderer failed: {0}")]
    Renderer(#[from] egui_wgpu::WgpuError),
}
