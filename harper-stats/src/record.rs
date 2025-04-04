use harper_core::{
    linting::{Lint, LintGroupConfig, LintKind},
    Document, FatStringToken,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Record {
    pub kind: RecordKind,
    /// Recorded as seconds from the Unix Epoch
    pub when: i64,
    pub uuid: Uuid,
}

impl Record {
    /// Record a new instance at the current system time.
    pub fn now(kind: RecordKind) -> Self {
        Self {
            kind,
            when: chrono::Utc::now().timestamp(),
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum RecordKind {
    Lint {
        kind: LintKind,
        context: Vec<FatStringToken>,
    },
    LintConfigUpdate(LintGroupConfig),
}

impl RecordKind {
    pub fn from_lint(lint: &Lint, doc: &Document) -> Self {
        Self::Lint {
            kind: lint.lint_kind,
            context: doc
                .fat_tokens_intersecting(lint.span)
                .into_iter()
                .map(|t| t.into())
                .collect(),
        }
    }
}
