use super::matcher;

use serde_json::Error as SerdeJsonError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("The provided file's item count was malformed.")]
    MalformedItemCount,
    #[error("Expected affix flag to be exactly one character.")]
    MultiCharacterFlag,
    #[error("Expected affix option to be a boolean.")]
    ExpectedBoolean,
    #[error("Expected affix option to be an unsigned integer.")]
    ExpectedUnsignedInteger,
    #[error("Could not parse because we encountered the end of the line.")]
    UnexpectedEndOfLine,
    #[error("Received malformed JSON at line {line}, column {column}: {message}")]
    MalformedJSON {
        message: String,
        line: usize,
        column: usize,
    },
    #[error("An error occurred with a condition: {0}")]
    Matcher(#[from] matcher::Error),
}

impl From<SerdeJsonError> for Error {
    fn from(e: SerdeJsonError) -> Self {
        Error::MalformedJSON {
            message: e.to_string(),
            line: e.line(),
            column: e.column(),
        }
    }
}
