use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Encountered a token that is unsupported by the parser.")]
    UnsupportedToken,
    #[error("Reached the end of the input token stream.")]
    EndOfInput,
    #[error("Unmatched brace")]
    UnmatchedBrace,
    #[error("Expected a comma here.")]
    ExpectedComma,
    #[error("Expected a valid keyword.")]
    UnexpectedKeyword,
    #[error("Expected a value to be defined.")]
    ExpectedVariableUndefined,
    #[error("Invalid LintKind")]
    InvalidLintKind,
}
