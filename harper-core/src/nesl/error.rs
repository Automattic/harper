use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Encountered a token that is unsupported by the parser.")]
    UnsupportedToken(String),
    #[error("Reached the end of the input token stream prematurely.")]
    EndOfInput,
    #[error("Unmatched brace")]
    UnmatchedBrace,
    #[error("Expected a comma here.")]
    ExpectedComma,
    #[error("Expected a valid keyword.")]
    UnexpectedToken(String),
    #[error("Expected a value to be defined.")]
    ExpectedVariableUndefined,
    #[error("Invalid LintKind")]
    InvalidLintKind,
}
