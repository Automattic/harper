use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Weir error: {0}")]
    Weir(#[from] crate::weir::Error),
    #[error("Missing manifest file '{0}'.")]
    MissingManifest(&'static str),
    #[error("Manifest field '{0}' is missing.")]
    MissingManifestField(&'static str),
    #[error("Manifest field '{0}' must be a string.")]
    InvalidManifestFieldType(&'static str),
    #[error("Duplicate manifest file '{0}'.")]
    DuplicateManifest(&'static str),
    #[error("Invalid rule filename '{0}'.")]
    InvalidRuleFileName(String),
}
