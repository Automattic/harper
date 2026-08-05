use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("system config directory is unavailable")]
    ConfigDirUnavailable,
    #[error("failed to serialize or deserialize config")]
    Serde(#[from] serde_json::Error),
    #[error("failed to access config file")]
    Io(#[from] io::Error),
}
