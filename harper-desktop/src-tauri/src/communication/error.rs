use thiserror::Error;

/// Errors produced while framing, serializing, or exchanging protocol messages.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("protocol io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("server closed the protocol stream before responding")]
    UnexpectedEof,
    #[error("unexpected response for {expected}")]
    UnexpectedResponse { expected: &'static str },
}
