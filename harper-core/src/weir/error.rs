use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Encountered a token that is unsupported by the parser.")]
    UnsupportedToken(String),
    #[error("Reached the end of the input token stream prematurely.")]
    EndOfInput,
    #[error("Unmatched brace")]
    UnmatchedBrace,
    #[error("Expected a comma here.")]
    ExpectedComma,
    #[error("Expected a valid keyword. Got: {0}")]
    UnexpectedToken(String),
    #[error("Expected a value to be defined.")]
    ExpectedVariableUndefined,
    #[error("Invalid LintKind")]
    InvalidLintKind,
    #[error("Invalid Replacement Strategy")]
    InvalidReplacementStrategy,
    #[error("Invalid Scope")]
    InvalidScope,
    #[error("Expected a variable type other than the one provided.")]
    ExpectedDifferentVariableType,
    #[error("Unable to resolve expression reference {0}")]
    UnableToResolveExpr(String),
    #[error("Line {line}, column {column}: {error}")]
    Located {
        line: usize,
        column: usize,
        error: Box<Error>,
    },
}

impl Error {
    /// Attach the line and column of the character at `offset` in `source`.
    /// Errors that already carry a location are returned unchanged.
    pub(crate) fn located(self, offset: usize, source: &[char]) -> Self {
        if matches!(self, Self::Located { .. }) {
            return self;
        }

        let before = &source[..offset.min(source.len())];
        let line_start = before.iter().rposition(|c| *c == '\n').map_or(0, |i| i + 1);

        Self::Located {
            line: before.iter().filter(|c| **c == '\n').count() + 1,
            column: before.len() - line_start + 1,
            error: Box::new(self),
        }
    }
}
